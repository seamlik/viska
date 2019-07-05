package viska.android;

import android.app.Service;
import android.content.Intent;
import android.os.Binder;
import android.os.IBinder;

public class ViskaService extends Service {

  @Override
  public IBinder onBind(Intent intent) {
    return new Binder() {
      public ViskaService getService() {
        return ViskaService.this;
      }
    };
  }
}
