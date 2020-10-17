package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import viska.database.ProfileService
import kotlin.IllegalStateException

@AndroidEntryPoint
abstract class InstanceActivity : AppCompatActivity() {

  @Inject lateinit var profileService: ProfileService

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    if (GlobalState.creatingAccount.value) {
      finish()
      throw IllegalStateException("Launching an InstanceActivity is forbidden during account creation")
    }

    if (!profileService.hasActiveAccount) {
      return
    }

    startForegroundService(Intent(this, DaemonService::class.java))

    synchronized(INSTANCES) {
      INSTANCES.add(this)
    }
  }

  override fun onDestroy() {
    synchronized(INSTANCES) {
      INSTANCES.remove(this)
    }
    super.onDestroy()
  }

  /** Must be invoked by child classes at the earliest stage of [onCreate]. */
  protected fun cancelIfNoActiveAccount() {
    if (!profileService.hasActiveAccount) {
      startActivity(Intent(this, NewProfileActivity::class.java))
      finish()
      return
    }
  }

  companion object {
    private val INSTANCES = mutableSetOf<InstanceActivity>()

    fun finishAll() {
      synchronized(INSTANCES) {
        INSTANCES.forEach { activity -> activity.finish() }
        INSTANCES.clear()
      }
    }
  }
}
