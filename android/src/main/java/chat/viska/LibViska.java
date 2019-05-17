package chat.viska;

public class LibViska {
  private LibViska() {
  }

  /**
   * Must be the first to invoke after invoking {@link #loadLibrary()}.
   */
  public static native void initialize();

  /**
   * Must be invoked before invoking any other methods.
   */
  public static void loadLibrary() {
    System.loadLibrary("viska");
  }
}
