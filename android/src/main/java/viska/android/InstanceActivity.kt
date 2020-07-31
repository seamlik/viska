package viska.android

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.Observer
import com.couchbase.lite.CouchbaseLiteException
import com.couchbase.lite.Database
import com.couchbase.lite.ListenerToken
import java.util.ArrayList
import java.util.function.Consumer
import viska.database.Profile
import viska.database.openProfile

abstract class InstanceActivity : AppCompatActivity() {

  protected lateinit var profile: Profile
  protected lateinit var db: Database
  private val tokens = ArrayList<ListenerToken>()

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    profile = openProfile() ?: return moveToNewProfileActivity()

    GlobalState.creatingAccount
        .observe(
            this,
            Observer { creatingAccount: Boolean ->
              if (creatingAccount) {
                finish()
              }
            })
  }

  override fun onStart() {
    super.onStart()
    db = profile.openDatabase()
    startForegroundService(Intent(this, DaemonService::class.java))
  }

  override fun onStop() {
    super.onStop()
    synchronized(tokens) {
      tokens.forEach(Consumer { token: ListenerToken? -> db.removeChangeListener(token!!) })
      tokens.clear()
    }
    try {
      db.close()
    } catch (e: CouchbaseLiteException) {
      Log.e(this.javaClass.canonicalName, "Failed to close database", e)
    }
  }

  private fun moveToNewProfileActivity() {
    Log.i(javaClass.simpleName, "No active account, switching to NewProfileActivity")
    startActivity(Intent(this, NewProfileActivity::class.java))
    finish()
  }

  protected fun storeListenerToken(token: ListenerToken) {
    synchronized(tokens) { tokens.add(token) }
  }
}
