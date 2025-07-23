-- Your SQL goes here
CREATE TABLE IF NOT EXISTS actions (
  tx_type VARCHAR(30) NOT NULL,
  tx_index BIGINT NOT NULL,
  tx_id VARCHAR(66) NOT NULL,
  sender VARCHAR(66) DEFAULT NULL,
  receiver VARCHAR(66) DEFAULT NULL,
  price BIGINT DEFAULT 0,
  nft_id VARCHAR(66) DEFAULT NULL,
  collection_id VARCHAR(66) DEFAULT NULL,
  market_name VARCHAR(30) DEFAULT NULL,
  market_contract_id VARCHAR(66) DEFAULT NULL,
  usd_price NUMERIC(20, 2) DEFAULT 0,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  block_height BIGINT NOT NULL,
  PRIMARY KEY (tx_index, tx_id)
)
