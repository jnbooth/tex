CREATE TABLE "namegen" (
  "kind"       "char"  NOT NULL,
  "name"       text  NOT NULL,
  "frequency"  int  NOT NULL,
  PRIMARY KEY ("kind", "name")
);
