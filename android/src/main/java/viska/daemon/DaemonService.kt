package viska.daemon

import io.grpc.ManagedChannelBuilder
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.random.Random
import org.bson.BsonBinary
import org.bson.BsonInt32
import org.bson.BsonString
import viska.database.ProfileService

@Singleton
class DaemonService @Inject constructor(profileService: ProfileService) : AutoCloseable {

  val nodeGrpcClient: NodeGrpcKt.NodeCoroutineStub
  val nodeGrpcServerHandle: Int

  init {
    if (profileService.accountId.isBlank()) {
      throw IllegalStateException("Cannot start daemon without an active account")
    }

    val localhost = "::1"

    // TODO: TLS
    // Node daemon
    val nodeGrpcPort = Random.nextInt(1, 65536)
    nodeGrpcServerHandle =
        viska.Module.start(
                BsonBinary(profileService.certificate),
                BsonBinary(profileService.key),
                BsonInt32(nodeGrpcPort),
                BsonString(profileService.baseDataDir.toString())
            )
            .asInt32()
            .value
    val channel = ManagedChannelBuilder.forAddress(localhost, nodeGrpcPort).usePlaintext().build()
    nodeGrpcClient = NodeGrpcKt.NodeCoroutineStub(channel)
  }

  override fun close() {
    viska.Module.stop(BsonInt32(nodeGrpcServerHandle))
  }
}
