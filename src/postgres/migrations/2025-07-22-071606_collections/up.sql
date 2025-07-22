-- Your SQL goes here
CREATE TABLE IF NOT EXISTS collections (
  id uuid NOT NULL,
  slug VARCHAR(66) UNIQUE,
  supply BIGINT DEFAULT 0,
  title VARCHAR(128),
  -- Add twitter VARCHAR(66),
  -- Add usd_volume BIGINT DEFAULT 0,
  -- Add verified BOOLEAN DEFAULT false,
  -- Add volume BIGINT DEFAULT 0,
  -- Add website VARCHAR(128),
  floor BIGINT DEFAULT NULL,
  -- Add discord VARCHAR(66),
  description TEXT,
  cover_url VARCHAR(512),
  contract_id uuid DEFAULT NULL,
  PRIMARY KEY (id)
);

CREATE FUNCTION update_floor_price ()
    RETURNS TRIGGER
    AS $$
BEGIN
	WITH listings AS (
    SELECT price, collection_id FROM listings
      JOIN nfts ON nfts.id = listings.nft_id
    WHERE listings.nft_id = NEW.nft_id AND listings.listed
    ORDER BY listings.price ASC
    LIMIT 1
  )
  UPDATE collections
    SET floor = listings.price
  FROM listings
  WHERE collections.id = listings.collection_id;
  
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER floor_price_changes_after_insert
    AFTER INSERT ON listings
    FOR EACH ROW
    EXECUTE FUNCTION update_floor_price ();

CREATE TRIGGER floor_price_changes_after_update
    AFTER UPDATE ON listings
    FOR EACH ROW
    EXECUTE FUNCTION update_floor_price ();