use crate::pb::solana::transactions::v1::CreatedTokens;
use crate::pb::solana::transactions::v1::{Transaction, Transactions};
use crate::pb::solana_token_tracker::types::v1::Output;
use crate::utils::{process_compiled_instruction, TokenParams};
use bs58;
use hex;
use serde::Deserialize;
use substreams::errors::Error;
use substreams::log;
use substreams::store::StoreNew;
use substreams::store::{StoreGet, StoreSet};
use substreams::store::{StoreGetInt64, StoreSetInt64};
use substreams_solana::pb::sf::solana::r#type::v1::{Block, ConfirmedTransaction};

const TARGET_PROGRAM_ID: &str = "5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki";
const DEFAULT_TOTAL_SUPPLY: &str = "1000000000";

/// Stage 1: detect creation events and emit CreatedTokens
#[substreams::handlers::map]
fn map_filter_transactions(blk: Block) -> Result<CreatedTokens, Error> {
    let mut created = CreatedTokens::default();
    for tx in blk.transactions.iter().filter(|tx| {
        tx.meta.as_ref().map_or(false, |m| {
            m.log_messages.iter().any(|l| {
                (l.starts_with("Program log: VPool ") && l.ends_with(" has graduated"))
                    || l.contains("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
            })
        })
    }) {
        if let Some(token) = tx.meta.as_ref().and_then(|m| {
            m.pre_token_balances
                .iter()
                .find(|b| b.mint.ends_with("virt"))
                .map(|b| b.mint.clone())
        }) {
            // record this token creation
            created.tokens.push(token);
        }
    }
    Ok(created)
}

/// Stage 1 Store: persist created tokens across blocks
/// Stage 1 Store: persist created tokens across blocks
#[substreams::handlers::store]
fn store_created_tokens(blk: Block, created: CreatedTokens, store: StoreSetInt64) {
    let order = blk.slot;
    let created_slot_i64 = blk.slot as i64;
    for token in created.tokens {
        store.set(order, token, &created_slot_i64);
    }
}

//// Stage 2: track all transfers/mints for tokens once created
#[substreams::handlers::map]
fn map_solana_token_events(block: Block, created_store: StoreGetInt64) -> Result<Output, Error> {
    let mut output = Output::default();
    let slot = block.slot;
    let timestamp = block.block_time.as_ref().unwrap().timestamp;

    for confirmed_trx in block.transactions_owned() {
        let accounts = confirmed_trx.resolved_accounts_as_strings();
        if let Some(trx) = confirmed_trx.transaction {
            let trx_hash = bs58::encode(&trx.signatures[0]).into_string();
            if let (Some(msg), Some(meta)) = (trx.message, confirmed_trx.meta.as_ref()) {
                for (i, inst) in msg.instructions.iter().enumerate() {
                    // identify token mint from first account of instruction
                    let maybe_token = inst.accounts.get(0).and_then(|idx| accounts.get(*idx as usize));
                    let token = match maybe_token {
                        Some(t) => t,
                        None => continue,
                    };
                    // ensure token was created on or before this block
                    match created_store.get_last(token.as_str()) {
                        Some(created_block_i64) if (created_block_i64 as u64) <= slot => {
                            // proceed
                        }
                        _ => continue,
                    }
                    // process instruction for this token
                    let params = vec![TokenParams {
                        token_contract: token.clone(),
                        token_decimals: 6.0,
                    }];
                    process_compiled_instruction(
                        &mut output,
                        timestamp,
                        &trx_hash,
                        meta,
                        i as u32,
                        inst,
                        &accounts,
                        &params,
                        slot,
                    );
                }
            }
        }
    }
    Ok(output)
}
