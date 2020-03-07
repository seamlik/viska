package viska.database;

import io.reactivex.Flowable;
import io.realm.Realm;
import io.realm.RealmResults;
import java.nio.charset.StandardCharsets;
import lombok.AllArgsConstructor;
import lombok.NonNull;

@AllArgsConstructor
public class Database implements AutoCloseable {
  private static final String PROFILE_ID = "id";

  @NonNull
  private final Realm realm;

  @Override
  public void close() {
    realm.close();
  }

  public String getAccountId() {
    final Profile raw = realm
        .where(Profile.class)
        .equalTo("name", PROFILE_ID)
        .findFirst();
    if (raw == null) {
      return "";
    } else {
      return new String(raw.value, StandardCharsets.UTF_8);
    }
  }

  public Flowable<Vcard> getVcard(final String id) {
    return realm
        .where(Vcard.class)
        .equalTo("id", id)
        .findFirst()
        .asFlowable();
  }

  public boolean isEmpty() {
    return realm.isEmpty();
  }

  public String getPath() {
    return realm.getPath();
  }

  public RealmResults<Peer> getRoster() {
    return realm.where(Peer.class).greaterThanOrEqualTo("role", 0).findAll();
  }

  public RealmResults<Chatroom> getChatrooms() {
    return realm.where(Chatroom.class).findAll();
  }
}
