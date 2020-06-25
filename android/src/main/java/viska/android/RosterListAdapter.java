package viska.android;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.DiffUtil;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Database;
import com.couchbase.lite.Result;
import java.util.Objects;
import viska.database.Peer;
import viska.database.PeerKt;
import viska.database.Vcard;

public class RosterListAdapter extends CouchbaseLiveQueryListAdapter<RosterListAdapter.ViewHolder> {

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
          new Peer(database, oldItem).getDocumentId(), new Peer(database, newItem).getDocumentId());
    }

    @Override
    public boolean areContentsTheSame(@NonNull Result oldItem, @NonNull Result newItem) {
      return Objects.equals(new Peer(database, oldItem), new Peer(database, newItem));
    }
  }

  private final Database database;

  @NonNull
  @Override
  public ViewHolder onCreateViewHolder(@NonNull final ViewGroup parent, int viewType) {
    return new ViewHolder(
        LayoutInflater.from(parent.getContext()).inflate(R.layout.roster_list_item, parent, false));
  }

  @Override
  public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
    final Vcard vcard = new Peer(database, getItem(position)).getVcard();
    if (vcard != null) {
      final TextView name = holder.itemView.findViewById(R.id.name);
      name.setText(vcard.getName());

      final ImageView avatar = holder.itemView.findViewById(R.id.avatar);
      avatar.setImageResource(R.drawable.person); // Custom view for SVG
    }
  }

  public RosterListAdapter(final Database database) {
    super(PeerKt.queryRoster(database), new Differ(database));
    this.database = database;
  }
}
