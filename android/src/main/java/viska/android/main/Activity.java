package viska.android.main;

import android.os.Bundle;
import android.view.MenuItem;
import android.view.ViewGroup;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.lifecycle.ViewModelProviders;
import com.google.android.material.navigation.NavigationView;
import viska.android.R;

public class Activity extends AppCompatActivity {

  private ViewModel model;
  private ViewGroup container;
  private DrawerLayout root;
  private MenuItem drawerMenuChatrooms;
  private MenuItem drawerMenuRoster;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.main);
    model = ViewModelProviders.of(this).get(ViewModel.class);

    container = findViewById(R.id.container_main);
    root = findViewById(R.id.root);

    final NavigationView drawer = findViewById(R.id.drawer);
    drawerMenuChatrooms = drawer.getMenu().findItem(R.id.chatrooms);
    drawerMenuRoster = drawer.getMenu().findItem(R.id.roster);

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
    ContentView view;
    switch (screen) {
      case CHATROOMS:
        drawerMenuChatrooms.setChecked(true);
        view = new viska.android.chatrooms.View(this);
        break;
      case ROSTER:
        drawerMenuRoster.setChecked(true);
        view = new viska.android.roster.View(this);
        break;
      default:
        return;
    }
    root.closeDrawers();
    container.removeAllViews();
    container.addView(view);
    view.getActionBar().setNavigationOnClickListener(it -> root.openDrawer(GravityCompat.START));
  }
}
