package viska.android;

import android.content.DialogInterface;
import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import android.view.MenuItem;
import android.view.View;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.lifecycle.MutableLiveData;
import androidx.lifecycle.ViewModelProviders;
import com.google.android.material.appbar.MaterialToolbar;
import com.google.android.material.dialog.MaterialAlertDialogBuilder;
import com.google.android.material.navigation.NavigationView;
import io.reactivex.rxjava3.disposables.Disposable;

public class MainActivity extends AppCompatActivity {

  public static class MainViewModel extends androidx.lifecycle.ViewModel {

    final MutableLiveData<Screen> screen = new MutableLiveData<>();

    MainViewModel() {
      screen.setValue(Screen.CHATROOMS);
    }
  }

  public enum Screen {
    CHATROOMS,
    ROSTER
  }

  private ViskaService.Connection viska;
  private Intent viskaIntent;
  private MainViewModel model;
  private DrawerLayout drawerLayout;
  private MenuItem drawerMenuChatrooms;
  private MenuItem drawerMenuRoster;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    final Application app = (Application) getApplication();
    viskaIntent = new Intent(this, ViskaService.class);

    if (!app.hasProfile()) {
      startActivity(new Intent(this, NewProfileActivity.class));
      finish();
      return;
    }
    startForegroundService(viskaIntent);
    viska = new ViskaService.Connection();
    bindService(viskaIntent, viska, 0);

    setContentView(R.layout.main);
    model = ViewModelProviders.of(this).get(MainViewModel.class);
    drawerLayout = findViewById(R.id.drawer_layout);

    final NavigationView drawer = findViewById(R.id.drawer);
    drawerMenuChatrooms = drawer.getMenu().findItem(R.id.chatrooms);
    drawerMenuRoster = drawer.getMenu().findItem(R.id.roster);

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    setSupportActionBar(actionBar);
    actionBar.setNavigationOnClickListener(it -> drawerLayout.openDrawer(GravityCompat.START));

    final View fab = findViewById(R.id.fab);
    fab.setOnClickListener(this::onFabClicked);

    final TextView description = drawer.getHeaderView(0).findViewById(R.id.description);
    final Disposable tokenAccountId = viska.getClient().subscribe(
        client -> runOnUiThread(() -> {
          try {
            description.setText(client.account_id_display());
          } catch (Exception err) {
            Log.e(getClass().getSimpleName(), "Failed to read from database.", err);
          }
        })
    );
    viska.putDisposable(tokenAccountId);

    model.screen.observe(this, this::changeScreen);
    drawer.setNavigationItemSelectedListener(this::onNavigationItemSelected);
  }

  @Override
  protected void onDestroy() {
    super.onDestroy();
    if (viska != null) {
      unbindService(viska);
      viska = null;
    }
  }

  private boolean onNavigationItemSelected(final MenuItem item) {
    switch (item.getItemId()) {
      case R.id.chatrooms:
        item.setChecked(true);
        model.screen.setValue(Screen.CHATROOMS);
        return true;
      case R.id.roster:
        item.setChecked(true);
        model.screen.setValue(Screen.ROSTER);
        return true;
      case R.id.exit:
        askExit();
        return true;
      default:
        return false;
    }
  }

  private void changeScreen(final Screen screen) {
    switch (screen) {
      case CHATROOMS:
        drawerMenuChatrooms.setChecked(true);
        getSupportActionBar().setTitle(R.string.title_chatrooms);
        // Change list adapter
        break;
      case ROSTER:
        drawerMenuRoster.setChecked(true);
        getSupportActionBar().setTitle(R.string.title_roster);
        // Change list adapter
        break;
      default:
        return;
    }
    drawerLayout.closeDrawers();
  }

  private void askExit() {
    final DialogInterface.OnClickListener listener = (dialog, which) -> {
      if (which != DialogInterface.BUTTON_POSITIVE) {
        return;
      }
      stopService(viskaIntent);
      finish();
    };

    new MaterialAlertDialogBuilder(this)
        .setTitle(R.string.dialog_exit_title)
        .setMessage(R.string.dialog_exit_text)
        .setPositiveButton(R.string.title_yes, listener)
        .setNegativeButton(R.string.title_no, listener)
        .create()
        .show();
  }

  private void onFabClicked(final View view) {
  }
}
