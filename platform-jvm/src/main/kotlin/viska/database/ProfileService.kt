package viska.database

import java.nio.file.Path

interface ProfileService {
  val accountId: String
  val certificate: ByteArray
  val key: ByteArray
  fun createProfile(mock: Boolean)
  val baseDataDir: Path
}
