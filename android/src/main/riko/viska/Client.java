package viska;

import org.checkerframework.checker.nullness.qual.Nullable;
import riko.DoubleFreeException;
import riko.UseAfterFreeException;

public class Client implements AutoCloseable {

  private final long handle;
  private boolean freed;

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
    if (freed) {
      throw new DoubleFreeException();
    }

    Rust_drop(handle);
    freed = true;
  }
  private native void Rust_drop(long handle);

  public byte @Nullable [] account_id() {
    if (freed) {
      throw new UseAfterFreeException();
    }

    return Rust_account_id(handle);
  }
  private static native byte[] Rust_account_id(long handle);

  public boolean isFreed() {
    return freed;
  }
}
