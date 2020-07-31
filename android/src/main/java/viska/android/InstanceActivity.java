package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import com.couchbase.lite.CouchbaseLiteException;
import com.couchbase.lite.Database;
import com.couchbase.lite.ListenerToken;
import java.util.ArrayList;
import viska.database.Profile;
import viska.database.ProfileKt;

public abstract class InstanceActivity extends AppCompatActivity {

  protected Profile profile;
  protected Database db;
  private final ArrayList<ListenerToken> tokens = new ArrayList<>();

  @Override
  protected void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    profile = ProfileKt.getProfile(this);
    if (profile == null) {
      Log.i(getClass().getSimpleName(), "No active account, switching to NewProfileActivity");
      moveToNewProfileActivity();
    }

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

    db = profile.openDatabase();
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
