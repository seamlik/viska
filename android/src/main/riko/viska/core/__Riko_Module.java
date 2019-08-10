package viska.core;

public class __Riko_Module {
  private __Riko_Module() {
  }
  public static void initialize() {
    __riko_initialize();
  }
  private static native void __riko_initialize();

  private static native void __riko_new_mock_profile(String profile_path);
  public static void new_mock_profile(final String profile_path) {
    __riko_new_mock_profile(profile_path);
  }
}
