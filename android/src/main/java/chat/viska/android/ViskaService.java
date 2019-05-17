package chat.viska.android;

import android.app.Service;
import android.content.Intent;
import android.os.Binder;
import android.os.IBinder;
import chat.viska.LibViska;

public class ViskaService extends Service {

  private static boolean libViskaInitialized = false;

  private static void initializeLibViska() {
    if (libViskaInitialized) {
      return;
    }
    LibViska.loadLibrary();
    LibViska.initialize();
    libViskaInitialized = true;
  }

  @Override
  public IBinder onBind(Intent intent) {
    return new Binder() {
      public ViskaService getService() {
        return ViskaService.this;
      }
    };
  }

  @Override
  public void onCreate() {
    super.onCreate();
    initializeLibViska();
  }
}
