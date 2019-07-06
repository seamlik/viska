package viska.mock_profile;

public class Module {
  private Module() {
  }
  private static native void Rust_new_mock_profile(String profile_path);
  public static void new_mock_profile(final String profile_path) {
    Rust_new_mock_profile(profile_path);
  }
}
