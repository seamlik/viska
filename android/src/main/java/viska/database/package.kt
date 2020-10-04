package viska.database

import com.google.protobuf.ByteString
import org.apache.commons.codec.binary.Hex

/** The canonical way to encode a binary ID into text. */
fun ByteArray.displayId() = Hex.encodeHexString(this, false) ?: ""

/** The canonical way to decode a binary ID from text. */
fun String.toBinaryId() = Hex.decodeHex(this) ?: ByteArray(0)

fun ByteArray.toProtobufByteString(): ByteString = ByteString.copyFrom(this)
