package viska.android;

import android.app.Notification;
import android.app.PendingIntent;
import android.app.Service;
import android.content.ComponentName;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.IBinder;
import io.reactivex.Single;
import io.reactivex.disposables.CompositeDisposable;
import io.reactivex.disposables.Disposable;
import io.reactivex.subjects.SingleSubject;
import viska.Client;

public class ViskaService extends Service {

  public static class Connection implements ServiceConnection {

    private SingleSubject<Client> client = SingleSubject.create();
    private final CompositeDisposable bin = new CompositeDisposable();

    @Override
    public void onServiceConnected(ComponentName name, final IBinder service) {
      client.onSuccess(((Binder) service).getService().client);
    }

    @Override
    public void onServiceDisconnected(ComponentName name) {
      client = SingleSubject.create();
      bin.clear();
    }

    public Single<Client> getClient() {
      return client;
    }

    public void putDisposable(final Disposable disposable) {
      bin.add(disposable);
    }
  }

  public class Binder extends android.os.Binder {
    public ViskaService getService() {
      return ViskaService.this;
    }
  }

  private Client client;

  @Override
  public IBinder onBind(Intent intent) {
    return new Binder();
  }

  @Override
  public void onCreate() {
    super.onCreate();
    final Application app = (Application) getApplication();

    final Notification notification = new Notification
        .Builder(this, Application.NOTIFICATION_CHANNEL_SYSTRAY)
        .setContentTitle(getString(R.string.notification_systray_title))
        .setContentIntent(PendingIntent.getActivity(
          this,
          0,
          new Intent(this, MainActivity.class),
          0
        ))
        .setCategory(Notification.CATEGORY_STATUS)
        .setSmallIcon(R.drawable.icon)
        .build();
    startForeground(R.id.notification_systray, notification);

    client = Client._new(app.getProfilePath().toString());
  }

  @Override
  public void onDestroy() {
    super.onDestroy();
    client.close();
  }
}
