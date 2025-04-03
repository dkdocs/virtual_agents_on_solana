use hex;
use crate::utils::{ process_compiled_instruction, TokenParams };
use anyhow::anyhow;
use serde::Deserialize;
use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use crate::pb::solana_token_tracker::types::v1::Output;
use substreams::errors::Error;
use substreams_database_change::tables::Tables;
use substreams_database_change::pb::database::{ DatabaseChanges };
use substreams_solana::pb::sf::solana::r#type::v1::{
    Block,
    CompiledInstruction,
    ConfirmedTransaction,
};
use crate::pb::sol::transactions::v1::{ Instruction, Transaction, Transactions };
use substreams::{ log };
use substreams::store::StoreGet;

use substreams::store::{
    self,
    DeltaProto,
    StoreNew,
    StoreSetIfNotExists,
    StoreSetIfNotExistsProto,
    StoreGetProto,
};
use bs58;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct TransactionFilterParams {
    program_id: String,
}

use serde_json::from_str;

#[derive(Debug, Deserialize)]
struct TokenJson {
    token: String,
    blockNumber: String,
    transactionHash: String,
    ownerAddress: String,
    timestamp: String,
    isGraduated: String,
}

const TARGET_PROGRAM_ID: &str = "5U3EU2ubXtK84QcRjWVmYt9RaDyA8gKxdUrPFXmZyaki";

#[substreams::handlers::map]
fn map_filter_transactions(blk: Block) -> Result<Transactions, substreams::errors::Error> {
    // Pre-allocate with a reasonable capacity to avoid reallocations
    let mut transactions = Vec::with_capacity(blk.transactions.len() / 4);
    
    // Process transactions in parallel if possible
    for tx in blk.transactions.iter().filter(|tx| has_target_program_log(tx)) {
        let msg = tx.transaction.as_ref().unwrap().message.as_ref().unwrap();
        let acct_keys = tx.resolved_accounts();
        
        // Extract virtual token more efficiently
        let virtual_token = tx.meta
            .as_ref()
            .and_then(|m| m.pre_token_balances.iter()
                .find(|balance| balance.mint.ends_with("virt"))
                .map(|balance| balance.mint.clone()));

        // Get signer more efficiently
        let signer = msg.account_keys
            .get(0)
            .map(|key| bs58::encode(key).into_string())
            .unwrap_or_else(|| "N/A".to_string());

        // Check if the transaction contains a 'graduated' event
        let is_graduated_log = log_decoded_transaction(tx, &blk);

        if is_graduated_log {
            if let Some(token) = virtual_token.as_ref() {
                log::info!("A {} token has graduated!", token);
            }
            
            let t = Transaction {
                symbol: "UNKNOWN".to_string(),
                name: "UNKNOWN".to_string(),
                created_at: blk.block_time.as_ref().unwrap().timestamp as u64,
                created_block_number: blk.slot,
                dev_wallet: signer.clone(),
                tx_hash: tx.transaction
                    .as_ref()
                    .unwrap()
                    .signatures.first()
                    .map(|sig| bs58::encode(sig).into_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                id: virtual_token.clone().unwrap_or_else(|| "N/A".to_string()),
                status: "1".to_string(),
                total_supply: "1000000000".to_string(),
            };
            transactions.push(t);
        }

        // Check for a specific log message indicating a meta creation event
        let contains_meta_creation_event = tx.meta
            .as_ref()
            .map_or(false, |m| m.log_messages
                .iter()
                .any(|log| log.contains("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")));

        if !contains_meta_creation_event {
            log::info!("No Virtual Token Found");
            continue;
        }

        let t = Transaction {
            symbol: "UNKNOWN".to_string(),
            name: "UNKNOWN".to_string(),
            created_block_number: blk.slot,
            created_at: blk.block_time.as_ref().unwrap().timestamp as u64,
            dev_wallet: signer,
            tx_hash: tx.transaction
                .as_ref()
                .unwrap()
                .signatures.first()
                .map(|sig| bs58::encode(sig).into_string())
                .unwrap_or_else(|| "N/A".to_string()),
            id: virtual_token.unwrap_or_else(|| "N/A".to_string()),
            status: "0".to_string(),
            total_supply: "1000000000".to_string(),
        };
        
        if !is_graduated_log {
            transactions.push(t);
        }
    }
    
    Ok(Transactions { transactions })
}

// MODULE TO GET TRANSFERS OF EACH TOKEN

#[substreams::handlers::map]
fn map_solana_token_events(
    params: String,
    block: Block,
) -> Result<Output, Error> {
    // Pre-allocate output with reasonable capacity
    let mut output = Output::default();
    
    // Parse token list once
    let token_list: Vec<TokenParams> = params.split(',')
        .map(|token| TokenParams {
            token_contract: token.to_string(),
            token_decimals: 6.0,  // Assigning static decimal value
        })
        .collect();
    
    log::info!("TOKEN LIST: {:?}", token_list);

    // Process transactions more efficiently
    for confirmed_trx in block.clone().transactions_owned() {
        let timestamp = block.block_time.as_ref().unwrap().timestamp;
        let accounts = confirmed_trx.resolved_accounts_as_strings();

        if let Some(trx) = confirmed_trx.transaction {
            let trx_hash = bs58::encode(&trx.signatures[0]).into_string();
            
            if let Some(msg) = trx.message {
                if let Some(meta) = confirmed_trx.meta.as_ref() {
                    for (i, compiled_instruction) in msg.instructions.iter().enumerate() {
                        process_compiled_instruction(
                            &mut output,
                            timestamp,
                            &trx_hash,
                            meta,
                            i as u32,
                            compiled_instruction,
                            &accounts,
                            &token_list,
                            block.slot
                        );
                    }
                }
            }
        }
    }

    Ok(output)
}

fn contains_program_id(transaction: &ConfirmedTransaction) -> bool {
    if let Some(tx) = transaction.transaction.as_ref() {
        if let Some(msg) = tx.message.as_ref() {
            let acct_keys = transaction.resolved_accounts();

            for inst in &msg.instructions {
                let program_id = bs58::encode(acct_keys[inst.program_id_index as usize].to_vec())
                    .into_string();
                
                if program_id == TARGET_PROGRAM_ID {
                    return true;
                }
            }
        }
    }
    false
}

fn base_64_to_hex<T: std::convert::AsRef<[u8]>>(num: T) -> String {
    format!("0x{}", hex::encode(num))
}

fn get_token_decimals(_token: &str) -> f64 {
    6.0
}

fn log_decoded_transaction(tx: &ConfirmedTransaction, blk: &Block) -> bool {
    if let Some(txn) = tx.transaction.as_ref() {
        if let Some(msg) = txn.message.as_ref() {
            // Check for graduated log message directly without full decoding
            if let Some(meta) = tx.meta.as_ref() {
                return meta.log_messages
                    .iter()
                    .any(|log| log.starts_with("Program log: VPool ") && log.ends_with(" has graduated"));
            }
        }
    }
    false
}

fn has_target_program_log(tx: &ConfirmedTransaction) -> bool {
    tx.meta
        .as_ref()
        .map_or(false, |meta| {
            meta.log_messages
                .iter()
                .any(|log| log.contains(TARGET_PROGRAM_ID))
        })
}
