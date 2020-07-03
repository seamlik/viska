package viska.android;

import android.app.Notification;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.os.IBinder;

public class ViskaService extends Service {

  public class Binder extends android.os.Binder {
    public ViskaService getService() {
      return ViskaService.this;
    }
  }

  private riko.Object daemon;

  @Override
  public IBinder onBind(Intent intent) {
    return new Binder();
  }

  @Override
  public void onCreate() {
    super.onCreate();
    final Notification notification =
        new Notification.Builder(this, ApplicationKt.NOTIFICATION_CHANNEL_SYSTRAY)
            .setContentTitle(getString(R.string.notification_systray_title))
            .setContentIntent(
                PendingIntent.getActivity(this, 0, new Intent(this, MainActivity.class), 0))
            .setCategory(Notification.CATEGORY_STATUS)
            .setSmallIcon(R.drawable.icon)
            .build();
    startForeground(R.id.notification_systray, notification);
  }

  @Override
  public void onDestroy() {
    super.onDestroy();
    if (daemon != null) {
      daemon.close();
      daemon = null;
    }
  }
}
