package viska.couchbase

import com.couchbase.lite.Blob
import viska.changelog.Changelog
import viska.database.toProtobufByteString

fun Blob.toBlob() =
    Changelog.Blob.newBuilder()
        .setMime(contentType)
        .setContent(content?.toProtobufByteString())
        .build()

fun Changelog.Blob.toCouchbaseBlob() = Blob(mime, content.toByteArray())
