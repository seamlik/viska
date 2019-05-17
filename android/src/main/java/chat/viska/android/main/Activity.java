package chat.viska.android.main;

import android.content.Intent;
import android.os.Bundle;
import chat.viska.R;
import chat.viska.android.ViskaService;

public class Activity extends android.app.Activity {

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.main);
    startService(new Intent(this, ViskaService.class));
  }
}
