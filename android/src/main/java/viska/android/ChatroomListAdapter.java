package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.DiffUtil;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Database;
import com.couchbase.lite.Result;
import java.util.Objects;
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

  public static class Differ extends DiffUtil.ItemCallback<Result> {
    private final Database database;

    public Differ(final Database database) {
      this.database = database;
    }

    @Override
    public boolean areItemsTheSame(@NonNull Result oldItem, @NonNull Result newItem) {
      return Objects.equals(
          new Chatroom(database, oldItem).getDocumentId(),
          new Chatroom(database, newItem).getDocumentId());
    }

    @Override
    public boolean areContentsTheSame(@NonNull Result oldItem, @NonNull Result newItem) {
      return Objects.equals(new Chatroom(database, oldItem), new Chatroom(database, newItem));
    }
  }

  private final Database database;

  public ChatroomListAdapter(final Database database) {
    super(ChatroomKt.queryChatrooms(database), new Differ(database));
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
