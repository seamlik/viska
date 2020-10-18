package viska.database

import com.couchbase.lite.Database

interface ProfileService {
  val accountId: String
  val certificate: ByteArray
  val key: ByteArray
  val hasActiveAccount: Boolean
  fun createProfile()
  val database: Database
}
