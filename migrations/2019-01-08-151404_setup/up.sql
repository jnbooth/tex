CREATE TABLE "page" (
  "url"    CHARACTER VARYING  PRIMARY KEY,
  "name"   CHARACTER VARYING  NOT NULL,
  "author" CHARACTER VARYING  NOT NULL,
  "votes"  INT  NOT NULL  DEFAULT 0
);
CREATE TABLE "reminder" (
  "id"      SERIAL  PRIMARY KEY,
  "nick"    CHARACTER VARYING  NOT NULL,
  "when"    TIMESTAMP  NOT NULL,
  "message" CHARACTER VARYING  NOT NULL
);
CREATE TABLE "seen" (
  "id"          SERIAL  PRIMARY KEY,
  "channel"     CHARACTER VARYING  NOT NULL,
  "nick"        CHARACTER VARYING  NOT NULL,
  "first"       CHARACTER VARYING  NOT NULL,
  "first_time"  TIMESTAMP  NOT NULL,
  "latest"      CHARACTER VARYING  NOT NULL,
  "latest_time" TIMESTAMP  NOT NULL,
  "total"       INT  NOT NULL  DEFAULT 1,
  CONSTRAINT SeenContext UNIQUE ("channel", "nick")
);
CREATE TABLE "silence" (
  "id"      SERIAL  PRIMARY KEY,
  "channel" CHARACTER VARYING  NOT NULL, 
  "command" CHARACTER VARYING  NOT NULL,
  CONSTRAINT SilenceContext UNIQUE ("channel", "command")
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