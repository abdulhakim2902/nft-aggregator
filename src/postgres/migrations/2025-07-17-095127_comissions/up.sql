-- Your SQL goes here
CREATE TABLE IF NOT EXISTS commissions (
  id uuid NOT NULL,
  royalty NUMERIC(10, 5),
  contract_id uuid DEFAULT NULL,
  nft_id uuid DEFAULT NULL,
  collection_id uuid DEFAULT NULL,
  -- Add marketname
  -- Add marketfee
  -- Add is_custodial 
  PRIMARY KEY (id)
)