package viska.android;

import android.content.Intent;
import android.os.Bundle;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import viska.database.Database;

public abstract class InstanceActivity extends AppCompatActivity {

  protected Database db;

  @Override
  protected void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    final Application app = (Application) getApplication();
    app.getViewModel().creatingAccount.observe(this, creatingAccount -> {
      if (creatingAccount) {
        finish();
      }
    });
  }

  @Override
  protected void onStart() {
    super.onStart();

    db = ((Application) getApplication()).getDatabase();
    if (db.isEmpty()) {
      startActivity(new Intent(this, NewProfileActivity.class));
      finish();
      return;
    }

    startForegroundService(new Intent(this, ViskaService.class));
  }

  @Override
  protected void onStop() {
    super.onStop();

    if (db != null) {
      db.close();
      db = null;
    }
  }
}
