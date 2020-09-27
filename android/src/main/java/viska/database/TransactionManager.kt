package viska.database

import javax.inject.Inject
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import viska.couchbase.ChatroomService
import viska.couchbase.MessageService
import viska.couchbase.PeerService
import viska.transaction.TransactionOuterClass

class TransactionManager
    @Inject
    constructor(
        private val chatroomService: ChatroomService,
        private val messageService: MessageService,
        private val peerService: PeerService,
    ) {

  suspend fun commit(transactions: Flow<TransactionOuterClass.Transaction>) {
    transactions.collect { transaction ->
      when (transaction.packet.operation) {
        TransactionOuterClass.Operation.ADD -> {
          when (transaction.packet.payloadCase) {
            TransactionOuterClass.Packet.PayloadCase.MESSAGE ->
                messageService.commit(
                    transaction.packet.key.toByteArray(), transaction.packet.message)
            TransactionOuterClass.Packet.PayloadCase.CHATROOM ->
                chatroomService.commit(
                    transaction.packet.key.toByteArray(), transaction.packet.chatroom)
            TransactionOuterClass.Packet.PayloadCase.PEER ->
                peerService.commit(transaction.packet.key.toByteArray(), transaction.packet.peer)
            else -> {
              throw BadTransactionException("Empty payload")
            }
          }
        }
        TransactionOuterClass.Operation.DELETE -> {
          when (transaction.packet.type) {
            TransactionOuterClass.PayloadType.MESSAGE ->
                messageService.delete(transaction.packet.key.toByteArray())
            TransactionOuterClass.PayloadType.CHATROOM ->
                chatroomService.delete(transaction.packet.key.toByteArray())
            TransactionOuterClass.PayloadType.PEER ->
                peerService.delete(transaction.packet.key.toByteArray())
            else -> throw BadTransactionException("Unrecognized payload type")
          }
        }
        else -> throw BadTransactionException("Unrecognized operation")
      }
    }
  }
}
