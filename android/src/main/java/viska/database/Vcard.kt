package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import java.util.Objects

class Vcard(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun getDocumentId(accountId: String) = "Vcard-$accountId"
  }

  val name
    get() = document.getString("name") ?: ""

  val timeUpdated
    get() = document.getDate("time-updated")

  val photo
    get() = document.getBlob("photo")

  override fun equals(other: Any?): Boolean {
    return if (other is Vcard) {
      Objects.equals(documentId, other.documentId) &&
          Objects.equals(name, other.name) &&
          Objects.equals(photo, other.photo)
    } else {
      false
    }
  }
}

fun Database.getVcard(accountId: String) =
    getDocument(Vcard.getDocumentId(accountId))?.run { Vcard(this@getVcard, this) }
