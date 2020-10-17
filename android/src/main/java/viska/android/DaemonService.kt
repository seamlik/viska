package viska.android

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import viska.daemon.DaemonService
import viska.database.ProfileService

@AndroidEntryPoint
class DaemonService : Service() {
  @Inject lateinit var profileService: ProfileService
  @Inject lateinit var daemonService: DaemonService

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
  }

  override fun onDestroy() {
    daemonService.close()
    super.onDestroy()
  }

  inner class Binder : android.os.Binder()

  override fun onBind(intent: Intent?): IBinder? {
    return Binder()
  }
}
