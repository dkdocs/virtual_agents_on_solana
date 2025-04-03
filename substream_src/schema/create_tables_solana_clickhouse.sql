CREATE DATABASE IF NOT EXISTS virtual_agents_on_solana;

-- Create tokens table
CREATE TABLE IF NOT EXISTS virtual_agents_on_solana.transfers (
    id String,
    tx_hash String,
    timestamp BigInt,
    from String,
    to String,
    amount Float64,
    token_address String,
    block_number BigInt,
    transfer_index Int64
) ENGINE = ReplacingMergeTree()
ORDER BY (id);


CREATE TABLE IF NOT EXISTS virtual_agents_on_solana.tokens (
    id String,
    symbol String,
    name String,
    created_at BigInt,
    graduated_at BigInt,
    dev_wallet String,
    total_supply String,
    launched_tx_hash String,
    graduated_tx_hash String,
    created_block_number BigInt,
    graduated_block_number BigInt,
    status Int
) ENGINE = ReplacingMergeTree()
ORDER BY (id);

-- Create block tracking table
CREATE TABLE IF NOT EXISTS virtual_agents_on_solana.block_tracking (
    id String,
    current_block_number UInt64
) ENGINE = ReplacingMergeTree()
ORDER BY (id);

-- Insert initial record
INSERT INTO virtual_agents_on_solana.block_tracking (id, current_block_number)
VALUES ('main', 319903923)
ON CONFLICT (id) DO NOTHING;

-- -- Create cursors_token table
-- CREATE TABLE IF NOT EXISTS virtual_agents_on_solana.cursors_token (
--     cursor String,
--     blockNumber String
-- ) ENGINE = ReplacingMergeTree()
-- ORDER BY tuple(); 

-- -- Create cursors_transfer table
-- CREATE TABLE IF NOT EXISTS virtual_agents_on_solana.cursors_transfer (
--     cursor String,
--     blockNumber String
-- ) ENGINE = ReplacingMergeTree()
-- ORDER BY tuple();
