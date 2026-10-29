#![no_main]

use libfuzzer_sys::fuzz_target;
use solana_parser::parse_instruction_with_idl;
use solana_parser_fuzz_core::arbitrary::ArbIdl;

const PROGRAM_ID: &str = "11111111111111111111111111111111";

// Prefix the first instruction's discriminator onto arbitrary arg bytes, so
// every input reaches arg decoding rather than failing discriminator lookup.
fuzz_target!(|input: (ArbIdl, Vec<u8>)| {
    let (ArbIdl(idl), args) = input;
    if let Some(disc) = idl
        .instructions
        .first()
        .and_then(|i| i.discriminator.as_ref())
    {
        let mut data = disc.clone();
        data.extend_from_slice(&args);
        let _ = parse_instruction_with_idl(&data, PROGRAM_ID, &idl);
    }
});
