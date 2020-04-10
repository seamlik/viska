package viska.database;

import io.realm.Realm;
import io.realm.RealmObject;
import io.realm.annotations.PrimaryKey;
import io.realm.annotations.RealmClass;
import io.realm.annotations.RealmNamingPolicy;
import io.realm.annotations.Required;
import java.util.Date;

@RealmClass(fieldNamingPolicy = RealmNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
public class Vcard extends RealmObject {
  @PrimaryKey @Required public String id;
  @Required public String name = "";
  public Date timeUpdated;
  public Blob avatar;

  /**
   * Gets a {@link Vcard} by an account ID or an empty one with default content if it is not
   * downloaded yet.
   */
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
