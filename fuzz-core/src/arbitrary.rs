//! `arbitrary::Arbitrary` impls for IDL types, for use with libFuzzer.
//!
//! Enabled via the `arbitrary` feature flag. Fuzz crates add:
//!
//! ```toml
//! [dependencies]
//! solana-parser-fuzz-core = { ..., features = ["arbitrary"] }
//! ```
//!
//! [`ArbIdl`] wraps [`Idl`] and implements `Arbitrary`, consuming bytes from
//! libFuzzer's `Unstructured` input to generate a valid IDL with unique
//! instruction names and correctly-computed Anchor discriminators.

use arbitrary::{Result, Unstructured};
use solana_parser::solana::idl_parser::compute_default_anchor_discriminator;
use solana_parser::solana::structs::{Idl, IdlField, IdlInstruction, IdlType};
use std::collections::HashSet;

/// A libFuzzer-compatible wrapper around [`Idl`].
///
/// Implements `arbitrary::Arbitrary` to generate structurally valid IDLs
/// without modifying the core `Idl` type.
pub struct ArbIdl(pub Idl);

fn arb_identifier(u: &mut Unstructured<'_>) -> Result<String> {
    let len: usize = u.int_in_range(1..=8)?;
    let mut s = String::with_capacity(len);
    s.push((b'a' + u.int_in_range::<u8>(0..=25)?) as char);
    for _ in 1..len {
        let c: u8 = u.int_in_range(0..=35)?;
        s.push(if c < 26 {
            (b'a' + c) as char
        } else {
            (b'0' + (c - 26)) as char
        });
    }
    Ok(s)
}

fn arb_primitive_type(u: &mut Unstructured<'_>) -> Result<IdlType> {
    Ok(match u.int_in_range::<u8>(0..=15)? {
        0 => IdlType::Bool,
        1 => IdlType::U8,
        2 => IdlType::U16,
        3 => IdlType::U32,
        4 => IdlType::U64,
        5 => IdlType::U128,
        6 => IdlType::I8,
        7 => IdlType::I16,
        8 => IdlType::I32,
        9 => IdlType::I64,
        10 => IdlType::I128,
        11 => IdlType::F32,
        12 => IdlType::F64,
        13 => IdlType::PublicKey,
        14 => IdlType::String,
        _ => IdlType::Bytes,
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ArbIdl {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let instruction_count: usize = u.int_in_range(1..=5)?;
        let mut instructions = Vec::with_capacity(instruction_count);
        let mut used_names: HashSet<String> = HashSet::new();

        for _ in 0..instruction_count {
            let mut name = arb_identifier(u)?;
            // ensure uniqueness with up to 4 suffix retries
            for i in 0u8..4 {
                if !used_names.contains(&name) {
                    break;
                }
                name = format!("{name}{i}");
            }
            if used_names.contains(&name) {
                continue;
            }
            used_names.insert(name.clone());

            let arg_count: usize = u.int_in_range(0..=4)?;
            let mut args = Vec::with_capacity(arg_count);
            for _ in 0..arg_count {
                args.push(IdlField {
                    name: arb_identifier(u)?,
                    r#type: arb_primitive_type(u)?,
                });
            }

            let disc = compute_default_anchor_discriminator(&name).unwrap_or_else(|_| vec![0u8; 8]);
            instructions.push(IdlInstruction {
                name,
                discriminator: Some(disc),
                accounts: vec![],
                args,
            });
        }

        // fallback: ensure at least one instruction if all were deduplicated away
        if instructions.is_empty() {
            let disc =
                compute_default_anchor_discriminator("transfer").unwrap_or_else(|_| vec![0u8; 8]);
            instructions.push(IdlInstruction {
                name: "transfer".to_string(),
                discriminator: Some(disc),
                accounts: vec![],
                args: vec![],
            });
        }

        Ok(ArbIdl(Idl {
            instructions,
            types: vec![],
        }))
    }
}
