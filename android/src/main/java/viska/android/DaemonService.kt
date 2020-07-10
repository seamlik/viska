package viska.android

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import com.couchbase.lite.Database
import io.grpc.Server
import io.grpc.netty.NettyServerBuilder
import java.net.InetSocketAddress
import viska.daemon.PlatformDaemon
import viska.database.TransactionManager

class DaemonService : Service() {
  private lateinit var database: Database
  private lateinit var transactionManager: TransactionManager
  private lateinit var grpcServer: Server

  override fun onCreate() {
    super.onCreate()
    val notification =
        Notification.Builder(this, NOTIFICATION_CHANNEL_SYSTRAY)
            .setContentTitle(getString(R.string.notification_systray_title))
            .setContentIntent(
                PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), 0))
            .setCategory(Notification.CATEGORY_STATUS)
            .setSmallIcon(R.drawable.icon)
            .build()
    startForeground(R.id.notification_systray, notification)

    database = viska.database.open()
    transactionManager = TransactionManager(database)

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
    database.close()

    super.onDestroy()
  }

  inner class Binder : android.os.Binder()

  override fun onBind(intent: Intent?): IBinder? {
    return Binder()
  }
}
