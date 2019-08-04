package viska.android;

import java.nio.file.Files;
import java.nio.file.Path;
import viska.Crate;

public class Application extends android.app.Application {

  private final ApplicationViewModel viewModel = new ApplicationViewModel();

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

  public ApplicationViewModel getViewModel() {
    return viewModel;
  }
}
