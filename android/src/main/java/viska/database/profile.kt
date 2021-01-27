package viska.database

import android.content.Context
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import dagger.Binds
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Inject
import javax.inject.Singleton
import org.bson.BsonString

@Singleton
class AndroidProfileService @Inject constructor(@ApplicationContext private val context: Context) :
    ProfileService {

  override val accountId: String
    get() = PreferenceManager.getDefaultSharedPreferences(context).getString("active-account", "")!!

  override fun createProfile(mock: Boolean) {

    val dirData = BsonString(context.filesDir.path)
    val accountId =
        if (mock) {
              Module.create_mock_profile(dirData)
            } else {
              Module.create_standard_profile(dirData)
            }
            .asBinary()
            .data
            .displayId()

    PreferenceManager.getDefaultSharedPreferences(context).edit(commit = true) {
      putString("active-account", accountId)
    }
  }

  override val profileConfig: ProfileConfig
    get() = ProfileConfig(context.filesDir.toPath())
}

@dagger.Module
@InstallIn(ActivityComponent::class, SingletonComponent::class)
abstract class ProfileServiceModule {
  @Binds abstract fun bind(impl: AndroidProfileService): ProfileService
}
