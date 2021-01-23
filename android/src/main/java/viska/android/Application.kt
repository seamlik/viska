package viska.android

import android.app.NotificationChannel
import android.app.NotificationManager
import androidx.core.content.getSystemService
import dagger.hilt.android.HiltAndroidApp
import viska.R

@HiltAndroidApp
class Application : android.app.Application() {
  override fun onCreate() {
    super.onCreate()

    System.loadLibrary("viska_android")
    viska_android.Module.initialize()
    initializeNotifications()
  }

  private fun initializeNotifications() {
    // TODO: Don't reset the channels every time the app starts
    val manager = getSystemService<NotificationManager>()!!
    val channelSystray =
        NotificationChannel(
            NOTIFICATION_CHANNEL_SYSTRAY,
            getString(R.string.notification_systray_name),
            NotificationManager.IMPORTANCE_NONE)
    channelSystray.setShowBadge(false)
    manager.createNotificationChannel(channelSystray)
  }
}

const val NOTIFICATION_CHANNEL_SYSTRAY = "systray"
