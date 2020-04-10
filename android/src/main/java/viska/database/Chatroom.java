package viska.database;

import io.realm.RealmList;
import io.realm.RealmObject;
import io.realm.RealmResults;
import io.realm.Sort;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import java.util.stream.Collectors;
import org.checkerframework.checker.nullness.qual.Nullable;

public class Chatroom extends RealmObject {
  @PrimaryKey @Required public String id;
  @Required public RealmList<String> members;
  public RealmList<Message> messages;
  public String name;

  /** Gets the calculated name to be shown to the user. */
  public String getDisplayName() {
    if (name == null) {
      return members.stream()
          .map(member -> Vcard.getById(getRealm(), member).name)
          .collect(Collectors.joining(", "));
    } else {
      return name;
    }
  }

  @Nullable
  public Message getLatestMessage() {
    return messages.sort("time", Sort.DESCENDING).first();
  }

  public RealmResults<Message> getConversation() {
    return messages.sort("time");
  }
}
