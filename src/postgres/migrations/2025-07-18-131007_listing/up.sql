-- Your SQL goes here
CREATE TABLE IF NOT EXISTS listings (
  id uuid NOT NULL, 
  block_height BIGINT,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  commission_id uuid DEFAULT NULL,
  contract_id uuid DEFAULT NULL,
  nft_id uuid NOT NULL,
  listed BOOLEAN DEFAULT NULL,
  market_name VARCHAR(128),
  nonce VARCHAR(128) DEFAULT NULL,
  price BIGINT DEFAULT 0,
  price_str VARCHAR(128) DEFAULT '0',
  seller VARCHAR(66),
  tx_index BIGINT,
  PRIMARY KEY (id)
)