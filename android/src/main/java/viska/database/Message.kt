package viska.database

import android.content.res.Resources
import androidx.core.content.MimeTypeFilter
import com.couchbase.lite.Array
import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Objects
import viska.android.R

class Message(document: DictionaryInterface) : Entity(document) {
  val content
    get() = document.getBlob("content") ?: throw DatabaseCorruptedException()

  val sender
    get() = document.getString("sender") ?: throw DatabaseCorruptedException()

  val time
    get() = document.getDate("time") ?: throw DatabaseCorruptedException()

  /** Chatroom members. */
  val participants
    get() =
        (document.getArray("participants") as Array?)?.map { it as String }?.toList() ?: emptyList()

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
                .like(Expression.string("Message-%"))
                .and(
                    Expression.property("participants")
                        .equalTo(Expression.list(chatroomMembers.toSortedSet().toList()))))
        .orderBy(Ordering.property("time").descending())
