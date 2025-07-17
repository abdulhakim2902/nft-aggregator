-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts (
  id uuid NOT NULL,
  media_url VARCHAR(512),
  name VARCHAR(128),
  owner VARCHAR(66),
  contract_id uuid DEFAULT NULL,
  collection_id uuid DEFAULT NULL,
  -- Add nft properties
  -- Add contract uuid
  -- Add ranking BIGINT DEFAULT NULL,
  -- Add rarity NUMERIC(78, 12) DEFAULT NULL,
  -- Add staked BOOLEAN DEFAULT false,
  -- Add staked_contract_id uuid DEFAULT NULL,
  -- Add staked_owner VARCHAR(66),
  token_id VARCHAR(128),
  -- Add trading_pool_id uuid DEFAULT NULL,
  -- Add chain state
  -- Add asset name
  burned BOOLEAN DEFAULT false,
  PRIMARY KEY (id)
);