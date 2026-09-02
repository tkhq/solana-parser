#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use solana_parser::parse_instruction_with_idl;
use solana_parser_fuzz_core::arbitrary::ArbIdl;

const PROGRAM_ID: &str = "11111111111111111111111111111111";

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(ArbIdl(idl)) = ArbIdl::arbitrary(&mut u) {
        let remaining = u.take_rest();
        if let Some(inst) = idl.instructions.first() {
            if let Some(disc) = &inst.discriminator {
                let mut input = disc.clone();
                input.extend_from_slice(remaining);
                let _ = parse_instruction_with_idl(&input, PROGRAM_ID, &idl);
            }
        }
    }
});
