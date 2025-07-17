-- Your SQL goes here
CREATE TABLE IF NOT EXISTS actions (
  id uuid DEFAULT gen_random_uuid(),
  tx_type VARCHAR(30) NOT NULL,
  tx_index BIGINT NOT NULL,
  tx_id VARCHAR(66) NOT NULL,
  sender VARCHAR(66) DEFAULT NULL,
  receiver VARCHAR(66) DEFAULT NULL,
  price BIGINT DEFAULT 0,
  nft_id uuid DEFAULT NULL,
  contract_id uuid DEFAULT NULL,
  collection_id uuid DEFAULT NULL,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  block_height BIGINT NOT NULL,
  PRIMARY KEY (id)
)
