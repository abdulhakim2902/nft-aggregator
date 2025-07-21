-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts (
  id uuid NOT NULL,
  media_url VARCHAR(512),
  name VARCHAR(128),
  owner VARCHAR(66),
  contract_id uuid DEFAULT NULL,
  collection_id uuid DEFAULT NULL,
  -- Add nft properties
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

CREATE FUNCTION update_nft_burn_status ()
    RETURNS TRIGGER
    AS $$
BEGIN
	IF NEW.tx_type = 'burn' THEN
	  INSERT INTO nfts(id, collection_id, contract_id, burned, owner)
      VALUES (NEW.nft_id, NEW.collection_id, NEW.contract_id, true, NULL)
    ON CONFLICT(id)
      DO UPDATE SET
        burned = true,
        owner = NULL;
	END IF;
  
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER burned_changes
    AFTER INSERT ON actions
    FOR EACH ROW
    EXECUTE FUNCTION update_nft_burn_status ();