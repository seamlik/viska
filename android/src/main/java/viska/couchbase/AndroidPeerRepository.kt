package viska.couchbase

import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.database.Database.Peer

class AndroidPeerRepository @Inject constructor(private val peerRepository: PeerRepository) {

  @Composable
  fun watchRoster(): StateFlow<List<Peer>> {
    val result = remember { MutableStateFlow(emptyList<Peer>()) }
    val token = remember { peerRepository.watchRoster { result.value = it } }
    onDispose { token.close() }
    return result
  }
}
