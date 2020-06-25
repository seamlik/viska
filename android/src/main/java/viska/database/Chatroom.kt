package viska.database

import com.couchbase.lite.Array
import com.couchbase.lite.DataSource
import com.couchbase.lite.Database
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.Ordering
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import java.util.Objects

class Chatroom(private val database: Database, document: DictionaryInterface) : Entity(document) {
  companion object {
    fun getDocumentId(chatroomId: String) = "Chatroom-$chatroomId"
    fun getChatroomIdFromMembers(members: Collection<String>): String = TODO()
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
                    .and(Expression.property("participants").equalTo(Expression.list(members))))
            .orderBy(Ordering.property("time").descending())
            .execute()
            .map { Message(it) }
            .firstOrNull()

  val members: List<String>
    get() = (document.getArray("members") as Array?)?.map { it as String }?.toList() ?: emptyList()

  override fun equals(other: Any?): Boolean {
    return if (other is Chatroom) {
      Objects.equals(documentId, other.documentId) &&
          Objects.equals(members, other.members) &&
          Objects.equals(name, other.name)
    } else {
      false
    }
  }
}

fun Database.queryChatrooms() =
    QueryBuilder.select(SelectResult.all())
        .from(DataSource.database(this))
        .where(Meta.id.like(Expression.string("Chatroom-%")))
