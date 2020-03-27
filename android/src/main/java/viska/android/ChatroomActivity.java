package viska.android;

import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.google.android.material.appbar.MaterialToolbar;
import com.uber.autodispose.AutoDispose;
import com.uber.autodispose.android.lifecycle.AndroidLifecycleScopeProvider;
import viska.database.Chatroom;

public class ChatroomActivity extends InstanceActivity {

  /**
   * Starts this activity.
   * @param id Chatroom ID
   */
  public static void start(final Context source, final String id) {
    final Uri uri = new Uri.Builder().scheme("viska").authority("chatroom").appendPath(id).build();
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

    final Chatroom chatroom = getChatroom();

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    setSupportActionBar(actionBar);
    chatroom
        .<Chatroom>asFlowable()
        .as(AutoDispose.autoDisposable(AndroidLifecycleScopeProvider.from(this)))
        .subscribe(it -> setTitle(it.getDisplayName()));

    final RecyclerView list = findViewById(R.id.list);
    list.setAdapter(new ConversationAdapter(chatroom.getConversation()));
  }

  private Chatroom getChatroom() {
    final Uri uri = getIntent().getData();
    final String id = uri == null ? null : uri.getLastPathSegment();
    return db.getChatroom(id == null ? "" : id);
  }
}
