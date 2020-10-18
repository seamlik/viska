package viska.couchbase

import android.content.res.Resources
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import androidx.core.content.MimeTypeFilter
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.android.R
import viska.changelog.Changelog
import viska.database.Database

class AndroidMessageRepository
    @Inject
    constructor(private val messageRepository: MessageRepository) {

  @Composable
  fun watchChatroomMessages(chatroomId: String): StateFlow<List<Database.Message>> {
    val result = remember { MutableStateFlow(emptyList<Database.Message>()) }
    val token =
        remember { messageRepository.watchChatroomMessages(chatroomId) { result.value = it } }
    onDispose { token.close() }
    return result
  }
}

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
