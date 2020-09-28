package viska.couchbase

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import com.couchbase.lite.Array
import com.couchbase.lite.Blob
import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableArray
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.time.Instant
import java.util.Date
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import org.bson.BsonArray
import org.bson.BsonBinary
import viska.database.DatabaseCorruptedException
import viska.database.Message
import viska.database.Module.chatroom_id
import viska.database.Module.display_id
import viska.database.ProfileService
import viska.transaction.TransactionOuterClass

class MessageService
    @Inject
    constructor(
        private val profileService: ProfileService,
    ) {

  private fun documentId(messageId: String) = "Message:${messageId.toUpperCase(Locale.ROOT)}"
  private fun documentId(messageId: ByteArray) =
      documentId(display_id(BsonBinary(messageId))!!.asString().value)

  fun commit(messageId: ByteArray, payload: TransactionOuterClass.Message) {
    val document = MutableDocument(documentId(messageId))

    payload.attachment?.also { attachment ->
      document.setBlob(
          "attachment",
          Blob(attachment.mime ?: "", attachment.content.toByteArray() ?: ByteArray(0)),
      )
    }
    document.setString("content", payload.content ?: "")
    document.setDate(
        "time", Date.from(Instant.ofEpochSecond(payload.time.seconds, payload.time.nanos.toLong())))

    val sender = display_id(BsonBinary(payload.sender.toByteArray()))!!.asString().value
    document.setString("sender", sender)

    val recipients =
        payload.recipientsList.map { display_id(BsonBinary(it.toByteArray()))!!.asString().value }
    document.setArray("recipients", MutableArray(recipients))

    // TODO: Create chatroom when receiving message
    val chatroomMembers = payload.recipientsList.map { it.toByteArray() }.toMutableList()
    chatroomMembers.add(payload.sender.toByteArray())
    val chatroomMembersBson = BsonArray(chatroomMembers.map { BsonBinary(it) })
    val chatroomId = chatroom_id(chatroomMembersBson)!!
    val chatroomIdText = display_id(chatroomId)!!.asString().value
    document.setString("chatroom-id", chatroomIdText)

    profileService.database.save(document)
  }

  fun getMessage(messageId: String): Message? {
    val database = profileService.database
    return database.getDocument(documentId(messageId))?.toMessage()
  }

  private fun DictionaryInterface.toMessage() =
      Message(
          content = getString("content") ?: "",
          attachment = getBlob("attachment")?.toBlob(),
          time = getDate("time")?.toInstant() ?: throw DatabaseCorruptedException("time"),
          chatroomId = getString("chatroom-id") ?: throw DatabaseCorruptedException("chatroom-id"),
          sender = getString("sender") ?: throw DatabaseCorruptedException("sender"),
          recipients =
              (getArray("recipients") as Array?)
                  ?.map { (it as String?) ?: "" }
                  ?.filter(String::isNotBlank)
                  ?.toSet()
                  ?: emptySet())

  private fun watchChatroomMessages(
      chatroomId: String, action: (List<Message>) -> Unit
  ): AutoCloseable {
    val chatroomIdNormalized = chatroomId.toUpperCase(Locale.ROOT)
    val query =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(profileService.database))
            .where(
                Meta.id
                    .like(Expression.string("Message:%"))
                    .and(
                        Expression.property("chatroom-id")
                            .equalTo(Expression.string(chatroomIdNormalized))))
            .orderBy(Ordering.property("time").descending())
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(
                MessageService::class.java.canonicalName,
                "Error querying messages of chatroom $chatroomIdNormalized",
                change.error)
          } else {
            action(change.results.allResults().map { it.toMessage() })
          }
        }
    query.execute()
    return LiveQueryToken(token, query)
  }

  @Composable
  fun watchChatroomMessages(chatroomId: String): StateFlow<List<Message>> {
    val result = remember { MutableStateFlow(emptyList<Message>()) }
    val token = remember { watchChatroomMessages(chatroomId) { result.value = it } }
    onDispose { token.close() }
    return result
  }

  fun delete(messageId: ByteArray) {
    TODO()
  }
}
