package viska;

import androidx.annotation.Nullable;
import org.apache.commons.codec.binary.Hex;

public class Utils {
  private Utils() {
  }

  /**
   * See {@code database::DisplayableId} on the Rust side.
   */
  public static String displayId(@Nullable final byte[] value) {
    if (value == null) {
      return "";
    } else {
      return Hex.encodeHexString(value);
    }
  }
}
