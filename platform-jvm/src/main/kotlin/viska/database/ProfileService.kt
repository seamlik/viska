package viska.database

interface ProfileService {
  val accountId: String
  val certificate: ByteArray
  val key: ByteArray
  fun createProfile(mock: Boolean)
}
