package viska.daemon

import com.google.protobuf.Empty
import io.grpc.Status
import io.grpc.StatusRuntimeException
import kotlinx.coroutines.flow.Flow
import viska.database.BadTransactionException
import viska.database.Database.TransactionPayload
import viska.database.TransactionManager

class PlatformDaemon(private val transactionManager: TransactionManager) :
    PlatformGrpcKt.PlatformCoroutineImplBase() {
  override suspend fun commitTransaction(requests: Flow<TransactionPayload>): Empty {
    try {
      transactionManager.commit(requests)
    } catch (e: BadTransactionException) {
      throw StatusRuntimeException(Status.INVALID_ARGUMENT)
    }
    return Empty.getDefaultInstance()
  }
}
