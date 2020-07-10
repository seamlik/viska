package viska.database

import android.content.res.Resources
import androidx.core.content.MimeTypeFilter
import com.couchbase.lite.Array
import com.couchbase.lite.Blob
import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
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
import java.util.Objects
import java.util.UUID
import viska.android.R
import viska.transaction.TransactionOuterClass

class Message(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun documentId(messageId: UUID) = "Message:$messageId"

    fun fromPayload(messageId: UUID, payload: TransactionOuterClass.Message): MutableDocument {
      val document = MutableDocument(documentId(messageId))

      document.setBlob(
          "content", Blob(payload.mime.ifEmpty { "text/plain" }, payload.content.toByteArray()))
      document.setBlob("sender", Blob(MIME_ACCOUNT_ID, payload.content.toByteArray()))
      document.setDate(
          "time",
          Date.from(Instant.ofEpochSecond(payload.time.seconds, payload.time.nanos.toLong())))
      document.setArray(
          "recipients",
          MutableArray(payload.recipientsList.map { Blob(MIME_ACCOUNT_ID, it.toByteArray()) }))

      return document
    }
  }

  val content
    get() = document.getBlob("content") ?: throw DatabaseCorruptedException("No message content")

  val sender
    get() = document.getBlob("sender") ?: throw DatabaseCorruptedException("No message sender")

  val time
    get() = document.getDate("time") ?: throw DatabaseCorruptedException("No message sent time")

  /** Chatroom members. */
  val recipients
    get() = (document.getArray("recipients") as Array?)?.map { it as Blob }?.toList() ?: emptyList()

  /** Generates a text previewing the content of this {@link Message}. */
  fun preview(resources: Resources) =
      when {
        MimeTypeFilter.matches(content.contentType, "text/*") -> {
          getContentAsText()
        }
        MimeTypeFilter.matches(content.contentType, "image/*") -> {
          resources.getString(R.string.placeholder_image)
        }
        MimeTypeFilter.matches(content.contentType, "audio/*") -> {
          resources.getString(R.string.placeholder_audio)
        }
        MimeTypeFilter.matches(content.contentType, "video/*") -> {
          resources.getString(R.string.placeholder_video)
        }
        else -> {
          resources.getString(R.string.placeholder_other)
        }
      }

  /** Gets the content as UTF-8 encoded text. */
  private fun getContentAsText() = content.content?.run { String(this) } ?: ""

  override fun equals(other: Any?): Boolean {
    return if (other is Message) {
      Objects.equals(documentId, other.documentId) &&
          Objects.equals(time, other.time) &&
          Objects.equals(sender, other.sender) &&
          Objects.equals(content, other.content)
    } else {
      false
    }
  }
}

fun Database.queryChatroomMessages(chatroomMembers: Collection<String>) =
    QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(this))
        .where(
            Meta.id
                .like(Expression.string("Message:%"))
                .and(
                    Expression.property("participants")
                        .equalTo(Expression.list(chatroomMembers.toSortedSet().toList()))))
        .orderBy(Ordering.property("time").descending())

fun Database.getMessage(messageId: UUID) =
    getDocument(Message.documentId(messageId))?.run { Message(this@getMessage, this) }
