package viska.couchbase

import com.couchbase.lite.Array
import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.MutableArray
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.OrderBy
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import com.google.protobuf.ByteString
import java.util.Locale
import java.util.logging.Level
import java.util.logging.Logger
import javax.inject.Inject
import viska.changelog.Changelog
import viska.database.Database
import viska.database.ProfileService
import viska.database.displayId
import viska.database.toBinaryId
import viska.database.toProtobufByteString

class MessageRepository @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(messageId: String) = "Message:${messageId.toUpperCase(Locale.ROOT)}"
  private fun documentId(messageId: ByteArray) = "Message:${messageId.displayId()}"

  fun commit(payload: Database.Message) {
    val messageId = payload.messageId.toByteArray().displayId()
    val document = MutableDocument(documentId(messageId))

    document.setString("type", TYPE)
    document.setString("content", payload.inner.content ?: "")
    document.setDouble("time", payload.inner.time)
    document.setString("chatroom-id", payload.chatroomId.toByteArray().displayId())
    document.setString("message-id", messageId)
    payload.inner.attachment?.let { attachment ->
      document.setBlob("attachment", attachment.toCouchbaseBlob())
    }

    val sender = payload.inner.sender.toByteArray() ?: ByteArray(0)
    val senderText = sender.displayId()
    document.setString("sender", senderText)

    val recipients = payload.inner.recipientsList.map { it.toByteArray().displayId() }
    document.setArray("recipients", MutableArray(recipients))

    profileService.database.save(document)
  }

  fun getMessage(messageId: String): Database.Message? {
    val database = profileService.database
    return database.getDocument(documentId(messageId))?.toMessage()
  }

  private fun DictionaryInterface.toMessage(): Database.Message {
    val builderInner = Changelog.Message.newBuilder()

    builderInner.content = getString("content") ?: ""
    builderInner.time = getDouble("time")
    builderInner.sender =
        getString("sender")?.toBinaryId()?.toProtobufByteString() ?: ByteString.EMPTY

    val recipients =
        (getArray("recipients") as Array?)
            ?.map { (it as String?)?.toBinaryId()?.toProtobufByteString() ?: ByteString.EMPTY }
            ?.filterNot(ByteString::isEmpty)
            ?.toList()
            ?: emptySet()
    builderInner.addAllRecipients(recipients)

    getBlob("attachment")?.let { attachment -> builderInner.attachment = attachment.toBlob() }

    return Database.Message.newBuilder()
        .setInner(builderInner.build())
        .setChatroomId(
            getString("chatroom-id")?.toBinaryId()?.toProtobufByteString() ?: ByteString.EMPTY)
        .build()
  }

  private fun queryChatroomMessages(chatroomId: String): OrderBy {
    val chatroomIdNormalized = chatroomId.toUpperCase(Locale.ROOT)
    val isMessage = Expression.property("type").equalTo(Expression.string(TYPE))
    val inChatroom =
        Expression.property("chatroom-id").equalTo(Expression.string(chatroomIdNormalized))
    return QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(profileService.database))
        .where(isMessage.and(inChatroom))
        .orderBy(Ordering.property("time").descending())
  }

  fun watchChatroomMessages(
      chatroomId: String,
      action: (List<Database.Message>) -> Unit,
  ): AutoCloseable {
    val query = queryChatroomMessages(chatroomId)
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Logger.getGlobal()
                .log(Level.SEVERE, "Error querying messages of chatroom $chatroomId", change.error)
          } else {
            action(change.results?.allResults()?.map { it.toMessage() } ?: emptyList())
          }
        }
    query.execute()
    return LiveQueryToken(token, query)
  }

  fun getChatroomLatestMessage(chatroomId: String) =
      queryChatroomMessages(chatroomId).limit(Expression.intValue(1)).execute().next()?.toMessage()

  fun findById(messageId: ByteArray) =
      profileService.database.getDocument(documentId(messageId)).toMessage()
}

private const val TYPE = "Message"
