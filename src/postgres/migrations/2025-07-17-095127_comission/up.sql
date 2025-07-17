-- Your SQL goes here
CREATE TABLE IF NOT EXISTS commissions (
  id uuid DEFAULT gen_random_uuid(),
  royalty NUMERIC(10, 5),
  contract_id uuid UNIQUE NOT NULL,
  -- Add marketname
  -- Add marketfee
  -- Add is_custodial 
  PRIMARY KEY (id)
)