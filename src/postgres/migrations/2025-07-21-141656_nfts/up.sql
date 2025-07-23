-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts (
  id VARCHAR(66) NOT NULL,
  name VARCHAR(128),
  owner VARCHAR(66),
  collection_id VARCHAR(66) DEFAULT NULL,
  attributes JSONB DEFAULT NULL,
  media_url VARCHAR DEFAULT NULL,
  image_data VARCHAR DEFAULT NULL,
  avatar_url VARCHAR DEFAULT NULL,
  image_url VARCHAR DEFAULT NULL,
  external_url VARCHAR DEFAULT NULL,
  description VARCHAR DEFAULT NULL,
  background_color VARCHAR DEFAULT NULL,
  animation_url VARCHAR DEFAULT NULL,
  youtube_url VARCHAR DEFAULT NULL,
  burned BOOLEAN DEFAULT false,
  version VARCHAR(10) DEFAULT 'v2',
  PRIMARY KEY (id)
);

CREATE FUNCTION update_nft_burn_status ()
    RETURNS TRIGGER
    AS $$
BEGIN
	IF NEW.tx_type = 'burn' THEN
	  INSERT INTO nfts(id, collection_id, burned, owner)
      VALUES (NEW.nft_id, NEW.collection_id, true, NULL)
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