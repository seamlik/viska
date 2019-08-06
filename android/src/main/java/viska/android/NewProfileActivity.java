package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import io.reactivex.Completable;
import io.reactivex.schedulers.Schedulers;
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
    newMockProfileButton.setOnClickListener(this::onNewMockProfile);

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

  private void onNewMockProfile(final View view) {
    final Application app = (Application) getApplication();
    final File tmpProfilePath = new File(getNoBackupFilesDir(), "tmp-profile");

    app.getViewModel().creatingAccount.setValue(true);
    Completable.fromAction(() -> {
      if (tmpProfilePath.exists()) {
        FileUtils.forceDelete(tmpProfilePath);
      }
      if (Files.exists(app.getProfilePath())) {
        FileUtils.forceDelete(app.getProfilePath().toFile());
      }
      switch (view.getId()) {
        case R.id.new_mock_profile:
          viska.mock_profile.Module.new_mock_profile(tmpProfilePath.toString());
          break;
        default:
          break;
      }
      Files.createDirectories(app.getProfilePath().getParent());
      Files.move(tmpProfilePath.toPath(), app.getProfilePath());
      app.getViewModel().creatingAccount.postValue(false);
    }).subscribeOn(Schedulers.io()).subscribe();
  }
}
