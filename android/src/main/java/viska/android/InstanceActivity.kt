package viska.android

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.Observer

abstract class InstanceActivity : AppCompatActivity() {

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    startForegroundService(Intent(this, DaemonService::class.java))

    GlobalState.creatingAccount.observe(
        this,
        Observer { creatingAccount: Boolean ->
          if (creatingAccount) {
            finish()
          }
        })
  }

  protected fun moveToNewProfileActivity() {
    Log.i(javaClass.simpleName, "No active account, switching to NewProfileActivity")
    startActivity(Intent(this, NewProfileActivity::class.java))
    finish()
  }
}
