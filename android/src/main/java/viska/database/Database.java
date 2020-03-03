package viska.database;

import io.reactivex.Flowable;
import io.realm.Realm;
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

  public Flowable<String> getAccountId() {
    return realm
        .where(Profile.class)
        .equalTo("name", PROFILE_ID)
        .findFirst()
        .<Profile>asFlowable()
        .map(nv -> new String(nv.value, StandardCharsets.UTF_8));
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

  public String path() {
    return realm.getPath();
  }
}
