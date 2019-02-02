CREATE TABLE "page" (
  "id"          text  PRIMARY KEY,
  "created_at"  timestamp with time zone  NOT NULL,
  "created_by"  text  NOT NULL,
  "rating"      integer  NOT NULL,
  "title"       text  NOT NULL,
  "updated"     timestamp  NOT NULL
);
CREATE INDEX ON "page" ("created_by");

CREATE TABLE "attribution" (
  "page_id"  text  NOT NULL,
  "user"     text  NOT NULL,
  "kind"     text  NOT NULL,
  PRIMARY KEY ("page_id", "user")
);
CREATE INDEX ON "attribution" ("user");

CREATE TABLE "tag" (
  "page_id"  text  NOT NULL  REFERENCES "page"("id")  ON DELETE CASCADE,
  "name"     text  NOT NULL,
  "updated"  timestamp  NOT NULL,
  PRIMARY KEY ("page_id", "name")
);
CREATE INDEX ON "tag" ("name");

CREATE TABLE "timer" (
  "name"     text  PRIMARY KEY,
  "minutes"  integer  NOT NULL
);

INSERT INTO "timer" ("name", "minutes") VALUES
    ('attributions', 60),
    ('bans', 60),
    ('pages', 15),
    ('staff', 60),
    ('titles', 5);

