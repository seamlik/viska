package viska.database

import android.content.Context
import android.util.Log
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import com.couchbase.lite.DatabaseConfiguration
import dagger.hilt.android.qualifiers.ApplicationContext
import java.nio.file.Files
import javax.inject.Inject
import javax.inject.Singleton
import org.bson.BsonBinary

@Singleton
class ProfileService @Inject constructor(@ApplicationContext private val context: Context) {

  private var _database = openDatabase(accountId)
  val database: Database
    get() = _database ?: error("No active account")

  private fun openDatabase(accountId: String): Database? {
    val config =
        if (accountId.isBlank()) {
          null
        } else {
          DatabaseConfiguration().apply {
            directory =
                context
                    .filesDir
                    .toPath()
                    .resolve("account")
                    .resolve(accountId)
                    .resolve("database")
                    .toString()
          }
        }
    return config?.let { Database("main", it) }
  }

  val accountId: String
    get() = PreferenceManager.getDefaultSharedPreferences(context).getString("active-account", "")!!

  val certificate: ByteArray
    get() {
      val account = accountId
      return if (account.isBlank()) {
        ByteArray(0)
      } else {
        val path =
            context
                .filesDir
                .toPath()
                .resolve("account")
                .resolve(accountId)
                .resolve("certificate.der")
        return Files.readAllBytes(path)
      }
    }

  val hasActiveAccount
    get() = _database != null

  fun createProfile() {
    _database?.run { close() }
    _database = null

    val bundle = viska.pki.Module.new_certificate()!!
    val certificate = bundle.asDocument().getBinary("certificate").data
    val key = bundle.asDocument().getBinary("key").data

    val accountId = Module.hash(BsonBinary(certificate))?.asBinary()?.data!!
    val accountIdText = accountId.displayId()
    Log.i(LOG_TAG, "Generated account $accountIdText")

    val profileDir = context.filesDir.toPath().resolve("account").resolve(accountIdText)
    Files.createDirectories(profileDir)
    Files.write(profileDir.resolve("certificate.der"), certificate)
    Files.write(profileDir.resolve("key.der"), key)

    PreferenceManager.getDefaultSharedPreferences(context).edit(commit = true) {
      putString("active-account", accountIdText)
    }

    _database = openDatabase(accountIdText)
  }
}
