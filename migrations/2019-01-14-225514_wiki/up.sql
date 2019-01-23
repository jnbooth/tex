CREATE TABLE "attribution" (
  "page"  CHARACTER VARYING  NOT NULL,
  "user"  CHARACTER VARYING  NOT NULL,
  "kind"  CHARACTER VARYING  NOT NULL,
  PRIMARY KEY ("page", "user")
);
CREATE INDEX ON "attribution" ("user");

CREATE TABLE "page" (
  "fullname" CHARACTER VARYING  PRIMARY KEY,
  "created_at" TIMESTAMP WITH TIME ZONE  NOT NULL,
  "created_by" CHARACTER VARYING  NOT NULL,
  "rating" INTEGER  NOT NULL,
  "title" CHARACTER VARYING  NOT NULL
);
CREATE INDEX ON "page" ("created_by");

CREATE TABLE "tag" (
  "name" CHARACTER VARYING  NOT NULL,
  "page" CHARACTER VARYING  NOT NULL,
  PRIMARY KEY ("name", "page")
);
CREATE INDEX ON "tag" ("name");
