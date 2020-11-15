package viska.couchbase

import android.content.res.Resources
import androidx.core.content.MimeTypeFilter
import viska.android.R
import viska.changelog.Changelog

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
