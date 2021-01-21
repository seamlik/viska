CREATE TABLE IF NOT EXISTS peer (
  account_id BLOB PRIMARY KEY NOT NULL,

  name       TEXT NOT NULL,
  role       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chatroom (
  chatroom_id       BLOB PRIMARY KEY NOT NULL,

  latest_message_id BLOB NOT NULL,
  time_updated      DOUBLE NOT NULL,

  -- changelog data
  name              TEXT NOT NULL,
  members           BLOB NOT NULL -- Protobuf viska.database.BytesArray of account IDs
);

CREATE TABLE IF NOT EXISTS message (
  message_id      BLOB PRIMARY KEY NOT NULL,

  chatroom_id     BLOB NOT NULL,

  -- changelog data
  attachment      BLOB NOT NULL,
  attachment_mime TEXT NOT NULL,
  content         TEXT NOT NULL,
  recipients      BLOB NOT NULL, -- Protobuf viska.database.BytesArray of account IDs
  sender          BLOB NOT NULL,
  time            DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS vcard (
  vcard_id   BLOB PRIMARY KEY NOT NULL,

  account_id BLOB NOT NULL,
  name       TEXT NOT NULL,
  photo      BLOB NOT NULL,
  photo_mime TEXT NOT NULL
);