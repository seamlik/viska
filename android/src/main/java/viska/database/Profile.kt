package viska.database

import android.content.Context
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import com.couchbase.lite.DatabaseConfiguration
import java.nio.file.Files
import org.bson.BsonString

class Profile(private val context: Context, val accountIdText: String) {

  val accountId = viska.pki.Module.parse_id(BsonString(accountIdText))!!.asBinary().data

  val certificate: ByteArray
    get() {
      val path =
          context.filesDir
              .toPath()
              .resolve("account")
              .resolve(accountIdText)
              .resolve("certificate.der")
      return Files.readAllBytes(path)
    }

  fun openDatabase(): Database {
    val config =
        DatabaseConfiguration().apply {
          directory =
              context.filesDir
                  .toPath()
                  .resolve("account")
                  .resolve(accountIdText)
                  .resolve("database")
                  .toString()
        }
    return Database("main", config)
  }
}

fun Context.openProfile(): Profile? {
  val accountIdText =
      PreferenceManager.getDefaultSharedPreferences(this).getString("active-account", null)
          ?: return null
  return Profile(this, accountIdText)
}
