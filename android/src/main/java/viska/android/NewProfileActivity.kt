package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.Text
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.Button
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.livedata.observeAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.setContent
import androidx.ui.tooling.preview.Preview
import viska.database.createNewProfile

class NewProfileActivity : AppCompatActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    setContent { Ui() }
  }

  @Composable
  @Preview
  private fun Ui() {
    val creatingAccount by GlobalState.creatingAccount.observeAsState(false)

    Column(
        verticalArrangement = Arrangement.SpaceEvenly,
        horizontalGravity = Alignment.CenterHorizontally,
        modifier = Modifier.fillMaxSize()) {
      Button(onClick = this@NewProfileActivity::newAccount, enabled = !creatingAccount) {
        Text(getString(R.string.new_account))
      }
      if (BuildConfig.DEBUG) {
        Button(onClick = this@NewProfileActivity::newMockProfile, enabled = !creatingAccount) {
          Text(getString(R.string.new_mock_profile))
        }
      }
    }
  }

  private fun newAccount() {
    GlobalState.creatingAccount.value = true
    createNewProfile()
    GlobalState.creatingAccount.value = false

    startActivity(Intent(this, MainActivity::class.java))
    finish()
  }

  private fun newMockProfile() {
    // Nothing!
  }
}
