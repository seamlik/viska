package viska.database;

import android.content.res.Resources;
import androidx.core.content.MimeTypeFilter;
import io.realm.RealmList;
import io.realm.RealmObject;
import io.realm.annotations.Index;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import java.nio.charset.StandardCharsets;
import java.util.Date;
import viska.android.R;

public class Message extends RealmObject {
  @PrimaryKey
  @Required
  public String id;

  @Index
  @Required
  public Date time;

  @Required
  public String sender;

  public Blob content;

  @Required
  public RealmList<String> participants;

  public boolean read = true;

  public String getPreview(final Resources resources) {
    if (content == null) {
      return "";
    } else if (content.mime == null || MimeTypeFilter.matches(content.mime, "text/*")) {
      return getContentAsText();
    } else if (MimeTypeFilter.matches(content.mime, "image/*")) {
      return resources.getString(R.string.placeholder_image);
    } else if (MimeTypeFilter.matches(content.mime, "audio/*")) {
      return resources.getString(R.string.placeholder_image);
    } else if (MimeTypeFilter.matches(content.mime, "video/*")) {
      return resources.getString(R.string.placeholder_video);
    } else {
      return resources.getString(R.string.placeholder_other);
    }
  }

  public String getContentAsText() {
    if (content == null) {
      return "";
    } else {
      return new String(content.content, StandardCharsets.UTF_8);
    }
  }
}
