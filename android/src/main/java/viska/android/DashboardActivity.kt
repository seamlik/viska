package viska.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.viewModels
import androidx.compose.foundation.Icon
import androidx.compose.foundation.Image
import androidx.compose.foundation.Text
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.preferredSize
import androidx.compose.foundation.lazy.LazyColumnFor
import androidx.compose.material.AlertDialog
import androidx.compose.material.Divider
import androidx.compose.material.DrawerState
import androidx.compose.material.DrawerValue
import androidx.compose.material.FloatingActionButton
import androidx.compose.material.IconButton
import androidx.compose.material.ListItem
import androidx.compose.material.MaterialTheme
import androidx.compose.material.ModalDrawerLayout
import androidx.compose.material.Scaffold
import androidx.compose.material.TextButton
import androidx.compose.material.TopAppBar
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ContextAmbient
import androidx.compose.ui.platform.setContent
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.ui.tooling.preview.Preview
import dagger.hilt.android.AndroidEntryPoint
import java.time.Instant
import javax.inject.Inject
import viska.couchbase.ChatroomService
import viska.couchbase.PeerService
import viska.couchbase.VcardService
import viska.database.Chatroom
import viska.database.Database.Vcard
import viska.database.Message
import viska.database.Peer

@AndroidEntryPoint
class DashboardActivity : InstanceActivity() {

  @Inject lateinit var chatroomService: ChatroomService
  @Inject lateinit var vcardService: VcardService
  @Inject lateinit var peerService: PeerService

  private val viewModel by viewModels<DashboardViewModel>()

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    setContent {
      MaterialTheme {
        val chatrooms = chatroomService.watchChatrooms()
        val vcard = vcardService.watchVcard(profileService.accountId)
        val roster = peerService.watchRoster()

        Page(
            viewModel = viewModel,
            chatrooms = chatrooms.collectAsState(),
            vcard = vcard.collectAsState(),
            accountId = profileService.accountId,
            roster = roster.collectAsState())
        ExitDialog(viewModel)
      }
    }
  }

  private fun exitApp() {
    stopService(Intent(this, DaemonService::class.java))
    finish()
  }

  @Composable
  private fun ExitDialog(viewModel: DashboardViewModel) {
    if (viewModel.exitDialogActive) {
      AlertDialog(
          title = { Text(stringResource(R.string.exit)) },
          text = { Text(stringResource(R.string.dialog_description_exit)) },
          confirmButton = {
            TextButton(onClick = this::exitApp) { Text(stringResource(android.R.string.ok)) }
          },
          dismissButton = {
            TextButton(onClick = { viewModel.exitDialogActive = false }) {
              Text(stringResource(android.R.string.cancel))
            }
          },
          onDismissRequest = { viewModel.exitDialogActive = false })
    }
  }
}

class DashboardViewModel : ViewModel() {
  enum class Screen {
    CHATROOMS,
    ROSTER
  }

  var screen by mutableStateOf(Screen.CHATROOMS)
  var exitDialogActive by mutableStateOf(false)
}

@Composable
@Preview
private fun PreviewPage() =
    MaterialTheme {
      Page(
          DashboardViewModel(),
          mutableStateOf(emptyList()),
          mutableStateOf(null),
          "a94eb927fae20e2cbdf417ae3eb920a5423635af772e30e33be78e15a3876259",
          mutableStateOf(emptyList()),
      )
    }

@Composable
private fun Page(
    viewModel: DashboardViewModel,
    chatrooms: State<List<Chatroom>>,
    vcard: State<Vcard?>,
    accountId: String,
    roster: State<List<Peer>>
) {
  val drawerState = rememberDrawerState(DrawerValue.Closed)
  ModalDrawerLayout(
      drawerState = drawerState,
      drawerContent = { DrawerContent(viewModel, drawerState, vcard, accountId) },
      bodyContent = {
        Scaffold(
            topBar = { AppBar(viewModel, drawerState) },
            floatingActionButton = {
              FloatingActionButton(onClick = {}) { Icon(Icons.Default.Add) }
            }) { _ ->
          when (viewModel.screen) {
            DashboardViewModel.Screen.CHATROOMS ->
                LazyColumnFor(items = chatrooms.value) { ChatroomItem(it) }
            DashboardViewModel.Screen.ROSTER ->
                LazyColumnFor(items = roster.value) { RosterItem(it) }
          }
        }
      })
}

@Composable
private fun AppBar(
    viewModel: DashboardViewModel,
    drawerState: DrawerState,
) {
  TopAppBar(
      title = {
        Text(
            when (viewModel.screen) {
              DashboardViewModel.Screen.CHATROOMS -> stringResource(R.string.chatrooms)
              DashboardViewModel.Screen.ROSTER -> stringResource(R.string.roster)
            })
      },
      navigationIcon = {
        IconButton(onClick = { drawerState.open() }) { Icon(Icons.Default.Menu) }
      })
}

@Composable
@Preview
private fun PreviewDrawerContent() {
  MaterialTheme {
    DrawerContent(
        DashboardViewModel(),
        rememberDrawerState(DrawerValue.Closed),
        mutableStateOf(null),
        "a94eb927fae20e2cbdf417ae3eb920a5423635af772e30e33be78e15a3876259")
  }
}

@Composable
private fun DrawerContent(
    viewModel: DashboardViewModel, drawerState: DrawerState, vcard: State<Vcard?>, accountId: String
) {
  Column {

    // Header
    ListItem(
        text = {
          Text(
              style = MaterialTheme.typography.h4,
              maxLines = 1,
              text = vcard.value?.name ?: stringResource(R.string.unknown_account))
        })
    ListItem(
        text = { Text(style = MaterialTheme.typography.body1, maxLines = 1, text = accountId) })

    Divider()

    val selectScreen =
        { screen: DashboardViewModel.Screen ->
          viewModel.screen = screen
          drawerState.close()
        }

    // Screens
    DrawerNavigationItem(
        selected = viewModel.screen == DashboardViewModel.Screen.CHATROOMS,
        onClick = { selectScreen(DashboardViewModel.Screen.CHATROOMS) },
        icon = { Icon(Icons.Default.Forum) },
        text = { Text(stringResource(R.string.chatrooms)) },
    )
    DrawerNavigationItem(
        selected = viewModel.screen == DashboardViewModel.Screen.ROSTER,
        onClick = { selectScreen(DashboardViewModel.Screen.ROSTER) },
        icon = { Icon(Icons.Default.Contacts) },
        text = { Text(stringResource(R.string.roster)) })

    Divider()

    // Misc buttons
    ListItem(
        modifier = Modifier.clickable(onClick = {}),
        icon = { Icon(Icons.Default.Settings) },
        text = { Text(stringResource(R.string.settings)) })
    ListItem(
        modifier = Modifier.clickable(onClick = { viewModel.exitDialogActive = true }),
        icon = { Icon(Icons.Default.ExitToApp) },
        text = { Text(stringResource(R.string.exit)) })
  }
}

@Composable
@Preview
private fun PreviewChatroomItem() {
  val chatroom =
      Chatroom(
          name = "A room",
          members = emptySet(),
          latestMessage =
              Message(
                  content = "Ahoj",
                  time = Instant.now(),
                  chatroomId = "",
                  sender = "",
                  recipients = emptySet(),
              ),
          timeUpdated = Instant.now(),
          chatroomId = "xxx",
      )
  MaterialTheme { ChatroomItem(chatroom) }
}

@Composable
private fun ChatroomItem(chatroom: Chatroom) {
  val context = ContextAmbient.current
  ListItem(
      modifier =
          Modifier.clickable(
              onClick = {
                if (chatroom.members.isNotEmpty()) {
                  ChatroomActivity.start(context, chatroom.chatroomId)
                }
              }),
      icon = { Image(asset = Icons.Default.Person, Modifier.preferredSize(48.dp)) },
      text = { Text(maxLines = 1, text = chatroom.name) },
      secondaryText = {
        Text(maxLines = 3, text = chatroom.latestMessage?.preview(context.resources) ?: "")
      },
  )
}

@Preview
@Composable
private fun PreviewRosterItem() = RosterItem(Peer(name = "A friend", accountId = "xxx"))

@Composable
private fun RosterItem(peer: Peer) {
  ListItem(
      modifier = Modifier.clickable(onClick = {}),
      icon = { Image(asset = Icons.Default.Person, Modifier.preferredSize(48.dp)) },
      text = { Text(maxLines = 1, text = peer.name) },
  )
}
