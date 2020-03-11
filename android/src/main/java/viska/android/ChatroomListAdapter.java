package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import io.realm.OrderedRealmCollection;
import io.realm.RealmRecyclerViewAdapter;
import viska.database.Chatroom;
import viska.database.Message;

public class ChatroomListAdapter
    extends RealmRecyclerViewAdapter<Chatroom, ChatroomListAdapter.ViewHolder> {


  public static class ViewHolder extends RecyclerView.ViewHolder {
    public ViewHolder(View itemView) {
      super(itemView);
    }
  }

  public ChatroomListAdapter(OrderedRealmCollection<Chatroom> data) {
    super(data, true, true);
  }

  @Override
  public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext()).inflate(R.layout.chatroom_list_item, parent, false)
    );
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Chatroom chatroom = getItem(position);

    final TextView name = holder.itemView.findViewById(R.id.name);
    name.setText(chatroom.getDisplayName());

    final TextView description = holder.itemView.findViewById(R.id.description);
    final Message latestMsg = chatroom.getLatestMessage();
    if (latestMsg == null) {
      description.setVisibility(View.GONE);
    } else {
      description.setVisibility(View.VISIBLE);
      description.setText(latestMsg.getPreview(holder.itemView.getResources()));
    }

    holder.itemView.setOnClickListener(
        view -> ChatroomActivity.start(holder.itemView.getContext(), chatroom.id)
    );
  }
}
