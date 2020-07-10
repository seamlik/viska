package viska.util

import java.nio.ByteBuffer
import java.util.UUID
import org.junit.Assert
import org.junit.Test

class UtilTest {
  @Test
  fun uuidFromBytes() {
    val randomUuid = UUID.randomUUID()

    val data = ByteArray(128 / 8)
    val encodeBuffer = ByteBuffer.wrap(data)

    // Encode
    encodeBuffer.putLong(randomUuid.mostSignificantBits)
    encodeBuffer.putLong(randomUuid.leastSignificantBits)

    // Decode
    val decodeBuffer = ByteBuffer.wrap(data)
    val decodedUuid = UUID(decodeBuffer.long, decodeBuffer.long)

    Assert.assertEquals(randomUuid, decodedUuid)
  }
}
