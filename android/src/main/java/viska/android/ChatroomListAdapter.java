package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Database;
import viska.database.Chatroom;
import viska.database.ChatroomKt;
import viska.database.Message;

public class ChatroomListAdapter
    extends CouchbaseLiveQueryListAdapter<ChatroomListAdapter.ViewHolder> {

  public static class ViewHolder extends RecyclerView.ViewHolder {
    public ViewHolder(View itemView) {
      super(itemView);
    }
  }

  private final Database database;

  public ChatroomListAdapter(final Database database) {
    super(
        ChatroomKt.queryChatrooms(database),
        new EntityDiffer<>(result -> new Chatroom(database, result)));
    this.database = database;
  }

  @Override
  public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext())
            .inflate(R.layout.chatroom_list_item, parent, false));
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Chatroom chatroom = new Chatroom(database, getItem(position));

    final TextView name = holder.itemView.findViewById(R.id.name);
    name.setText(chatroom.getDisplayName());

    final TextView description = holder.itemView.findViewById(R.id.description);
    final Message latestMsg = chatroom.getLatestMessage();
    if (latestMsg == null) {
      description.setVisibility(View.GONE);
    } else {
      description.setVisibility(View.VISIBLE);
      description.setText(latestMsg.preview(holder.itemView.getResources()));
    }

    holder.itemView.setOnClickListener(
        view -> ChatroomActivity.start(holder.itemView.getContext(), chatroom.getMembers()));
  }
}
