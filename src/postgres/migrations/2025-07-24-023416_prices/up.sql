-- Your SQL goes here
CREATE TABLE IF NOT EXISTS prices (
  created_at TIMESTAMP(6) with time zone NOT NULL,
  price NUMERIC(20, 2) NOT NULL,
  PRIMARY KEY (created_at)
)