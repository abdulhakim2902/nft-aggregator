-- Your SQL goes here
CREATE TABLE IF NOT EXISTS bids (
  id uuid NOT NULL, 
  bidder VARCHAR(66) NOT NULL,
  accepted_tx_id VARCHAR(66) DEFAULT NULL,
  canceled_tx_id VARCHAR(66) DEFAULT NULL,
  collection_id uuid DEFAULT NULL,
  contract_id uuid DEFAULT NULL,
  created_tx_id VARCHAR(66) DEFAULT NULL,
  expires_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  market_contract_id uuid DEFAULT NULL,
  nonce VARCHAR(128),
  nft_id uuid DEFAULT NULL,
  price BIGINT,
  price_str VARCHAR(128),
  receiver VARCHAR(66),
  remaining_count BIGINT,
  status VARCHAR(20),
  bid_type VARCHAR(20),
  PRIMARY KEY (id)
);