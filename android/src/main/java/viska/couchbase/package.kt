package viska.couchbase

import viska.database.Blob

fun com.couchbase.lite.Blob.toBlob() = Blob(content = content ?: ByteArray(0), type = contentType)
