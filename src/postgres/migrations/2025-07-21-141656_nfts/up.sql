-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts (
  id VARCHAR(66) NOT NULL,
  name VARCHAR(128),
  owner VARCHAR(66),
  collection_id VARCHAR(66) DEFAULT NULL,
  properties JSONB DEFAULT NULL,
  image_data VARCHAR DEFAULT NULL,
  avatar_url VARCHAR DEFAULT NULL,
  image_url VARCHAR DEFAULT NULL,
  external_url VARCHAR DEFAULT NULL,
  description VARCHAR DEFAULT NULL,
  background_color VARCHAR DEFAULT NULL,
  animation_url VARCHAR DEFAULT NULL,
  youtube_url VARCHAR DEFAULT NULL,
  burned BOOLEAN DEFAULT false,
  version VARCHAR(10) DEFAULT 'v2',
  created_at timestamp(6) WITH time zone DEFAULT NOW(),
  PRIMARY KEY (id)
);
