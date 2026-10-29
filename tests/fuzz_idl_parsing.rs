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
use proptest::strategy::ValueTree;
use proptest::test_runner::{TestCaseError, TestRunner};
use solana_parser::solana::structs::{
    Defined, Idl, IdlField, IdlType, IdlTypeDefinition, IdlTypeDefinitionType, ProgramType,
};
use solana_parser::{decode_idl_data, parse_instruction_with_idl, SolanaParsedInstructionData};
use solana_parser_fuzz_core::proptest as arb;
use std::sync::Arc;

const TEST_PROGRAM_ID: &str = "11111111111111111111111111111111";

// ── Local strategies ─────────────────────────────────────────────────────────
//
// Core strategies (`arb_identifier`, `arb_primitive_idl_type`, `arb_idl_type`,
// `arb_idl_instruction`, `arb_idl`, `arb_idl_json`, `arb_bytes_for_type`,
// `arb_valid_instruction_bytes`) live in `solana_parser_fuzz_core::proptest`.

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
            arb::idl_to_json(&idl)
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
            arb::idl_to_json(&idl)
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

// ── Generator coverage ───────────────────────────────────────────────────────
//
// The crash-safety properties only reach instruction parsing when the generated
// JSON decodes. These checks fail if a generator stops producing decodable IDLs,
// which would otherwise make those properties pass while testing nothing.

/// Samples `cases` values from `strategy` and asserts every one decodes.
fn assert_generated_idls_decode<S: Strategy<Value = String>>(name: &str, strategy: S, cases: u32) {
    let mut runner = TestRunner::deterministic();
    for case in 0..cases {
        let json = strategy.new_tree(&mut runner).unwrap().current();
        if let Err(e) = decode_idl_data(&json) {
            panic!("{name}: case {case} failed to decode: {e}\n{json}");
        }
    }
}

#[test]
fn generated_idl_json_decodes() {
    assert_generated_idls_decode("arb_idl_json", arb::arb_idl_json(), 256);
    assert_generated_idls_decode("arb_vec_arg_idl_json", arb_vec_arg_idl_json(), 256);
    assert_generated_idls_decode(
        "arb_defined_struct_idl_json",
        arb_defined_struct_idl_json(),
        256,
    );
}

/// A decoded `arb_idl_json` IDL dispatches correctly-encoded bytes to `Ok`,
/// so `fuzz_idl_parsing_never_panics` reaches argument parsing.
#[test]
fn generated_idl_json_reaches_instruction_parsing() {
    let strategy = arb::arb_idl_json().prop_flat_map(|json| {
        let idl = decode_idl_data(&json).unwrap_or_else(|e| panic!("decode failed: {e}\n{json}"));
        let types = Arc::new(idl.types.clone());
        let bytes = arb::arb_valid_instruction_bytes(&idl.instructions[0], types);
        (Just(idl), bytes)
    });
    let mut runner = TestRunner::deterministic();
    for _ in 0..256 {
        let (idl, bytes) = strategy.new_tree(&mut runner).unwrap().current();
        let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, &idl);
        let parsed = result.unwrap_or_else(|e| panic!("valid bytes rejected: {e}"));
        assert_eq!(parsed.instruction_name, idl.instructions[0].name);
    }
}

/// Decodes generated IDL JSON, failing the case instead of skipping it, so a
/// generator regression cannot turn a property into a no-op.
fn decode_generated(idl_json: &str) -> Result<Idl, TestCaseError> {
    decode_idl_data(idl_json).map_err(|e| {
        TestCaseError::fail(format!("generated IDL failed to decode: {e}\n{idl_json}"))
    })
}

// ── Shared checks ────────────────────────────────────────────────────────────

/// Parses `data` against `idl`, either as-is or, when `use_valid_disc` is set,
/// behind the discriminator of a selected instruction.
///
/// Using a valid discriminator for half of all inputs ensures the argument-
/// decoding paths are covered, not just discriminator matching. Either way the
/// parser must not panic; on the valid-discriminator branch an `Ok` must also
/// name the selected instruction. `Err` is always acceptable — random arg bytes
/// may be too short or malformed.
fn check_dispatch(
    idl: &Idl,
    use_valid_disc: bool,
    inst_idx: usize,
    data: &[u8],
) -> Result<(), TestCaseError> {
    if !use_valid_disc || idl.instructions.is_empty() {
        let _ = parse_instruction_with_idl(data, TEST_PROGRAM_ID, idl);
        return Ok(());
    }
    let inst = &idl.instructions[inst_idx % idl.instructions.len()];
    let Some(disc) = &inst.discriminator else {
        return Ok(());
    };
    let mut d = disc.clone();
    d.extend_from_slice(data);
    if let Ok(result) = parse_instruction_with_idl(&d, TEST_PROGRAM_ID, idl) {
        prop_assert_eq!(
            &result.instruction_name,
            &inst.name,
            "discriminator must dispatch to the selected instruction"
        );
    }
    Ok(())
}

/// Runs [`check_dispatch`] against a fixed IDL with random data up to a little
/// past MAX_CURSOR_LENGTH (1232) bytes.
fn check_never_panics(name: &str, idl: &Idl) {
    let strategy = (
        any::<bool>(),
        any::<usize>(),
        prop::collection::vec(any::<u8>(), 0..1300usize),
    );
    let config = ProptestConfig {
        source_file: Some(file!()),
        ..ProptestConfig::default()
    };
    TestRunner::new(config)
        .run(&strategy, |(use_valid_disc, inst_idx, data)| {
            check_dispatch(idl, use_valid_disc, inst_idx, &data)
        })
        .unwrap_or_else(|e| panic!("{name}: {e}"));
}

/// For every instruction in a fixed IDL, generates correctly borsh-encoded
/// bytes and asserts the parser returns `Ok` with the expected instruction name.
///
/// Uses `TestRunner::run` directly because the strategy depends on a runtime
/// IDL, which the `proptest!` macro can't express.
fn check_valid_data_parses_ok(name: &str, idl: &Idl) {
    if idl.instructions.is_empty() {
        return;
    }
    let types = Arc::new(idl.types.clone());
    let instructions = idl.instructions.clone();
    let strategy = (0..instructions.len()).prop_flat_map(move |inst_idx| {
        arb::arb_valid_instruction_bytes(&instructions[inst_idx], types.clone())
            .prop_map(move |bytes| (inst_idx, bytes))
    });
    let config = ProptestConfig {
        source_file: Some(file!()),
        ..ProptestConfig::default()
    };
    TestRunner::new(config)
        .run(&strategy, |(inst_idx, bytes)| {
            let expected = &idl.instructions[inst_idx].name;
            let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, idl);
            prop_assert!(
                result.is_ok(),
                "instruction '{expected}' rejected correctly-encoded input: {result:?}"
            );
            prop_assert_eq!(&result.unwrap().instruction_name, expected);
            Ok(())
        })
        .unwrap_or_else(|e| panic!("{name}: {e}"));
}

// ── Crash-safety property tests ──────────────────────────────────────────────

proptest! {
    // Default is 256 cases. Override with PROPTEST_CASES=5000 for deeper fuzzing.
    #![proptest_config(ProptestConfig::default())]

    /// Core crash-safety test: a random IDL paired with instruction data that is
    /// either fully random or behind a valid discriminator (see `check_dispatch`).
    #[test]
    fn fuzz_idl_parsing_never_panics(
        idl_json in arb::arb_idl_json(),
        use_valid_disc in any::<bool>(),
        inst_idx in any::<usize>(),
        data in prop::collection::vec(any::<u8>(), 0..1300usize),
    ) {
        let idl = decode_generated(&idl_json)?;
        check_dispatch(&idl, use_valid_disc, inst_idx, &data)?;
    }

    /// `decode_idl_data` must not panic on completely arbitrary string input.
    #[test]
    fn fuzz_decode_idl_data_arbitrary_input(s in any::<String>()) {
        let _ = decode_idl_data(&s);
    }

    /// IDLs with defined struct types must not panic regardless of instruction
    /// bytes, and must dispatch correctly when given a valid discriminator.
    #[test]
    fn fuzz_defined_struct_types_never_panics(
        idl_json in arb_defined_struct_idl_json(),
        use_valid_disc in any::<bool>(),
        inst_idx in any::<usize>(),
        data in prop::collection::vec(any::<u8>(), 0..200usize),
    ) {
        let idl = decode_generated(&idl_json)?;
        check_dispatch(&idl, use_valid_disc, inst_idx, &data)?;
    }

    /// Valid input must always parse successfully.
    ///
    /// Unlike the crash-safety tests, this asserts `Ok`: a failure is a genuine
    /// parser bug, rejecting data that matches its own IDL contract.
    #[test]
    fn fuzz_valid_data_always_parses_ok(
        (idl, inst_idx, bytes) in arb_idl_and_valid_bytes(),
    ) {
        if idl.instructions.is_empty() || bytes.is_empty() { return Ok(()); }
        let expected_name = idl.instructions[inst_idx].name.clone();
        let result = parse_instruction_with_idl(&bytes, TEST_PROGRAM_ID, &idl);
        prop_assert!(result.is_ok(),
            "parser rejected correctly-encoded input for instruction '{expected_name}': {result:?}");
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
        let idl = decode_generated(&idl_json)?;
        let mut data = length_prefix.to_le_bytes().to_vec();
        data.extend_from_slice(&trailing);
        check_dispatch(&idl, true, 0, &data)?;
    }
}

// ── Valid-data roundtrip tests ────────────────────────────────────────────────
//
// Each test builds an IDL from JSON, prefixes hand-encoded arg bytes with an
// instruction's computed discriminator, and checks the decoded values.

/// Decodes `idl` and parses `args` behind its `inst_idx`-th instruction's discriminator.
fn parse_args(
    idl: &serde_json::Value,
    inst_idx: usize,
    args: &[u8],
) -> Result<SolanaParsedInstructionData, Box<dyn std::error::Error>> {
    let idl = decode_idl_data(&idl.to_string()).unwrap();
    let mut data = idl.instructions[inst_idx].discriminator.clone().unwrap();
    data.extend_from_slice(args);
    parse_instruction_with_idl(&data, TEST_PROGRAM_ID, &idl)
}

/// An IDL with one instruction `name` taking `args`.
fn single_instruction_idl(name: &str, args: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "instructions": [{"name": name, "accounts": [], "args": args}],
        "types": []
    })
}

#[test]
fn roundtrip_no_args() {
    let idl = single_instruction_idl("initialize", serde_json::json!([]));
    let result = parse_args(&idl, 0, &[]).unwrap();
    assert_eq!(result.instruction_name, "initialize");
    assert!(result.program_call_args.is_empty());
}

#[test]
fn roundtrip_mixed_primitive_args() {
    let idl = single_instruction_idl(
        "swap",
        serde_json::json!([
            {"name": "amountIn",  "type": "u64"},
            {"name": "minOut",    "type": "u64"},
            {"name": "slippage",  "type": "u16"},
            {"name": "isExact",   "type": "bool"},
        ]),
    );
    let mut args = 1000u64.to_le_bytes().to_vec();
    args.extend_from_slice(&900u64.to_le_bytes());
    args.extend_from_slice(&50u16.to_le_bytes());
    args.push(1u8);

    let result = parse_args(&idl, 0, &args).unwrap();
    assert_eq!(result.instruction_name, "swap");
    let a = &result.program_call_args;
    assert_eq!(a["amountIn"], serde_json::json!(1000));
    assert_eq!(a["minOut"], serde_json::json!(900));
    assert_eq!(a["slippage"], serde_json::json!(50));
    assert_eq!(a["isExact"], serde_json::json!(true));
}

#[test]
fn roundtrip_container_args() {
    let cases: Vec<(serde_json::Value, Vec<u8>, serde_json::Value)> = vec![
        // Option: 0x01 + inner for Some, 0x00 for None
        (
            serde_json::json!({"option": "u16"}),
            [&[1u8][..], &300u16.to_le_bytes()].concat(),
            serde_json::json!(300),
        ),
        (
            serde_json::json!({"option": "u16"}),
            vec![0u8],
            serde_json::Value::Null,
        ),
        // Vec: u32 LE length prefix + elements
        (
            serde_json::json!({"vec": "u8"}),
            [&3u32.to_le_bytes()[..], &[10, 20, 30]].concat(),
            serde_json::json!([10, 20, 30]),
        ),
        // Array: exactly n elements, no length prefix
        (
            serde_json::json!({"array": ["u32", 3]}),
            [100u32, 200, 300]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            serde_json::json!([100, 200, 300]),
        ),
    ];
    for (ty, args, expected) in cases {
        let idl = single_instruction_idl("op", serde_json::json!([{"name": "v", "type": ty}]));
        let result = parse_args(&idl, 0, &args).unwrap();
        assert_eq!(result.program_call_args["v"], expected, "type {ty}");
    }
}

#[test]
fn roundtrip_multiple_instructions_distinct_dispatch() {
    // IDL with 3 instructions; verify each is dispatched by its own discriminator.
    let idl = serde_json::json!({
        "instructions": [
            {"name": "initialize", "accounts": [], "args": []},
            {"name": "deposit",    "accounts": [], "args": [{"name": "amount", "type": "u32"}]},
            {"name": "withdraw",   "accounts": [], "args": [
                {"name": "amount", "type": "u32"},
                {"name": "all",    "type": "bool"},
            ]},
        ],
        "types": []
    });

    let r = parse_args(&idl, 0, &[]).unwrap();
    assert_eq!(r.instruction_name, "initialize");
    assert!(r.program_call_args.is_empty());

    let r = parse_args(&idl, 1, &99u32.to_le_bytes()).unwrap();
    assert_eq!(r.instruction_name, "deposit");
    assert_eq!(r.program_call_args["amount"], serde_json::json!(99));

    let r = parse_args(&idl, 2, &[&50u32.to_le_bytes()[..], &[0u8]].concat()).unwrap();
    assert_eq!(r.instruction_name, "withdraw");
    assert_eq!(r.program_call_args["amount"], serde_json::json!(50));
    assert_eq!(r.program_call_args["all"], serde_json::json!(false));
}

/// An instruction whose single arg is a defined struct with primitive fields
/// is decoded correctly end-to-end, with fields nested under the arg name.
#[test]
fn roundtrip_defined_struct_arg() {
    let idl = serde_json::json!({
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
    });
    let mut args = 5000u64.to_le_bytes().to_vec();
    args.extend_from_slice(&10u32.to_le_bytes());
    args.push(1u8);

    let result = parse_args(&idl, 0, &args).unwrap();
    assert_eq!(result.instruction_name, "createOrder");
    let params = &result.program_call_args["params"];
    assert_eq!(params["price"], serde_json::json!(5000));
    assert_eq!(params["quantity"], serde_json::json!(10));
    assert_eq!(params["side"], serde_json::json!(true));
}

/// A Vec length prefix that vastly exceeds the backing data must be rejected
/// cleanly (Err), not panic or over-allocate.
///
/// SizeGuard budget = MAX_CURSOR_LENGTH (1232) × MAX_ALLOC_PER_CURSOR_LENGTH (24) = 29 568 bytes.
#[test]
fn size_guard_rejects_over_budget_vec_length() {
    // 10M × 1 byte, and 100k × 8 bytes = 800 000 bytes: both far over budget.
    for (elem, claimed_len) in [("u8", 10_000_000u32), ("u64", 100_000)] {
        let idl = single_instruction_idl(
            "write",
            serde_json::json!([{"name": "v", "type": {"vec": elem}}]),
        );
        let result = parse_args(&idl, 0, &claimed_len.to_le_bytes());
        assert!(
            result.is_err(),
            "Vec<{elem}> of {claimed_len}: expected Err"
        );
    }
}

/// A defined enum with 256 unit variants uses every one-byte variant index;
/// generating and parsing valid data for it must work for all of them.
#[test]
fn enum_with_256_variants_parses_valid_data() {
    let variants: Vec<serde_json::Value> = (0..256)
        .map(|i| serde_json::json!({"name": format!("v{i}")}))
        .collect();
    let idl = serde_json::json!({
        "instructions": [{"name": "pick", "accounts": [], "args": [
            {"name": "choice", "type": {"defined": "Choice"}}
        ]}],
        "types": [{"name": "Choice", "type": {"kind": "enum", "variants": variants}}]
    });
    let idl = decode_idl_data(&idl.to_string()).unwrap();
    check_valid_data_parses_ok("256-variant enum", &idl);
}

// ── Embedded-IDL property tests ───────────────────────────────────────────────
//
// Run the crash-safety and valid-data checks against every IDL embedded in the
// library.
//
//   cargo test --test fuzz_idl_parsing embedded
//   PROPTEST_CASES=1000 cargo test --test fuzz_idl_parsing embedded

fn embedded_idls() -> impl Iterator<Item = (&'static str, Idl)> {
    ProgramType::all().iter().map(|p| {
        let name = p.program_name();
        let idl = decode_idl_data(p.idl_json())
            .unwrap_or_else(|e| panic!("{name}: decode_idl_data failed: {e}"));
        (name, idl)
    })
}

#[test]
fn embedded_idls_never_panic() {
    for (name, idl) in embedded_idls() {
        check_never_panics(name, &idl);
    }
}

#[test]
fn embedded_idls_valid_data_always_parses_ok() {
    for (name, idl) in embedded_idls() {
        check_valid_data_parses_ok(name, &idl);
    }
}

// ── Real-IDL property tests (driven by IDL_FILE env var) ─────────────────────
//
// Skipped when IDL_FILE is unset. Use them to test IDLs not in the embedded set.
//
//   IDL_FILE=/path/to/custom.json cargo test --test fuzz_idl_parsing real_idl
//   IDL_FILE=/path/to/custom.json PROPTEST_CASES=1000 cargo test --test fuzz_idl_parsing real_idl

fn load_idl_from_env() -> Option<(String, Idl)> {
    let path = std::env::var("IDL_FILE").ok()?;
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("IDL_FILE={path}: {e}"));
    match decode_idl_data(&json) {
        Ok(idl) => Some((path, idl)),
        Err(e) => {
            // IDL failed validation (e.g. duplicate type names, cyclic references).
            // Skip these tests — they are not valid inputs for real_idl_* tests.
            eprintln!("IDL_FILE={path}: skipping — decode failed: {e}");
            None
        }
    }
}

#[test]
fn real_idl_never_panics() {
    if let Some((path, idl)) = load_idl_from_env() {
        check_never_panics(&path, &idl);
    }
}

#[test]
fn real_idl_valid_data_always_parses_ok() {
    if let Some((path, idl)) = load_idl_from_env() {
        check_valid_data_parses_ok(&path, &idl);
    }
}
