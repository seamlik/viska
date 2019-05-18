package chat.viska.android.main;

import android.content.Intent;
import android.os.Bundle;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import chat.viska.R;
import chat.viska.android.ViskaService;
import com.google.android.material.appbar.MaterialToolbar;

public class Activity extends AppCompatActivity {

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.main);

    final MaterialToolbar actionBar = findViewById(R.id.action_bar);
    setSupportActionBar(actionBar);

    final DrawerLayout drawer = findViewById(R.id.drawer);
    actionBar.setNavigationOnClickListener(view -> drawer.openDrawer(GravityCompat.START));

    startService(new Intent(this, ViskaService.class));
  }
}
