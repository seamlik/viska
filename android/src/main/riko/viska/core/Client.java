package viska.core;

import riko.Heaped;
import riko.Marshaler;
import riko.Returned;

public class Client extends Heaped {

  private Client(final int handle) {
    super(handle);
  }

  public static Client create(final String profile_path) {
    final Returned<Integer> result = Marshaler.fromBytes(__riko_create(
        Marshaler.toBytes(profile_path)
    ));
    return new Client(result.unwrap());
  }
  private static native byte[] __riko_create(byte[] arg_1);

  @Override
  protected void drop() {
    __riko_drop(handle);
  }
  private native void __riko_drop(int handle);

  public byte[] account_id() {
    assertAlive();
    final Returned<byte[]> result = Marshaler.fromBytes(__riko_account_id(handle));
    return result.unwrap();
  }
  private static native byte[] __riko_account_id(int handle);
}
