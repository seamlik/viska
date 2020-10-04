package viska.couchbase

import com.couchbase.lite.Blob
import viska.common.Common
import viska.database.toProtobufByteString

fun Blob.toBlob() =
    Common.Blob.newBuilder()
        .setType(contentType)
        .setContent(content?.toProtobufByteString())
        .build()

fun Common.Blob.toCouchbaseBlob() = Blob(type, content.toByteArray())
