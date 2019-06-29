package viska.android.main;

import android.os.Bundle;
import android.view.MenuItem;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.lifecycle.ViewModelProviders;
import com.google.android.material.appbar.MaterialToolbar;
import com.google.android.material.navigation.NavigationView;
import viska.android.R;

public class Activity extends AppCompatActivity {

  private ViewModel model;
  private DrawerLayout drawerLayout;
  private MenuItem drawerMenuChatrooms;
  private MenuItem drawerMenuRoster;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.main);

    model = ViewModelProviders.of(this).get(ViewModel.class);
    drawerLayout = findViewById(R.id.drawer_layout);

    final NavigationView drawer = findViewById(R.id.drawer);
    drawerMenuChatrooms = drawer.getMenu().findItem(R.id.chatrooms);
    drawerMenuRoster = drawer.getMenu().findItem(R.id.roster);

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    setSupportActionBar(actionBar);
    actionBar.setNavigationOnClickListener(it -> drawerLayout.openDrawer(GravityCompat.START));

    model.screens.observe(this, this::changeScreen);
    drawer.setNavigationItemSelectedListener(this::onNavigationItemSelected);
  }

  private boolean onNavigationItemSelected(final MenuItem item) {
    switch (item.getItemId()) {
      case R.id.chatrooms:
        item.setChecked(true);
        model.screens.setValue(Screen.CHATROOMS);
        return true;
      case R.id.roster:
        item.setChecked(true);
        model.screens.setValue(Screen.ROSTER);
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
}