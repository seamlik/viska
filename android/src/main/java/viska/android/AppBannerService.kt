package viska.android

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import viska.R

/** Shows a permanent notification so that the Viska daemon can run in the background. */
class AppBannerService : Service() {

  override fun onCreate() {
    super.onCreate()
    // notification
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

  class Binder : android.os.Binder()

  override fun onBind(intent: Intent?): IBinder {
    return Binder()
  }
}
