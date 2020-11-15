CREATE TABLE IF NOT EXISTS peer (
  account_id BLOB PRIMARY KEY,

  name       TEXT,
  role       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chatroom (
  chatroom_id       BLOB PRIMARY KEY,

  latest_message_id BLOB,
  time_updated      REAL NOT NULL,

  -- changelog data
  name              TEXT,
  members           BLOB NOT NULL -- Protobuf viska.database.BytesArray of account IDs
);

CREATE TABLE IF NOT EXISTS message (
  message_id      BLOB PRIMARY KEY,

  chatroom_id     BLOB,

  -- changelog data
  attachment      BLOB
  attachment_mime TEXT
  content         TEXT,
  recipients      BLOB NOT NULL, -- Protobuf viska.database.BytesArray of account IDs
  sender          BLOB,
  time            REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS vcard (
  vcard_id   BLOB PRIMARY KEY,

  account_id BLOB,
  name       TEXT,
  photo      BLOB,
  photo_mime TEXT
);