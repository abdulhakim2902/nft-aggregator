-- Your SQL goes here
CREATE TABLE IF NOT EXISTS actions (
  id uuid DEFAULT gen_random_uuid(),
  tx_type VARCHAR(30) NOT NULL,
  tx_index BIGINT NOT NULL,
  tx_id VARCHAR(66) NOT NULL,
  sender VARCHAR(66) DEFAULT NULL,
  receiver VARCHAR(66) DEFAULT NULL,
  price BIGINT DEFAULT 0,
  nft_id uuid NOT NULL,
  collection_id uuid NOT NULL,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  block_height BIGINT NOT NULL,
  PRIMARY KEY (id)
)

-- CREATE FUNCTION update_receiver_to_mint_action ()
--     RETURNS TRIGGER
--     AS $$
-- BEGIN
--     UPDATE actions
--     SET
--     WHERE actions.type = 'mint'
--       AND actions.
-- END
-- $$
-- LANGUAGE plpgsql;

-- CREATE TRIGGER update_mint_action_after_insert_transfer_action
--     AFTER INSERT ON actions
--     FOR EACH ROW
--     EXECUTE FUNCTION update_receiver_to_mint_action ();