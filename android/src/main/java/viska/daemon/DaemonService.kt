package viska.daemon

import android.util.Log
import io.grpc.ManagedChannelBuilder
import io.grpc.Server
import io.grpc.netty.NettyServerBuilder
import java.net.InetSocketAddress
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.random.Random
import org.bson.BsonBinary
import org.bson.BsonInt32
import viska.database.ProfileService

@Singleton
class DaemonService
    @Inject
    constructor(profileService: ProfileService, platformDaemon: PlatformDaemon) : AutoCloseable {

  private val platformGrpcServer: Server
  val nodeGrpcClient: NodeGrpcKt.NodeCoroutineStub
  val nodeGrpcServerHandle: Int

  init {
    if (!profileService.hasActiveAccount) {
      throw IllegalStateException("Cannot start daemon without an active account")
    }

    val localhost = "::1"

    // TODO: TLS

    // Platform daemon
    platformGrpcServer =
        NettyServerBuilder.forAddress(InetSocketAddress(localhost, 0))
            .addService(platformDaemon)
            .build()
    platformGrpcServer.start()
    Log.i(
        javaClass.canonicalName,
        "Running PlatformDaemon gRPC server at port ${platformGrpcServer.port}")

    // Node daemon
    val nodeGrpcPort = Random.nextInt(1, 65536)
    nodeGrpcServerHandle =
        viska.Module.start(
                BsonBinary(profileService.certificate),
                BsonBinary(profileService.key),
                BsonInt32(platformGrpcServer.port),
                BsonInt32(nodeGrpcPort),
            )!!
            .asInt32()
            .value
    val channel = ManagedChannelBuilder.forAddress(localhost, nodeGrpcPort).usePlaintext().build()
    nodeGrpcClient = NodeGrpcKt.NodeCoroutineStub(channel)
  }

  override fun close() {
    viska.Module.stop(BsonInt32(nodeGrpcServerHandle))
    platformGrpcServer.shutdown()
  }
}
