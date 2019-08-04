package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.view.MenuItem;
import android.view.View;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.lifecycle.ViewModelProviders;
import com.google.android.material.appbar.MaterialToolbar;
import com.google.android.material.navigation.NavigationView;

public class MainActivity extends AppCompatActivity {

  private MainViewModel model;
  private DrawerLayout drawerLayout;
  private MenuItem drawerMenuChatrooms;
  private MenuItem drawerMenuRoster;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    final Application app = (Application) getApplication();
    if (!app.hasProfile()) {
      startActivity(new Intent(this, NewProfileActivity.class));
      finish();
      return;
    }

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

    model.screen.observe(this, this::changeScreen);
    drawer.setNavigationItemSelectedListener(this::onNavigationItemSelected);
  }

  private boolean onNavigationItemSelected(final MenuItem item) {
    switch (item.getItemId()) {
      case R.id.chatrooms:
        item.setChecked(true);
        model.screen.setValue(MainScreen.CHATROOMS);
        return true;
      case R.id.roster:
        item.setChecked(true);
        model.screen.setValue(MainScreen.ROSTER);
        return true;
      default:
        return false;
    }
  }

  private void changeScreen(final MainScreen screen) {
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

  private void onFabClicked(final View view) {
  }
}
