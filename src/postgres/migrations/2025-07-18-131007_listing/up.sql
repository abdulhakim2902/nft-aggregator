-- Your SQL goes here
CREATE TABLE IF NOT EXISTS listings (
  block_height BIGINT,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  market_contract_id VARCHAR(66) DEFAULT NULL,
  collection_id VARCHAR(66),
  nft_id VARCHAR(66) NOT NULL,
  listed BOOLEAN DEFAULT NULL,
  market_name VARCHAR(128),
  nonce VARCHAR(128) DEFAULT NULL,
  price BIGINT DEFAULT NULL,
  price_str VARCHAR(128) DEFAULT '',
  seller VARCHAR(66),
  tx_index BIGINT,
  PRIMARY KEY (market_contract_id, nft_id)
)