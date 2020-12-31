package viska.database

import android.content.Context
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import dagger.Binds
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityComponent
import dagger.hilt.android.components.ServiceComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import java.nio.file.Files
import javax.inject.Inject
import javax.inject.Singleton
import org.bson.BsonString

@Singleton
class AndroidProfileService @Inject constructor(@ApplicationContext private val context: Context) :
    ProfileService {

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

  override fun createProfile(mock: Boolean) {

    val profileDir = BsonString(context.filesDir.toPath().resolve("account").toString())
    val accountIdText =
        if (mock) {
          Module.create_mock_profile(profileDir).asString().value
        } else {
          Module.create_standard_profile(profileDir).asString().value
        }

    PreferenceManager.getDefaultSharedPreferences(context).edit(commit = true) {
      putString("active-account", accountIdText)
    }
  }
}

@dagger.Module
@InstallIn(ServiceComponent::class, ActivityComponent::class, SingletonComponent::class)
abstract class ProfileServiceModule {
  @Binds abstract fun bind(impl: AndroidProfileService): ProfileService
}
