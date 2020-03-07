package viska.database;

import io.realm.RealmList;
import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;

public class Chatroom extends RealmObject {
  @PrimaryKey
  @Required
  public String id;

  @Required
  public RealmList<String> members;

  public RealmList<Message> messages;
  public String name;
}
