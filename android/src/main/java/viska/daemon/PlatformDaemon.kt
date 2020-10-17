package viska.daemon

import com.google.protobuf.Empty
import io.grpc.Status
import io.grpc.StatusRuntimeException
import javax.inject.Inject
import kotlinx.coroutines.flow.Flow
import viska.couchbase.MessageRepository
import viska.couchbase.PeerRepository
import viska.database.BadTransactionException
import viska.database.Database.TransactionPayload
import viska.database.TransactionManager

class PlatformDaemon
    @Inject
    constructor(
        private val transactionManager: TransactionManager,
        private val peerRepository: PeerRepository,
        private val messageRepository: MessageRepository,
    ) : PlatformGrpcKt.PlatformCoroutineImplBase() {
  override suspend fun commitTransaction(requests: Flow<TransactionPayload>): Empty {
    try {
      transactionManager.commit(requests)
    } catch (e: BadTransactionException) {
      throw StatusRuntimeException(Status.INVALID_ARGUMENT)
    }
    return Empty.getDefaultInstance()
  }

  override suspend fun findPeerById(request: BytesValue): Database.Peer {
    return peerRepository.findById(request.toByteArray())
  }

  override suspend fun findMessageById(request: BytesValue): Database.Message {
    return messageRepository.findById(request.toByteArray())
  }
}
