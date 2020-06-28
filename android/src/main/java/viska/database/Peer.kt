package viska.database

import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Objects

class Peer(database: Database, document: DictionaryInterface) : Entity(database, document) {
  val role
    get() = document.getInt("role")

  val vcard
    get() = database.getVcard(documentId.removePrefix("Peer-"))

  override fun equals(other: Any?): Boolean {
    return if (other is Peer) {
      Objects.equals(documentId, other.documentId) && Objects.equals(role, other.role)
    } else {
      false
    }
  }
}

fun Database.queryRoster() =
    QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(this))
        .where(
            Meta.id
                .like(Expression.string("Peer-%"))
                .and(Expression.property("role").greaterThanOrEqualTo(Expression.intValue(0))))
