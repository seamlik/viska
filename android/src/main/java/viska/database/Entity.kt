package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Document
import com.couchbase.lite.MutableDocument

abstract class Entity(
    protected val database: Database, protected val document: DictionaryInterface
) {
  val documentId
    get() = document.getString("id") ?: throw DatabaseCorruptedException("No document ID")

  fun save() {
    val mutableDocument =
        when (document) {
          is MutableDocument -> {
            document
          }
          is Document -> {
            document.toMutable()
          }
          else -> {
            MutableDocument(document.toMap())
          }
        }
    database.save(mutableDocument)
  }

  fun delete() {
    when (document) {
      is Document -> database.delete(document)
      else -> database.getDocument(documentId)?.also { database.delete(it) }
    }
  }

  override fun equals(other: Any?): Boolean {
    TODO("Entity implementations must implement this method")
  }
}
