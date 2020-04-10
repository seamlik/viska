package viska.database;

import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import org.checkerframework.checker.nullness.qual.Nullable;

public class Peer extends RealmObject {
  @PrimaryKey @Required public String id;
  public int role = 0;

  @Nullable
  public Vcard getVcard() {
    return getRealm().where(Vcard.class).equalTo("id", id).findFirst();
  }
}
