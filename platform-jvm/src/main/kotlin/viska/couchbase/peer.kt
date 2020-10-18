package viska.couchbase

import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import com.google.protobuf.ByteString
import java.util.Locale
import java.util.logging.Level
import java.util.logging.Logger
import javax.inject.Inject
import viska.changelog.Changelog
import viska.database.Database.Peer
import viska.database.ProfileService
import viska.database.displayId
import viska.database.toBinaryId
import viska.database.toProtobufByteString

class PeerRepository @Inject constructor(private val profileService: ProfileService) {

  private fun documentId(accountId: String) = "${TYPE}:${accountId.toUpperCase(Locale.ROOT)}"
  private fun documentId(accountId: ByteArray) = "${TYPE}:${accountId.displayId()}"

  fun commit(payload: Peer) {
    val accountId = payload.inner.accountId.toByteArray().displayId()
    val document = MutableDocument(documentId(accountId))

    document.setString("type", TYPE)
    document.setString("name", payload.inner.name)
    document.setString("role", payload.inner.role.name)
    document.setString("accountId", accountId)

    profileService.database.save(document)
  }

  protected fun DictionaryInterface.toPeer(): Peer {
    val inner =
        Changelog.Peer.newBuilder()
            .setAccountId(
                getString("account-id")?.toBinaryId()?.toProtobufByteString() ?: ByteString.EMPTY)
            .setName(getString("name") ?: "")
            .build()
    return Peer.newBuilder().setInner(inner).build()
  }

  fun findById(accountId: ByteArray) =
      profileService.database.getDocument(documentId(accountId)).toPeer()

  fun watchRoster(action: (List<Peer>) -> Unit): AutoCloseable {
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
            Logger.getGlobal().log(Level.SEVERE, "Error querying roster", change.error)
          } else {
            action(change.results?.allResults()?.map { it.toPeer() } ?: emptyList())
          }
        }
    return LiveQueryToken(token, query)
  }
}

private const val TYPE = "Peer"
