package viska.couchbase

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import com.couchbase.lite.Array
import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableArray
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import dagger.Lazy
import java.lang.IllegalArgumentException
import java.time.Instant
import java.util.Date
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import org.bson.BsonArray
import org.bson.BsonBinary
import viska.database.Chatroom
import viska.database.DatabaseCorruptedException
import viska.database.Module.chatroom_id
import viska.database.Module.display_id
import viska.database.ProfileService
import viska.transaction.TransactionOuterClass

class ChatroomService
    @Inject
    constructor(
        private val profileService: ProfileService,
        private val messageService: Lazy<MessageService>,
        private val vcardService: VcardService,
    ) {

  private fun documentId(chatroomId: String) = "Chatroom:${chatroomId.toUpperCase(Locale.ROOT)}"
  private fun documentId(chatroomId: ByteArray) =
      documentId(display_id(BsonBinary(chatroomId))!!.asString().value)

  private fun watchChatrooms(action: (List<Chatroom>) -> Unit): AutoCloseable {
    // TODO: Order by latest message
    val query =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(profileService.database))
            .where(Meta.id.like(Expression.string("Chatroom:%")))
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(
                ChatroomService::class.java.canonicalName,
                "Error querying list of chatrooms",
                change.error)
          } else {
            action(change.results.allResults().map { it.toChatroom() })
          }
        }
    query.execute()
    return LiveQueryToken(token, query)
  }

  @Composable
  fun watchChatrooms(): StateFlow<List<Chatroom>> {
    val result = remember { MutableStateFlow(emptyList<Chatroom>()) }
    val token = remember { watchChatrooms { result.value = it } }
    onDispose { token.close() }
    return result
  }

  private fun watchChatroom(chatroomId: String, action: (Chatroom) -> Unit): AutoCloseable {
    val documentId = documentId(chatroomId)
    val token =
        profileService.database.addDocumentChangeListener(documentId) { change ->
          action(profileService.database.getDocument(documentId).toChatroom())
        }
    return DocumentChangeToken(token, profileService.database)
  }

  @Composable
  fun watchChatroom(chatroomId: String): StateFlow<Chatroom?> {
    val result = remember { MutableStateFlow(null as Chatroom?) }
    val token = remember { watchChatroom(chatroomId) { result.value = it } }
    onDispose { token.close() }
    return result
  }

  private fun DictionaryInterface.toChatroom(): Chatroom {
    val latestMessageId = getString("latest-message-id") ?: ""
    val latestMessage =
        if (latestMessageId.isBlank()) {
          null
        } else {
          messageService.get().getMessage(latestMessageId)
        }
    val chatroomId =
        getString("chatroom-id")?.run {
          ifBlank { throw DatabaseCorruptedException("chatroom-id") }
        }
            ?: ""

    return Chatroom(
        name = getString("name") ?: "",
        members =
            (getArray("members") as Array?)
                ?.filterIsInstance(String::class.java)
                ?.filter { it.isNotEmpty() }
                ?.toSet()
                ?: emptySet(),
        latestMessage = latestMessage,
        timeUpdated = getDate("time-updated")?.toInstant()
                ?: throw DatabaseCorruptedException("time-updated"),
        chatroomId = chatroomId)
  }

  fun commit(chatroomId: ByteArray, payload: TransactionOuterClass.Chatroom) {
    val document = MutableDocument(documentId(chatroomId))

    document.setString("name", payload.name)
    document.setArray(
        "members",
        MutableArray(
            payload.membersList.map {
              display_id(BsonBinary(it.toByteArray()))!!.asString().value!!
            }),
    )
    document.setString("chatroom-id", display_id(BsonBinary(chatroomId))!!.asString().value)
    document.setDate("time-updated", Date())
    // TODO

    profileService.database.save(document)
  }

  fun Database.getChatroom(chatroomId: String) = getDocument(documentId(chatroomId))?.toChatroom()

  fun delete(chatroomId: ByteArray) {
    TODO()
  }

  fun updateForMessage(members: Set<ByteArray>, messageId: String, messageTime: Instant): Chatroom {
    if (members.isEmpty()) {
      throw IllegalArgumentException("Empty room")
    }

    val chatroomIdBson = chatroom_id(BsonArray(members.map { BsonBinary(it) }))!!.asBinary()!!
    val chatroomIdText = display_id(chatroomIdBson)!!.asString().value!!

    return profileService.database.getDocument(chatroomIdText)?.toMutable()?.let { document ->
      // Assign latest message to the chatroom if it's newer
      messageService.get().getChatroomLatestMessage(chatroomIdText)?.let { latestMessage ->
        if (document.getDate("time-updated")?.toInstant()?.isBefore(latestMessage.time) != false) {
          document.setDate("time-updated", Date.from(latestMessage.time))
        }
        profileService.database.save(document)
      }
      return@let document.toChatroom()
    }
        ?: createForMessage(members, chatroomIdText, messageId, messageTime)
  }

  private fun createForMessage(
      members: Set<ByteArray>,
      chatroomId: String,
      latestMessageId: String,
      latestMessageTime: Instant
  ): Chatroom {
    val document = MutableDocument(documentId(chatroomId))

    document.setString("chatroom-id", chatroomId)
    document.setString("latest-message-id", latestMessageId)
    document.setDate("time-updated", Date.from(latestMessageTime))

    val membersText = members.map { display_id(BsonBinary(it))!!.asString().value!! }
    document.setArray("members", MutableArray(membersText))

    val name = membersText.joinToString(" | ") { vcardService.get(it)?.name ?: it }
    document.setString("name", name)

    profileService.database.save(document)
    return document.toChatroom()
  }
}
