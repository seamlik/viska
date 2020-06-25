package viska.database

import com.couchbase.lite.DictionaryInterface

open class Entity(protected val document: DictionaryInterface) {
  val documentId
    get() = document.getString("id") ?: throw DatabaseCorruptedException()
}
