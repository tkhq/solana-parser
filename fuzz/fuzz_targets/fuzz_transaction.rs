#![no_main]

use libfuzzer_sys::fuzz_target;
use solana_parser::parse_transaction;

// The first byte is a mode byte, the rest is the payload:
// - bit 0 selects `full_transaction` (signatures + message vs bare message).
// - bit 1 clear: hex-encode the payload, so it reaches transaction
//   deserialization, instruction dispatch and IDL lookup instead of dying in
//   `parse_transaction`'s input decoding.
// - bit 1 set: pass the payload through as a string, so the input-encoding
//   checks themselves (hex today, other encodings if added) are fuzzed too.
// Goal: no panics on any path.
fuzz_target!(|data: &[u8]| {
    let Some((&mode, payload)) = data.split_first() else {
        return;
    };
    let full_transaction = mode & 1 == 1;
    let input = if mode & 2 == 0 {
        hex::encode(payload)
    } else {
        String::from_utf8_lossy(payload).into_owned()
    };
    let _ = parse_transaction(input, full_transaction, None);
});
