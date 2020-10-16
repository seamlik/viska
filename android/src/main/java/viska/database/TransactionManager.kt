package viska.database

import javax.inject.Inject
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import viska.couchbase.ChatroomRepository
import viska.couchbase.MessageRepository
import viska.couchbase.PeerRepository
import viska.couchbase.VcardRepository

class TransactionManager
    @Inject
    constructor(
        private val chatroomRepository: ChatroomRepository,
        private val messageRepository: MessageRepository,
        private val peerRepository: PeerRepository,
        private val vcardRepository: VcardRepository,
    ) {

  suspend fun commit(payloads: Flow<Database.TransactionPayload>) {
    // TODO: Batch operation
    payloads.collect { payload ->
      when (payload.contentCase) {
        Database.TransactionPayload.ContentCase.ADD_VCARD -> {
          vcardRepository.commit(payload.addVcard)
        }
        Database.TransactionPayload.ContentCase.ADD_MESSAGE -> {
          messageRepository.commit(payload.addMessage)
        }
        Database.TransactionPayload.ContentCase.ADD_PEER -> {
          peerRepository.commit(payload.addPeer)
        }
        Database.TransactionPayload.ContentCase.ADD_CHATROOM -> {
          chatroomRepository.commit(payload.addChatroom)
        }
        else -> {
          throw BadTransactionException("Empty payload")
        }
      }
    }
  }
}
