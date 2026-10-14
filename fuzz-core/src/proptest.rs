//! Proptest strategies for IDL types.
//!
//! Enabled via the `proptest` feature flag. Consumers add:
//!
//! ```toml
//! [dev-dependencies]
//! solana-parser-fuzz-core = { ..., features = ["proptest"] }
//! ```
//!
//! The key strategies:
//! - [`arb_primitive_idl_type`] — leaf `IdlType` variants (no recursion)
//! - [`arb_idl_type`] — any `IdlType` including nested containers
//! - [`arb_idl_instruction`] — a random `IdlInstruction` with random discriminator
//! - [`arb_idl`] — a complete `Idl` with unique instruction names/discriminators
//! - [`arb_idl_json`] — `arb_idl` serialized to JSON
//! - [`arb_bytes_for_type`] — valid borsh-encoded bytes for a given `IdlType`
//! - [`arb_valid_instruction_bytes`] — discriminator + valid borsh arg bytes

use proptest::prelude::*;
use solana_parser::solana::idl_parser::compute_default_anchor_discriminator;
use solana_parser::solana::structs::{
    Defined, EnumFields, Idl, IdlField, IdlInstruction, IdlType, IdlTypeDefinition,
    IdlTypeDefinitionType,
};
use std::sync::Arc;

// ── Identifier strategy ───────────────────────────────────────────────────────

/// Generates a short, valid identifier string (lowercase ASCII).
pub fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

// ── IdlType strategies ────────────────────────────────────────────────────────

/// Generates a primitive (non-recursive) `IdlType`.
pub fn arb_primitive_idl_type() -> impl Strategy<Value = IdlType> {
    prop_oneof![
        Just(IdlType::Bool),
        Just(IdlType::U8),
        Just(IdlType::U16),
        Just(IdlType::U32),
        Just(IdlType::U64),
        Just(IdlType::U128),
        Just(IdlType::I8),
        Just(IdlType::I16),
        Just(IdlType::I32),
        Just(IdlType::I64),
        Just(IdlType::I128),
        Just(IdlType::F32),
        Just(IdlType::F64),
        Just(IdlType::PublicKey),
        Just(IdlType::String),
        Just(IdlType::Bytes),
    ]
}

/// Generates any `IdlType`, including nested `Vec`, `Option`, and `Array`.
///
/// Does not generate `Defined` references — use `arb_defined_struct_idl_json`
/// in the test file for that case.
pub fn arb_idl_type() -> impl Strategy<Value = IdlType> {
    arb_primitive_idl_type().prop_recursive(
        3,  // max depth
        16, // expected total nodes
        4,  // max items per collection node
        |inner| {
            prop_oneof![
                inner.clone().prop_map(|t| IdlType::Option(Box::new(t))),
                inner.clone().prop_map(|t| IdlType::Vec(Box::new(t))),
                (inner, 1usize..=4usize).prop_map(|(t, n)| IdlType::Array(Box::new(t), n)),
            ]
        },
    )
}

// ── Instruction strategy ──────────────────────────────────────────────────────

/// Generates a random `IdlInstruction` with an arbitrary 8-byte discriminator.
pub fn arb_idl_instruction() -> impl Strategy<Value = IdlInstruction> {
    (
        arb_identifier(),
        prop::array::uniform8(any::<u8>()),
        prop::collection::vec(
            (arb_identifier(), arb_idl_type()).prop_map(|(n, t)| IdlField { name: n, r#type: t }),
            0..=4usize,
        ),
    )
        .prop_map(|(name, disc, args)| IdlInstruction {
            name,
            discriminator: Some(disc.to_vec()),
            accounts: vec![],
            args,
        })
}

// ── Idl strategy ─────────────────────────────────────────────────────────────

/// Generates a complete `Idl` with 1–5 instructions.
///
/// Instruction names are unique (drawn from a hash set), so discriminators
/// computed via the Anchor formula are also unique. This guarantees that
/// `arb_valid_instruction_bytes` produces bytes that parse back to `Ok`.
///
/// Uses only primitive arg types (no `Defined` references) so the `types`
/// vec is always empty and `TypeResolver` always succeeds.
pub fn arb_idl() -> impl Strategy<Value = Idl> {
    prop::collection::hash_set("[a-z][a-z0-9]{0,7}", 1..=5usize).prop_flat_map(|name_set| {
        let names: Vec<String> = name_set.into_iter().collect();
        let n = names.len();
        prop::collection::vec(
            prop::collection::vec(
                (arb_identifier(), arb_primitive_idl_type()).prop_map(|(field_name, t)| IdlField {
                    name: field_name,
                    r#type: t,
                }),
                0..=4usize,
            ),
            n..=n,
        )
        .prop_map(move |all_args| {
            let instructions = names
                .iter()
                .zip(all_args)
                .map(|(name, args)| {
                    // The fallback is defensive only: `arb_identifier()` produces
                    // names matching `[a-z][a-z0-9]{0,7}`, and Anchor's discriminator
                    // is a sha256 truncation of `global:<name>`, which is infallible
                    // for any non-empty name. Reachable only if the strategy invariant
                    // ever changes and an empty name slips through.
                    let disc =
                        compute_default_anchor_discriminator(name).unwrap_or_else(|_| vec![0u8; 8]);
                    IdlInstruction {
                        name: name.clone(),
                        discriminator: Some(disc),
                        accounts: vec![],
                        args,
                    }
                })
                .collect();
            Idl {
                instructions,
                types: vec![],
            }
        })
    })
}

/// Generates a valid IDL serialized as a JSON string.
pub fn arb_idl_json() -> impl Strategy<Value = String> {
    arb_idl().prop_map(|idl| serde_json::to_string(&idl).expect("Idl always serializes"))
}

// ── Borsh byte strategies ─────────────────────────────────────────────────────

/// Generates valid borsh-encoded bytes for a given `IdlType`.
///
/// The generated bytes are exactly what `parse_type` in `idl_parser` would
/// consume — no extra bytes, no missing bytes.
///
/// Sizes are kept small (strings ≤ 32 chars, vecs ≤ 4 elements) so the
/// `SizeGuard` inside the parser never rejects them.
pub fn arb_bytes_for_type(
    ty: &IdlType,
    types: Arc<Vec<IdlTypeDefinition>>,
) -> BoxedStrategy<Vec<u8>> {
    match ty {
        // Fixed-width primitives
        IdlType::Bool => any::<bool>().prop_map(|b| vec![b as u8]).boxed(),
        IdlType::U8 => any::<u8>().prop_map(|v| vec![v]).boxed(),
        IdlType::I8 => any::<i8>().prop_map(|v| vec![v as u8]).boxed(),
        IdlType::U16 => any::<u16>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::I16 => any::<i16>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::U32 => any::<u32>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::I32 => any::<i32>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::U64 => any::<u64>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::I64 => any::<i64>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::U128 => any::<u128>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        IdlType::I128 => any::<i128>().prop_map(|v| v.to_le_bytes().to_vec()).boxed(),
        // Filter out NaN/infinity: serde_json's From<f32> panics on non-finite values.
        IdlType::F32 => any::<f32>()
            .prop_filter("finite f32", |f| f.is_finite())
            .prop_map(|v| v.to_le_bytes().to_vec())
            .boxed(),
        IdlType::F64 => any::<f64>()
            .prop_filter("finite f64", |f| f.is_finite())
            .prop_map(|v| v.to_le_bytes().to_vec())
            .boxed(),
        // 32-byte public key
        IdlType::PublicKey => prop::array::uniform32(any::<u8>())
            .prop_map(|b| b.to_vec())
            .boxed(),
        // Length-prefixed UTF-8 string (u32 LE len + bytes)
        IdlType::String => "[a-zA-Z0-9 ]{0,32}"
            .prop_map(|s: String| {
                let b = s.as_bytes();
                let mut out = (b.len() as u32).to_le_bytes().to_vec();
                out.extend_from_slice(b);
                out
            })
            .boxed(),
        // Length-prefixed raw bytes (u32 LE len + bytes)
        IdlType::Bytes => prop::collection::vec(any::<u8>(), 0..=32usize)
            .prop_map(|bytes| {
                let mut out = (bytes.len() as u32).to_le_bytes().to_vec();
                out.extend(bytes);
                out
            })
            .boxed(),
        // Option: 0x00 for None, 0x01 + inner for Some
        IdlType::Option(inner) => {
            let inner: IdlType = *inner.clone();
            let types2 = types.clone();
            any::<bool>()
                .prop_flat_map(move |is_some| {
                    if is_some {
                        arb_bytes_for_type(&inner, types2.clone())
                            .prop_map(|b| {
                                let mut out = vec![1u8];
                                out.extend(b);
                                out
                            })
                            .boxed()
                    } else {
                        Just(vec![0u8]).boxed()
                    }
                })
                .boxed()
        }
        // Vec: u32 LE length + elements (0–4 elements to stay within SizeGuard)
        IdlType::Vec(inner) => {
            let inner: IdlType = *inner.clone();
            let types2 = types.clone();
            (0u32..=4u32)
                .prop_flat_map(move |len| {
                    let inner2 = inner.clone();
                    let types3 = types2.clone();
                    prop::collection::vec(
                        arb_bytes_for_type(&inner2, types3),
                        len as usize..=len as usize,
                    )
                    .prop_map(move |parts| {
                        let mut out = len.to_le_bytes().to_vec();
                        for part in parts {
                            out.extend(part);
                        }
                        out
                    })
                })
                .boxed()
        }
        // Array: exactly n elements, no length prefix
        IdlType::Array(inner, size) => {
            let inner: IdlType = *inner.clone();
            let size = *size;
            let types2 = types.clone();
            prop::collection::vec(arb_bytes_for_type(&inner, types2), size..=size)
                .prop_map(|parts| {
                    let mut out = Vec::new();
                    for part in parts {
                        out.extend(part);
                    }
                    out
                })
                .boxed()
        }
        // Defined: look up the type definition and encode it
        IdlType::Defined(defined) => {
            let name = match defined {
                Defined::String(s) => s.clone(),
                Defined::Object { name } => name.clone(),
            };
            if let Some(ty_def) = types.iter().find(|t| t.name == name) {
                arb_bytes_for_type_def(&ty_def.r#type.clone(), types)
            } else {
                // Type not found: return empty bytes (parse will Err, not panic)
                Just(vec![]).boxed()
            }
        }
    }
}

fn arb_bytes_for_type_def(
    ty_def: &IdlTypeDefinitionType,
    types: Arc<Vec<IdlTypeDefinition>>,
) -> BoxedStrategy<Vec<u8>> {
    match ty_def {
        IdlTypeDefinitionType::Struct { fields } => {
            let strats: Vec<BoxedStrategy<Vec<u8>>> = fields
                .iter()
                .map(|f| arb_bytes_for_type(&f.r#type, types.clone()))
                .collect();
            concat_strategies(strats)
        }
        IdlTypeDefinitionType::Enum { variants } => {
            if variants.is_empty() {
                return Just(vec![]).boxed();
            }
            let variants = variants.clone();
            let n = variants.len();
            let types2 = types.clone();
            (0u8..n as u8)
                .prop_flat_map(move |idx| {
                    let variant = &variants[idx as usize];
                    let field_bytes = match &variant.fields {
                        Some(EnumFields::Named(fields)) => {
                            let strats: Vec<_> = fields
                                .iter()
                                .map(|f| arb_bytes_for_type(&f.r#type, types2.clone()))
                                .collect();
                            concat_strategies(strats)
                        }
                        Some(EnumFields::Tuple(tys)) => {
                            let strats: Vec<_> = tys
                                .iter()
                                .map(|t| arb_bytes_for_type(t, types2.clone()))
                                .collect();
                            concat_strategies(strats)
                        }
                        None => Just(vec![]).boxed(),
                    };
                    field_bytes
                        .prop_map(move |mut b| {
                            let mut out = vec![idx];
                            out.append(&mut b);
                            out
                        })
                        .boxed()
                })
                .boxed()
        }
        IdlTypeDefinitionType::Alias { value } => arb_bytes_for_type(value, types),
    }
}

/// Generates valid borsh-encoded bytes for a complete instruction:
/// discriminator bytes followed by correctly-encoded bytes for each arg.
///
/// These bytes are guaranteed to be exactly consumed by `parse_data_into_args`,
/// so `parse_instruction_with_idl` will return `Ok` for a well-formed IDL.
pub fn arb_valid_instruction_bytes(
    inst: &IdlInstruction,
    types: Arc<Vec<IdlTypeDefinition>>,
) -> BoxedStrategy<Vec<u8>> {
    let disc = inst.discriminator.clone().unwrap_or_default();
    let arg_strats: Vec<BoxedStrategy<Vec<u8>>> = inst
        .args
        .iter()
        .map(|arg| arb_bytes_for_type(&arg.r#type, types.clone()))
        .collect();
    concat_strategies(arg_strats)
        .prop_map(move |args| {
            let mut out = disc.clone();
            out.extend(args);
            out
        })
        .boxed()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Folds a list of byte-vec strategies into one that concatenates all results.
fn concat_strategies(strats: Vec<BoxedStrategy<Vec<u8>>>) -> BoxedStrategy<Vec<u8>> {
    strats
        .into_iter()
        .fold(Just(Vec::new()).boxed(), |acc, next| {
            (acc, next)
                .prop_map(|(mut a, b)| {
                    a.extend(b);
                    a
                })
                .boxed()
        })
}
