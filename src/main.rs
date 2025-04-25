use std::{env, fs};

mod solana;

use solana::structs::Idl;

use crate::solana::parser::parse_transaction;
use crate::solana::idl_parser::decode_idl_data;
use crate::solana::structs::SolanaParsedTransactionPayload;
use crate::solana::idl_db::IDL_DB;
use crate::solana::parser::IDL_DIRECTORY;


fn main() {
    let args: Vec<String> = env::args().collect();
    
    // TEMPORARILY DISABLE CHECK
    // if args.len() != 4 {
    //     println!("Usage: `cargo run parse <unsigned transaction> --message OR cargo run parse <unsinged transaction> --transaction`");
    //     return;
    // }

    let command = &args[1];
        match command.as_str() {
            "parse" => {
                let unsigned_tx = &args[3];
                let flag = if args.len() > 3 { Some(&args[2]) } else { None };
                match flag {
                    Some(flag) if flag == "--message" || flag == "--transaction" => {
                        let is_transaction = flag == "--transaction";
                        let result = parse_transaction(unsigned_tx.to_string(), is_transaction);
                        match result {
                            Ok(response) => {
                                print_parsed_transaction(response.solana_parsed_transaction.payload.unwrap());
                            },
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    _ => {
                        println!("Invalid or missing flag. Use either --message or --transaction.");
                    }
                }
            }
            "parse-all-idls" => {
                let mut idls: Vec<Idl> = vec![];
                for entry in IDL_DB {
                    let (name, program_id, file_name) = entry;
                    let idl_file_name = IDL_DIRECTORY.to_string() + file_name;
                    println!("PARSING IDL: {}", idl_file_name.clone());
                    let idl_json = fs::read_to_string(idl_file_name).unwrap();
                    idls.push(decode_idl_data(&idl_json, program_id, name).unwrap())
                }
                println!("{:#?}", idls)
            }
            _ => println!("Unknown command: {}", command),
        }
}

fn print_parsed_transaction(transaction_payload: SolanaParsedTransactionPayload) {
    println!("Solana Parsed Transaction Payload:");
    println!("  Unsigned Payload: {}", transaction_payload.unsigned_payload);
    if let Some(metadata) = transaction_payload.transaction_metadata {
        println!("  Transaction Metadata:");
        println!("    Signatures: {:?}", metadata.signatures);
        println!("    Account Keys: {:?}", metadata.account_keys);
        println!("    Program Keys: {:?}", metadata.program_keys);
        println!("    Recent Blockhash: {}", metadata.recent_blockhash);
        println!("    Instructions:");
        for (i, instruction) in metadata.instructions.iter().enumerate() {
            println!("      Instruction {}:", i + 1);
            println!("        Program Key: {}", instruction.program_key);
            println!("        Accounts: {:?}", instruction.accounts);
            println!("        Instruction Data (hex): {}", instruction.instruction_data_hex);
            println!("        Address Table Lookups: {:?}", instruction.address_table_lookups);
        }
        println!("    Transfers:");
        for (i, transfer) in metadata.transfers.iter().enumerate() {
            println!("      Transfer {}:", i + 1);
            println!("        From: {}", transfer.from);
            println!("        To: {}", transfer.to);
            println!("        Amount: {}", transfer.amount);
        }
        println!("    SPL Transfers:");
        for (i, spl_transfer) in metadata.spl_transfers.iter().enumerate() {
            println!("      SPL Transfer {}:", i + 1);
            println!("        From: {}", spl_transfer.from);
            println!("        To: {}", spl_transfer.to);
            println!("        Owner: {}", spl_transfer.owner);
            for (j, signer) in spl_transfer.signers.iter().enumerate() {
                println!("        Signer {}: {}", j + 1, signer);
            }
            println!("        Amount: {}", spl_transfer.amount);
            if let Some(token_mint) = spl_transfer.token_mint.clone() {
                println!("        Mint: {}", token_mint);
            }
            if let Some(decimals) = spl_transfer.decimals.clone() {
                println!("        Decimals: {}", decimals);
            }
            if let Some(fee) = spl_transfer.fee.clone() {
                println!("        Fee: {}", fee);
            }
        }
        println!("    Address Table Lookups: {:?}", metadata.address_table_lookups);
    }
}
