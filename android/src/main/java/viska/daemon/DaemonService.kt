package viska.daemon

import android.content.Context
import android.content.Intent
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton
import viska.android.AppBannerService
import viska.database.ProfileService

@Singleton
class DaemonService
@Inject
constructor(
    @ApplicationContext private val context: Context,
    private val profileService: ProfileService,
) {
  private var daemon: DaemonWrapper? = null

  fun getOrCreate(): DaemonWrapper {
    val currentDaemon = daemon
    if (currentDaemon == null) {
      val d = DaemonWrapper(profileService)
      daemon = d
      context.startForegroundService(Intent(context, AppBannerService::class.java))
      return d
    } else {
      return currentDaemon
    }
  }

  fun stop() {
    daemon?.close()
    daemon = null
    context.stopService(Intent(context, AppBannerService::class.java))
  }
}
