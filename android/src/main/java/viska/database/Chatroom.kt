package viska.database

import com.couchbase.lite.Array
import com.couchbase.lite.Blob
import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableArray
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Objects
import org.bson.BsonBinary
import viska.transaction.TransactionOuterClass

class Chatroom(database: Database, document: DictionaryInterface) : Entity(database, document) {
  companion object {
    fun documentId(chatroomIdUpper: String) = "Chatroom:$chatroomIdUpper"
    fun documentId(chatroomId: ByteArray) =
        "Chatroom:${viska.pki.Module.display_id(BsonBinary(chatroomId))!!.asString().value}"
    fun getChatroomIdFromMembers(members: Collection<String>): String = TODO()

    fun fromPayload(
        chatroomId: ByteArray, payload: TransactionOuterClass.Chatroom
    ): MutableDocument {
      val document = MutableDocument(documentId(chatroomId))

      document.setString("name", payload.name)
      document.setArray(
          "members",
          MutableArray(payload.membersList.map { Blob(MIME_ACCOUNT_ID, it.toByteArray()) }))

      return document
    }
  }

  val name
    get() = document.getString("name") ?: ""

  /** Calculated name to be shown to the user. */
  val displayName: String
    get() {
      val definedName = name
      if (definedName.isNotEmpty()) {
        return definedName
      }

      val members = members
      if (members.isEmpty()) {
        return "Empty room"
      }

      return members.joinToString(",") { memberAccountId ->
        QueryBuilder.select(SelectResult.property("name"))
            .from(DataSource.database(database))
            .where(Expression.property("id").equalTo(Expression.string(memberAccountId.toString())))
            .execute()
            .firstOrNull()
            ?.getString("name")
            ?: "Unknown"
      }
    }

  val latestMessage: Message?
    get() =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(database))
            .where(
                Expression.property("type")
                    .equalTo(Expression.string("Message"))
                    .and(
                        Expression.property("recipients")
                            .equalTo(Expression.list(members.map { Blob(MIME_ACCOUNT_ID, it) }))))
            .orderBy(Ordering.property("time").descending())
            .execute()
            .map { Message(database, it) }
            .firstOrNull()

  val members: List<ByteArray>
    get() =
        (document.getArray("members") as Array?)
            ?.map { (it as Blob).content ?: ByteArray(0) }
            ?.toList()
            ?: emptyList()

  override fun equals(other: Any?): Boolean {
    return if (other is Chatroom) {
      Objects.equals(documentId, other.documentId) &&
          members.size == other.members.size &&
          members.zip(other.members).all { (left, right) -> left.contentEquals(right) } &&
          Objects.equals(name, other.name)
    } else {
      false
    }
  }
}

fun Database.queryChatrooms() =
    QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(this))
        .where(Meta.id.like(Expression.string("Chatroom:%")))

fun Database.getChatroom(chatroomId: ByteArray) =
    getDocument(Chatroom.documentId(chatroomId))?.run { Chatroom(this@getChatroom, this) }
