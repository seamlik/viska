package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import io.realm.OrderedRealmCollection;
import io.realm.RealmRecyclerViewAdapter;
import viska.database.Message;
import viska.database.Vcard;

public class ConversationAdapter
    extends RealmRecyclerViewAdapter<Message, ConversationAdapter.ViewHolder> {

  public static class ViewHolder extends RecyclerView.ViewHolder {
    public ViewHolder(View itemView) {
      super(itemView);
    }
  }

  public ConversationAdapter(OrderedRealmCollection<Message> data) {
    super(data, true, true);
  }

  @Override
  public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext())
            .inflate(R.layout.conversation_item, parent, false));
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Message message = getItem(position);

    final TextView content = holder.itemView.findViewById(R.id.content);
    content.setText(message.getPreview(holder.itemView.getResources()));

    final TextView name = holder.itemView.findViewById(R.id.name);
    final Vcard vcard = Vcard.getById(message.getRealm(), message.sender);
    name.setText(vcard.name);
  }
}
