use crate::constants;
use crate::pb::solana_token_tracker::types::v1::{Burn, InitializedAccount, Mint, Output, Transfer};
use std::ops::Div;
use substreams::errors::Error;

use substreams_solana::pb::sf::solana::r#type::v1::{CompiledInstruction, TokenBalance, TransactionStatusMeta};

use substreams::log;
use substreams_solana_program_instructions::token_instruction_2022::TokenInstruction;

#[derive(Debug)]
pub struct TokenParams {
    pub token_contract: String,
    pub token_decimals: f64,
}

pub fn process_compiled_instruction(
    output: &mut Output,
    timestamp: i64,
    trx_hash: &String,
    meta: &TransactionStatusMeta,
    inst_index: u32,
    inst: &CompiledInstruction,
    accounts: &Vec<String>,
    parameters: &Vec<TokenParams>,
    block_number: u64,
) {
    let instruction_program_account = &accounts[inst.program_id_index as usize];

    if instruction_program_account == constants::TOKEN_PROGRAM {
        if let Err(err) = process_token_instruction(
            trx_hash,
            timestamp,
            &inst.data,
            &inst.accounts,
            meta,
            accounts,
            output,
            parameters,
            inst_index.to_string(),
            block_number,
        ) {
            log::info!(
                "trx_hash {} top level transaction without inner instructions: {}",
                trx_hash,
                err
            );
        }
    }

    process_inner_instructions(
        output,
        inst_index,
        meta,
        accounts,
        trx_hash,
        timestamp,
        parameters,
        block_number,
    );
}

pub fn process_inner_instructions(
    output: &mut Output,
    instruction_index: u32,
    meta: &TransactionStatusMeta,
    accounts: &Vec<String>,
    trx_hash: &String,
    timestamp: i64,
    parameters: &Vec<TokenParams>,
    block_number: u64,
) {
    let mut inner_instruction_index = 1;

    if let Some(inner_inst) = meta
        .inner_instructions
        .iter()
        .find(|inst| inst.index == instruction_index)
    {
        for inner_instruction in inner_inst.instructions.iter() {
            let instruction_program_account = &accounts[inner_instruction.program_id_index as usize];

            if instruction_program_account == constants::TOKEN_PROGRAM {
                inner_instruction_index += 1;
                let joined_instruction_id = format!("{}.{}", instruction_index, inner_instruction_index);

                if let Err(err) = process_token_instruction(
                    trx_hash,
                    timestamp,
                    &inner_instruction.data,
                    &inner_instruction.accounts,
                    meta,
                    accounts,
                    output,
                    parameters,
                    joined_instruction_id,
                    block_number,
                ) {
                    log::info!("trx_hash {} filtering inner instructions: {}", trx_hash, err);
                }
            }
        }
    }
}

fn process_token_instruction(
    trx_hash: &String,
    timestamp: i64,
    data: &Vec<u8>,
    inst_accounts: &Vec<u8>,
    meta: &TransactionStatusMeta,
    accounts: &Vec<String>,
    output: &mut Output,
    parameters: &Vec<TokenParams>,
    instruction_index: String,
    block_number: u64,
) -> Result<(), Error> {
    match TokenInstruction::unpack(data) {
        Err(err) => {
            log::info!("unpacking token instruction {:?}", err);
            return Err(anyhow::anyhow!("unpacking token instruction: {}", err));
        }
        Ok(instruction) => match instruction {
            TokenInstruction::Transfer { amount: amt } | TokenInstruction::TransferChecked { amount: amt, .. } => {
                let source = &accounts[inst_accounts[0] as usize];
                let destination = &accounts[inst_accounts[1] as usize];
                let authority = &accounts[inst_accounts[2] as usize];

                for parameter in parameters.iter() {
                    if is_token_transfer(&meta.pre_token_balances, authority, &parameter.token_contract) {
                        output.transfers.push(Transfer {
                            id: format!("{}-{}", trx_hash, instruction_index),
                            tx_hash: trx_hash.to_owned(),
                            timestamp,
                            from: source.to_owned(),
                            to: destination.to_owned(),
                            amount: amount_to_decimals(amt as f64, parameter.token_decimals),
                            token_address: parameter.token_contract.to_string(),
                            block_number,
                        });
                    }
                }
            }
            TokenInstruction::MintTo { amount: amt } | TokenInstruction::MintToChecked { amount: amt, .. } => {
                let mint = fetch_account_to(accounts, inst_accounts[0]);

                for parameter in parameters.iter() {
                    if mint == parameter.token_contract {
                        let account_to = fetch_account_to(accounts, inst_accounts[1]);
                        output.mints.push(Mint {
                            trx_hash: trx_hash.to_owned(),
                            timestamp,
                            to: account_to,
                            amount: amount_to_decimals(amt as f64, parameter.token_decimals),
                        });
                        return Ok(());
                    }
                }
            }
            _ => {}
        },
    }
    Ok(())
}

fn amount_to_decimals(amount: f64, decimal: f64) -> f64 {
    amount / 10.0_f64.powf(decimal)
}

fn fetch_account_to(account_keys: &Vec<String>, position: u8) -> String {
    account_keys[position as usize].to_owned()
}

fn is_token_transfer(pre_token_balances: &Vec<TokenBalance>, account: &String, contract_address: &String) -> bool {
    pre_token_balances
        .iter()
        .any(|token_balance| token_balance.owner == *account && token_balance.mint == *contract_address)
}

#[cfg(test)]
mod test {
    use crate::utils::amount_to_decimals;

    #[test]
    pub fn test_amount_to_decimals() {
        let amount = 4983184141.0;
        let expected = 4.983184141;

        let actual = amount_to_decimals(amount, 9.0);
        println!("expected {:?} actual {:?}", expected, actual);
        assert_eq!(expected, actual)
    }
}
