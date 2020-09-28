package viska.couchbase

import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import com.couchbase.lite.DictionaryInterface
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.database.DatabaseCorruptedException
import viska.database.ProfileService
import viska.database.Vcard

class VcardService @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(accountId: String) = "Vcard:${accountId.toUpperCase(Locale.ROOT)}"

  private fun DictionaryInterface.toVcard(): Vcard {
    val accountId = getString("account-id") ?: ""
    if (accountId.isBlank()) {
      throw DatabaseCorruptedException("account-id")
    }

    return Vcard(
        name = getString("name") ?: "",
        timeUpdated = getDate("time-updated")?.toInstant(),
        photo = getBlob("photo")?.toBlob(),
        accountId = accountId)
  }

  fun watchVcard(accountId: String, action: (Vcard?) -> Unit): AutoCloseable {
    val token =
        profileService.database.addDocumentChangeListener(documentId(accountId)) { change ->
          action(profileService.database.getDocument(change.documentID)?.toVcard())
        }

    return DocumentChangeToken(token, profileService.database)
  }

  @Composable
  fun watchVcard(accountId: String): StateFlow<Vcard?> {
    val result = remember { MutableStateFlow(null as Vcard?) }
    val token = remember { watchVcard(accountId) { result.value = it } }
    onDispose { token.close() }
    return result
  }

  fun get(accountId: String) = profileService.database.getDocument(documentId(accountId))?.toVcard()
}
