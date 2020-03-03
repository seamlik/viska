package viska.database;

import io.realm.RealmList;
import io.realm.RealmObject;
import io.realm.annotations.Index;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import java.util.Date;

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
}
