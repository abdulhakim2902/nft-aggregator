-- Your SQL goes here
CREATE TABLE IF NOT EXISTS commissions (
  id uuid NOT NULL,
  royalty NUMERIC(10, 5),
  nft_id VARCHAR(664) DEFAULT NULL,
  collection_id VARCHAR(664) DEFAULT NULL,
  -- Add marketname
  -- Add marketfee
  -- Add is_custodial 
  PRIMARY KEY (id)
)