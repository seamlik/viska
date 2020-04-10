package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import io.realm.OrderedRealmCollection;
import io.realm.RealmRecyclerViewAdapter;
import viska.database.Peer;
import viska.database.Vcard;

public class RosterListAdapter
    extends RealmRecyclerViewAdapter<Peer, RosterListAdapter.ViewHolder> {

  public static class ViewHolder extends RecyclerView.ViewHolder {

    public ViewHolder(View itemView) {
      super(itemView);
    }
  }

  @NonNull
  @Override
  public ViewHolder onCreateViewHolder(@NonNull final ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext()).inflate(R.layout.roster_list_item, parent, false));
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Vcard vcard = getItem(position).getVcard();
    if (vcard != null) {
      final TextView name = holder.itemView.findViewById(R.id.name);
      name.setText(vcard.name);

      final ImageView avatar = holder.itemView.findViewById(R.id.avatar);
      avatar.setImageResource(R.drawable.person); // Custom view for SVG
    }
  }

  public RosterListAdapter(final OrderedRealmCollection<Peer> data) {
    super(data, true, true);
  }
}
