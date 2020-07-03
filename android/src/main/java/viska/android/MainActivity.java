package viska.android;

import android.content.DialogInterface;
import android.content.Intent;
import android.os.Bundle;
import android.view.MenuItem;
import android.view.View;
import android.widget.TextView;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.lifecycle.MutableLiveData;
import androidx.lifecycle.ViewModelProvider;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.couchbase.lite.Document;
import com.couchbase.lite.ListenerToken;
import com.google.android.flexbox.FlexboxLayoutManager;
import com.google.android.flexbox.JustifyContent;
import com.google.android.material.appbar.MaterialToolbar;
import com.google.android.material.dialog.MaterialAlertDialogBuilder;
import com.google.android.material.navigation.NavigationView;
import org.apache.commons.codec.binary.Hex;
import viska.database.DatabaseCorruptedException;
import viska.database.ProfileKt;
import viska.database.Vcard;

public class MainActivity extends InstanceActivity {

  public static class MainViewModel extends androidx.lifecycle.ViewModel {

    final MutableLiveData<Screen> screen = new MutableLiveData<>();

    public MainViewModel() {
      screen.setValue(Screen.CHATROOMS);
    }
  }

  public enum Screen {
    CHATROOMS,
    ROSTER,
  }

  private MainViewModel model;
  private FlexboxLayoutManager flexboxLayoutManager;
  private LinearLayoutManager linearLayoutManager;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    setContentView(R.layout.main);
    model = new ViewModelProvider(this).get(MainViewModel.class);

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    final DrawerLayout drawerLayout = findViewById(R.id.drawer_layout);
    setSupportActionBar(actionBar);
    actionBar.setNavigationOnClickListener(it -> drawerLayout.openDrawer(GravityCompat.START));

    final View fab = findViewById(R.id.fab);
    fab.setOnClickListener(this::onFabClicked);

    final NavigationView drawer = findViewById(R.id.drawer);
    drawer.setNavigationItemSelectedListener(this::onNavigationItemSelected);

    flexboxLayoutManager = new FlexboxLayoutManager(this);
    flexboxLayoutManager.setJustifyContent(JustifyContent.CENTER);
    linearLayoutManager = new LinearLayoutManager(this);
  }

  @Override
  protected void onStart() {
    super.onStart();

    model.screen.observe(this, this::changeScreen);
    model.screen.setValue(model.screen.getValue());

    final NavigationView drawer = findViewById(R.id.drawer);

    final String accountId;
    try {
      accountId = Hex.encodeHexString(ProfileKt.getProfile(db).getAccountId(), false);
    } catch (DatabaseCorruptedException err) {
      moveToNewProfileActivity();
      return;
    }

    final TextView description = drawer.getHeaderView(0).findViewById(R.id.description);
    description.setText(accountId);

    final TextView name = drawer.getHeaderView(0).findViewById(R.id.name);
    final ListenerToken token =
        db.addDocumentChangeListener(
            Vcard.Companion.getDocumentId(accountId),
            change -> {
              final Document vcard = change.getDatabase().getDocument(change.getDocumentID());
              if (vcard != null) {
                name.setText(vcard.getString("name"));
              }
            });
    storeListenerToken(token);
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
    final NavigationView drawer = findViewById(R.id.drawer);
    final MenuItem drawerMenuChatrooms = drawer.getMenu().findItem(R.id.chatrooms);
    final MenuItem drawerMenuRoster = drawer.getMenu().findItem(R.id.roster);
    final RecyclerView list = findViewById(R.id.list);
    if (list.getAdapter() != null) {
      ((CouchbaseLiveQueryListAdapter) list.getAdapter()).unsubscribe();
    }
    switch (screen) {
      case CHATROOMS:
        drawerMenuChatrooms.setChecked(true);
        getSupportActionBar().setTitle(R.string.chatrooms);
        list.setLayoutManager(linearLayoutManager);
        final ChatroomListAdapter chatroomListAdapter = new ChatroomListAdapter(db);
        list.setAdapter(chatroomListAdapter);
        break;
      case ROSTER:
        drawerMenuRoster.setChecked(true);
        getSupportActionBar().setTitle(R.string.roster);
        list.setLayoutManager(flexboxLayoutManager);
        final RosterListAdapter rosterListAdapter = new RosterListAdapter(db);
        list.setAdapter(rosterListAdapter);
        break;
      default:
        return;
    }

    final DrawerLayout drawerLayout = findViewById(R.id.drawer_layout);
    drawerLayout.closeDrawers();
  }

  @Override
  protected void onStop() {
    final RecyclerView list = findViewById(R.id.list);
    if (list.getAdapter() != null) {
      ((CouchbaseLiveQueryListAdapter) list.getAdapter()).unsubscribe();
    }

    super.onStop();
  }

  private void askExit() {
    final DialogInterface.OnClickListener listener =
        (dialog, which) -> {
          if (which != DialogInterface.BUTTON_POSITIVE) {
            return;
          }
          stopService(new Intent(this, ViskaService.class));
          finish();
        };

    new MaterialAlertDialogBuilder(this)
        .setTitle(R.string.exit)
        .setMessage(R.string.dialog_description_exit)
        .setPositiveButton(android.R.string.ok, listener)
        .setNegativeButton(android.R.string.cancel, listener)
        .create()
        .show();
  }

  private void onFabClicked(final View view) {}
}
