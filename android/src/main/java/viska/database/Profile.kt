package viska.database

import com.couchbase.lite.Blob
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.MutableDocument
import org.bson.BsonBinary

class Profile(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun fromPkiBundle(database: Database, certificate: ByteArray, key: ByteArray): Profile {
      val document = MutableDocument("profile")
      document.setBlob("certificate", Blob("application/x-x509-ca-cert", certificate))
      document.setBlob("key", Blob("application/pkcs8", key))
      return Profile(database, document)
    }
  }

  val certificate
    get() =
        document.getBlob("certificate")?.content
            ?: throw DatabaseCorruptedException("No account certificate")

  val accountId: ByteArray
    get() = viska.pki.Module.hash(BsonBinary(certificate))!!.asBinary().data
}

val Database.profile
  get() = Profile(this, getDocument("profile") ?: throw DatabaseCorruptedException("No profile"))
