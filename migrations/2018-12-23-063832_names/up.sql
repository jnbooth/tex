CREATE TABLE "name_male" (
  "name"        CHARACTER VARYING  PRIMARY KEY,
  "frequency"   INT  NOT NULL,
  "probability" DOUBLE PRECISION  NOT NULL
);
CREATE TABLE "name_female" (
  "name"        CHARACTER VARYING  PRIMARY KEY,
  "frequency"   INT  NOT NULL,
  "probability" DOUBLE PRECISION  NOT NULL
);
CREATE TABLE "name_last" (
  "name"        CHARACTER VARYING  PRIMARY KEY,
  "frequency"   INT  NOT NULL,
  "probability" DOUBLE PRECISION  NOT NULL
);
