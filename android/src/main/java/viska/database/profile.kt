package viska.database

import android.content.Context
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import com.couchbase.lite.DatabaseConfiguration
import dagger.Binds
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityComponent
import dagger.hilt.android.components.ApplicationComponent
import dagger.hilt.android.components.ServiceComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import java.nio.file.Files
import javax.inject.Inject
import javax.inject.Singleton
import org.bson.BsonString

@Singleton
class AndroidProfileService @Inject constructor(@ApplicationContext private val context: Context) :
    ProfileService {

  private var _database = openDatabase(accountId)
  override val database: Database
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

  override val accountId: String
    get() = PreferenceManager.getDefaultSharedPreferences(context).getString("active-account", "")!!

  override val certificate: ByteArray
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

  override val key: ByteArray
    get() {
      val account = accountId
      return if (account.isBlank()) {
        ByteArray(0)
      } else {
        val path =
            context.filesDir.toPath().resolve("account").resolve(accountId).resolve("key.der")
        return Files.readAllBytes(path)
      }
    }

  override val hasActiveAccount
    get() = _database != null

  override fun createProfile() {
    _database?.run { close() }
    _database = null

    val profileDir = context.filesDir.toPath().resolve("account")
    Files.createDirectories(profileDir)
    val accountIdText =
        Module.create_standard_profile(BsonString(profileDir.toString())).asString().value

    PreferenceManager.getDefaultSharedPreferences(context).edit(commit = true) {
      putString("active-account", accountIdText)
    }

    _database = openDatabase(accountIdText)
  }
}

@dagger.Module
@InstallIn(ServiceComponent::class, ActivityComponent::class, ApplicationComponent::class)
abstract class ProfileServiceModule {
  @Binds abstract fun bind(impl: AndroidProfileService): ProfileService
}
