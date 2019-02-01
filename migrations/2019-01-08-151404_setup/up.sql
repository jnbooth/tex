CREATE TABLE "memo" (
  "channel"  TEXT  NOT NULL,
  "user"     TEXT  NOT NULL,
  "message"  TEXT  NOT NULL,
  PRIMARY KEY ("channel", "user")
);

CREATE TABLE "reminder" (
  "id"       serial  PRIMARY KEY,
  "user"     text  NOT NULL,
  "when"     timestamp  NOT NULL,
  "message"  text  NOT NULL
);

CREATE TABLE "seen" (
  "channel"      text  NOT NULL,
  "user"         text  NOT NULL,
  "first"        text  NOT NULL,
  "first_time"   timestamp  NOT NULL  DEFAULT current_timestamp,
  "latest"       text  NOT NULL,
  "latest_time"  timestamp  NOT NULL  DEFAULT current_timestamp,
  "total"        int  NOT NULL  DEFAULT 1,
  PRIMARY KEY ("channel", "user")
);

CREATE TABLE "silence" (
  "channel"  text  NOT NULL, 
  "command"  text  NOT NULL,
  PRIMARY KEY ("channel", "command")
);

CREATE TABLE "tell" (
  "id"       serial  PRIMARY KEY, 
  "target"   text  NOT NULL,
  "sender"   text  NOT NULL,
  "time"     timestamp  NOT NULL,
  "message"  text  NOT NULL
);
