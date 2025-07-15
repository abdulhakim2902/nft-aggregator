-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts (
  id uuid NOT NULL,
  media_type VARCHAR(10),
  media_url VARCHAR(512),
  name VARCHAR(128),
  owner VARCHAR(66),
  collection_id uuid DEFAULT NULL,
  -- Add nft properties
  -- Add contract uuid
  -- Add ranking BIGINT DEFAULT NULL,
  -- Add rarity NUMERIC(78, 12) DEFAULT NULL,
  -- Add staked BOOLEAN DEFAULT false,
  -- Add staked_contract_id uuid DEFAULT NULL,
  -- Add staked_owner VARCHAR(66),
  token_id VARCHAR(66),
  -- Add token_id_index BIGINT DEFAULT NULL,
  -- Add trading_pool_id uuid DEFAULT NULL,
  -- How to check the version
  -- version BIGINT DEFAULT 0,
  -- Add chain state
  -- Add asset name
  burned BOOLEAN DEFAULT false,
  PRIMARY KEY (id)
);