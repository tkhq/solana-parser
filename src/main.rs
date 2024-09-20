use std::env;

mod solana;

use crate::solana::solana::parse_transaction;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        println!("Usage: `cargo run parse <unsigned transaction>`");
        return;
    }

    let command = &args[1];
        match command.as_str() {
            "parse" => parse(),
            _ => println!("Unknown command: {}", command),
        }
}

fn parse() {
    let result = parse_transaction();
    match result {
        Ok(transaction) => println!("Parsed transaction: {:?}", transaction),
        Err(err) => println!("Error parsing transaction: {}", err),
    }
}
