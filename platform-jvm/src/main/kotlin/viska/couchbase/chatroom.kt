package viska.couchbase

import com.couchbase.lite.Array
import com.couchbase.lite.DataSource
import com.couchbase.lite.DictionaryInterface
import com.couchbase.lite.Expression
import com.couchbase.lite.Meta
import com.couchbase.lite.MutableArray
import com.couchbase.lite.MutableDocument
import com.couchbase.lite.QueryBuilder
import com.couchbase.lite.SelectResult
import com.google.protobuf.ByteString
import java.time.Instant
import java.util.Locale
import java.util.logging.Level
import java.util.logging.Logger
import javax.inject.Inject
import viska.changelog.Changelog
import viska.database.Database.Chatroom
import viska.database.Database.Message
import viska.database.ProfileService
import viska.database.displayId
import viska.database.toBinaryId
import viska.database.toFloat
import viska.database.toProtobufByteString

class ChatroomRepository
    @Inject
    constructor(
        private val profileService: ProfileService,
        private val messageRepository: MessageRepository,
    ) {

  private fun documentId(chatroomId: String) = "Chatroom:${chatroomId.toUpperCase(Locale.ROOT)}"
  private fun documentId(chatroomId: ByteArray) = "Chatroom:${chatroomId.displayId()}"

  fun watchChatrooms(action: (List<ChatroomQueryResult>) -> Unit): AutoCloseable {
    // TODO: Order by latest message
    val query =
        QueryBuilder.select(SelectResult.all())
            .from(DataSource.database(profileService.database))
            .where(Meta.id.like(Expression.string("Chatroom:%")))
    val token =
        query.addChangeListener { change ->
          if (change.error != null) {
            Logger.getGlobal().log(Level.SEVERE, "Error querying list of chatrooms", change.error)
          } else {
            action(
                change.results?.allResults()?.map { result ->
                  val chatroom = result.toChatroom()
                  val latestMessage =
                      messageRepository.getChatroomLatestMessage(result.getString("chatroom-id")!!)
                  ChatroomQueryResult(chatroom, latestMessage)
                }
                    ?: emptyList(),
            )
          }
        }
    query.execute()
    return LiveQueryToken(token, query)
  }

  fun watchChatroom(chatroomId: String, action: (Chatroom) -> Unit): AutoCloseable {
    val documentId = documentId(chatroomId)
    val token =
        profileService.database.addDocumentChangeListener(documentId) { _ ->
          action(profileService.database.getDocument(documentId).toChatroom())
        }
    return DocumentChangeToken(token, profileService.database)
  }

  private fun DictionaryInterface.toChatroom(): Chatroom {

    val builderInner = Changelog.Chatroom.newBuilder()
    builderInner.name = getString("name") ?: ""

    val members =
        (getArray("members") as Array?)
            ?.filterIsInstance(String::class.java)
            ?.filter { it.isNotEmpty() }
            ?.map { it.toBinaryId().toProtobufByteString() }
            ?: emptyList()
    builderInner.addAllMembers(members)

    val builder =
        Chatroom.newBuilder()
            .setInner(builderInner.build())
            .setLatestMessageId(
                getString("latest-message-id")?.toBinaryId()?.toProtobufByteString()
                    ?: ByteString.EMPTY)
            .setTimeUpdated(getDouble("time-updated"))
    getString("chatroom-id")?.let { builder.setChatroomId(it.toBinaryId().toProtobufByteString()) }
    return builder.build()
  }

  fun commit(payload: Chatroom) {
    val chatroomId = payload.chatroomId.toByteArray().displayId()
    val document = MutableDocument(documentId(chatroomId))

    document.setString("type", TYPE)
    document.setString("name", payload.inner.name)
    document.setArray(
        "members",
        MutableArray(payload.inner.membersList.map { it.toByteArray().displayId() }),
    )
    document.setString("chatroom-id", chatroomId)
    document.setDouble("time-updated", Instant.now().toFloat())
    document.setString("latest-message-id", payload.latestMessageId.toByteArray().displayId())

    profileService.database.save(document)
  }

  fun findById(chatroomId: ByteArray) =
      profileService.database.getDocument(documentId(chatroomId)).toChatroom()
}

private const val TYPE = "Chatroom"

data class ChatroomQueryResult(val chatroom: Chatroom, val latestMessage: Message?)
