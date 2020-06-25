package viska.android;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import androidx.lifecycle.MutableLiveData;
import com.couchbase.lite.CouchbaseLite;
import lombok.Getter;

public class Application extends android.app.Application {

  public static class ViewModel extends androidx.lifecycle.ViewModel {

    public final MutableLiveData<Boolean> creatingAccount = new MutableLiveData<>();

    public ViewModel() {
      creatingAccount.setValue(false);
    }
  }

  public static final String NOTIFICATION_CHANNEL_SYSTRAY = "systray";

  @Getter private final ViewModel viewModel = new ViewModel();

  @Override
  public void onCreate() {
    super.onCreate();
    System.loadLibrary("viska_android");
    viska_android.Module.initialize();
    initializeNotifications();
    CouchbaseLite.init(this);
  }

  /** Initializes notifications. */
  public void initializeNotifications() {
    final NotificationManager manager = getSystemService(NotificationManager.class);
    final NotificationChannel channelSystray =
        new NotificationChannel(
            NOTIFICATION_CHANNEL_SYSTRAY,
            getString(R.string.notification_systray_name),
            NotificationManager.IMPORTANCE_NONE);
    channelSystray.setShowBadge(false);
    manager.createNotificationChannel(channelSystray);
  }
}
