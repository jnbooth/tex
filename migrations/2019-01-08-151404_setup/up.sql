CREATE TABLE "memo" (
  "channel" CHARACTER VARYING  NOT NULL,
  "user"    CHARACTER VARYING  NOT NULL,
  "message" CHARACTER VARYING  NOT NULL,
  PRIMARY KEY ("channel", "user")
);

CREATE TABLE "reminder" (
  "id"      SERIAL  PRIMARY KEY,
  "user"    CHARACTER VARYING  NOT NULL,
  "when"    TIMESTAMP  NOT NULL,
  "message" CHARACTER VARYING  NOT NULL
);

CREATE TABLE "seen" (
  "channel"     CHARACTER VARYING  NOT NULL,
  "user"        CHARACTER VARYING  NOT NULL,
  "first"       CHARACTER VARYING  NOT NULL,
  "first_time"  TIMESTAMP  NOT NULL,
  "latest"      CHARACTER VARYING  NOT NULL,
  "latest_time" TIMESTAMP  NOT NULL,
  "total"       INT  NOT NULL  DEFAULT 1,
  PRIMARY KEY ("channel", "user")
);

CREATE TABLE "silence" (
  "channel" CHARACTER VARYING  NOT NULL, 
  "command" CHARACTER VARYING  NOT NULL,
  PRIMARY KEY ("channel", "command")
);

CREATE TABLE "tell" (
  "id"      SERIAL  PRIMARY KEY,
  "target"  CHARACTER VARYING  NOT NULL,
  "sender"  CHARACTER VARYING  NOT NULL,
  "time"    TIMESTAMP  NOT NULL,
  "message" CHARACTER VARYING  NOT NULL
);

CREATE TABLE "user" (
  "nick"     CHARACTER VARYING  PRIMARY KEY,
  "auth"     INT  NOT NULL  DEFAULT 0,
  "pronouns" CHARACTER VARYING
);
