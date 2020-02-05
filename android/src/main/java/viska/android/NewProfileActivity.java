package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import io.reactivex.rxjava3.core.Completable;
import io.reactivex.rxjava3.schedulers.Schedulers;
import java.io.File;
import java.nio.file.Files;
import org.apache.commons.io.FileUtils;

public class NewProfileActivity extends AppCompatActivity {

  @Override
  public void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.new_profile);
    final Application app = (Application) getApplication();
    final ProgressBar progressBar = findViewById(R.id.progress);
    final Button newAccountButton = findViewById(R.id.new_account);

    final Button newMockProfileButton = findViewById(R.id.new_mock_profile);
    newMockProfileButton.setOnClickListener(view -> onNewMockProfile());

    app.getViewModel().creatingAccount.observe(this, running -> {
      if (running) {
        progressBar.setVisibility(View.VISIBLE);
        newAccountButton.setVisibility(View.GONE);
        newMockProfileButton.setVisibility(View.GONE);
      } else {
        if (app.hasProfile()) {
          startActivity(new Intent(this, MainActivity.class));
          finish();
        }
        progressBar.setVisibility(View.GONE);
        newAccountButton.setVisibility(View.VISIBLE);
        newMockProfileButton.setVisibility(BuildConfig.DEBUG ? View.VISIBLE : View.GONE);
      }
    });
  }

  private void onNewMockProfile() {
    final Application app = (Application) getApplication();
    app.getViewModel().creatingAccount.setValue(true);
    Completable.fromAction(() -> {
      final File profile = app.getDatabaseProfilePath().toFile();
      if (profile.exists()) {
        FileUtils.forceDelete(profile);
      }
      final File cache = app.getDatabaseCachePath().toFile();
      if (cache.exists()) {
        FileUtils.forceDelete(cache);
      }
      viska.mock_profiles.Module.new_mock_profile(profile.toString(), cache.toString());
      app.getViewModel().creatingAccount.postValue(false);
    }).subscribeOn(Schedulers.io()).subscribe();
  }
}
