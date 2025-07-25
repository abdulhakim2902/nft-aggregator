-- Your SQL goes here
CREATE TABLE IF NOT EXISTS collections (
  id VARCHAR(66) NOT NULL,
  slug VARCHAR(66) UNIQUE,
  supply BIGINT DEFAULT 0,
  title VARCHAR(128),
  twitter VARCHAR,
  verified BOOLEAN DEFAULT false,
  website VARCHAR,
  discord VARCHAR,
  description TEXT,
  cover_url VARCHAR(512),
  PRIMARY KEY (id)
);
