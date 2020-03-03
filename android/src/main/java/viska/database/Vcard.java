package viska.database;

import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import java.util.Date;

public class Vcard extends RealmObject {
  @PrimaryKey
  @Required
  public String id;

  public String name;
  public Date time_updated;
  public Blob avatar;
}
