use std::collections::HashMap;
use std::env;
use std::fs;

mod solana;

use crate::solana::parser::parse_transaction;
use crate::solana::structs::{
    IdlSource, SolanaParsedInstructionData, SolanaParsedTransactionPayload,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        print_usage();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "parse" => {
            let flag = &args[2];
            let unsigned_tx = &args[3];

            match flag.as_str() {
                "--message" | "--transaction" => {
                    let is_transaction = flag == "--transaction";

                    // Check for optional custom IDL parameters
                    let custom_idls = parse_custom_idl_args(&args[4..]);

                    let result =
                        parse_transaction(unsigned_tx.to_string(), is_transaction, custom_idls);

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
                    println!("Invalid flag. Use either --message or --transaction.");
                    print_usage();
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run parse --message <unsigned_tx_hex>");
    println!("  cargo run parse --transaction <unsigned_tx_hex>");
    println!();
    println!("Optional custom IDL parameters:");
    println!("  --custom-idl <program_id> <idl_json_file_or_string> [--override]");
    println!();
    println!("Examples:");
    println!("  cargo run parse --message <tx_hex>");
    println!("  cargo run parse --message <tx_hex> --custom-idl <program_id> /path/to/idl.json");
    println!("  cargo run parse --message <tx_hex> --custom-idl <program_id> /path/to/idl.json --override");
}

fn parse_custom_idl_args(args: &[String]) -> Option<HashMap<String, (String, bool)>> {
    if args.is_empty() {
        return None;
    }

    let mut custom_idls = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--custom-idl" {
            if i + 2 >= args.len() {
                eprintln!(
                    "Error: --custom-idl requires <program_id> and <idl_json_file_or_string>"
                );
                return None;
            }

            let program_id = args[i + 1].clone();
            let idl_arg = &args[i + 2];

            // Check if it's a file path or JSON string
            let idl_json = if std::path::Path::new(idl_arg).exists() {
                match fs::read_to_string(idl_arg) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading IDL file {}: {}", idl_arg, e);
                        return None;
                    }
                }
            } else {
                idl_arg.clone()
            };

            // Check for --override flag
            let override_builtin = if i + 3 < args.len() && args[i + 3] == "--override" {
                i += 1; // Skip the --override flag
                true
            } else {
                false
            };

            custom_idls.insert(program_id, (idl_json, override_builtin));
            i += 3;
        } else {
            i += 1;
        }
    }

    if custom_idls.is_empty() {
        None
    } else {
        Some(custom_idls)
    }
}

fn print_parsed_transaction(transaction_payload: SolanaParsedTransactionPayload) {
    println!("Solana Parsed Transaction Payload:");
    println!(
        "  Unsigned Payload: {}",
        transaction_payload.unsigned_payload
    );
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
            print_parsed_instruction_data(instruction.parsed_instruction.clone());
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

fn print_parsed_instruction_data(p_inst_data: Option<SolanaParsedInstructionData>) {
    println!("        Parsed Instruction Data:");
    if let Some(parsed_data) = p_inst_data {
        println!(
            "          Instruction Name: {}",
            parsed_data.instruction_name
        );

        // Display IDL source information
        match &parsed_data.idl_source {
            IdlSource::BuiltIn(program_type) => {
                println!(
                    "          IDL Source: Built-in ({})",
                    program_type.program_name()
                );
            }
            IdlSource::Custom => {
                println!("          IDL Source: Custom");
            }
        }
        println!("          IDL Hash: {}", parsed_data.idl_hash);

        println!("          Named Accounts:");
        for k in parsed_data.named_accounts.keys() {
            let acct_string = parsed_data.named_accounts[k].clone();
            println!("            {}: {}", k, acct_string);
        }
        println!("          Args:");
        for k in parsed_data.program_call_args.keys() {
            let arg_json = parsed_data.program_call_args[k].clone();
            println!("            {}: {:#?}", k, arg_json);
        }
    } else {
        println!("          NO PARSED INSTRUCTION DATA -- NO MATCHING IDL")
    }
}
