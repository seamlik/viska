package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import kotlin.IllegalStateException
import viska.database.ProfileService

@AndroidEntryPoint
abstract class InstanceActivity : AppCompatActivity() {

  @Inject lateinit var profileService: ProfileService

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    if (GlobalState.creatingAccount.value) {
      finish()
      throw IllegalStateException(
          "Launching an InstanceActivity is forbidden during account creation")
    }

    if (profileService.accountId.isBlank()) {
      startActivity(Intent(this, NewProfileActivity::class.java))
      finish()
      throw ActivityRedirectedException()
    }

    startForegroundService(Intent(this, DaemonService::class.java))

    synchronized(INSTANCES) { INSTANCES.add(this) }
  }

  override fun onDestroy() {
    synchronized(INSTANCES) { INSTANCES.remove(this) }
    super.onDestroy()
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
