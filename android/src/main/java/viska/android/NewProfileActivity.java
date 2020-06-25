package viska.android;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import com.couchbase.lite.Database;
import com.uber.autodispose.AutoDispose;
import com.uber.autodispose.android.lifecycle.AndroidLifecycleScopeProvider;
import io.reactivex.Completable;
import io.reactivex.schedulers.Schedulers;
import viska.database.DatabaseKt;

public class NewProfileActivity extends AppCompatActivity {

  @Override
  public void onCreate(@Nullable Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.new_profile);

    final Button newMockProfileButton = findViewById(R.id.new_mock_profile);
    newMockProfileButton.setOnClickListener(view -> onNewMockProfile());

    final Application app = (Application) getApplication();
    final ProgressBar progressBar = findViewById(R.id.progress);
    final Button newAccountButton = findViewById(R.id.new_account);
    app.getViewModel()
        .creatingAccount
        .observe(
            this,
            running -> {
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
    Completable.fromAction(
            () -> {
              final Database db = DatabaseKt.open();
              DatabaseKt.createDemoProfile(db);
              db.close();
              app.getViewModel().creatingAccount.postValue(false);
            })
        .observeOn(Schedulers.io())
        .subscribeOn(Schedulers.from(getMainExecutor()))
        .as(AutoDispose.autoDisposable(AndroidLifecycleScopeProvider.from(this)))
        .subscribe(
            () -> {
              startActivity(new Intent(this, MainActivity.class));
              finish();
            });
  }
}
