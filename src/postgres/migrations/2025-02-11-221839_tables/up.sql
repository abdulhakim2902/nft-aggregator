-- Processor status table
CREATE TABLE processor_status (
  processor VARCHAR(100) PRIMARY KEY NOT NULL,
  last_success_version BIGINT NOT NULL,
  last_updated TIMESTAMP NOT NULL DEFAULT NOW(),
  last_transaction_timestamp TIMESTAMP
);