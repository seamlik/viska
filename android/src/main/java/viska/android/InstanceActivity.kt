package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import kotlin.IllegalStateException
import viska.daemon.DaemonService
import viska.database.ProfileService

@AndroidEntryPoint
abstract class InstanceActivity : AppCompatActivity() {

  @Inject lateinit var profileService: ProfileService
  @Inject lateinit var daemonService: DaemonService

  private var exiting = false

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    if (GlobalState.creatingAccount.value) {
      finish()
      throw IllegalStateException(
          "Launching an InstanceActivity is forbidden during account creation")
    }

    if (exiting) {
      finish()
      throw IllegalStateException("Application is exiting")
    }

    synchronized(INSTANCES) { INSTANCES.add(this) }
  }

  override fun onDestroy() {
    synchronized(INSTANCES) {
      INSTANCES.remove(this)
      if (exiting && INSTANCES.isEmpty()) {
        daemonService.stop()
      }
    }
    super.onDestroy()
  }

  protected fun exitApp() {
    exiting = true
    finishAll()
  }

  protected fun redirectToNewProfile() {
    startActivity(Intent(this, NewProfileActivity::class.java))
    finish()
  }

  companion object {
    private val INSTANCES = mutableSetOf<InstanceActivity>()

    fun finishAll() {
      synchronized(INSTANCES) { INSTANCES.forEach { activity -> activity.finish() } }
    }
  }
}
