package viska.util

import java.nio.BufferUnderflowException
import java.nio.ByteBuffer
import java.util.UUID

fun uuidFromBytes(data: ByteArray): UUID {
  val buffer = ByteBuffer.wrap(data)
  try {
    return UUID(buffer.long, buffer.long)
  } catch (e: BufferUnderflowException) {
    throw IllegalArgumentException("Not a UUID", e)
  }
}
