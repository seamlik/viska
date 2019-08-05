package viska.android;

import androidx.lifecycle.MutableLiveData;
import java.nio.file.Files;
import java.nio.file.Path;
import viska.Crate;

public class Application extends android.app.Application {

  public static class ViewModel extends androidx.lifecycle.ViewModel {

    public final MutableLiveData<Boolean> creatingAccount = new MutableLiveData<>();

    public ViewModel() {
      creatingAccount.setValue(false);
    }
  }

  private final ViewModel viewModel = new ViewModel();

  @Override
  public void onCreate() {
    super.onCreate();
    Crate.loadLibrary();
    Module.initialize();
  }

  public Path getProfileDatabasePath() {
    return getNoBackupFilesDir().toPath().resolve("profile").resolve("database");
  }

  public boolean hasProfile() {
    return Files.isDirectory(getProfileDatabasePath());
  }

  public ViewModel getViewModel() {
    return viewModel;
  }
}
