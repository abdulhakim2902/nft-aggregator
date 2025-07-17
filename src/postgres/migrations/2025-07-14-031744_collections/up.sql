-- Your SQL goes here
CREATE TABLE IF NOT EXISTS collections (
  id uuid NOT NULL,
  slug VARCHAR(66) UNIQUE,
  supply BIGINT DEFAULT 0,
  title VARCHAR(128),
  -- Add twitter VARCHAR(66),
  -- Add usd_volume BIGINT DEFAULT 0,
  -- Add verified BOOLEAN DEFAULT false,
  -- Add volume BIGINT DEFAULT 0,
  -- Add website VARCHAR(128),
  -- Add floor BIGINT DEFAULT 0,
  -- Add discord VARCHAR(66),
  description TEXT,
  cover_url VARCHAR(512),
  contract_id uuid DEFAULT NULL,
  PRIMARY KEY (id)
);