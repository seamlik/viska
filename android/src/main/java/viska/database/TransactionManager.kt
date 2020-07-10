package viska.database

import com.couchbase.lite.Database
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import viska.transaction.TransactionOuterClass
import viska.util.uuidFromBytes

class TransactionManager(private val database: Database) {
  suspend fun commit(transactions: Flow<TransactionOuterClass.Transaction>) {
    transactions.collect { transaction ->
      when (transaction.packet.operation) {
        TransactionOuterClass.Operation.ADD -> {
          when (transaction.packet.payloadCase) {
            TransactionOuterClass.Packet.PayloadCase.MESSAGE -> {
              val document =
                  Message.fromPayload(
                      uuidFromBytes(transaction.packet.key.toByteArray()),
                      transaction.packet.message)
              database.save(document)
            }
            TransactionOuterClass.Packet.PayloadCase.CHATROOM -> {
              val document =
                  Chatroom.fromPayload(
                      transaction.packet.key.toByteArray(), transaction.packet.chatroom)
              database.save(document)
            }
            TransactionOuterClass.Packet.PayloadCase.PEER -> {
              val document =
                  Peer.fromPayload(transaction.packet.key.toByteArray(), transaction.packet.peer)
              database.save(document)
            }
            else -> {
              throw BadTransactionException("Empty payload")
            }
          }
        }
        TransactionOuterClass.Operation.DELETE -> {
          when (transaction.packet.type) {
            TransactionOuterClass.PayloadType.MESSAGE ->
                database.getMessage(uuidFromBytes(transaction.packet.key.toByteArray()))?.delete()
            TransactionOuterClass.PayloadType.CHATROOM ->
                database.getChatroom(transaction.packet.key.toByteArray())?.delete()
            TransactionOuterClass.PayloadType.PEER ->
                database.getPeer(transaction.packet.key.toByteArray())?.delete()
            else -> throw BadTransactionException("Unrecognized payload type")
          }
        }
        else -> throw BadTransactionException("Unrecognized operation")
      }
    }
  }
}
