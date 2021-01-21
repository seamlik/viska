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
  name              TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chatroom_members (
  id                BLOB PRIMARY KEY NOT NULL, -- UUID

  chatroom_id       BLOB NOT NULL REFERENCES chatroom(chatroom_id) ON DELETE CASCADE,
  member_account_id BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS message (
  message_id      BLOB PRIMARY KEY NOT NULL,

  chatroom_id     BLOB NOT NULL,

  -- changelog data
  attachment      BLOB REFERENCES object(object_id) ON DELETE SET NULL,
  content         TEXT NOT NULL,
  recipients      BLOB NOT NULL, -- Protobuf viska.database.BytesArray of account IDs
  sender          BLOB NOT NULL,
  time            DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS vcard (
  vcard_id   BLOB PRIMARY KEY NOT NULL,

  account_id BLOB NOT NULL,
  name       TEXT NOT NULL,
  photo      BLOB REFERENCES object(object_id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS object (
  object_id BLOB PRIMARY KEY NOT NULL, -- UUID

  content   BLOB NOT NULL,
  mime      TEXT NOT NULL
);