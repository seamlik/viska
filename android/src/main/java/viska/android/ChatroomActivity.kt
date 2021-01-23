package viska.android

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.preferredSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.ListItem
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Scaffold
import androidx.compose.material.Text
import androidx.compose.material.TopAppBar
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.setContent
import androidx.compose.ui.unit.dp
import com.google.protobuf.ByteString
import com.google.protobuf.BytesValue
import dagger.Lazy
import dagger.hilt.android.AndroidEntryPoint
import java.lang.IllegalArgumentException
import javax.inject.Inject
import viska.daemon.Daemon.Message
import viska.database.displayId
import viska.database.toBinaryId

@AndroidEntryPoint
class ChatroomActivity : InstanceActivity() {

  @Inject lateinit var daemonService: Lazy<viska.daemon.DaemonService>

  override fun onCreate(savedInstanceState: Bundle?) {
    try {
      super.onCreate(savedInstanceState)
    } catch (_: ActivityRedirectedException) {
      return
    }

    setContent {
      MaterialTheme {
        val chatroomIdProtobuf = BytesValue.of(ByteString.copyFrom(chatroomId.toBinaryId()))
        val chatroom by daemonService
            .get()
            .nodeGrpcClient
            .watchChatroom(chatroomIdProtobuf)
            .collectAsState(null)
        if (chatroom == null) {
          finish()
          return@MaterialTheme
        }

        val messagesSubscription by daemonService
            .get()
            .nodeGrpcClient
            .watchChatroomMessages(chatroomIdProtobuf)
            .collectAsState(null)

        Scaffold(topBar = { TopAppBar(title = { Text(text = chatroom?.name ?: "") }) }) { _ ->
          LazyColumn {
            items(messagesSubscription?.messagesList ?: emptyList()) { MessageItem(it) }
          }
        }
      }
    }
  }

  private val chatroomId: String
    get() {
      val uriPath = intent.data?.pathSegments ?: emptyList()
      if (uriPath.size != 2 && "chatroom" != uriPath[0]) {
        throw IllegalArgumentException("Bad chatroom URI")
      }
      return uriPath[1]
    }

  companion object {

    fun start(source: Context, chatroomId: ByteArray) {
      // Will be like:
      // viska://chatroom/87956192A8143476909113CDA0D4077E092E26E10CC7DAC43E68F694EA68A036
      val uri =
          Uri.Builder()
              .scheme("viska")
              .authority("chatroom")
              .appendPath(chatroomId.displayId())
              .build()
      val intent = Intent(source, ChatroomActivity::class.java)
      intent.data = uri
      source.startActivity(intent)
    }
  }
}

@Composable
private fun MessageItem(message: Message) {
  ListItem(
      icon = { Image(imageVector = Icons.Default.Person, Modifier.preferredSize(48.dp)) },
      text = { Text(text = message.content) })
}
