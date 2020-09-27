package viska.android

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import dagger.hilt.android.AndroidEntryPoint
import io.grpc.Server
import io.grpc.netty.NettyServerBuilder
import java.net.InetSocketAddress
import javax.inject.Inject
import viska.daemon.PlatformDaemon
import viska.database.ProfileService
import viska.database.TransactionManager

@AndroidEntryPoint
class DaemonService : Service() {
  @Inject lateinit var profileService: ProfileService
  @Inject lateinit var transactionManager: TransactionManager
  private lateinit var grpcServer: Server

  override fun onCreate() {
    super.onCreate()
    val notification =
        Notification.Builder(this, NOTIFICATION_CHANNEL_SYSTRAY)
            .setContentTitle(getString(R.string.notification_systray_title))
            .setContentIntent(
                PendingIntent.getActivity(this, 0, Intent(this, DashboardActivity::class.java), 0))
            .setCategory(Notification.CATEGORY_STATUS)
            .setSmallIcon(R.drawable.icon)
            .build()
    startForeground(R.id.notification_systray, notification)

    val grpcServerPort = "::1"
    grpcServer =
        NettyServerBuilder.forAddress(InetSocketAddress(grpcServerPort, 0))
            .addService(PlatformDaemon(transactionManager))
            .build()
    grpcServer.start()
    Log.i(
        javaClass.canonicalName,
        "Running PlatformDaemon gRPC server at [${grpcServerPort}]:${grpcServer.port}")
  }

  override fun onDestroy() {
    grpcServer.shutdown()
    profileService.close()

    super.onDestroy()
  }

  inner class Binder : android.os.Binder()

  override fun onBind(intent: Intent?): IBinder? {
    return Binder()
  }
}
