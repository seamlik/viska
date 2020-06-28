package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Database;
import java.util.Collection;
import viska.database.Message;
import viska.database.MessageKt;
import viska.database.Vcard;
import viska.database.VcardKt;

public class ConversationAdapter
    extends CouchbaseLiveQueryListAdapter<ConversationAdapter.ViewHolder> {

  public static class ViewHolder extends RecyclerView.ViewHolder {
    public ViewHolder(View itemView) {
      super(itemView);
    }
  }

  private final Database database;

  public ConversationAdapter(final Database database, final Collection<String> chatroomMembers) {
    super(
        MessageKt.queryChatroomMessages(database, chatroomMembers),
        new EntityDiffer<>(result -> new Message(database, result)));
    this.database = database;
  }

  @Override
  public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext())
            .inflate(R.layout.conversation_item, parent, false));
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Message message = new Message(database, getItem(position));

    final TextView content = holder.itemView.findViewById(R.id.content);
    content.setText(message.preview(holder.itemView.getResources()));

    final String sender = message.getSender();
    final Vcard vcard = VcardKt.getVcard(database, sender);
    final TextView name = holder.itemView.findViewById(R.id.name);
    name.setText(vcard.getName());
  }
}
