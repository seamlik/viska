package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface

class Profile(database: Database, document: DictionaryInterface) : Entity(database, document) {
  val accountId
    get() = document.getString("account-id") ?: throw DatabaseCorruptedException()
}

val Database.profile
  get() = Profile(this, this.getDocument("profile") ?: throw DatabaseCorruptedException())
