//! Property-based fuzz tests for IDL instruction parsing.
//!
//! These tests verify that `decode_idl_data` and `parse_instruction_with_idl`
//! (from `solana_parser`) never panic regardless of:
//!
//! - IDL shape: varying instruction counts, argument counts, and argument types
//! - Instruction data bytes: fully random, correct-discriminator prefix, empty, overlong
//! - Defined types (structs) referenced from instruction args
//! - Nested container types: `Vec<Option<T>>`, `Option<Vec<T>>`
//! - SizeGuard boundary: large Vec/String length prefixes with little backing data
//!
//! Run: `cargo test --test fuzz_idl_parsing`
//! More iterations: `PROPTEST_CASES=5000 cargo test --test fuzz_idl_parsing`

use proptest::prelude::*;
use solana_parser::solana::structs::{
    Defined, Idl, IdlField, IdlType, IdlTypeDefinition, IdlTypeDefinitionType,
};
use solana_parser::{decode_idl_data, parse_instruction_with_idl};
use solana_parser_fuzz_core::proptest as arb;
use std::sync::Arc;

const TEST_PROGRAM_ID: &str = "11111111111111111111111111111111";

// ── Local strategies ─────────────────────────────────────────────────────────
//
// Core strategies (`arb_identifier`, `arb_primitive_idl_type`, `arb_idl_type`,
// `arb_idl_field`, `arb_idl_instruction`, `arb_idl`, `arb_idl_json`,
// `arb_bytes_for_type`, `arb_valid_instruction_bytes`) live in
// `solana_parser_fuzz_core::proptest` so they're reusable from both
// `cargo test --features proptest` and the libfuzzer harnesses under `fuzz/`.

/// IDL JSON with a defined struct type correlated between `types` and instruction args.
///
/// Exercises the `Defined` type resolution path through `types`.
fn arb_defined_struct_idl_json() -> impl Strategy<Value = String> {
    (
        arb::arb_identifier(),
        prop::collection::vec(
            (arb::arb_identifier(), arb::arb_primitive_idl_type())
                .prop_map(|(n, t)| IdlField { name: n, r#type: t }),
            1..=8,
        ),
        arb::arb_idl_instruction(),
        prop::collection::vec(arb::arb_idl_instruction(), 0..=4),
    )
        .prop_map(|(struct_name, fields, mut main_inst, mut extra_insts)| {
            main_inst.args = vec![IdlField {
                name: "data".to_string(),
                r#type: IdlType::Defined(Defined::String(struct_name.clone())),
            }];
            extra_insts.push(main_inst);
            let idl = Idl {
                instructions: extra_insts,
                types: vec![IdlTypeDefinition {
                    name: struct_name,
                    r#type: IdlTypeDefinitionType::Struct { fields },
                }],
            };
            serde_json::to_string(&idl).unwrap()
        })
}

/// IDL JSON where the single instruction has a `Vec` arg.
///
/// Used to stress-test the SizeGuard, which guards against large length-prefix
/// attacks (e.g. claiming a Vec of 10,000,000 u8 when the cursor has 4 bytes).
fn arb_vec_arg_idl_json() -> impl Strategy<Value = String> {
    arb::arb_idl_instruction().prop_flat_map(|base_inst| {
        arb::arb_idl_type().prop_map(move |elem_type| {
            let mut inst = base_inst.clone();
            inst.args = vec![IdlField {
                name: "data".to_string(),
                r#type: IdlType::Vec(Box::new(elem_type)),
            }];
            let idl = Idl {
                instructions: vec![inst],
                types: vec![],
            };
            serde_json::to_string(&idl).unwrap()
        })
    })
}

/// Strategy that produces `(idl, instruction_index, valid_borsh_bytes)`.
///
/// The bytes are always correctly encoded for the selected instruction's arg
/// layout — so `parse_instruction_with_idl` is expected to return `Ok`.
fn arb_idl_and_valid_bytes() -> impl Strategy<Value = (Idl, usize, Vec<u8>)> {
    arb::arb_idl().prop_flat_map(|idl| {
        let n = idl.instructions.len();
        let types = Arc::new(idl.types.clone());
        let instructions = idl.instructions.clone();
        let idl_owned = idl.clone();
        (0..n).prop_flat_map(move |inst_idx| {
            let byte_strat =
                arb::arb_valid_instruction_bytes(&instructions[inst_idx], types.clone());
            let idl_c = idl_owned.clone();
            byte_strat.prop_map(move |bytes| (idl_c.clone(), inst_idx, bytes))
        })
    })
}

// ── Crash-safety property tests ──────────────────────────────────────────────

proptest! {
    // Default is 256 cases. Override with PROPTEST_CASES=5000 for deeper fuzzing.
    #![proptest_config(ProptestConfig::default())]

    /// Core crash-safety test: a random IDL paired with instruction data that is
    /// either (a) fully random bytes or (b) a valid discriminator prefix followed
    /// by random arg bytes — 50/50 split.
    ///
    /// Using a valid discriminator for half of all inputs ensures the argument-
    /// decoding code paths are covered, not just the discriminator-matching paths.
    ///
    /// On the valid-discriminator branch: if parsing returns `Ok`, the instruction
    /// name must be non-empty — confirming that the parse code path was taken, not
    /// just that an `Err` was returned silently.
    #[test]
    fn fuzz_idl_parsing_never_panics(
        idl_json in arb::arb_idl_json(),
        use_valid_disc in any::<bool>(),
        inst_idx in any::<usize>(),
        data in prop::collection::vec(any::<u8>(), 0..200usize),
    ) {
        if let Ok(idl) = decode_idl_data(&idl_json) {
            if use_valid_disc && !idl.instructions.is_empty() {
                let inst = &idl.instructions[inst_idx % idl.instructions.len()];
                if let Some(disc) = &inst.discriminator {
                    let mut d = disc.clone();
                    d.extend_from_slice(&data);
                    if let Ok(result) = parse_instruction_with_idl(&d, TEST_PROGRAM_ID, &idl) {
                        prop_assert!(!result.instruction_name.is_empty(),
                            "Ok result must have a non-empty instruction name");
                    }
                    // Err is also acceptable — random arg bytes may be too short or malformed
                }
            } else {
                // Random bytes: only crash-safety matters, not the Ok/Err outcome
                let _ = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
            }
        }
    }

    /// `decode_idl_data` must not panic on completely arbitrary string input.
    #[test]
    fn fuzz_decode_idl_data_arbitrary_input(s in any::<String>()) {
        let _ = decode_idl_data(&s);
    }

    /// Take a valid 8-byte discriminator from a randomly-selected instruction
    /// (not always the first) and append random arg bytes up to MAX_CURSOR_LENGTH
    /// (1232).  The parser must return `Ok` or a clean `Err` — never a panic.
    ///
    /// On `Ok`: the instruction name must match the selected instruction, confirming
    /// that discriminator dispatch routed to the correct handler.
    #[test]
    fn fuzz_valid_discriminator_random_args(
        idl_json in arb::arb_idl_json(),
        inst_idx in any::<usize>(),
        arg_bytes in prop::collection::vec(any::<u8>(), 0..1300usize),
    ) {
        if let Ok(idl) = decode_idl_data(&idl_json) {
            if !idl.instructions.is_empty() {
                let inst = &idl.instructions[inst_idx % idl.instructions.len()];
                let expected_name = inst.name.clone();
                if let Some(disc) = &inst.discriminator {
                    let mut data = disc.clone();
                    data.extend_from_slice(&arg_bytes);
                    if let Ok(result) = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl) {
                        prop_assert_eq!(&result.instruction_name, &expected_name,
                            "discriminator must dispatch to the correct instruction");
                    }
                    // Err is acceptable — random arg bytes may be too short or malformed
                }
            }
        }
    }

    /// IDLs with defined struct types must not panic regardless of instruction bytes.
    /// Uses the same 50/50 valid-discriminator mix as the core test.
    ///
    /// On the valid-discriminator branch: if parsing returns `Ok`, the instruction
    /// name must match the selected instruction, confirming that defined-type
    /// resolution was attempted (not short-circuited before dispatch).
    #[test]
    fn fuzz_defined_struct_types_never_panics(
        idl_json in arb_defined_struct_idl_json(),
        use_valid_disc in any::<bool>(),
        inst_idx in any::<usize>(),
        data in prop::collection::vec(any::<u8>(), 0..200usize),
    ) {
        if let Ok(idl) = decode_idl_data(&idl_json) {
            if use_valid_disc && !idl.instructions.is_empty() {
                let inst = &idl.instructions[inst_idx % idl.instructions.len()];
                let expected_name = inst.name.clone();
                if let Some(disc) = &inst.discriminator {
                    let mut d = disc.clone();
                    d.extend_from_slice(&data);
                    if let Ok(result) = parse_instruction_with_idl(&d, TEST_PROGRAM_ID, &idl) {
                        prop_assert_eq!(&result.instruction_name, &expected_name,
                            "defined-type instruction must dispatch to the correct handler");
                    }
                    // Err is acceptable — random arg bytes may not satisfy struct field layout
                }
            } else {
                let _ = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
            }
        }
    }

    /// Valid input must always parse successfully.
    ///
    /// Unlike the other crash-safety tests, this one asserts `result.is_ok()` —
    /// not merely "didn't panic". The bytes are generated by `arb_idl_and_valid_bytes`,
    /// which produces a correctly borsh-encoded payload for every instruction layout.
    ///
    /// A failure here indicates a genuine parser bug: the parser rejected data
    /// that it should have accepted according to its own IDL contract.
    ///
    /// On `Ok`: instruction name must match the selected instruction, confirming
    /// discriminator dispatch and arg decoding both succeeded.
    #[test]
    fn fuzz_valid_data_always_parses_ok(
        (idl, inst_idx, bytes) in arb_idl_and_valid_bytes(),
    ) {
        if idl.instructions.is_empty() || bytes.is_empty() { return Ok(()); }
        let expected_name = idl.instructions[inst_idx].name.clone();
        let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, &idl);
        prop_assert!(result.is_ok(),
            "parser rejected correctly-encoded input for instruction '{expected_name}': {:?}", result);
        prop_assert_eq!(&result.unwrap().instruction_name, &expected_name);
    }

    /// SizeGuard stress: a Vec arg instruction with a valid discriminator followed
    /// by an arbitrary u32 length prefix and a short trailing payload.
    ///
    /// The SizeGuard must prevent the parser from allocating memory proportional
    /// to the claimed length when the cursor contains far fewer bytes
    /// (budget = MAX_CURSOR_LENGTH × MAX_ALLOC_PER_CURSOR_LENGTH = 1232 × 24 = 29 568 bytes).
    #[test]
    fn fuzz_size_guard_vec_length_prefix(
        idl_json in arb_vec_arg_idl_json(),
        length_prefix in any::<u32>(),
        trailing in prop::collection::vec(any::<u8>(), 0..=8usize),
    ) {
        if let Ok(idl) = decode_idl_data(&idl_json) {
            if !idl.instructions.is_empty() {
                // There is exactly one instruction in arb_vec_arg_idl_json
                let inst = &idl.instructions[0];
                if let Some(disc) = &inst.discriminator {
                    let mut data = disc.clone();
                    data.extend_from_slice(&length_prefix.to_le_bytes());
                    data.extend_from_slice(&trailing);
                    let _ = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
                }
            }
        }
    }
}

// ── Valid-data roundtrip tests ────────────────────────────────────────────────
//
// These tests construct an IDL, extract the computed discriminator, then build
// correctly-serialized instruction data and assert that parsing succeeds.

#[test]
fn roundtrip_no_args() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "initialize", "accounts": [], "args": []}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let result = parse_instruction_with_idl(disc, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.instruction_name, "initialize");
    assert!(result.program_call_args.is_empty());
}

#[test]
fn roundtrip_single_u64_arg() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "deposit", "accounts": [], "args": [
            {"name": "amount", "type": "u64"}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    data.extend_from_slice(&42u64.to_le_bytes());

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.instruction_name, "deposit");
    assert_eq!(result.program_call_args["amount"], serde_json::json!(42));
}

#[test]
fn roundtrip_mixed_primitive_args() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "swap", "accounts": [], "args": [
            {"name": "amountIn",  "type": "u64"},
            {"name": "minOut",    "type": "u64"},
            {"name": "slippage",  "type": "u16"},
            {"name": "isExact",   "type": "bool"},
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    data.extend_from_slice(&1000u64.to_le_bytes()); // amountIn
    data.extend_from_slice(&900u64.to_le_bytes()); // minOut
    data.extend_from_slice(&50u16.to_le_bytes()); // slippage
    data.push(1u8); // isExact = true

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.instruction_name, "swap");
    assert_eq!(
        result.program_call_args["amountIn"],
        serde_json::json!(1000)
    );
    assert_eq!(result.program_call_args["minOut"], serde_json::json!(900));
    assert_eq!(result.program_call_args["slippage"], serde_json::json!(50));
    assert_eq!(result.program_call_args["isExact"], serde_json::json!(true));
}

#[test]
fn roundtrip_option_some() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "setFee", "accounts": [], "args": [
            {"name": "feeBps", "type": {"option": "u16"}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    data.push(1u8); // Some
    data.extend_from_slice(&300u16.to_le_bytes());

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.program_call_args["feeBps"], serde_json::json!(300));
}

#[test]
fn roundtrip_option_none() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "setFee", "accounts": [], "args": [
            {"name": "feeBps", "type": {"option": "u16"}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    data.push(0u8); // None

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.program_call_args["feeBps"], serde_json::Value::Null);
}

#[test]
fn roundtrip_vec_u8() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "writeData", "accounts": [], "args": [
            {"name": "payload", "type": {"vec": "u8"}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let elements: [u8; 3] = [10, 20, 30];
    let mut data = disc.clone();
    data.extend_from_slice(&(elements.len() as u32).to_le_bytes()); // u32 length prefix
    data.extend_from_slice(&elements);

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(
        result.program_call_args["payload"],
        serde_json::json!([10, 20, 30])
    );
}

#[test]
fn roundtrip_array_u32() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "setParams", "accounts": [], "args": [
            {"name": "limits", "type": {"array": ["u32", 3]}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    for val in [100u32, 200, 300] {
        data.extend_from_slice(&val.to_le_bytes());
    }

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(
        result.program_call_args["limits"],
        serde_json::json!([100, 200, 300])
    );
}

#[test]
fn roundtrip_multiple_instructions_distinct_dispatch() {
    // IDL with 3 instructions; verify each is dispatched by its own discriminator.
    let idl_json = serde_json::json!({
        "instructions": [
            {"name": "initialize", "accounts": [], "args": []},
            {"name": "deposit",    "accounts": [], "args": [{"name": "amount", "type": "u32"}]},
            {"name": "withdraw",   "accounts": [], "args": [
                {"name": "amount", "type": "u32"},
                {"name": "all",    "type": "bool"},
            ]},
        ],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();

    // initialize — no args
    let disc0 = idl.instructions[0].discriminator.as_ref().unwrap();
    let r = parse_instruction_with_idl(disc0, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(r.instruction_name, "initialize");
    assert!(r.program_call_args.is_empty());

    // deposit — single u32
    let disc1 = idl.instructions[1].discriminator.as_ref().unwrap();
    let mut data1 = disc1.clone();
    data1.extend_from_slice(&99u32.to_le_bytes());
    let r = parse_instruction_with_idl(&data1, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(r.instruction_name, "deposit");
    assert_eq!(r.program_call_args["amount"], serde_json::json!(99));

    // withdraw — u32 + bool
    let disc2 = idl.instructions[2].discriminator.as_ref().unwrap();
    let mut data2 = disc2.clone();
    data2.extend_from_slice(&50u32.to_le_bytes());
    data2.push(0u8); // all = false
    let r = parse_instruction_with_idl(&data2, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(r.instruction_name, "withdraw");
    assert_eq!(r.program_call_args["amount"], serde_json::json!(50));
    assert_eq!(r.program_call_args["all"], serde_json::json!(false));
}

// ── Defined type (struct) roundtrip tests ────────────────────────────────────

/// An instruction whose single arg is a defined struct with primitive fields
/// is decoded correctly end-to-end.
#[test]
fn roundtrip_defined_struct_arg() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "createOrder", "accounts": [], "args": [
            {"name": "params", "type": {"defined": "OrderParams"}}
        ]}],
        "types": [{
            "name": "OrderParams",
            "type": {"kind": "struct", "fields": [
                {"name": "price",    "type": "u64"},
                {"name": "quantity", "type": "u32"},
                {"name": "side",     "type": "bool"},
            ]}
        }]
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    let mut data = disc.clone();
    data.extend_from_slice(&5000u64.to_le_bytes()); // price
    data.extend_from_slice(&10u32.to_le_bytes()); // quantity
    data.push(1u8); // side = buy

    // Must parse and return Ok with the struct contents.
    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl).unwrap();
    assert_eq!(result.instruction_name, "createOrder");
    // Struct fields are nested under the "params" key.
    let params = &result.program_call_args["params"];
    assert_eq!(params["price"], serde_json::json!(5000));
    assert_eq!(params["quantity"], serde_json::json!(10));
    assert_eq!(params["side"], serde_json::json!(true));
}

// ── SizeGuard boundary tests ──────────────────────────────────────────────────

/// A Vec<u8> arg with a length prefix that vastly exceeds the backing data
/// must be rejected cleanly (Err), not panic or over-allocate.
///
/// SizeGuard budget = MAX_CURSOR_LENGTH (1232) × MAX_ALLOC_PER_CURSOR_LENGTH (24) = 29 568 bytes.
#[test]
fn size_guard_huge_vec_length_prefix_is_rejected_cleanly() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "writeData", "accounts": [], "args": [
            {"name": "payload", "type": {"vec": "u8"}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    // Claim 10 000 000 elements but provide zero backing bytes.
    let mut data = disc.clone();
    data.extend_from_slice(&10_000_000u32.to_le_bytes());

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
    // Must be Err, not a panic or OOM.
    assert!(
        result.is_err(),
        "expected Err for over-budget Vec length, got Ok"
    );
}

/// Same as above but with a Vec<u64> (8 bytes/element) — smaller element count
/// is still enough to exceed the budget relative to cursor length.
#[test]
fn size_guard_vec_u64_over_budget() {
    let idl_json = serde_json::json!({
        "instructions": [{"name": "setRates", "accounts": [], "args": [
            {"name": "rates", "type": {"vec": "u64"}}
        ]}],
        "types": []
    })
    .to_string();

    let idl = decode_idl_data(&idl_json).unwrap();
    let disc = idl.instructions[0].discriminator.as_ref().unwrap();

    // 100 000 × 8 bytes = 800 000 bytes, far exceeds the 29 568-byte budget.
    let mut data = disc.clone();
    data.extend_from_slice(&100_000u32.to_le_bytes());

    let result = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
    assert!(
        result.is_err(),
        "expected Err for over-budget Vec<u64> length"
    );
}

// ── Embedded-IDL property tests ───────────────────────────────────────────────
//
// These two tests replace the old fuzz_all_idls.sh shell script. They run the
// same crash-safety and valid-data property tests against every IDL embedded in
// the library, with no external tooling required.
//
// Usage:
//   cargo test --test fuzz_idl_parsing embedded
//   PROPTEST_CASES=1000 cargo test --test fuzz_idl_parsing embedded

const EMBEDDED_IDLS: &[(&str, &str)] = &[
    ("ape_pro", solana_parser::solana::embedded_idls::APE_PRO_IDL),
    (
        "candy_machine",
        solana_parser::solana::embedded_idls::CANDY_MACHINE_IDL,
    ),
    ("drift", solana_parser::solana::embedded_idls::DRIFT_IDL),
    ("jupiter", solana_parser::solana::embedded_idls::JUPITER_IDL),
    (
        "jupiter_agg_v6",
        solana_parser::solana::embedded_idls::JUPITER_AGG_V6_IDL,
    ),
    (
        "jupiter_limit",
        solana_parser::solana::embedded_idls::JUPITER_LIMIT_IDL,
    ),
    ("kamino", solana_parser::solana::embedded_idls::KAMINO_IDL),
    (
        "lifinity",
        solana_parser::solana::embedded_idls::LIFINITY_IDL,
    ),
    ("meteora", solana_parser::solana::embedded_idls::METEORA_IDL),
    (
        "openbook",
        solana_parser::solana::embedded_idls::OPENBOOK_IDL,
    ),
    ("orca", solana_parser::solana::embedded_idls::ORCA_IDL),
    ("raydium", solana_parser::solana::embedded_idls::RAYDIUM_IDL),
    ("stabble", solana_parser::solana::embedded_idls::STABBLE_IDL),
];

fn proptest_cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256)
}

/// Crash-safety test against every embedded IDL.
///
/// For each IDL: 50/50 valid/random discriminator mix; on Ok asserts correct dispatch.
#[test]
fn embedded_idls_never_panic() {
    for &(name, idl_json) in EMBEDDED_IDLS {
        let idl = decode_idl_data(idl_json)
            .unwrap_or_else(|e| panic!("{name}: decode_idl_data failed: {e}"));

        let config = ProptestConfig::with_cases(proptest_cases());
        let mut runner = proptest::test_runner::TestRunner::new(config);
        let strategy = (
            any::<bool>(),
            any::<usize>(),
            prop::collection::vec(any::<u8>(), 0..1300usize),
        );

        let idl_ref = idl.clone();
        runner
            .run(&strategy, move |(use_valid_disc, inst_idx, data)| {
                if use_valid_disc && !idl_ref.instructions.is_empty() {
                    let inst = &idl_ref.instructions[inst_idx % idl_ref.instructions.len()];
                    let expected = inst.name.clone();
                    if let Some(disc) = &inst.discriminator {
                        let mut d = disc.clone();
                        d.extend_from_slice(&data);
                        if let Ok(result) =
                            parse_instruction_with_idl(&d, TEST_PROGRAM_ID, &idl_ref)
                        {
                            prop_assert_eq!(
                                &result.instruction_name,
                                &expected,
                                "wrong instruction dispatched"
                            );
                        }
                    }
                } else {
                    let _ = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl_ref);
                }
                Ok(())
            })
            .unwrap_or_else(|e| panic!("{name}: embedded_idls_never_panic failed: {e}"));
    }
}

/// Valid-data parse test against every embedded IDL.
///
/// For each IDL and each instruction: generates correctly borsh-encoded bytes
/// and asserts the parser returns Ok with the expected instruction name.
#[test]
fn embedded_idls_valid_data_always_parses_ok() {
    for &(name, idl_json) in EMBEDDED_IDLS {
        let idl = decode_idl_data(idl_json)
            .unwrap_or_else(|e| panic!("{name}: decode_idl_data failed: {e}"));

        if idl.instructions.is_empty() {
            continue;
        }

        let config = ProptestConfig::with_cases(proptest_cases());
        let mut runner = proptest::test_runner::TestRunner::new(config);

        let n = idl.instructions.len();
        let types = Arc::new(idl.types.clone());
        let instructions = idl.instructions.clone();
        let strategy = (0..n).prop_flat_map(move |inst_idx| {
            arb::arb_valid_instruction_bytes(&instructions[inst_idx], types.clone())
                .prop_map(move |bytes| (inst_idx, bytes))
        });

        let idl_ref = idl.clone();
        let name_str = name.to_string();
        runner
            .run(&strategy, move |(inst_idx, bytes)| {
                let expected = &idl_ref.instructions[inst_idx].name;
                let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, &idl_ref);
                prop_assert!(
                    result.is_ok(),
                    "{name_str}: instruction '{expected}' rejected correctly-encoded input: {:?}",
                    result
                );
                prop_assert_eq!(&result.unwrap().instruction_name, expected);
                Ok(())
            })
            .unwrap_or_else(|e| {
                panic!("{name}: embedded_idls_valid_data_always_parses_ok failed: {e}")
            });
    }
}

// ── Real-IDL property tests (driven by IDL_FILE env var) ─────────────────────
//
// These tests are skipped when IDL_FILE is unset. Use them to test IDLs not
// in the embedded set.
//
// Usage:
//   IDL_FILE=/path/to/custom.json cargo test --test fuzz_idl_parsing real_idl
//   IDL_FILE=/path/to/custom.json PROPTEST_CASES=1000 cargo test --test fuzz_idl_parsing real_idl

fn load_idl_from_env() -> Option<(String, solana_parser::solana::structs::Idl)> {
    let path = std::env::var("IDL_FILE").ok()?;
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("IDL_FILE={path}: {e}"));
    match decode_idl_data(&json) {
        Ok(idl) => Some((json, idl)),
        Err(e) => {
            // IDL failed validation (e.g. duplicate type names, cyclic references).
            // Skip these tests — they are not valid inputs for real_idl_* tests.
            eprintln!("IDL_FILE={path}: skipping — decode failed: {e}");
            None
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::default())]

    /// Crash-safety test against a real IDL loaded from IDL_FILE.
    ///
    /// Uses the same 50/50 valid/random discriminator mix as
    /// `fuzz_idl_parsing_never_panics`. On `Ok` with a valid discriminator,
    /// asserts the instruction name matches the selected instruction.
    #[test]
    fn real_idl_never_panics(
        use_valid_disc in any::<bool>(),
        inst_idx in any::<usize>(),
        data in prop::collection::vec(any::<u8>(), 0..1300usize),
    ) {
        let Some((_, idl)) = load_idl_from_env() else { return Ok(()); };
        if use_valid_disc && !idl.instructions.is_empty() {
            let inst = &idl.instructions[inst_idx % idl.instructions.len()];
            let expected_name = inst.name.clone();
            if let Some(disc) = &inst.discriminator {
                let mut d = disc.clone();
                d.extend_from_slice(&data);
                if let Ok(result) = parse_instruction_with_idl(&d, TEST_PROGRAM_ID, &idl) {
                    prop_assert_eq!(&result.instruction_name, &expected_name,
                        "discriminator must dispatch to the correct instruction");
                }
            }
        } else {
            let _ = parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl);
        }
    }
}

/// Valid-data parse test against a real IDL loaded from IDL_FILE.
///
/// Uses TestRunner::run directly so the strategy can be built from the
/// runtime-loaded IDL (not possible with the proptest! macro, which requires
/// strategies to be fully determined at compile time).
///
/// For every instruction in the IDL, generates correctly borsh-encoded bytes
/// (discriminator + all args) and asserts the parser returns Ok with the
/// expected instruction name.
#[test]
fn real_idl_valid_data_always_parses_ok() {
    let Some((_, idl)) = load_idl_from_env() else {
        return;
    };
    let n = idl.instructions.len();
    if n == 0 {
        return;
    }

    let types = Arc::new(idl.types.clone());
    let instructions = idl.instructions.clone();

    let strategy = (0..n).prop_flat_map(move |inst_idx| {
        arb::arb_valid_instruction_bytes(&instructions[inst_idx], types.clone())
            .prop_map(move |bytes| (inst_idx, bytes))
    });

    let config = ProptestConfig::default();
    let mut runner = proptest::test_runner::TestRunner::new(config);
    let idl_ref = idl.clone();
    runner
        .run(&strategy, move |(inst_idx, bytes)| {
            let expected = &idl_ref.instructions[inst_idx].name;
            let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, &idl_ref);
            prop_assert!(
                result.is_ok(),
                "instruction '{expected}' rejected correctly-encoded input: {:?}",
                result
            );
            prop_assert_eq!(&result.unwrap().instruction_name, expected);
            Ok(())
        })
        .expect("real_idl_valid_data_always_parses_ok failed");
}
