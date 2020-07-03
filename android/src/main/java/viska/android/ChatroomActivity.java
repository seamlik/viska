package viska.android;

import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Document;
import com.couchbase.lite.ListenerToken;
import com.google.android.material.appbar.MaterialToolbar;
import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import viska.database.Chatroom;

public class ChatroomActivity extends InstanceActivity {

  /**
   * Starts this activity.
   *
   * @param chatroomMembers Account ID of the chatroom members.
   */
  public static void start(final Context source, final Collection<String> chatroomMembers) {
    // Will be like viska://chatroom/account1+account2+...
    final Uri uri =
        new Uri.Builder()
            .scheme("viska")
            .authority("chatroom")
            .appendPath(String.join("+", chatroomMembers))
            .build();
    final Intent intent = new Intent(source, ChatroomActivity.class);
    intent.setData(uri);
    source.startActivity(intent);
  }

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.chatroom);

    final RecyclerView list = findViewById(R.id.list);
    list.setLayoutManager(new LinearLayoutManager(this));
  }

  @Override
  protected void onStart() {
    super.onStart();

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    setSupportActionBar(actionBar);

    final List<String> chatroomMembers = getChatroomMembers();
    if (chatroomMembers.isEmpty()) {
      return;
    }

    final String chatroomId = Chatroom.Companion.getChatroomIdFromMembers(chatroomMembers);
    final ListenerToken token =
        db.addDocumentChangeListener(
            Chatroom.Companion.getDocumentId(chatroomId),
            change -> {
              final Document document = change.getDatabase().getDocument(change.getDocumentID());
              if (document != null) {
                final Chatroom chatroom = new Chatroom(change.getDatabase(), document);
                setTitle(chatroom.getDisplayName());
              }
            });
    storeListenerToken(token);

    final RecyclerView list = findViewById(R.id.list);
    final ConversationAdapter adapter = new ConversationAdapter(db, chatroomMembers);
    list.setAdapter(adapter);
  }

  @Override
  protected void onStop() {
    final RecyclerView list = findViewById(R.id.list);
    ((CouchbaseLiveQueryListAdapter) list.getAdapter()).unsubscribe();

    super.onStop();
  }

  private List<String> getChatroomMembers() {
    final Uri uri = getIntent().getData();
    return uri == null
        ? Collections.emptyList()
        : Arrays.asList(uri.getLastPathSegment().split("\\+"));
  }
}
