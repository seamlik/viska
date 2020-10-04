package viska.database

import javax.inject.Inject
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import viska.couchbase.ChatroomService
import viska.couchbase.MessageService
import viska.couchbase.PeerService
import viska.couchbase.VcardService

class TransactionManager
    @Inject
    constructor(
        private val chatroomService: ChatroomService,
        private val messageService: MessageService,
        private val peerService: PeerService,
        private val vcardService: VcardService,
    ) {

  suspend fun commit(payloads: Flow<Database.TransactionPayload>) {
    // TODO: Batch operation
    payloads.collect { payload ->
      when (payload.contentCase) {
        Database.TransactionPayload.ContentCase.ADD_VCARD -> {
          vcardService.commit(payload.addVcard)
        }
        else -> {
          throw BadTransactionException("Empty payload")
        }
      }
    }
  }
}
