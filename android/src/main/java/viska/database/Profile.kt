package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface

class Profile(private val document: DictionaryInterface) {
  val accountId
    get() = document.getString("account-id") ?: throw DatabaseCorruptedException()
}

val Database.profile
  get() = Profile(this.getDocument("profile") ?: throw DatabaseCorruptedException())
