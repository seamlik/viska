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
import com.couchbase.lite.OrderBy
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import dagger.Lazy
import java.time.Instant
import java.util.Date
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import org.bson.BsonBinary
import viska.database.BadTransactionException
import viska.database.DatabaseCorruptedException
import viska.database.Message
import viska.database.Module.message_id
import viska.database.ProfileService
import viska.database.displayId
import viska.transaction.TransactionOuterClass

class MessageService
    @Inject
    constructor(
        private val profileService: ProfileService,
        private val chatroomService: Lazy<ChatroomService>,
    ) {

  private fun documentId(messageId: String) = "Message:${messageId.toUpperCase(Locale.ROOT)}"
  private fun documentId(messageId: ByteArray) = "Message:${messageId.displayId()}"

  fun commit(messageId: ByteArray, payload: TransactionOuterClass.Message) {
    val messageIdCalculated = message_id(BsonBinary(payload.toByteArray()))!!.asBinary().data!!
    if (!messageIdCalculated.contentEquals(messageId)) {
      throw BadTransactionException("Mismatch Message ID")
    }

    val messageIdText = documentId(messageId)

    val document = MutableDocument(messageIdText)

    payload.attachment?.also { attachment ->
      document.setBlob(
          "attachment",
          Blob(attachment.mime ?: "", attachment.content.toByteArray() ?: ByteArray(0)),
      )
    }
    document.setString("content", payload.content ?: "")

    val time = Instant.ofEpochSecond(payload.time.seconds, payload.time.nanos.toLong())
    document.setDate("time", Date.from(time))

    val sender = payload.sender.toByteArray() ?: ByteArray(0)
    if (sender.isEmpty()) {
      throw BadTransactionException("No sender")
    }
    val senderText = sender.displayId()
    document.setString("sender", senderText)

    val recipients = payload.recipientsList.map { it.toByteArray().displayId() }
    document.setArray("recipients", MutableArray(recipients))

    val chatroomMembers =
        payload
            .recipientsList
            .filterNotNull()
            .map { it.toByteArray() ?: ByteArray(0) }
            .toMutableSet()
    chatroomMembers.add(sender)

    profileService.database.inBatch {
      val chatroom = chatroomService.get().updateForMessage(chatroomMembers, messageIdText, time)
      document.setString("chatroom-id", chatroom.chatroomId)

      profileService.database.save(document)
    }
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

  private fun queryChatroomMessages(chatroomId: String): OrderBy {
    val chatroomIdNormalized = chatroomId.toUpperCase(Locale.ROOT)
    return QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(profileService.database))
        .where(
            Meta.id
                .like(Expression.string("Message:%"))
                .and(
                    Expression.property("chatroom-id")
                        .equalTo(Expression.string(chatroomIdNormalized))))
        .orderBy(Ordering.property("time").descending())
  }

  private fun watchChatroomMessages(
      chatroomId: String,
      action: (List<Message>) -> Unit,
  ): AutoCloseable {
    val query = queryChatroomMessages(chatroomId)
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(
                MessageService::class.java.canonicalName,
                "Error querying messages of chatroom $chatroomId",
                change.error)
          } else {
            action(change.results?.allResults()?.map { it.toMessage() } ?: emptyList())
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

  fun getChatroomLatestMessage(chatroomId: String) =
      queryChatroomMessages(chatroomId).limit(Expression.intValue(1)).execute().next()?.toMessage()
}
