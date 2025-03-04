use std::env;

mod solana;

use crate::solana::parser::parse_transaction;
use crate::solana::structs::SolanaParsedTransactionPayload;

fn main() {
    let args: Vec<String> = env::args().collect();

    if !(args.len() == 4 || args.len() == 6) {
        println!("Usage: `cargo run parse <unsigned transaction> --message OR cargo run parse <unsigned transaction> --transaction`. You can optionally include the format flag (base64 or hex) after the flag.");
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "parse" => {
            let unsigned_tx = &args[3];
            let flag = if args.len() > 3 { Some(&args[2]) } else { None };
            println!("Parsing transaction: {}", unsigned_tx);
            println!("Flag: {:?}", flag);
            let format_flag = if args.len() > 4 {
                Some(args[5].as_str())
            } else {
                Some("hex")
            }; // Default to "hex"
            println!("Format Flag: {:?}", format_flag);
            match (flag, format_flag) {
                (Some(flag), Some(format_flag))
                    if (flag == "--message" || flag == "--transaction")
                        && (format_flag == "base64" || format_flag == "hex") =>
                {
                    let is_transaction = flag == "--transaction";
                    let result = parse_transaction(
                        unsigned_tx.to_string(),
                        is_transaction,
                        format_flag.to_string(),
                    );
                    match result {
                        Ok(response) => {
                            print_parsed_transaction(
                                response.solana_parsed_transaction.payload.unwrap(),
                            );
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                _ => {
                    println!("Invalid or missing flag. Use either --message or --transaction followed by format flag (base64 or hex).");
                }
            }
        }
        _ => println!("Unknown command: {}", command),
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
