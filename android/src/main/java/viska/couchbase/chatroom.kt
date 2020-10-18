package viska.couchbase

import androidx.compose.runtime.Composable
import androidx.compose.runtime.onDispose
import androidx.compose.runtime.remember
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import viska.database.Database.Chatroom
import viska.database.Database.Message

class AndroidChatroomRepository
    @Inject
    constructor(private val chatroomRepository: ChatroomRepository) {

  @Composable
  fun watchChatrooms(): StateFlow<List<ChatroomQueryResult>> {
    val result = remember { MutableStateFlow(emptyList<ChatroomQueryResult>()) }
    val token = remember { chatroomRepository.watchChatrooms { result.value = it } }
    onDispose { token.close() }
    return result
  }

  @Composable
  fun watchChatroom(chatroomId: String): StateFlow<Chatroom?> {
    val result = remember { MutableStateFlow(null as Chatroom?) }
    val token = remember { chatroomRepository.watchChatroom(chatroomId) { result.value = it } }
    onDispose { token.close() }
    return result
  }
}

data class ChatroomQueryResult(val chatroom: Chatroom, val latestMessage: Message?)
