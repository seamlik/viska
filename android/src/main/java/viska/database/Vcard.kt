package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import java.util.Objects
import org.bson.BsonBinary

class Vcard(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun documentId(accountId: ByteArray) =
        "Vcard:${viska.pki.Module.display_id(BsonBinary(accountId))!!.asString().value}"

    fun documentId(accountIdUppercase: String) = "Vcard:$accountIdUppercase"
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

fun Database.getVcard(accountId: ByteArray) =
    getDocument(Vcard.documentId(accountId))?.run { Vcard(this@getVcard, this) }

fun Database.getVcard(accountIdUppercase: String) =
    getDocument(Vcard.documentId(accountIdUppercase))?.run { Vcard(this@getVcard, this) }
