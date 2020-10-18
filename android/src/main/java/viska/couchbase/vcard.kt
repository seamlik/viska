package viska.couchbase

import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.database.Database

class AndroidVcardRepository @Inject constructor(private val vcardRepository: VcardRepository) {

  @Composable
  fun watchVcard(accountId: String): StateFlow<Database.Vcard?> {
    val result = remember { MutableStateFlow(null as Database.Vcard?) }
    val token = remember { vcardRepository.watchVcard(accountId) { result.value = it } }
    onDispose { token.close() }
    return result
  }
}
