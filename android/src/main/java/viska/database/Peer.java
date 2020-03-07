package viska.database;

import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;

public class Peer extends RealmObject {
  @PrimaryKey
  @Required
  public String id;

  public int role = 0;
}
