package viska.android;

public class Module {
  private Module() {
  }
  public static void initialize() {
    Rust_initialize();
  }
  private static native void Rust_initialize();
}
