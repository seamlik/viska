package viska.core;

import riko.Marshaler;

public class __Riko_Module {
  private __Riko_Module() {
  }
  public static void initialize() {
    __riko_initialize();
  }
  private static native void __riko_initialize();

  private static native void __riko_new_mock_profile(byte[] arg_1);
  public static void new_mock_profile(final String dst) {
    __riko_new_mock_profile(
      Marshaler.toBytes(dst)
    );
  }
}
