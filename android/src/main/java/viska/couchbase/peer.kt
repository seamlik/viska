package viska.couchbase

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.changelog.Changelog
import viska.database.Database.Peer
import viska.database.DatabaseCorruptedException
import viska.database.ProfileService
import viska.database.displayId
import viska.database.toBinaryId
import viska.database.toProtobufByteString

class PeerRepository @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(accountId: String) = "${TYPE}:${accountId.toUpperCase(Locale.ROOT)}"

  fun commit(payload: Peer) {
    val accountId = payload.inner.accountId.toByteArray().displayId()
    val document = MutableDocument(documentId(accountId))

    document.setString("type", TYPE)
    document.setString("name", payload.inner.name)
    document.setString("role", payload.inner.role.name)
    document.setString("accountId", accountId)

    profileService.database.save(document)
  }

  private fun DictionaryInterface.toPeer(): Peer {
    val accountId = getString("account-id") ?: ""
    if (accountId.isBlank()) {
      throw DatabaseCorruptedException("account-id")
    }
    val inner =
        Changelog.Peer.newBuilder()
            .setAccountId(accountId.toBinaryId().toProtobufByteString())
            .setName(getString("name") ?: "")
            .build()
    return Peer.newBuilder().setInner(inner).build()
  }

  @Composable
  fun watchRoster(): StateFlow<List<Peer>> {
    val result = MutableStateFlow(emptyList<Peer>())
    val isPeer = Expression.property("type").equalTo(Expression.string(TYPE))
    val roleIsFriend =
        Expression.property("role").equalTo(Expression.string(Changelog.PeerRole.FRIEND.name))
    val query =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(profileService.database))
            .where(isPeer.and(roleIsFriend))
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(
                ChatroomRepository::class.java.canonicalName, "Error querying roster", change.error)
          } else {
            result.value = change.results?.allResults()?.map { it.toPeer() } ?: emptyList()
          }
        }
    onDispose { query.removeChangeListener(token) }
    return result
  }

  fun delete(accountId: ByteArray) {
    TODO()
  }
}

private const val TYPE = "Peer"
