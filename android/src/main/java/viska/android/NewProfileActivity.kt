package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.Button
import androidx.compose.material.LinearProgressIndicator
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.setContent
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import dagger.Lazy
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import viska.daemon.DaemonService
import viska.database.ProfileService

@AndroidEntryPoint
class NewProfileActivity : AppCompatActivity() {

  @Inject lateinit var profileService: ProfileService
  @Inject lateinit var daemonService: Lazy<DaemonService>

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    setContent {
      val creatingAccount by GlobalState.creatingAccount.collectAsState()
      Page(creatingAccount, { createProfile(false) }, { createProfile(true) })
    }
  }

  private fun createProfile(mock: Boolean) {
    GlobalState.creatingAccount.value = true
    InstanceActivity.finishAll()

    GlobalScope.launch(Dispatchers.IO) {
      try {
        profileService.createProfile(mock)
      } finally {
        GlobalState.creatingAccount.value = false
        startActivity(Intent(this@NewProfileActivity, DashboardActivity::class.java))
        finish()
      }
    }
  }
}

@Composable
private fun Page(creatingAccount: Boolean, newAccount: () -> Unit, newMockProfile: () -> Unit) {
  Column(
      verticalArrangement = Arrangement.SpaceEvenly,
      horizontalAlignment = Alignment.CenterHorizontally,
      modifier = Modifier.fillMaxSize(),
  ) {
    Button(onClick = newAccount, enabled = !creatingAccount) {
      Text(stringResource(R.string.new_account))
    }

    if (BuildConfig.DEBUG) {
      Button(onClick = newMockProfile, enabled = !creatingAccount) {
        Text(stringResource(R.string.new_mock_profile))
      }
    }

    if (creatingAccount) {
      LinearProgressIndicator()
    }
  }
}

@Preview
@Composable
private fun CreatingAccount() {
  Page(true, {}, {})
}

@Preview
@Composable
private fun NotCreatingAccount() {
  Page(false, {}, {})
}
