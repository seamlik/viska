package viska.database;

import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;

public class Profile extends RealmObject {
  @PrimaryKey @Required public String name;
  public byte[] value;
}
