package viska;

public class Client implements AutoCloseable {

  private final long handle;

  private Client(final long handle) {
    this.handle = handle;
  }

  public static Client _new(final String profile_path) {
    final long handle = Rust_new(profile_path);
    return new Client(handle);
  }
  private static native int Rust_new(String profile_path);

  @Override
  public void close() {
    Rust_drop(handle);
  }
  private native void Rust_drop(long handle);
}
