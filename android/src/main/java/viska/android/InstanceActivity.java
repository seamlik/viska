package viska.android;

import static viska.database.DatabaseKt.open;

import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import com.couchbase.lite.CouchbaseLiteException;
import com.couchbase.lite.Database;
import com.couchbase.lite.ListenerToken;
import java.util.ArrayList;

public abstract class InstanceActivity extends AppCompatActivity {

  protected Database db;
  private final ArrayList<ListenerToken> tokens = new ArrayList<>();

  @Override
  protected void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    GlobalState.INSTANCE
        .getCreatingAccount()
        .observe(
            this,
            creatingAccount -> {
              if (creatingAccount) {
                finish();
              }
            });
  }

  @Override
  protected void onStart() {
    super.onStart();

    db = open();
    if (db.getCount() == 0) {
      startActivity(new Intent(this, NewProfileActivity.class));
      finish();
      return;
    }

    startForegroundService(new Intent(this, DaemonService.class));
  }

  @Override
  protected void onStop() {
    super.onStop();

    if (db != null) {
      synchronized (tokens) {
        tokens.forEach(db::removeChangeListener);
        tokens.clear();
      }
      try {
        db.close();
      } catch (CouchbaseLiteException e) {
        Log.e(this.getClass().getCanonicalName(), "Failed to close database", e);
      }
      db = null;
    }
  }

  protected void moveToNewProfileActivity() {
    startActivity(new Intent(this, NewProfileActivity.class));
    finish();
  }

  protected void storeListenerToken(final ListenerToken token) {
    synchronized (tokens) {
      tokens.add(token);
    }
  }
}
