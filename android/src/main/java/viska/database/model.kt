package viska.database

import android.content.res.Resources
import androidx.core.content.MimeTypeFilter
import java.time.Instant
import viska.android.R

data class Chatroom(
    val name: String,
    val members: Set<String>,
    val latestMessage: Message? = null,
    val timeUpdated: Instant,
    val chatroomId: String,
)

data class Message(
    val content: String,
    val attachment: Blob? = null,
    val time: Instant,
    val sender: String,
    val recipients: Set<String>,
    val chatroomId: String
) {

  /** Generates a text previewing the content of this {@link Message}. */
  fun preview(resources: Resources) =
      if (content.isBlank()) {
        when {
          MimeTypeFilter.matches(attachment?.type, "image/*") -> {
            resources.getString(R.string.placeholder_image)
          }
          MimeTypeFilter.matches(attachment?.type, "audio/*") -> {
            resources.getString(R.string.placeholder_audio)
          }
          MimeTypeFilter.matches(attachment?.type, "video/*") -> {
            resources.getString(R.string.placeholder_video)
          }
          else -> {
            resources.getString(R.string.placeholder_other)
          }
        }
      } else {
        content
      }
}

data class Blob(val content: ByteArray, val type: String)

data class Vcard(
    val name: String, val accountId: String, val photo: Blob?, val timeUpdated: Instant?)

data class Peer(val name: String, val accountId: String)
