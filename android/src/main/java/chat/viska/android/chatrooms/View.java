package chat.viska.android.chatrooms;

import android.content.Context;
import androidx.appcompat.widget.Toolbar;
import chat.viska.R;
import chat.viska.android.main.ContentView;

public class View extends ContentView {

  public View(Context context) {
    super(context, null, 0);
    inflateStub(R.layout.chatrooms);

    final Toolbar actionBar = findViewById(R.id.action_bar);
    actionBar.setTitle(R.string.title_chatrooms);
  }
}
