package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.Observer
import dagger.hilt.android.AndroidEntryPoint
import java.lang.IllegalStateException
import javax.inject.Inject
import viska.database.ProfileService

@AndroidEntryPoint
abstract class InstanceActivity : AppCompatActivity() {

  @Inject lateinit var profileService: ProfileService

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    if (!profileService.hasActiveAccount) {
      startActivity(Intent(this, NewProfileActivity::class.java))
      finish()
      throw IllegalStateException("No active account, switching to NewProfileActivity")
    }

    startForegroundService(Intent(this, DaemonService::class.java))

    GlobalState.creatingAccount.observe(
        this,
        Observer { creatingAccount: Boolean ->
          if (creatingAccount) {
            finish()
          }
        })
  }
}
