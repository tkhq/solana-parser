#![no_main]

use libfuzzer_sys::fuzz_target;
use solana_parser::decode_idl_data;

// Feed arbitrary bytes as IDL JSON into the deserializer.
// Goals: no panics, no unbounded allocation, correct error handling.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = decode_idl_data(s);
    }
});
