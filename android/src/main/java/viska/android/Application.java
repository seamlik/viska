package viska.android;

import android.app.NotificationChannel;
import android.app.NotificationManager;
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

  public static final String NOTIFICATION_CHANNEL_SYSTRAY = "systray";

  private final ViewModel viewModel = new ViewModel();

  @Override
  public void onCreate() {
    super.onCreate();
    Crate.loadLibrary();
    Module.initialize();
    initializeNotifications();
  }

  public Path getProfilePath() {
    return getNoBackupFilesDir().toPath().resolve("profile");
  }

  public boolean hasProfile() {
    return Files.isDirectory(getProfilePath());
  }

  public ViewModel getViewModel() {
    return viewModel;
  }

  /**
   * Initializes notifications.
   */
  public void initializeNotifications() {
    final NotificationManager manager = getSystemService(NotificationManager.class);
    final NotificationChannel channelSystray = new NotificationChannel(
        NOTIFICATION_CHANNEL_SYSTRAY,
        getString(R.string.notification_systray_name),
        NotificationManager.IMPORTANCE_NONE
    );
    channelSystray.setShowBadge(false);
    manager.createNotificationChannel(channelSystray);
  }
}
