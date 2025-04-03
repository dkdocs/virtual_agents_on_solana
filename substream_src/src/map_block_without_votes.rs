use substreams_solana::pb::sf::solana::r#type::v1::Block;

// Vote111111111111111111111111111111111111111
static VOTE_INSTRUCTION: &'static [u8] = &[7, 97, 72, 29, 53, 116, 116, 187, 124, 77, 118, 36, 235, 211, 189, 179, 216, 53, 94, 115, 209, 16, 67, 252, 13, 163, 83, 128, 0, 0, 0, 0];

#[substreams::handlers::map]
fn map_block_without_votes(mut block: Block) -> Result<Block, substreams::errors::Error> {
    // Pre-calculate the length to avoid reallocations
    let original_len = block.transactions.len();
    
    // Filter transactions more efficiently
    block.transactions.retain(|trx| {
        // Early return for invalid transactions
        let meta = match trx.meta.as_ref() {
            Some(meta) => meta,
            None => return false,
        };
        
        if meta.err.is_some() {
            return false;
        }

        let transaction = match trx.transaction.as_ref() {
            Some(transaction) => transaction,
            None => return false,
        };
        
        let message = match transaction.message.as_ref() {
            Some(message) => message,
            None => return false,
        };

        // Check if the transaction contains a vote instruction
        !message.account_keys.contains(&VOTE_INSTRUCTION.to_vec())
    });

    // Log the number of filtered transactions for debugging
    if original_len != block.transactions.len() {
        substreams::log::info!(
            "Filtered {} vote transactions from block",
            original_len - block.transactions.len()
        );
    }

    Ok(block)
}
