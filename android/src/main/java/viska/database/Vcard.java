package viska.database;

import io.realm.Realm;
import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.Required;
import java.util.Date;

public class Vcard extends RealmObject {
  @PrimaryKey
  @Required
  public String id;

  @Required
  public String name = "";

  public Date time_updated;
  public Blob avatar;

  public static Vcard getById(final Realm realm, final String id) {
    realm.beginTransaction();

    Vcard result = realm.where(Vcard.class).equalTo("id", id).findFirst();
    if (result == null) {
      result = realm.createObject(Vcard.class, id);
    }

    realm.commitTransaction();
    return result;
  }
}
