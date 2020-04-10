package viska.database;

import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;

public class Blob extends RealmObject {
  @PrimaryKey @Required public String id;
  public String mime;
  public byte[] content;
}
