-- Your SQL goes here
CREATE TABLE IF NOT EXISTS collections (
  id uuid DEFAULT gen_random_uuid(),
  slug VARCHAR(66) UNIQUE,
  supply BIGINT DEFAULT 0,
  title VARCHAR(128),
  twitter VARCHAR(66),
  usd_volume BIGINT DEFAULT 0,
  verified BOOLEAN DEFAULT false,
  volume BIGINT DEFAULT 0,
  website VARCHAR(128),
  floor BIGINT DEFAULT 0,
  discord VARCHAR(66),
  description TEXT,
  cover_url VARCHAR(512),
  PRIMARY KEY (id)
);