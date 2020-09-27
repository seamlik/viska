package viska.couchbase

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import org.bson.BsonBinary
import viska.database.DatabaseCorruptedException
import viska.database.Module.display_id
import viska.database.Peer
import viska.database.ProfileService
import viska.transaction.TransactionOuterClass

class PeerService @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(accountId: String) = "Peer:${accountId.toUpperCase(Locale.ROOT)}"

  fun commit(accountId: ByteArray, payload: TransactionOuterClass.Peer) {
    val accountIdText = display_id(BsonBinary(accountId))!!.asString().value!!
    val document = MutableDocument(documentId(accountIdText))

    document.setString("name", payload.name)
    document.setString("role", payload.role.name)
    document.setString("accountId", accountIdText)

    profileService.database.save(document)
  }

  private fun DictionaryInterface.toPeer(): Peer {
    val accountId = getString("account-id") ?: ""
    if (accountId.isBlank()) {
      throw DatabaseCorruptedException("account-id")
    }
    return Peer(name = getString("name") ?: "", accountId = accountId)
  }

  @Composable
  fun watchRoster(): StateFlow<List<Peer>> {
    val result = MutableStateFlow(emptyList<Peer>())
    val query =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(profileService.database))
            .where(
                Meta.id
                    .like(Expression.string("Peer:%"))
                    .and(
                        Expression.property("role")
                            .equalTo(
                                Expression.string(TransactionOuterClass.PeerRole.FRIEND.name))))
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Log.e(ChatroomService::class.java.canonicalName, "Error querying roster", change.error)
          } else {
            result.value = change.results.allResults().map { it.toPeer() }
          }
        }
    onDispose { query.removeChangeListener(token) }
    return result
  }

  fun delete(accountId: ByteArray) {
    TODO()
  }
}
