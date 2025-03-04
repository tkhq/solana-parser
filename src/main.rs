use std::env;

mod solana;

use crate::solana::parser::parse_transaction;
use crate::solana::structs::SolanaParsedTransactionPayload;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("solana-parser")
        .subcommand(
            Command::new("parse")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["message", "transaction"])
                        .required(true),
                )
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("The transaction/message to parse"),
                )
                .arg(
                    Arg::new("encoding")
                        .long("encoding")
                        .value_parser(["base64", "hex"])
                        .default_value("hex"),
                ),
        )
        .get_matches();

    if let Some(parse_matches) = matches.subcommand_matches("parse") {
        let unsigned_tx = parse_matches.get_one::<String>("input").unwrap();
        let is_transaction = parse_matches.get_one::<String>("format").unwrap() == "transaction";
        let encoding = parse_matches.get_one::<String>("encoding").unwrap();

        println!("Parsing transaction: {}", unsigned_tx);
        println!(
            "Format: {}",
            if is_transaction {
                "transaction"
            } else {
                "message"
            }
        );
        println!("Encoding: {}", encoding);

        match parse_transaction(
            unsigned_tx.to_string(),
            is_transaction,
            encoding.to_string(),
        ) {
            Ok(response) => {
                print_parsed_transaction(response.solana_parsed_transaction.payload.unwrap());
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn print_parsed_transaction(transaction_payload: SolanaParsedTransactionPayload) {
    println!("Solana Parsed Transaction Payload:");
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
            println!(
                "        Instruction Data (hex): {}",
                instruction.instruction_data_hex
            );
            println!(
                "        Address Table Lookups: {:?}",
                instruction.address_table_lookups
            );
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
        println!(
            "    Address Table Lookups: {:?}",
            metadata.address_table_lookups
        );
    }
}
