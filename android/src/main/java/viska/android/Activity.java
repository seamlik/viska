package viska.android;

import android.os.Bundle;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;

public abstract class Activity extends AppCompatActivity {
  @Override
  protected void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    final Application app = (Application) getApplication();
    app.getViewModel().creatingAccount.observe(this, creatingAccount -> {
      if (!(this instanceof NewProfileActivity) && creatingAccount) {
        finish();
      }
    });
  }
}
