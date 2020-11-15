package viska.daemon

import com.google.protobuf.BytesValue
import com.google.protobuf.Empty
import io.grpc.Status
import io.grpc.StatusRuntimeException
import javax.inject.Inject
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import viska.couchbase.PeerRepository
import viska.database.Database
import viska.database.Database.TransactionPayload

class PlatformDaemon
    @Inject
    constructor(
        private val peerRepository: PeerRepository,
    ) : PlatformGrpcKt.PlatformCoroutineImplBase() {
  override suspend fun commitTransaction(requests: Flow<TransactionPayload>): Empty {
    // TODO: Batch operation
    requests.collect { payload ->
      when (payload.contentCase) {
        TransactionPayload.ContentCase.ADD_PEER -> {
          peerRepository.commit(payload.addPeer)
        }
        else -> {
          throw StatusRuntimeException(Status.INVALID_ARGUMENT)
        }
      }
    }
    return Empty.getDefaultInstance()
  }

  override suspend fun findPeerById(request: BytesValue): Database.Peer {
    return peerRepository.findById(request.toByteArray())
  }
}
