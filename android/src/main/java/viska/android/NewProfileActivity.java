package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import androidx.annotation.Nullable;
import io.reactivex.Completable;
import io.reactivex.disposables.CompositeDisposable;
import io.reactivex.disposables.Disposable;
import io.reactivex.schedulers.Schedulers;
import java.io.File;
import java.io.FileOutputStream;
import org.apache.commons.io.IOUtils;
import viska.database.Database;

public class NewProfileActivity extends Activity {

  private final CompositeDisposable subscriptions = new CompositeDisposable();

  @Override
  public void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.new_profile);

    final Button newMockProfileButton = findViewById(R.id.new_mock_profile);
    newMockProfileButton.setOnClickListener(view -> onNewMockProfile());

    final Application app = (Application) getApplication();
    final ProgressBar progressBar = findViewById(R.id.progress);
    final Button newAccountButton = findViewById(R.id.new_account);
    app.getViewModel().creatingAccount.observe(this, running -> {
      if (running) {
        progressBar.setVisibility(View.VISIBLE);
        newAccountButton.setVisibility(View.GONE);
        newMockProfileButton.setVisibility(View.GONE);
      } else {
        progressBar.setVisibility(View.GONE);
        newAccountButton.setVisibility(View.VISIBLE);
        newMockProfileButton.setVisibility(BuildConfig.DEBUG ? View.VISIBLE : View.GONE);
      }
    });
  }

  private void onNewMockProfile() {
    final Application app = (Application) getApplication();
    app.getViewModel().creatingAccount.setValue(true);
    Disposable sub = Completable
        .fromAction(() -> {
          final Database db = app.getDatabase();
          final String dbPath = db.path();
          db.close();

          IOUtils.copy(
              getResources().getAssets().open("demo.realm"),
              new FileOutputStream(new File(dbPath))
          );

          app.getViewModel().creatingAccount.postValue(false);
        })
        .observeOn(Schedulers.io())
        .subscribeOn(Schedulers.from(getMainExecutor()))
        .subscribe(() -> {
          startActivity(new Intent(this, MainActivity.class));
          finish();
        });
    subscriptions.add(sub);
  }
}
