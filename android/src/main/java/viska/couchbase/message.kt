package viska.couchbase

import android.content.res.Resources
import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import androidx.core.content.MimeTypeFilter
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
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.android.R
import viska.changelog.Changelog
import viska.database.BadTransactionException
import viska.database.Database
import viska.database.DatabaseCorruptedException
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
    if (sender.isEmpty()) {
      throw BadTransactionException("No sender")
    }
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
        getString("sender")?.toBinaryId()?.toProtobufByteString()
            ?: throw DatabaseCorruptedException("sender")

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
            getString("chatroom-id")?.toBinaryId()?.toProtobufByteString()
                ?: throw DatabaseCorruptedException("chatroom-id"))
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

  private fun watchChatroomMessages(
      chatroomId: String,
      action: (List<Database.Message>) -> Unit,
  ): AutoCloseable {
    val query = queryChatroomMessages(chatroomId)
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(
                MessageRepository::class.java.canonicalName,
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
  fun watchChatroomMessages(chatroomId: String): StateFlow<List<Database.Message>> {
    val result = remember { MutableStateFlow(emptyList<Database.Message>()) }
    val token = remember { watchChatroomMessages(chatroomId) { result.value = it } }
    onDispose { token.close() }
    return result
  }

  fun delete(messageId: ByteArray) {
    TODO()
  }

  fun getChatroomLatestMessage(chatroomId: String) =
      queryChatroomMessages(chatroomId).limit(Expression.intValue(1)).execute().next()?.toMessage()

  fun findById(messageId: ByteArray) =
      profileService.database.getDocument(documentId(messageId)).toMessage()
}

private const val TYPE = "Message"

/** Generates a text previewing the content of this {@link Message}. */
fun Changelog.Message.preview(resources: Resources) =
    if (content.isBlank()) {
      when {
        MimeTypeFilter.matches(attachment?.mime, "image/*") -> {
          resources.getString(R.string.placeholder_image)
        }
        MimeTypeFilter.matches(attachment?.mime, "audio/*") -> {
          resources.getString(R.string.placeholder_audio)
        }
        MimeTypeFilter.matches(attachment?.mime, "video/*") -> {
          resources.getString(R.string.placeholder_video)
        }
        else -> {
          resources.getString(R.string.placeholder_other)
        }
      }
    } else {
      content
    }
