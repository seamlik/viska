package viska.database

import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Objects
import org.bson.BsonBinary
import viska.transaction.TransactionOuterClass

class Peer(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun documentId(accountId: ByteArray) =
        "Peer:${viska.pki.Module.display_id(BsonBinary(accountId))!!.asString().value}"
    fun fromPayload(accountId: ByteArray, payload: TransactionOuterClass.Peer): MutableDocument {
      val document = MutableDocument(documentId(accountId))

      document.setString("name", payload.name)
      document.setInt("role", payload.role)

      return document
    }
  }

  val role
    get() = document.getInt("role")

  val vcard
    get() = database.getVcard(documentId.removePrefix("Peer:"))

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
                .like(Expression.string("Peer:%"))
                .and(Expression.property("role").greaterThanOrEqualTo(Expression.intValue(0))))

fun Database.getPeer(accountId: ByteArray) =
    getDocument(Peer.documentId(accountId))?.run { Vcard(this@getPeer, this) }
