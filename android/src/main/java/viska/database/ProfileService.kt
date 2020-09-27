package viska.database

import android.content.Context
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import com.couchbase.lite.DatabaseConfiguration
import dagger.hilt.android.qualifiers.ApplicationContext
import java.nio.file.Files
import javax.inject.Inject

class ProfileService @Inject constructor(@ApplicationContext private val context: Context) :
    AutoCloseable {

  override fun close() {
    _database?.close()
  }

  private var _database = null as Database?
  val database: Database
    get() = _database ?: openDatabase()

  private fun openDatabase(): Database {
    val account = accountId
    val config =
        if (account.isBlank()) {
          null
        } else {
          DatabaseConfiguration().apply {
            directory =
                context
                    .filesDir
                    .toPath()
                    .resolve("account")
                    .resolve(account)
                    .resolve("database")
                    .toString()
          }
        }
    val result = config?.let { Database("main", it) }
    if (result == null) {
      throw NoActiveAccountException()
    } else {
      _database = result
      return result
    }
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

  val hasActiveAccount = accountId.isNotBlank()
}
