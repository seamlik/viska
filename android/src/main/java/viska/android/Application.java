package viska.android;

import viska.Crate;

public class Application extends android.app.Application {

  @Override
  public void onCreate() {
    super.onCreate();
    Crate.loadLibrary();
    Module.initialize();
  }
}
