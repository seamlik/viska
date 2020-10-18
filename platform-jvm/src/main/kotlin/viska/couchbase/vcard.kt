package viska.couchbase

import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.MutableDocument
import java.util.Locale
import javax.inject.Inject
import viska.database.Database.Vcard
import viska.database.ProfileService
import viska.database.displayId
import viska.database.toBinaryId
import viska.database.toProtobufByteString

class VcardRepository @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(accountId: String) = "Vcard:${accountId.toUpperCase(Locale.ROOT)}"
  private fun documentId(accountId: ByteArray) = "Vcard:${accountId.displayId()}"

  private fun DictionaryInterface.toVcard(): Vcard {
    val builder = Vcard.newBuilder()
    builder.name = getString("name") ?: ""
    getString("account-id")?.let { builder.accountId = it.toBinaryId().toProtobufByteString() }
    builder.timeUpdated = getDouble("time-updated")
    getBlob("photo")?.let { photo -> builder.photo = photo.toBlob() }
    return builder.build()
  }

  fun commit(payload: Vcard) {
    val document = MutableDocument(documentId(payload.accountId.toByteArray()))
    document.setString("name", payload.name)
    document.setString("account-id", payload.accountId.toByteArray().displayId())
    document.setDouble("time-updated", payload.timeUpdated)
    payload.photo?.let { photo -> document.setBlob("photo", photo.toCouchbaseBlob()) }
    profileService.database.save(document)
  }

  fun watchVcard(accountId: String, action: (Vcard?) -> Unit): AutoCloseable {
    val token =
        profileService.database.addDocumentChangeListener(documentId(accountId)) { change ->
          action(profileService.database.getDocument(change.documentID)?.toVcard())
        }

    return DocumentChangeToken(token, profileService.database)
  }

  fun get(accountId: String) = profileService.database.getDocument(documentId(accountId))?.toVcard()
}
