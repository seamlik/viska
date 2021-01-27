package viska.daemon

import io.grpc.ManagedChannel
import io.grpc.ManagedChannelBuilder
import java.util.concurrent.TimeUnit
import kotlin.random.Random
import org.bson.BsonBinary
import org.bson.BsonDocument
import org.bson.BsonInt32
import org.bson.BsonString
import viska.android.ActivityRedirectedException
import viska.database.ProfileService
import viska.database.toBinaryId

class DaemonWrapper(profileService: ProfileService) : AutoCloseable {

  private val nodeGrpcChannel: ManagedChannel
  val nodeGrpcClient: NodeGrpcKt.NodeCoroutineStub
  private val nodeGrpcServerHandle: Int

  init {
    if (profileService.accountId.isBlank()) {
      throw ActivityRedirectedException()
    }

    val localhost = "::1"

    val profileConfig = profileService.profileConfig
    val profileConfigBson = BsonDocument()
    profileConfigBson["dir_data"] = BsonString(profileConfig.dirData.toString())

    // TODO: TLS
    // Node daemon
    val nodeGrpcPort = Random.nextInt(49152, 65536) // IANA private range
    nodeGrpcServerHandle =
        viska.Module.start(
                BsonBinary(profileService.accountId.toBinaryId()),
                profileConfigBson,
                BsonInt32(nodeGrpcPort),
            )
            .asInt32()
            .value
    nodeGrpcChannel =
        ManagedChannelBuilder.forAddress(localhost, nodeGrpcPort).usePlaintext().build()
    nodeGrpcClient = NodeGrpcKt.NodeCoroutineStub(nodeGrpcChannel)
  }

  override fun close() {
    nodeGrpcChannel.shutdown().awaitTermination(8, TimeUnit.SECONDS)
    viska.Module.stop(BsonInt32(nodeGrpcServerHandle))
  }
}
