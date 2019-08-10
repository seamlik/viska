package viska.core;

import org.checkerframework.checker.nullness.qual.Nullable;
import riko.DoubleFreeException;
import riko.Marshaler;
import riko.Returned;
import riko.UseAfterFreeException;
import riko.UserException;

public class Client implements AutoCloseable {

  private final int handle;
  private boolean freed;

  private Client(final int handle) {
    this.handle = handle;
  }

  public static Client create(final String profile_path) throws UserException {
    final Returned<Integer> result = Marshaler.fromBytes(__riko_create(
        Marshaler.toBytes(profile_path)
    ));
    return new Client(result.unwrap());
  }
  private static native byte[] __riko_create(byte[] arg_1);

  @Override
  public void close() {
    if (freed) {
      throw new DoubleFreeException();
    }

    __riko_drop(handle);
    freed = true;
  }
  private native void __riko_drop(int handle);

  public byte @Nullable [] account_id() {
    if (freed) {
      throw new UseAfterFreeException();
    }

    return __riko_account_id(handle);
  }
  private static native byte[] __riko_account_id(int handle);

  public boolean isFreed() {
    return freed;
  }
}
