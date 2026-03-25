use serde_json::{Map, Number, Value};
use std::{collections::HashMap, vec};

use super::*;
use crate::solana::idl_parser;
use crate::solana::parser::{SolanaTransaction, TOKEN_2022_PROGRAM_KEY, TOKEN_PROGRAM_KEY};
use crate::solana::structs::{
    IdlSource, ProgramType, SolTransfer, SolanaAccount, SolanaAddressTableLookup,
    SolanaInstruction, SolanaSingleAddressTableLookup,
};

// Test-only IDL directory for test fixtures
const TEST_IDL_DIRECTORY: &str = "src/solana/idls/";
use parser::SOL_SYSTEM_PROGRAM_KEY;
use structs::SolanaMetadata;

#[test]
fn parses_valid_legacy_transactions() {
    let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // All expected values
    let exp_signature = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let sender_account_key = "3uC8tBZQQA1RCKv9htCngTfYm4JK4ezuYx4M4nFsZQVp";
    let recipient_account_key = "tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C";
    let expected_account_keys = vec![
        sender_account_key.to_string(),
        recipient_account_key.to_string(),
        SOL_SYSTEM_PROGRAM_KEY.to_string(),
    ];
    let expected_program_keys = vec![SOL_SYSTEM_PROGRAM_KEY.to_string()];
    let expected_instruction_hex = "020000006f00000000000000";
    let expected_amount_opt = Some("111".to_string());
    let expected_amount = expected_amount_opt.clone().unwrap();

    // All output checks
    assert_eq!(tx_metadata.signatures, vec![exp_signature]);
    assert_eq!(tx_metadata.account_keys, expected_account_keys);
    assert_eq!(tx_metadata.program_keys, expected_program_keys);
    assert_eq!(tx_metadata.instructions.len(), 1);

    // Assert instructions array is correct
    let inst = &tx_metadata.instructions[0];
    assert_eq!(inst.program_key, SOL_SYSTEM_PROGRAM_KEY.to_string());
    assert_eq!(inst.accounts.len(), 2);
    assert_eq!(inst.accounts[0].account_key, sender_account_key);
    assert!(inst.accounts[0].signer);
    assert!(inst.accounts[0].writable);
    assert_eq!(inst.accounts[1].account_key, recipient_account_key);
    assert!(!inst.accounts[1].signer);
    assert!(inst.accounts[1].writable);
    assert_eq!(inst.instruction_data_hex, expected_instruction_hex);

    // Assert transfers array is correct
    assert_eq!(tx_metadata.transfers.len(), 1);
    assert_eq!(tx_metadata.transfers[0].amount, expected_amount);
    assert_eq!(tx_metadata.transfers[0].from, sender_account_key);
    assert_eq!(tx_metadata.transfers[0].to, recipient_account_key);
}

#[test]
fn parses_valid_legacy_transaction_message_only() {
    let unsigned_payload = "010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, false, None).unwrap(); // check that a message is parsed correctly
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // All expected values
    let sender_account_key = "3uC8tBZQQA1RCKv9htCngTfYm4JK4ezuYx4M4nFsZQVp";
    let recipient_account_key = "tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C";
    let expected_account_keys = vec![
        sender_account_key.to_string(),
        recipient_account_key.to_string(),
        SOL_SYSTEM_PROGRAM_KEY.to_string(),
    ];
    let expected_program_keys = vec![SOL_SYSTEM_PROGRAM_KEY.to_string()];
    let expected_instruction_hex = "020000006f00000000000000";
    let expected_amount_opt = Some("111".to_string());
    let expected_amount = expected_amount_opt.clone().unwrap();

    // All output checks
    assert_eq!(tx_metadata.signatures, vec![] as Vec<String>); // Check that there is an empty signature
    assert_eq!(tx_metadata.account_keys, expected_account_keys);
    assert_eq!(tx_metadata.program_keys, expected_program_keys);
    assert_eq!(tx_metadata.instructions.len(), 1);

    // Assert instructions array is correct
    let inst = &tx_metadata.instructions[0];
    assert_eq!(inst.program_key, SOL_SYSTEM_PROGRAM_KEY.to_string());
    assert_eq!(inst.accounts.len(), 2);
    assert_eq!(inst.accounts[0].account_key, sender_account_key);
    assert!(inst.accounts[0].signer);
    assert!(inst.accounts[0].writable);
    assert_eq!(inst.accounts[1].account_key, recipient_account_key);
    assert!(!inst.accounts[1].signer);
    assert!(inst.accounts[1].writable);
    assert_eq!(inst.instruction_data_hex, expected_instruction_hex);

    // Assert transfers array is correct
    assert_eq!(tx_metadata.transfers.len(), 1);
    assert_eq!(tx_metadata.transfers[0].amount, expected_amount);
    assert_eq!(tx_metadata.transfers[0].from, sender_account_key);
    assert_eq!(tx_metadata.transfers[0].to, recipient_account_key);
}

#[test]
fn parses_invalid_transactions() {
    // Invalid bytes, odd number length string
    let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, true, None);

    let inst_error_message = parsed_tx.unwrap_err().to_string(); // Unwrap the error
    assert_eq!(
        inst_error_message,
        "unsigned Solana transaction provided is invalid hex"
    );

    // Invalid length for Instruction Data Array
    let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, true, None);

    let inst_error_message = parsed_tx.unwrap_err().to_string(); // Convert to String
    assert_eq!(
    inst_error_message,
    "Error parsing full transaction. If this is just a message instead of a full transaction, parse using the --message flag. Parsing Error: \"Unsigned transaction provided is incorrectly formatted, error while parsing Instruction Data Array\""
);

    // Invalid length for Accounts Array
    let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001192b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, true, None);

    let inst_error_message = parsed_tx.unwrap_err().to_string(); // Unwrap the error
    assert_eq!(
            inst_error_message,
            "Error parsing full transaction. If this is just a message instead of a full transaction, parse using the --message flag. Parsing Error: \"Unsigned transaction provided is incorrectly formatted, error while parsing Accounts\""
        );
}

#[test]
fn parses_v0_transactions() {
    // You can also ensure that the output of this transaction makes sense yourself using the below references
    // Transaction reference: https://solscan.io/tx/4tkFaZQPGNYTBag6sNTawpBnAodqiBNF494y86s2qBLohQucW1AHRaq9Mm3vWTSxFRaUTmtdYp67pbBRz5RDoAdr
    // Address Lookup Table Account key: https://explorer.solana.com/address/6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy/entries

    // Invalid bytes, odd number length string
    let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100070ae05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc186a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b6600000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d616419cee70b839eb4eadd1411aa73eea6fd8700da5f0ea730136db1dd6fb2de660804000502c05c150004000903caa200000000000007060002000e03060101030200020c0200000080f0fa02000000000601020111070600010009030601010515060002010509050805100f0a0d01020b0c0011060524e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000060302000001090158b73fa66d1fb4a0562610136ebc84c7729542a8d792cb9bd2ad1bf75c30d5a404bdc2c1ba0497bcbbbf".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, true, None).unwrap();
    let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

    // verify that the signatures array contains a single placeholder signature
    let exp_signature = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    assert_eq!(transaction_metadata.signatures, vec![exp_signature]);

    verify_jupiter_message(transaction_metadata)
}

#[test]
fn parses_v0_transaction_message_only() {
    // NOTE: This is the same transaction as above however only the message is included, without the signatures array
    // You can also ensure that the output of this transaction makes sense yourself using the below references
    // Transaction reference: https://solscan.io/tx/4tkFaZQPGNYTBag6sNTawpBnAodqiBNF494y86s2qBLohQucW1AHRaq9Mm3vWTSxFRaUTmtdYp67pbBRz5RDoAdr
    // Address Lookup Table Account key: https://explorer.solana.com/address/6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy/entries

    // Invalid bytes, odd number length string
    let unsigned_payload = "800100070ae05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc186a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b6600000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d616419cee70b839eb4eadd1411aa73eea6fd8700da5f0ea730136db1dd6fb2de660804000502c05c150004000903caa200000000000007060002000e03060101030200020c0200000080f0fa02000000000601020111070600010009030601010515060002010509050805100f0a0d01020b0c0011060524e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000060302000001090158b73fa66d1fb4a0562610136ebc84c7729542a8d792cb9bd2ad1bf75c30d5a404bdc2c1ba0497bcbbbf".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_payload, false, None).unwrap();
    let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

    // verify that the signatures array is empty
    assert_eq!(transaction_metadata.signatures, vec![] as Vec<String>);

    verify_jupiter_message(transaction_metadata)
}

#[allow(clippy::too_many_lines)]
fn verify_jupiter_message(transaction_metadata: SolanaMetadata) {
    // All Expected accounts
    let signer_acct_key = "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp"; // Signer account key
    let usdc_mint_acct_key = "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8"; // USDC Mint account key
    let receiving_acct_key = "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw"; // receiving account key
    let compute_budget_acct_key = "ComputeBudget111111111111111111111111111111"; // compute budget program account key
    let jupiter_program_acct_key = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"; // Jupiter program account key
    let token_acct_key = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // token program account key
    let assoc_token_acct_key = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"; // associated token program account key
    let jupiter_event_authority_key = "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf"; // Jupiter aggregator event authority account key
    let usdc_acct_key = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC account key

    // All expected static account keys
    let expected_static_acct_keys = vec![
        signer_acct_key,
        usdc_mint_acct_key,
        receiving_acct_key,
        SOL_SYSTEM_PROGRAM_KEY,
        compute_budget_acct_key,
        jupiter_program_acct_key,
        token_acct_key,
        assoc_token_acct_key,
        jupiter_event_authority_key,
        usdc_acct_key,
    ];
    // all expected program account keys
    let expected_program_keys = vec![
        SOL_SYSTEM_PROGRAM_KEY,
        compute_budget_acct_key,
        jupiter_program_acct_key,
        token_acct_key,
        assoc_token_acct_key,
    ];

    // All expected full account objects
    let signer_acct = SolanaAccount {
        account_key: signer_acct_key.to_string(),
        signer: true,
        writable: true,
    };
    let usdc_mint_acct = SolanaAccount {
        account_key: usdc_mint_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let receiving_acct = SolanaAccount {
        account_key: receiving_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let system_program_acct = SolanaAccount {
        account_key: SOL_SYSTEM_PROGRAM_KEY.to_string(),
        signer: false,
        writable: false,
    };
    let jupiter_program_acct = SolanaAccount {
        account_key: jupiter_program_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let token_acct = SolanaAccount {
        account_key: token_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let jupiter_event_authority_acct = SolanaAccount {
        account_key: jupiter_event_authority_key.to_string(),
        signer: false,
        writable: false,
    };
    let usdc_acct = SolanaAccount {
        account_key: usdc_acct_key.to_string(),
        signer: false,
        writable: false,
    };

    let lookup_table_key = "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy";

    // Assert that accounts and programs are correct
    assert_eq!(expected_static_acct_keys, transaction_metadata.account_keys);
    assert_eq!(expected_program_keys, transaction_metadata.program_keys);

    // Assert ALL instructions as expected --> REFERENCE: https://solscan.io/tx/4tkFaZQPGNYTBag6sNTawpBnAodqiBNF494y86s2qBLohQucW1AHRaq9Mm3vWTSxFRaUTmtdYp67pbBRz5RDoAdr

    // Instruction 1 -- SetComputeUnitLimit
    let exp_instruction_1 = SolanaInstruction {
        program_key: compute_budget_acct_key.to_string(),
        accounts: vec![],
        address_table_lookups: vec![],
        instruction_data_hex: "02c05c1500".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_1, transaction_metadata.instructions[0]);

    // Instruction 2 -- SetComputeUnitPrice
    let exp_instruction_2 = SolanaInstruction {
        program_key: compute_budget_acct_key.to_string(),
        accounts: vec![],
        address_table_lookups: vec![],
        instruction_data_hex: "03caa2000000000000".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_2, transaction_metadata.instructions[1]);

    // Instruction 3 - CreateIdempotent
    let exp_instruction_3 = SolanaInstruction {
        program_key: assoc_token_acct_key.to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
        accounts: vec![
            signer_acct.clone(),
            receiving_acct.clone(),
            signer_acct.clone(),
            system_program_acct.clone(),
            token_acct.clone(),
        ],
        address_table_lookups: vec![SolanaSingleAddressTableLookup {
            address_table_key: lookup_table_key.to_string(),
            index: 151,
            writable: false,
        }],
        instruction_data_hex: "01".to_string(),
    };
    assert_eq!(exp_instruction_3, transaction_metadata.instructions[2]);

    // Instruction 4 -- This is a basic SOL transfer
    let exp_instruction_4 = SolanaInstruction {
        program_key: SOL_SYSTEM_PROGRAM_KEY.to_string(),
        accounts: vec![signer_acct.clone(), receiving_acct.clone()],
        address_table_lookups: vec![],
        instruction_data_hex: "0200000080f0fa0200000000".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_4, transaction_metadata.instructions[3]);

    // Instruction 5 -- SyncNative
    let exp_instruction_5 = SolanaInstruction {
        program_key: token_acct_key.to_string(),
        accounts: vec![receiving_acct.clone()],
        address_table_lookups: vec![],
        instruction_data_hex: "11".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_5, transaction_metadata.instructions[4]);

    // Instruction 6 -- CreateIdempotent
    let exp_instruction_6 = SolanaInstruction {
        program_key: assoc_token_acct_key.to_string(),
        accounts: vec![
            signer_acct.clone(),
            usdc_mint_acct.clone(),
            signer_acct.clone(),
            usdc_acct.clone(),
            system_program_acct.clone(),
            token_acct.clone(),
        ],
        address_table_lookups: vec![],
        instruction_data_hex: "01".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_6, transaction_metadata.instructions[5]);

    // Instruction 7 Jupiter Aggregator V6 Route
    let mut lookups_7: Vec<SolanaSingleAddressTableLookup> = vec![];
    let lookups_7_inds: Vec<i32> = vec![187, 188, 189, 186, 194, 193, 191];
    let lookups_7_writable: Vec<bool> = vec![false, false, true, true, true, true, false];
    for i in 0..lookups_7_inds.len() {
        lookups_7.push(SolanaSingleAddressTableLookup {
            address_table_key: lookup_table_key.to_string(),
            index: lookups_7_inds[i],
            writable: lookups_7_writable[i],
        });
    }
    let exp_instruction_7 = &transaction_metadata.instructions[6];
    assert_eq!(
        exp_instruction_7.program_key,
        jupiter_program_acct_key.to_string()
    );
    assert_eq!(
        exp_instruction_7.accounts,
        vec![
            token_acct.clone(),
            signer_acct.clone(),
            receiving_acct.clone(),
            usdc_mint_acct.clone(),
            jupiter_program_acct.clone(),
            usdc_acct.clone(),
            jupiter_program_acct.clone(),
            jupiter_event_authority_acct.clone(),
            jupiter_program_acct.clone(),
            usdc_mint_acct.clone(),
            receiving_acct.clone(),
            signer_acct.clone(),
            token_acct.clone(),
            jupiter_program_acct.clone(),
        ]
    );
    assert_eq!(exp_instruction_7.address_table_lookups, lookups_7);
    assert_eq!(
        exp_instruction_7.instruction_data_hex,
        "e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000"
    );
    // Verify Jupiter IDL was parsed with correct metadata
    let parsed = exp_instruction_7
        .parsed_instruction
        .as_ref()
        .expect("Jupiter instruction should be parsed");
    assert_eq!(parsed.instruction_name, "route");
    assert!(matches!(
        parsed.idl_source,
        IdlSource::BuiltIn(ProgramType::JupiterAggregatorV6)
    ));

    // Instruction 8 -- CloseAccount
    let exp_instruction_8 = SolanaInstruction {
        program_key: token_acct_key.to_string(),
        accounts: vec![
            receiving_acct.clone(),
            signer_acct.clone(),
            signer_acct.clone(),
        ],
        address_table_lookups: vec![],
        instruction_data_hex: "09".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_8, transaction_metadata.instructions[7]);

    // ASSERT top level transfers array
    let exp_transf_arr: Vec<SolTransfer> = vec![SolTransfer {
        amount: "50000000".to_string(),
        to: receiving_acct_key.to_string(),
        from: signer_acct_key.to_string(),
    }];
    assert_eq!(exp_transf_arr, transaction_metadata.transfers);

    // ASSERT Address table lookups
    let exp_lookups: Vec<SolanaAddressTableLookup> = vec![SolanaAddressTableLookup {
        address_table_key: lookup_table_key.to_string(),
        writable_indexes: vec![189, 194, 193, 186],
        readonly_indexes: vec![151, 188, 187, 191],
    }];
    assert_eq!(exp_lookups, transaction_metadata.address_table_lookups);
}

#[test]
#[allow(clippy::too_many_lines)]
fn parses_valid_v0_transaction_with_complex_address_table_lookups() {
    // You can also ensure that the output of this transaction makes sense yourself using the below references
    // Transaction reference: https://solscan.io/tx/5onMRTbaqb1xjotSedpvSzS4bzPzRSYFwoBb5Rc7qaVwYUy2o9nNE59j8p23zkMBDTd7JRZ4phMfkPz6VikDW32P
    // Address Lookup Table Accounts:
    // https://solscan.io/account/BkAbXZuNv1prbDh5q6HAQgkGgkX14UpBSfDnuLHKoQho
    // https://solscan.io/account/3yg3PND9XDBd7VnZAoHXFRvyFfjPzR8RNb1G1AS9GwH6

    let unsigned_transaction = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100090fe05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc115376f3f97590a9c65d068b64e24a1f0f3ab9798c17fdddc38bf54d15ab56df477047a381c391538f7a3ba42bafe841d453f26d52e71a66443f6af1edd748afd86a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b66e9d4488b07fe399b1a9155e5821b697d43016c0a3c4f3bbca2afb41d0163305700000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f0000000000106ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859ac1ae3d087f29237062548f70c4c04aec2a995694986e7cbb467520621d38630b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61f0686da7719b0fd854cbc86dd72ec0c438b509b1e57ad61ea9dc8de9efbbcdba0707000502605f04000700090327530500000000000b0600040009060a0101060200040c0200000080969800000000000a0104011108280a0c0004020503090e08080d081d180201151117131614100f120c1c0a08081e1f190c01051a1b0a29c1209b3341d69c810502000000136400011c016401028096980000000000b2a31700000000006400000a030400000109029fa3b18857ed4adbd196e5fa77c76029c0ea1084a9671d2ad0643a027d29ad8a0a410104400705021103090214002c3c0b092d97db350aa90b53afe1d13d3a5b6ff46c97be630ca2779983794df503fbfeff02fdfc".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let transaction_metadata = parsed_tx.transaction_metadata().unwrap();
    let parsed_tx_sigs = transaction_metadata.signatures;
    assert_eq!(1, parsed_tx_sigs.len());
    assert_eq!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(), parsed_tx_sigs[0]);

    // All Expected accounts
    let signer_acct_key = "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp"; // Signer account key
    let pyth_intermediate_acct_key = "2Rpb7v4vNS6d4k936hVA3176BW3yxqvrpXx21C8tY9xw"; // PYTH account key
    let wsol_intermediate_acct_key = "91bUbswo6Di8235jAPwim1At4cPZLbG2pkpneyqKg4NQ"; // WSOL account key
    let usdc_destination_acct_key = "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8"; // USDC account key
    let wsol_mint_acct_key = "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw"; // WSOL Mint account key
    let usdc_intermediate_acct_key = "Gjmjory7TWKJXD2Jc6hKzAG991wWutFhtbXudzJqgx3p"; // USDC account key
    let compute_budget_acct_key = "ComputeBudget111111111111111111111111111111"; // compute budget program account key
    let jupiter_program_acct_key = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"; // Jupiter program account key
    let wsol_acct_key = "So11111111111111111111111111111111111111112"; // WSOL program account key
    let token_acct_key = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // token program account key
    let assoc_token_acct_key = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"; // associated token program account key
    let jupiter_event_authority_key = "CapuXNQoDviLvU1PxFiizLgPNQCxrsag1uMeyk6zLVps"; // Jupiter aggregator event authority account key
    let jupiter_aggregator_authority_key = "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf"; // Jupiter aggregator authority account key
    let usdc_acct_key = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC account key

    // All expected static account keys
    let expected_static_acct_keys = vec![
        signer_acct_key,
        pyth_intermediate_acct_key,
        wsol_intermediate_acct_key,
        usdc_destination_acct_key,
        wsol_mint_acct_key,
        usdc_intermediate_acct_key,
        SOL_SYSTEM_PROGRAM_KEY,
        compute_budget_acct_key,
        jupiter_program_acct_key,
        wsol_acct_key,
        token_acct_key,
        assoc_token_acct_key,
        jupiter_event_authority_key,
        jupiter_aggregator_authority_key,
        usdc_acct_key,
    ];
    // all expected program account keys
    let expected_program_keys = vec![
        SOL_SYSTEM_PROGRAM_KEY,
        compute_budget_acct_key,
        jupiter_program_acct_key,
        token_acct_key,
        assoc_token_acct_key,
    ];

    // All expected full account objects
    let signer_acct = SolanaAccount {
        account_key: signer_acct_key.to_string(),
        signer: true,
        writable: true,
    };
    let usdc_destination_acct = SolanaAccount {
        account_key: usdc_destination_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let system_program_acct = SolanaAccount {
        account_key: SOL_SYSTEM_PROGRAM_KEY.to_string(),
        signer: false,
        writable: false,
    };
    let jupiter_program_acct = SolanaAccount {
        account_key: jupiter_program_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let token_acct = SolanaAccount {
        account_key: token_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let jupiter_event_authority_acct: SolanaAccount = SolanaAccount {
        account_key: jupiter_event_authority_key.to_string(),
        signer: false,
        writable: false,
    };
    let jupiter_aggregator_authority_acct = SolanaAccount {
        account_key: jupiter_aggregator_authority_key.to_string(),
        signer: false,
        writable: false,
    };
    let usdc_acct = SolanaAccount {
        account_key: usdc_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let wsol_acct = SolanaAccount {
        account_key: wsol_acct_key.to_string(),
        signer: false,
        writable: false,
    };
    let wsol_mint_acct = SolanaAccount {
        account_key: wsol_mint_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let pyth_intermediate_acct = SolanaAccount {
        account_key: pyth_intermediate_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let wsol_intermediate_acct = SolanaAccount {
        account_key: wsol_intermediate_acct_key.to_string(),
        signer: false,
        writable: true,
    };
    let usdc_intermediate_acct = SolanaAccount {
        account_key: usdc_intermediate_acct_key.to_string(),
        signer: false,
        writable: true,
    };

    let lookup_table_key_1 = "BkAbXZuNv1prbDh5q6HAQgkGgkX14UpBSfDnuLHKoQho";
    let lookup_table_key_2 = "3yg3PND9XDBd7VnZAoHXFRvyFfjPzR8RNb1G1AS9GwH6";

    // Assert that accounts and programs are correct
    assert_eq!(expected_static_acct_keys, transaction_metadata.account_keys);
    assert_eq!(expected_program_keys, transaction_metadata.program_keys);

    // Assert ALL instructions as expected --> REFERENCE: https://solscan.io/tx/5onMRTbaqb1xjotSedpvSzS4bzPzRSYFwoBb5Rc7qaVwYUy2o9nNE59j8p23zkMBDTd7JRZ4phMfkPz6VikDW32P

    // Assert expected number of instructions
    assert_eq!(7, transaction_metadata.instructions.len());

    // Instruction 1 -- SetComputeUnitLimit
    let exp_instruction_1 = SolanaInstruction {
        program_key: compute_budget_acct_key.to_string(),
        accounts: vec![],
        address_table_lookups: vec![],
        instruction_data_hex: "02605f0400".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_1, transaction_metadata.instructions[0]);

    // Instruction 2 -- SetComputeUnitPrice
    let exp_instruction_2 = SolanaInstruction {
        program_key: compute_budget_acct_key.to_string(),
        accounts: vec![],
        address_table_lookups: vec![],
        instruction_data_hex: "032753050000000000".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_2, transaction_metadata.instructions[1]);

    // Instruction 3 - CreateIdempotent
    let exp_instruction_3 = SolanaInstruction {
        program_key: assoc_token_acct_key.to_string(),
        accounts: vec![
            signer_acct.clone(),
            wsol_mint_acct.clone(),
            signer_acct.clone(),
            wsol_acct.clone(),
            system_program_acct.clone(),
            token_acct.clone(),
        ],
        address_table_lookups: vec![],
        instruction_data_hex: "01".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_3, transaction_metadata.instructions[2]);

    // Instruction 4 -- This is a basic SOL transfer
    let exp_instruction_4 = SolanaInstruction {
        program_key: SOL_SYSTEM_PROGRAM_KEY.to_string(),
        accounts: vec![signer_acct.clone(), wsol_mint_acct.clone()],
        address_table_lookups: vec![],
        instruction_data_hex: "020000008096980000000000".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_4, transaction_metadata.instructions[3]);

    // Instruction 5 -- SyncNative
    let exp_instruction_5 = SolanaInstruction {
        program_key: token_acct_key.to_string(),
        accounts: vec![wsol_mint_acct.clone()],
        address_table_lookups: vec![],
        instruction_data_hex: "11".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_5, transaction_metadata.instructions[4]);

    // Instruction 6 -- Jupiter Aggregator v6: sharedAccountsRoute
    let exp_instruction_6 = &transaction_metadata.instructions[5];
    assert_eq!(
        exp_instruction_6.program_key,
        jupiter_program_acct_key.to_string()
    );
    assert_eq!(
        exp_instruction_6.accounts,
        vec![
            token_acct.clone(),
            jupiter_event_authority_acct.clone(),
            signer_acct.clone(),
            wsol_mint_acct.clone(),
            wsol_intermediate_acct.clone(),
            usdc_intermediate_acct.clone(),
            usdc_destination_acct.clone(),
            wsol_acct.clone(),
            usdc_acct.clone(),
            jupiter_program_acct.clone(),
            jupiter_program_acct.clone(),
            jupiter_aggregator_authority_acct.clone(),
            jupiter_program_acct.clone(),
            wsol_intermediate_acct.clone(),
            pyth_intermediate_acct.clone(),
            jupiter_event_authority_acct.clone(),
            token_acct.clone(),
            jupiter_program_acct.clone(),
            jupiter_program_acct.clone(),
            jupiter_event_authority_acct.clone(),
            pyth_intermediate_acct.clone(),
            usdc_intermediate_acct.clone(),
            token_acct.clone(),
        ]
    );
    assert_eq!(
        exp_instruction_6.instruction_data_hex,
        "c1209b3341d69c810502000000136400011c016401028096980000000000b2a3170000000000640000"
    );
    assert_eq!(
        exp_instruction_6.address_table_lookups,
        vec![
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 0,
                writable: false,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 9,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 2,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 4,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 3,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 7,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 17,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 5,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 1,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 65,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 64,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_1.to_string(),
                index: 20,
                writable: false,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_2.to_string(),
                index: 253,
                writable: false,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_2.to_string(),
                index: 252,
                writable: false,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_2.to_string(),
                index: 251,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_2.to_string(),
                index: 254,
                writable: true,
            },
            SolanaSingleAddressTableLookup {
                address_table_key: lookup_table_key_2.to_string(),
                index: 255,
                writable: true,
            },
        ]
    );
    // Verify Jupiter IDL was parsed
    assert!(exp_instruction_6.parsed_instruction.is_some());
    let parsed = exp_instruction_6.parsed_instruction.as_ref().unwrap();
    assert_eq!(parsed.instruction_name, "shared_accounts_route");
    assert_eq!(parsed.discriminator, "c1209b3341d69c81");
    assert!(matches!(
        parsed.idl_source,
        IdlSource::BuiltIn(ProgramType::JupiterAggregatorV6)
    ));

    // Instruction 7 -- Close Account
    let exp_instruction_7: SolanaInstruction = SolanaInstruction {
        program_key: token_acct_key.to_string(),
        accounts: vec![
            wsol_mint_acct.clone(),
            signer_acct.clone(),
            signer_acct.clone(),
        ],
        address_table_lookups: vec![],
        instruction_data_hex: "09".to_string(),
        parsed_instruction: None,
        idl_parse_error: None,
    };
    assert_eq!(exp_instruction_7, transaction_metadata.instructions[6]);

    // ASSERT top level transfers array
    let exp_transf_arr: Vec<SolTransfer> = vec![SolTransfer {
        amount: "10000000".to_string(),
        to: wsol_mint_acct_key.to_string(),
        from: signer_acct_key.to_string(),
    }];
    assert_eq!(exp_transf_arr, transaction_metadata.transfers);

    // ASSERT Address table lookups
    let exp_lookups: Vec<SolanaAddressTableLookup> = vec![
        SolanaAddressTableLookup {
            address_table_key: lookup_table_key_1.to_string(),
            writable_indexes: vec![65, 1, 4, 64, 7, 5, 2, 17, 3, 9],
            readonly_indexes: vec![20, 0],
        },
        SolanaAddressTableLookup {
            address_table_key: lookup_table_key_2.to_string(),
            writable_indexes: vec![251, 254, 255],
            readonly_indexes: vec![253, 252],
        },
    ];
    assert_eq!(exp_lookups, transaction_metadata.address_table_lookups);
}

#[test]
fn parses_transaction_with_multi_byte_compact_array_header() {
    // The purpose of this test is to ensure that transactions with compact array headers that are multiple bytes long are parsed correctly
    // multiple byte array headers are possible based on the compact-u16 format as described in solana documentation here: https://solana.com/docs/core/transactions#compact-array-format

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100071056837517cb604056d3d10dca4553663be1e7a8f0cb7a78abd50862eb2073fbd827a00dfa20ba5511ba322e07293a47397c9e842de88aa4d359ff9a0073f88217740218a08252e747966f313cef860d86d095a76f033098f0cb383d4a5078cc8dedfee61094ac6637619b7c78339527ef0a4460a9e32a0f37fda8c68aea1b751dd39306efe4d9bbb93cfa6c484c1016bb7a52fe3feeca3157d7d0791a25f345798b18611a5b9ca7a6a37eac499749d95233f53f18ec5e692915e2582ade72d68962bedcf3cccf19cbf35daaa34926be22b1fc6bfc7a0938bbb6ee5593046168974592a5996b45dcf0f07ef85b77388f204a784bbf8b212806048c3f9276485de4c353609a6762251896323d9bcd3e70b7bf0ddb03ff381afd5601e994ab9b5f9c0306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a90479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e80d0720fe448de59d8811e24d6df917dc8d0d98b392ddf4dd2b622a747a60fded9b48fc124b1d8ff29225062e50ea775462ef424c3c21bda18e3bf47b835bbdd90909000502c027090009000903098b0200000000000a06000100150b0c01010b0200010c0200000000e1f505000000000c010101110a06000200160b0c01010d1d0c0001020d160d0e0d1710171112010215161317000c0c18170304050d23e517cb977ae3ad2a010000002664000100e1f505000000006d2d4a01000000002100000c0301000001090f0c001916061a020708140b0c0a8f02828362be28ce44327e16490100000000000000000000000000000000000000000000000000000000000000000000210514000000532f27101965dd16442e59d40670faf5ebb142e40000000000000000000000000000000000000000000000075858938cec63c6b3140000009528cf48a8deb982b5549d72abbb764ffdbce3010056837517cb604056d3d10dca4553663be1e7a8f0cb7a78abd50862eb2073fbd800140000009528cf48a8deb982b5549d72abbb764ffdbce301000001000000009a06e62b93010000420000000101000000d831640000000000000000000000000000000000b3c663ec8c935858070000000000000000000000000000000000000000000000000000000000000000026f545fe588dd627fb93f2295f47652ccd56feab015ec282c500bf33679e3b3d10423222928042b2a26256a88a76573c8d9d435fad46f194977a3aead561e0c01a6d9b5873c9f05e4dd8e010302020c".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

    // sanity check signatures
    let parsed_tx_sigs = transaction_metadata.signatures;
    assert_eq!(1, parsed_tx_sigs.len());
    assert_eq!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(), parsed_tx_sigs[0]);
}

#[test]
fn parse_spl_token_transfer() {
    // The below transaction hex involves two instructions, one to the system program which is irrelevant for this test, and an SPL token transfer described below.

    // The below transaction is an SPL token transfer of the following type:
    // - A full transaction (legacy message)
    // - Calls the original Token Program (NOT 2022)
    // - Calls the TransferCheckedWithFee instruction
    // - Not a mutisig owner

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000307533b5b0116e5bd434b30300c28f3814712637545ae345cc63d2f23709c75894d3bcae0fb76cc461d85bd05a078f887cf646fd27011e12edaaeb5091cdb976044a1460dfb457c122a8fe4d4c180b21a6078e67ea08c271acfd1b7ff3d88a2bbf4ca107ce11d55b05bdb209feaeeac8120fea5598cabbf91df2862fc36c5cf83a2000000000000000000000000000000000000000000000000000000000000000006a7d517192c568ee08a845f73d29788cf035c3145b21ab344d8062ea940000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9eefd656548c17a30f2d97998a7ec413e2304464841f817bfc5c73c2c9a36bf6f020403020500040400000006030301000903a086010000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // initialize expected values
    let exp_from = "EbmwLZmuugxuQb8ksm4TBXf2qPbSK8N4uxNmakvRaUyX";
    let exp_to = "52QUutfwWMDDVNZSjovpmtD1ZmMe3Uf3n1ENE7JgBMkP";
    let exp_owner = "6buLKuZFhVNtAFkyRituTZNNVyjHSYLx4NyfD8cKr1uW";
    let exp_amount = "100000";

    // Test assertions for SPL Transfer fields
    let spl_transfer = &tx_metadata.spl_transfers[0];
    assert_eq!(spl_transfer.from, exp_from);
    assert_eq!(spl_transfer.to, exp_to);
    assert_eq!(spl_transfer.owner, exp_owner);
    assert_eq!(spl_transfer.amount, exp_amount);
    assert_eq!(spl_transfer.signers, Vec::<String>::new());
    assert_eq!(spl_transfer.decimals, None);
    assert_eq!(spl_transfer.token_mint, None);
    assert_eq!(spl_transfer.fee, None);

    // Test Program called in the instruction
    assert_eq!(tx_metadata.instructions[1].program_key, TOKEN_PROGRAM_KEY);
    assert_eq!(
        tx_metadata.instructions[1].instruction_data_hex,
        "03a086010000000000"
    )
}

#[test]
fn parse_spl_token_22_transfer_checked_with_fee() {
    // The below transaction is an SPL token transfer of the following type:
    // - A full transaction (legacy message)
    // - Calls Token Program 2022
    // - Calls the TransferCheckedWithFee instruction
    // - Not a mutisig owner

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000205864624d78f936e02c49acfd0320a66b8baec813f00df938ed2505b1242504fa9e3db1d9522e05705cf23ac1d3f5a1db2ef9f23ff78d7fcf699da1cf4902463263bcae0fb76cc461d85bd05a078f887cf646fd27011e12edaaeb5091cdb97604406ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfcbc07c56e60ad3d3f177382eac6548fba1fd32cfd90ca02b3e7cfa185fdce7398b97a42135e0503573230dfadebb740b6e206b513208e90a489f2b46684462bc801030401040200131a0100ca9a3b00000000097b00000000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, false, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // Initialize expected value
    let exp_from = "GLTLPbA1XJctLCsaErmbzgouaLsLm2CLGzWyi8xangNq";
    let exp_to = "52QUutfwWMDDVNZSjovpmtD1ZmMe3Uf3n1ENE7JgBMkP";
    let exp_owner = "A39fhEiRvz4YsSrrpqU8z3zF6n1t9S48CsDjL2ibDFrx";
    let exp_amount = "1000000000";
    let exp_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
    let exp_decimals = "9";
    let exp_fee = "123";

    // Test assertions for SPL Transfer fields
    let spl_transfer = &tx_metadata.spl_transfers[0];
    assert_eq!(spl_transfer.from, exp_from);
    assert_eq!(spl_transfer.to, exp_to);
    assert_eq!(spl_transfer.owner, exp_owner);
    assert_eq!(spl_transfer.amount, exp_amount);
    assert_eq!(spl_transfer.signers, Vec::<String>::new());
    assert_eq!(spl_transfer.decimals, Some(exp_decimals.to_string()));
    assert_eq!(spl_transfer.token_mint, Some(exp_mint.to_string()));
    assert_eq!(spl_transfer.fee, Some(exp_fee.to_string()));

    // Test Program called in the instruction
    assert_eq!(
        tx_metadata.instructions[0].program_key,
        TOKEN_2022_PROGRAM_KEY
    )
}

#[test]
fn parse_spl_token_program_tranfer_multiple_signers() {
    // The below Transaction is an SPL token transfer of the following type:
    // - Versioned Transaction Message
    // - Calls the original Token Program (NOT 2022)
    // - Calls the simple Transfer instruction
    // - Mutlisig owner with 2 signers

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "8003020106864624d78f936e02c49acfd0320a66b8baec813f00df938ed2505b1242504fa98b2e0a1e9310dc03bfc0432ac8c9f290d15cbc57b2ed367f43aeefc28c7a4d7a5078df268c218e5c9ebe650a7f90c8879bba318b35ce9046cb505b7ed5724a9de3db1d9522e05705cf23ac1d3f5a1db2ef9f23ff78d7fcf699da1cf4902463263bcae0fb76cc461d85bd05a078f887cf646fd27011e12edaaeb5091cdb97604406ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9b97a42135e0503573230dfadebb740b6e206b513208e90a489f2b46684462bc80105050304000102090300ca9a3b0000000000".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, false, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // Initialize expected value
    let exp_from = "GLTLPbA1XJctLCsaErmbzgouaLsLm2CLGzWyi8xangNq";
    let exp_to = "52QUutfwWMDDVNZSjovpmtD1ZmMe3Uf3n1ENE7JgBMkP";
    let exp_owner = "A39fhEiRvz4YsSrrpqU8z3zF6n1t9S48CsDjL2ibDFrx";
    let exp_signer_1 = "ANJPUpqXC1Qn8uhHVXLTsRKjving6kPfjCATJzg7EJjB";
    let exp_signer_2 = "6R8WtdoanEVNJfkeGfbQDMsCrqeHE1sGXjsReJsSbmxQ";
    let exp_amount = "1000000000";

    // Test assertions for SPL Transfer fields
    let spl_transfer = &tx_metadata.spl_transfers[0];
    assert_eq!(spl_transfer.from, exp_from);
    assert_eq!(spl_transfer.to, exp_to);
    assert_eq!(spl_transfer.owner, exp_owner);
    assert_eq!(spl_transfer.amount, exp_amount);
    assert_eq!(spl_transfer.signers, vec![exp_signer_1, exp_signer_2]);
    assert_eq!(spl_transfer.decimals, None);
    assert_eq!(spl_transfer.token_mint, None);
    assert_eq!(spl_transfer.fee, None);

    // Test Program called in the instruction
    assert_eq!(tx_metadata.instructions[0].program_key, TOKEN_PROGRAM_KEY)
}

#[test]
fn parse_spl_token_program_2022_transfer_checked_multiple_signers() {
    // The below Transaction is an SPL token transfer of the following type:
    // - Versioned Transaction Message
    // - Calls Token Program 2022
    // - Calls the TransferChecked instruction
    // - Mutlisig owner with 2 signers

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "8003020207864624d78f936e02c49acfd0320a66b8baec813f00df938ed2505b1242504fa98b2e0a1e9310dc03bfc0432ac8c9f290d15cbc57b2ed367f43aeefc28c7a4d7a5078df268c218e5c9ebe650a7f90c8879bba318b35ce9046cb505b7ed5724a9de3db1d9522e05705cf23ac1d3f5a1db2ef9f23ff78d7fcf699da1cf4902463263bcae0fb76cc461d85bd05a078f887cf646fd27011e12edaaeb5091cdb97604406ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfcbc07c56e60ad3d3f177382eac6548fba1fd32cfd90ca02b3e7cfa185fdce7398b97a42135e0503573230dfadebb740b6e206b513208e90a489f2b46684462bc80105060306040001020a0c00ca9a3b000000000900".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, false, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    // Initialize expected value
    let exp_from = "GLTLPbA1XJctLCsaErmbzgouaLsLm2CLGzWyi8xangNq";
    let exp_to = "52QUutfwWMDDVNZSjovpmtD1ZmMe3Uf3n1ENE7JgBMkP";
    let exp_owner = "A39fhEiRvz4YsSrrpqU8z3zF6n1t9S48CsDjL2ibDFrx";
    let exp_signer_1 = "ANJPUpqXC1Qn8uhHVXLTsRKjving6kPfjCATJzg7EJjB";
    let exp_signer_2 = "6R8WtdoanEVNJfkeGfbQDMsCrqeHE1sGXjsReJsSbmxQ";
    let exp_amount = "1000000000";
    let exp_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
    let exp_decimals = 9;

    // Test assertions for SPL Transfer fields
    let spl_transfer = &tx_metadata.spl_transfers[0];
    assert_eq!(spl_transfer.from, exp_from);
    assert_eq!(spl_transfer.to, exp_to);
    assert_eq!(spl_transfer.owner, exp_owner);
    assert_eq!(spl_transfer.amount, exp_amount);
    assert_eq!(spl_transfer.signers, vec![exp_signer_1, exp_signer_2]);
    assert_eq!(spl_transfer.decimals, Some(exp_decimals.to_string()));
    assert_eq!(spl_transfer.token_mint, Some(exp_mint.to_string()));
    assert_eq!(spl_transfer.fee, None);

    // Test Program called in the instruction
    assert_eq!(
        tx_metadata.instructions[0].program_key,
        TOKEN_2022_PROGRAM_KEY
    )
}

#[test]
fn parse_spl_transfer_using_address_table_lookups() {
    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001000b10b9334994c55889c1e129158c59a9b3b16fd9bfc9bedd105a8e1d7b7a8644110772f445b3a19ac048d2a928fe0774cf7b8b5efa7c6457cbccbc82ecf0eac93c792343cde9faec81dfd6963f83ea57e8075f2db9eb0c461d195737e143f9b16909c52568e818f6871d033a00dba9ae878df8ba008104e34fb0332d685f3eacdf6a5149b5337cf8079ab25763ae8e8f95a9b09d2325dcc2ee5f8e8640b7eacf470d283d0dd282354fef0ae3b0e227d37cd89ca266fb17ddf8f7cb7ccefbe4ebdc5506a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000d1a3910dca452ccc0c6d513e570b0a5cee7edf44fa74e1410cd405fba63e96100306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f8591e8c4fab8994494c8f1e5c1287445b2917d60c43c79aa959162f5d6000598d32000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a92ccd355fe72bcf08d5ee763f52bb9603e025ef8e1d0340f28a576313251507310479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8ee501f6575c6376b0fc00c38a8f474ed66466d3cc3bf159e8d2be46427a83a9c0a08000903a8d002000000000008000502e7e1060005020607090022bb6ad79d0c1600090600010a130b0c01010c04021301000a0c9c0100000000000006090600030d130b0c01010c04021303000a0c4603000000000000060906000400140b0c01010e120c0002040e140e0f0e150010111204020c1624e517cb977ae3ad2a010000003d016400013e9c070000000000c6c53a0000000000e803000c030400000109015de6c0e5b44625227af5ec45b683057e191d6d7bf7ff43e3d25f31d5d5e81dac03b86fba04c013b970".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let _ = parsed_tx.transaction_metadata().unwrap();
}

#[test]
fn parse_spl_transfer_using_address_table_lookups_mint() {
    // This transaction contains two SPL transfer instructions
    // BOTH Spl transfer instructions use an Address Table look up to represent the Token Mint address
    // The parser will return the flag ADDRESS_TABLE_LOOKUP for these non statically included addresses

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001000b10b9334994c55889c1e129158c59a9b3b16fd9bfc9bedd105a8e1d7b7a8644110772f445b3a19ac048d2a928fe0774cf7b8b5efa7c6457cbccbc82ecf0eac93c792343cde9faec81dfd6963f83ea57e8075f2db9eb0c461d195737e143f9b16909c52568e818f6871d033a00dba9ae878df8ba008104e34fb0332d685f3eacdf6a5149b5337cf8079ab25763ae8e8f95a9b09d2325dcc2ee5f8e8640b7eacf470d283d0dd282354fef0ae3b0e227d37cd89ca266fb17ddf8f7cb7ccefbe4ebdc5506a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000d1a3910dca452ccc0c6d513e570b0a5cee7edf44fa74e1410cd405fba63e96100306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f8591e8c4fab8994494c8f1e5c1287445b2917d60c43c79aa959162f5d6000598d32000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a92ccd355fe72bcf08d5ee763f52bb9603e025ef8e1d0340f28a576313251507310479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8ee501f6575c6376b0fc00c38a8f474ed66466d3cc3bf159e8d2be46427a83a9c0a08000903a8d002000000000008000502e7e1060005020607090022bb6ad79d0c1600090600010a130b0c01010c04021301000a0c9c0100000000000006090600030d130b0c01010c04021303000a0c4603000000000000060906000400140b0c01010e120c0002040e140e0f0e150010111204020c1624e517cb977ae3ad2a010000003d016400013e9c070000000000c6c53a0000000000e803000c030400000109015de6c0e5b44625227af5ec45b683057e191d6d7bf7ff43e3d25f31d5d5e81dac03b86fba04c013b970".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    let spl_transfers = tx_metadata.spl_transfers;

    // SPL transfer 1 (Uses an address table lookup token mint address)
    let spl_transfer_1 = spl_transfers[0].clone();
    assert_eq!(
        spl_transfer_1.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_1.to,
        "8jjWmLhYdqrtFMcEkiMDdqkEN85cvEFnNS4LFgNf5NRv".to_string()
    );
    assert_eq!(
        spl_transfer_1.token_mint,
        Some("ADDRESS_TABLE_LOOKUP".to_string())
    ); // EMPTY BECAUSE OF ATLU
    assert_eq!(
        spl_transfer_1.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );

    // SPL transfer 2 (Uses an address table lookup token mint address)
    let spl_transfer_2 = spl_transfers[1].clone();
    assert_eq!(
        spl_transfer_2.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_2.to,
        "EGaQJtKJ4zctDY4VjP98f2KDtEeeso8JGk5a4E8X1EaV".to_string()
    );
    assert_eq!(
        spl_transfer_2.token_mint,
        Some("ADDRESS_TABLE_LOOKUP".to_string())
    ); // EMPTY BECAUSE OF ATLU
    assert_eq!(
        spl_transfer_2.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );
}

#[test]
fn parse_spl_transfer_using_address_table_lookups_recipient() {
    // This transaction contains two SPL transfer instructions
    // 1. Uses an Address Table look up to represent the Token Mint address
    // 2. Uses an Address Table look up to represent the Recipient address
    // The parser will return the flag ADDRESS_TABLE_LOOKUP for these non statically included addresses

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001000b10b9334994c55889c1e129158c59a9b3b16fd9bfc9bedd105a8e1d7b7a8644110772f445b3a19ac048d2a928fe0774cf7b8b5efa7c6457cbccbc82ecf0eac93c792343cde9faec81dfd6963f83ea57e8075f2db9eb0c461d195737e143f9b16909c52568e818f6871d033a00dba9ae878df8ba008104e34fb0332d685f3eacdf6a5149b5337cf8079ab25763ae8e8f95a9b09d2325dcc2ee5f8e8640b7eacf470d283d0dd282354fef0ae3b0e227d37cd89ca266fb17ddf8f7cb7ccefbe4ebdc5506a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000d1a3910dca452ccc0c6d513e570b0a5cee7edf44fa74e1410cd405fba63e96100306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f8591e8c4fab8994494c8f1e5c1287445b2917d60c43c79aa959162f5d6000598d32000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a92ccd355fe72bcf08d5ee763f52bb9603e025ef8e1d0340f28a576313251507310479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8ee501f6575c6376b0fc00c38a8f474ed66466d3cc3bf159e8d2be46427a83a9c0a08000903a8d002000000000008000502e7e1060005020607090022bb6ad79d0c1600090600010a130b0c01010c04021301000a0c9c0100000000000006090600030d130b0c01010c04020313000a0c4603000000000000060906000400140b0c01010e120c0002040e140e0f0e150010111204020c1624e517cb977ae3ad2a010000003d016400013e9c070000000000c6c53a0000000000e803000c030400000109015de6c0e5b44625227af5ec45b683057e191d6d7bf7ff43e3d25f31d5d5e81dac03b86fba04c013b970".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    let spl_transfers = tx_metadata.spl_transfers;

    // SPL transfer 1 (Uses an address table lookup token mint address)
    let spl_transfer_1 = spl_transfers[0].clone();
    assert_eq!(
        spl_transfer_1.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_1.to,
        "8jjWmLhYdqrtFMcEkiMDdqkEN85cvEFnNS4LFgNf5NRv".to_string()
    );
    assert_eq!(
        spl_transfer_1.token_mint,
        Some("ADDRESS_TABLE_LOOKUP".to_string())
    ); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
    assert_eq!(
        spl_transfer_1.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );

    // SPL transfer 2 (Uses an address table lookup for receiving "to" address)
    let spl_transfer_2 = spl_transfers[1].clone();
    assert_eq!(
        spl_transfer_2.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(spl_transfer_2.to, "ADDRESS_TABLE_LOOKUP".to_string()); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
    assert_eq!(
        spl_transfer_2.token_mint,
        Some("EGaQJtKJ4zctDY4VjP98f2KDtEeeso8JGk5a4E8X1EaV".to_string())
    );
    assert_eq!(
        spl_transfer_2.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );
}

#[test]
fn parse_spl_transfer_using_address_table_lookups_sender() {
    // This transaction contains two SPL transfer instructions
    // 1. Uses an Address Table look up to represent the Token Mint address
    // 2. Uses an Address Table look up to represent the Sending address
    // The parser will return the flag ADDRESS_TABLE_LOOKUP for these non statically included addresses

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001000b10b9334994c55889c1e129158c59a9b3b16fd9bfc9bedd105a8e1d7b7a8644110772f445b3a19ac048d2a928fe0774cf7b8b5efa7c6457cbccbc82ecf0eac93c792343cde9faec81dfd6963f83ea57e8075f2db9eb0c461d195737e143f9b16909c52568e818f6871d033a00dba9ae878df8ba008104e34fb0332d685f3eacdf6a5149b5337cf8079ab25763ae8e8f95a9b09d2325dcc2ee5f8e8640b7eacf470d283d0dd282354fef0ae3b0e227d37cd89ca266fb17ddf8f7cb7ccefbe4ebdc5506a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000d1a3910dca452ccc0c6d513e570b0a5cee7edf44fa74e1410cd405fba63e96100306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f8591e8c4fab8994494c8f1e5c1287445b2917d60c43c79aa959162f5d6000598d32000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a92ccd355fe72bcf08d5ee763f52bb9603e025ef8e1d0340f28a576313251507310479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8ee501f6575c6376b0fc00c38a8f474ed66466d3cc3bf159e8d2be46427a83a9c0a08000903a8d002000000000008000502e7e1060005020607090022bb6ad79d0c1600090600010a130b0c01010c04021301000a0c9c0100000000000006090600030d130b0c01010c04130303000a0c4603000000000000060906000400140b0c01010e120c0002040e140e0f0e150010111204020c1624e517cb977ae3ad2a010000003d016400013e9c070000000000c6c53a0000000000e803000c030400000109015de6c0e5b44625227af5ec45b683057e191d6d7bf7ff43e3d25f31d5d5e81dac03b86fba04c013b970".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    let spl_transfers = tx_metadata.spl_transfers;

    // SPL transfer 1 (Uses an address table lookup token mint address)
    let spl_transfer_1 = spl_transfers[0].clone();
    assert_eq!(
        spl_transfer_1.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_1.to,
        "8jjWmLhYdqrtFMcEkiMDdqkEN85cvEFnNS4LFgNf5NRv".to_string()
    );
    assert_eq!(
        spl_transfer_1.token_mint,
        Some("ADDRESS_TABLE_LOOKUP".to_string())
    ); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
    assert_eq!(
        spl_transfer_1.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );

    // SPL transfer 2 (Uses an address table lookup for sending "from" address)
    let spl_transfer_2 = spl_transfers[1].clone();
    assert_eq!(spl_transfer_2.from, "ADDRESS_TABLE_LOOKUP".to_string()); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
    assert_eq!(
        spl_transfer_2.to,
        "EGaQJtKJ4zctDY4VjP98f2KDtEeeso8JGk5a4E8X1EaV".to_string()
    );
    assert_eq!(
        spl_transfer_2.token_mint,
        Some("EGaQJtKJ4zctDY4VjP98f2KDtEeeso8JGk5a4E8X1EaV".to_string())
    );
    assert_eq!(
        spl_transfer_2.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );
}

#[test]
fn parse_spl_transfer_using_address_table_lookups_owner() {
    // This transaction contains two SPL transfer instructions
    // 1. Uses an Address Table look up to represent the Token Mint address
    // 2. Uses an Address Table look up to represent the Owner address
    // The parser will return the flag ADDRESS_TABLE_LOOKUP for these non statically included addresses

    // ensure that transaction gets parsed without errors
    let unsigned_transaction = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001000b10b9334994c55889c1e129158c59a9b3b16fd9bfc9bedd105a8e1d7b7a8644110772f445b3a19ac048d2a928fe0774cf7b8b5efa7c6457cbccbc82ecf0eac93c792343cde9faec81dfd6963f83ea57e8075f2db9eb0c461d195737e143f9b16909c52568e818f6871d033a00dba9ae878df8ba008104e34fb0332d685f3eacdf6a5149b5337cf8079ab25763ae8e8f95a9b09d2325dcc2ee5f8e8640b7eacf470d283d0dd282354fef0ae3b0e227d37cd89ca266fb17ddf8f7cb7ccefbe4ebdc5506a7d51718c774c928566398691d5eb68b5eb8a39b4b6d5c73555b2100000000d1a3910dca452ccc0c6d513e570b0a5cee7edf44fa74e1410cd405fba63e96100306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000008c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f8591e8c4fab8994494c8f1e5c1287445b2917d60c43c79aa959162f5d6000598d32000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a92ccd355fe72bcf08d5ee763f52bb9603e025ef8e1d0340f28a576313251507310479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138fb43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8ee501f6575c6376b0fc00c38a8f474ed66466d3cc3bf159e8d2be46427a83a9c0a08000903a8d002000000000008000502e7e1060005020607090022bb6ad79d0c1600090600010a130b0c01010c04021301000a0c9c0100000000000006090600030d130b0c01010c04020003130a0c4603000000000000060906000400140b0c01010e120c0002040e140e0f0e150010111204020c1624e517cb977ae3ad2a010000003d016400013e9c070000000000c6c53a0000000000e803000c030400000109015de6c0e5b44625227af5ec45b683057e191d6d7bf7ff43e3d25f31d5d5e81dac03b86fba04c013b970".to_string();
    let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
    let tx_metadata = parsed_tx.transaction_metadata().unwrap();

    let spl_transfers = tx_metadata.spl_transfers;

    // SPL transfer 1 (Uses an address table lookup token mint address)
    let spl_transfer_1 = spl_transfers[0].clone();
    assert_eq!(
        spl_transfer_1.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_1.to,
        "8jjWmLhYdqrtFMcEkiMDdqkEN85cvEFnNS4LFgNf5NRv".to_string()
    );
    assert_eq!(
        spl_transfer_1.token_mint,
        Some("ADDRESS_TABLE_LOOKUP".to_string())
    ); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
    assert_eq!(
        spl_transfer_1.owner,
        "DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string()
    );

    // SPL transfer 2 (Uses an address table lookup for owner address)
    let spl_transfer_2 = spl_transfers[1].clone();
    assert_eq!(
        spl_transfer_2.from,
        "3NfEggXMdHJPTYV4pkbHjh4iC3q5NoLkXTwyWAB1QSkp".to_string()
    );
    assert_eq!(
        spl_transfer_2.to,
        "EGaQJtKJ4zctDY4VjP98f2KDtEeeso8JGk5a4E8X1EaV".to_string()
    );
    assert_eq!(
        spl_transfer_2.token_mint,
        Some("DTwnQq6QdYRibHtyzWM5MxqsBuDTiUD8aeaFcjesnoKt".to_string())
    );
    assert_eq!(spl_transfer_2.owner, "ADDRESS_TABLE_LOOKUP".to_string()); // Shows the flag ADDRESS_TABLE_LOOKUP because an ATLU is used for this address in the transaction
}

#[cfg(test)]
mod account_validation_tests {
    use super::*;

    #[test]
    fn parses_system_transfer_with_extra_accounts() {
        // This transaction contains a system program Transfer instruction with 3 account keys
        // instead of the normal 2. The Solana runtime is permissive and ignores extra accounts
        // beyond the required from/to pair. This test ensures our parser handles this correctly
        // (e.g., Jupiter limit order v2 deposits).
        //
        // Built from the parses_valid_legacy_transactions test tx but with an extra account
        // (index 2) added to the transfer instruction's account list.
        let unsigned_transaction = "010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000204ef9a11d81cd82d96ce3273cda2d2e6b32ac5e89f935c18ec44b5fd91c340eed80d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3111111111111111111111111111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000000002d9084e1e78a3966ef658e17f3c3c8f855c8955eb53e8d7d9ce94134202a60ab0103030001020c020000006400000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
        let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

        let sender_account_key = "H8Jhb6qEnby1XHkxSY4eoLzsdbfZFG2Nuu1dALLeb3Hq";
        let recipient_account_key = "tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C";

        // The transfer instruction has 3 accounts but only the first 2 matter for the transfer
        assert_eq!(transaction_metadata.instructions.len(), 1);
        let inst = &transaction_metadata.instructions[0];
        assert_eq!(inst.program_key, SOL_SYSTEM_PROGRAM_KEY.to_string());
        assert_eq!(inst.accounts.len(), 3, "instruction should have 3 accounts");

        // Transfer should still be parsed correctly using only the first 2 accounts
        assert_eq!(transaction_metadata.transfers.len(), 1);
        let transfer = &transaction_metadata.transfers[0];
        assert_eq!(transfer.from, sender_account_key);
        assert_eq!(transfer.to, recipient_account_key);
        assert_eq!(transfer.amount, "100");
    }

    #[test]
    fn parses_system_transfer_with_extra_accounts_jupiter_craft_deposit() {
        // Similar to the above test but using a real failing intent from jupiter_craft_deposit - https://dev.jup.ag/api-reference/trigger/v2/deposit-craft#craft-deposit
        // This transaction has multiple instructions (7 total), and the system program
        // Transfer instruction (the 3rd) has 3 account keys instead of the normal 2.
        // The Solana runtime is permissive and ignores extra accounts beyond the
        // required from/to pair. This test ensures our parser handles this correctly.
        let unsigned_transaction = "030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000301050a08d8167cfcc1631e875bfa6ec73a6e298b2eadcd40ad7463185d27914d89c74df844a911736270327b0cd88425f1b61e6d284b23c4aa4a9caa0e35f9f60cfa110d48c72acace14f57019f9901701c7816493ef985d70b906135419fce82b3bd52f295689e2507cb5c3bc2d130db02cc6802e45754e73578814077578702ca32c7cc6545eb0a3ce6b9fad3977e58635742d3ef0e960965b10c768663cd93f54c800000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f0000000000106ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9ed4fd7dde19b705818da89bd93b254bed62ea69c13ad7d52b32dd648aee86dad0706000502a44b00000600090390d003000000000005030003020c02000000f01d1f000000000007060301080905000993f17b64f484ae76ff09030403010903000e27070000000009030300010109050200000c02000000000e270700000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true, None).unwrap();
        let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

        let sender_account_key = "bXNWGA4KcB8fz15DF9RJqf54nE5ZyS6rJBP8Jz8Dhm6";
        let recipient_account_key = "4B6iqgbER5yJNJs7TjuzUaVxdb3PApP3NeGecH8RvK5M";

        // The 3rd instruction is a transfer instruction that has 3 accounts but only the first 2 matter for the transfer
        assert_eq!(transaction_metadata.instructions.len(), 7);
        let inst = &transaction_metadata.instructions[2];
        assert_eq!(inst.program_key, SOL_SYSTEM_PROGRAM_KEY.to_string());
        assert_eq!(inst.accounts.len(), 3, "instruction should have 3 accounts");

        // Transfer should still be parsed correctly using only the first 2 accounts
        assert_eq!(transaction_metadata.transfers.len(), 2);
        let transfer = &transaction_metadata.transfers[0];
        assert_eq!(transfer.from, sender_account_key);
        assert_eq!(transfer.to, recipient_account_key);
        assert_eq!(transfer.amount, "2039280");
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;
    use std::fs;

    // JUPITER AGGREGATOR V6 TESTS -- IDL PARSING
    #[test]
    fn test_idl_parsing_jupiter_instruction_1() {
        let jup_idl_file = "jupiter_agg_v6.json";

        // Instruction #3 at this link -- https://solscan.io/tx/JcGkMSRd5iyXH5dar6MnjyrpEhQi5XR9mV2TfQpxLbTZH53geZsToDY4nzbcLGnvV32RvuEe4BPTSTkobV6rjEx
        // Instruction Name: route
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode(
                "e517cb977ae3ad2a0200000007640001266401026ca35b0400000000a5465c0400000000000000",
            )
            .unwrap(),
        )
        .unwrap();

        // Instruction #6 at this link -- https://solscan.io/tx/eQqquSewgVCP3jBkKeTstnEGGMuGaT7g7G7uPL9kfFgJk163BYFHhqE3hxNTsaoWWX6gJHRYNr2sSeRtEP3nnA3
        // Instruction Name: shared_accounts_route
        let _ = get_idl_parsed_value_given_data(
                jup_idl_file,
                &hex::decode("c1209b3341d69c810502000000266400014701006401028a28000000000000f405000000000000460014")
                    .unwrap(),
            )
            .unwrap();

        // Instruction #5 at this link -- https://solscan.io/tx/huK6BK5iUUvGZHHGJZbPrTjQrWgQY7vDQ4umFQTw8he1rvxPV6DH41XcwgEMZWXr9irgGKomxotQGzueCARqfkM
        // Instruction Name: set_token_ledger
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode("e455b9704e4f4d02").unwrap(),
        )
        .unwrap();

        // Instruction #14 at this link -- https://solscan.io/tx/huK6BK5iUUvGZHHGJZbPrTjQrWgQY7vDQ4umFQTw8he1rvxPV6DH41XcwgEMZWXr9irgGKomxotQGzueCARqfkM
        // Instruction Name: shared_account_route_with_token_ledger
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode("e6798f50779f6aaa0402000000076400012f0000640102201fd10200000000080700")
                .unwrap(),
        )
        .unwrap();

        // Instruction #9 at this link -- https://solscan.io/tx/34JeXyn1YgX2VXFeasJwFhp135dWN4xaHJquYUNP6ymgHksvM9wi4GSR6DRXmHwJ2vguB5swuauG9GPP9zEDCMds
        // Instruction Name: route_with_token_ledger
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode(
                "96564774a75d0e680300000011006400013d016401021a6402036142950900000000320000",
            )
            .unwrap(),
        )
        .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/DB5hZyNoUSo7JxNmpZkrx3Fz4rUPRLhyfkvBSp2RaqiMnEaJkgV7gUhrUAcKKtoJFzGpmsBQdmUuka2fQHY4WyR
        // Instruction Name: shared_accounts_exact_out_route
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode(
                "b0d169a89a7d453e07020000001a6400011a6401028beb033902000000ec0a3e0500000000f40100",
            )
            .unwrap(),
        )
        .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/dUQfdooGYYxRRMNYYD3fCJ6aPVCK8aEcHsgZiujZK9yZqaS3zWTfFmygQHWSRdcKYwz19u7HtHqcxvuUrbQBa79
        // Instruction Name: claim_token
        let _ = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode("74ce1bbfa613004908").unwrap(),
        )
        .unwrap();
    }

    // APE PRO SMART WALLET PROGRAM -- IDL PARSING TESTS
    #[test]
    fn test_idl_ape_pro() {
        let ape_idl_file = "ape_pro.json";

        // Instruction #4 at this link -- https://solscan.io/tx/5Yifk8KmHjGWxLP3haiMsyzuUBT7EUoqzHphW5qQoazvAqhzREf3Lwu5mRFEK7o9xHMrLuM1UavDGmRetP8sDKrB
        // Instruction Name:  preFlashSwapApprove
        let _ = get_idl_parsed_value_given_data(ape_idl_file, &hex::decode("315e88b6bffd5c280246218496010000cd6e86c4290c162621bd683dcf8de9da5aed6ffcbe1c3e0fb03581739510ba8137e3cba7093f1e888c7726a4f56809f90d38ed4cf1d85e2aae04b51016dee2ec01809698000000000000ca9a3b000000000000000000000000092d00000000ffffffffffffffff").unwrap()).unwrap();

        // Instruction #4 at this link -- https://solscan.io/tx/4JwtqYCqan3DPSbjGqwxYKzuhSZndSdX5b1JEieJr4AZD2pm1nEzCnGHKRNPRf7NvurvnvNSiECy1jV9hxiPgvy7
        // Instruction Name:  postSwap
        let _ = get_idl_parsed_value_given_data(
            ape_idl_file,
            &hex::decode("9fd5b739b38a75a1").unwrap(),
        )
        .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/4K89QAFV5EiB4AEp344XxKYxB2gdqDEcFEWj8jN6F8teKURY8zwGQbNkGoP1vFN6JvDBo9Z9M6V97qX9RG4xBYh4
        // Instruction Name:  withdrawToken
        let _ = get_idl_parsed_value_given_data(ape_idl_file, &hex::decode("88ebb505656d395121353b84960100009ef7641be0a9b34d7f8a97129c3f5711a7eaa5d35d9b7fc5b09f5746cec2165378babd55434c969c16946f4414ef59e78136f2c3c0a7bf619ecb08b4361d0af301bfa79caf05000000").unwrap()).unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/5BoC44EDb33DAhxmkZrFkQsBnG6ALJ3A5VzYXYkeCvkcWSLEXorZ2X4jeHP8V42kST8tNCxDkTE4cnt1efjSAFSn
        // Instruction Name:  flashSwapApprove
        let _ = get_idl_parsed_value_given_data(ape_idl_file, &hex::decode("f5d2c7fb443c609454baaf9492010000ed4f9590d2243e5fe524dcf665c758b4567a5e1e58949f504f1688fcd77fdc3b4a552d0e8caf9499d41c6b013bc1a6f39ec1191b8ba6e97a3bea0e5fc22453750040420f00000000004c38c175510000000000000000000000").unwrap()).unwrap();
    }

    // RaydiumCMPP -- IDL PARSING TESTS
    #[test]
    fn test_idl_raydium() {
        let rd_idl_file = "raydium.json";

        // Instruction #4 at this link -- https://solscan.io/tx/1FyuW7Pwxip4RZ5BpR3jwnDZQ5z9uUQ4Pa74hBUZw2z1k4eVWgAGBkXptSamNCkWxV58a4aXt4UHWywCERRP54a
        // Instruction Name: swapBaseInput
        let _ = get_idl_parsed_value_given_data(
            rd_idl_file,
            &hex::decode("8fbe5adac41e33de1027000000000000b80d020200000000").unwrap(),
        )
        .unwrap();

        // Instruction #5 at this link -- https://solscan.io/tx/5LETS5M435N9tEUynZy2DZX5PToNJSZZ4Cxab89zF9iHk5p6PPpgPFUsKn2aa8GVGyCHukQ7a69tafweYGS3Rk2A
        // Instruction Name: withdraw
        let _ = get_idl_parsed_value_given_data(
            rd_idl_file,
            &hex::decode("b712469c946da1229c2fef7dba0200007679e100000000000a90393145d5fa08")
                .unwrap(),
        )
        .unwrap();

        // Instruction #3.1 at this link -- https://solscan.io/tx/4s6ajUvRdsLLorb6w1N9GyJDmmzDksUrSgHHLfdZ4NPjgDe7emGWTFBt2JSWMxq9pq1bxx6bMqfgGGZ68pPb5LjU
        // Instruction Name: createAmmConfig
        let _ = get_idl_parsed_value_given_data(
                rd_idl_file,
                &hex::decode("8934edd4d7756c6804008813000000000000c0d4010000000000409c00000000000080d1f00800000000")
                    .unwrap(),
            )
            .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/5frS5mvdNJvnnLH4VhboGKzGMWTnL1HPuc9jbgpFTRavC7qZjcV92LNQBCPQLUhpsUybjfXqxy9xWgwgjuu4Hy9h
        // Instruction Name: collectProtocolFee
        let _ = get_idl_parsed_value_given_data(
            rd_idl_file,
            &hex::decode("8888fcddc2427e59ffffffffffffffffffffffffffffffff").unwrap(),
        )
        .unwrap();
    }

    // Kamino Program -- IDL PARSING TESTS
    #[test]
    fn test_idl_kamino() {
        let kamino_idl_file = "kamino.json";

        // Instruction #3 at this link -- https://solscan.io/tx/2RhYLjmRbKry2tyHKpaTfGYaEXSUmUtBTxWLRZ347nfKVVknBGgYNf5BrcykjVRjcmc8zNwhanLDcQy8ex6SfS3s
        // Instruction Name: initializeStrategy
        let _ = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("d0779091b23969fc0100000000000000ea000000000000000000000000000000")
                .unwrap(),
        )
        .unwrap();

        // Instruction #3.1 at this link -- https://solscan.io/tx/2qWFsUVJKkKYz19fzQnf2eaapYEXQXgnvmHvNZksmw7Co89Ez9BTZB7bkBrP6CicjbpwvM1VLmHwshQV58WVUsUF
        // Instruction Name: insertCollateralInfo
        let _ = get_idl_parsed_value_given_data(kamino_idl_file, &hex::decode("1661044ea6bc33beea000000000000000b86be66bc1f98b47d20a3be615a4905a825b826864e2a0f4c948467d33ee7090000000000000000000000000000000000000000000000000000000000000000e2003e00ffffffffe0001400ffffffff774d0000000000000000000000000000000000000000000000000000000000002c010000000000004001000000000000000000000000000000ffffffffffffffff").unwrap()).unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/PuKd8JeLhvSwA8VxN7hkeV5ZHdyj1hk8EoFawjn2X9YVu9GrHza93yPT1AxvMgTFzckkSBCVXhtc4eocwVxx68u
        // Instruction Name: flashSwapUnevenVaultsStart
        let _ = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("816fae0c0a3c95c19786991c0000000000").unwrap(),
        )
        .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/5fJXiyDoUxffhPm2riHTugu2zSsx16Rq1ezGv4oomjw6jEUgyrPLG3vrFkGazXgDspQpcFsJ8UapAUuNX8ph6yD8
        // Instruction Name: collectFeesAndRewards
        let _ = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("71124b08b61f69ba").unwrap(),
        )
        .unwrap();

        // Instruction #3 at this link -- https://solscan.io/tx/oYVVbND3bbpBuL3eb9jyyET2wWu1nEKxacC6BsRHP4KdsA9WHNjD7tHSEcNJkt4R83NFTquESp2xrhR92DRFCEW
        // Instruction Name: invest
        let _ = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("0df5b467feb67904").unwrap(),
        )
        .unwrap();

        // Instruction #2 at this link -- https://solscan.io/tx/yWdjGMsPP4zNVFpbqXRSVTwZJQogXzqjpV4kFLMH5G7T4Dc6hjrQ8QCkJpKLPi2sq8PyEBgedntRLVervR8Eg6o
        // Instruction Name: initializeSharesMetadata
        let _ = get_idl_parsed_value_given_data(kamino_idl_file, &hex::decode("030fac72c8008320180000004b616d696e6f20774d2d5553444320285261796469756d29080000006b574d2d555344435800000068747470733a2f2f6170692e6b616d696e6f2e66696e616e63652f6b746f6b656e732f4273334c5757747165435641714b75315032574839674b5a344d4a736b57484e786631453666686f7667597a2f6d65746164617461").unwrap()).unwrap();
    }

    // Meteora Program -- IDL PARSING TESTS
    #[test]
    fn test_idl_meteora() {
        let mtr_idl_file = "meteora.json";

        // Instruction #3.6 at this link -- https://solscan.io/tx/3Veo71aP5mVi4VT35hYDQRaxqvdkEbcgPmjh9dsBVNK2XLVusrvk3PLzrS36sYsY2h3wL2S7Z76DYW3B13uispv4
        // Instruction Name: swap
        let _ = get_idl_parsed_value_given_data(
            mtr_idl_file,
            &hex::decode("f8c69e91e17587c8d0b49c7a000000000000000000000000").unwrap(),
        )
        .unwrap();
    }

    // Test Calculating Default Anchor discriminator across different contracts
    #[test]
    fn test_anchor_default_discriminator_generation() {
        // Test Ape Pro contract instruction
        // Check that the instruction name collectFeesAndRewards parses to the correct instruction discriminator
        // Reference for the correct instruction discriminator is the first 8 bytes of instruction #3 at this link - https://solscan.io/tx/5fJXiyDoUxffhPm2riHTugu2zSsx16Rq1ezGv4oomjw6jEUgyrPLG3vrFkGazXgDspQpcFsJ8UapAUuNX8ph6yD8
        let inst_name = "collectFeesAndRewards";
        let exp_disc = hex::decode("71124b08b61f69ba").unwrap();
        let calc_disc = idl_parser::compute_default_anchor_discriminator(inst_name).unwrap();
        assert_eq!(exp_disc, calc_disc);

        // Test Orca Whirlpools contract instruction
        // Check that the instruction name openPosition parses to the correct instruction discriminator
        // Reference for the correct instruction discriminator is the first 8 bytes of instruction #3.5 at this link - https://solscan.io/tx/3TxJ1wMMBpfXucDzv5wbgXY2qx5W6pbxRGwPmkNgAhBRBqZViZNUwBWBzi2UXvsbpPf4rjQj3j2GFLT4gmX9WpKQ
        let inst_name = "openPosition";
        let exp_disc = hex::decode("87802f4d0f98f031").unwrap();
        let calc_disc = idl_parser::compute_default_anchor_discriminator(inst_name).unwrap();
        assert_eq!(exp_disc, calc_disc);

        // Test OpenBook instruction
        // Check that the instruction name consumeEvents parses to the correct instruction discriminator
        // Reference for the correct instruction discriminator is the first 8 bytes of instruction #4.1 at this link - https://solscan.io/tx/sCJp1GLAZfGNVGBZxhCE3ZQ4uyXPy8ui7L63bWF2bSPbEUhEkzPtztzr7G18DaJK64yR88RojTXPwDdvkNeDgzL
        let inst_name = "consumeEvents";
        let exp_disc = hex::decode("dd91b1341f2f3fc9").unwrap();
        let calc_disc = idl_parser::compute_default_anchor_discriminator(inst_name).unwrap();
        assert_eq!(exp_disc, calc_disc);

        // Test empty instruction name -- ERROR CASE
        let empty_inst_name = "";
        let empty_err_str = idl_parser::compute_default_anchor_discriminator(empty_inst_name)
            .unwrap_err()
            .to_string();
        assert_eq!(
                empty_err_str,
                "attempted to compute the default anchor instruction discriminator for an instruction with no name"
                    .to_string()
            );
    }

    // Test cycle detection in defined types resolution
    #[test]
    fn test_cyclic_types_idl() {
        let idl_json_string =
            fs::read_to_string(TEST_IDL_DIRECTORY.to_string() + "cyclic.json").unwrap();
        let cyclic_err_str = idl_parser::decode_idl_data(&idl_json_string)
            .unwrap_err()
            .to_string();
        assert_eq!(
            cyclic_err_str,
            "defined types cycle check failed. Recursive type found: TypeA".to_string()
        );
    }

    // Test type name collision detection
    #[test]
    fn test_type_names_collision() {
        let idl_json_string =
            fs::read_to_string(TEST_IDL_DIRECTORY.to_string() + "collision.json").unwrap();
        let cyclic_err_str = idl_parser::decode_idl_data(&idl_json_string)
            .unwrap_err()
            .to_string();
        assert_eq!(
            cyclic_err_str,
            "multiple types with the same name detected: TypeA".to_string()
        );
    }

    // Test detection of extraneous bytes at the end of an instruction
    #[test]
    fn test_extraneous_bytes() {
        let kamino_idl_file = "kamino.json";
        // Below is Kamino instruction data with an extra byte "00" added on to the end
        // Original instruction #3 at this link -- https://solscan.io/tx/oYVVbND3bbpBuL3eb9jyyET2wWu1nEKxacC6BsRHP4KdsA9WHNjD7tHSEcNJkt4R83NFTquESp2xrhR92DRFCEW
        let kamino_extra_bytes_err = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("0df5b467feb6790400").unwrap(),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(
            kamino_extra_bytes_err,
            "extra unexpected bytes remaining at the end of instruction call data".to_string()
        );

        let rd_pid = "raydium.json";
        // Below is Raydium instruction data with some extra bytes "0012" added on to the end
        // Original Instruction #4 at this link -- https://solscan.io/tx/1FyuW7Pwxip4RZ5BpR3jwnDZQ5z9uUQ4Pa74hBUZw2z1k4eVWgAGBkXptSamNCkWxV58a4aXt4UHWywCERRP54a
        let raydium_extra_bytes_err = get_idl_parsed_value_given_data(
            rd_pid,
            &hex::decode("8fbe5adac41e33de1027000000000000b80d0202000000000012").unwrap(),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(
            raydium_extra_bytes_err,
            "extra unexpected bytes remaining at the end of instruction call data".to_string()
        );
    }

    #[test]
    fn test_insufficient_data_cases() {
        let kamino_idl_file = "kamino.json";
        // Below is Kamino instruction data with an extra byte "00" added on to the end
        // Instruction #2 at this link -- https://solscan.io/tx/2RhYLjmRbKry2tyHKpaTfGYaEXSUmUtBTxWLRZ347nfKVVknBGgYNf5BrcykjVRjcmc8zNwhanLDcQy8ex6SfS3s
        // Instruction Name: initializeStrategy
        let u64_parse_err = get_idl_parsed_value_given_data(
            kamino_idl_file,
            &hex::decode("d0779091b23969fc0100000000000000ea0000000000000000000000000000").unwrap(),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(
            u64_parse_err,
            "failed to parse IDL argument with error: failed to fill whole buffer"
        );

        // Instruction #2 at this link -- https://solscan.io/tx/yWdjGMsPP4zNVFpbqXRSVTwZJQogXzqjpV4kFLMH5G7T4Dc6hjrQ8QCkJpKLPi2sq8PyEBgedntRLVervR8Eg6o
        // Instruction Name: initializeSharesMetadata
        let string_parse_err = get_idl_parsed_value_given_data(kamino_idl_file, &hex::decode("030fac72c8008320180000004b616d696e6f20774d2d5553444320285261796469756d29080000006b574d2d555344435800000068747470733a2f2f6170692e6b616d696e6f2e66696e616e63652f6b746f6b656e732f4273334c5757747165435641714b75315032574839674b5a344d4a736b57484e786631453666686f7667597a2f6d65746164").unwrap()).unwrap_err().to_string();
        assert_eq!(
            string_parse_err,
            "failed to parse IDL argument with error: failed to fill whole buffer"
        );
    }

    // This test tests ALL the parsed values making sure all values for the below instruction parsed, as provided by instruction
    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_full_jupiter_idl_call() {
        let jup_idl_file = "jupiter_agg_v6.json";

        // Instruction #3 at this link -- https://solscan.io/tx/JcGkMSRd5iyXH5dar6MnjyrpEhQi5XR9mV2TfQpxLbTZH53geZsToDY4nzbcLGnvV32RvuEe4BPTSTkobV6rjEx
        // Instruction Name: route
        let value_map = get_idl_parsed_value_given_data(
            jup_idl_file,
            &hex::decode(
                "e517cb977ae3ad2a0200000007640001266401026ca35b0400000000a5465c0400000000000000",
            )
            .unwrap(),
        )
        .unwrap();

        // Initialize expected map
        let expected_map = HashMap::from_iter([
            ("slippage_bps".to_string(), Value::Number(Number::from(0))),
            (
                "in_amount".to_string(),
                Value::Number(Number::from(73114476)),
            ),
            (
                "platform_fee_bps".to_string(),
                Value::Number(Number::from(0)),
            ),
            (
                "quoted_out_amount".to_string(),
                Value::Number(Number::from(73156261)),
            ),
            (
                "route_plan".to_string(),
                Value::Array(vec![
                    Value::Object(Map::from_iter([
                        ("input_index".to_string(), Value::Number(Number::from(0))),
                        ("output_index".to_string(), Value::Number(Number::from(1))),
                        ("percent".to_string(), Value::Number(Number::from(100))),
                        (
                            "swap".to_string(),
                            Value::Object(Map::from_iter([("Raydium".to_string(), Value::Null)])),
                        ),
                    ])),
                    Value::Object(Map::from_iter([
                        ("input_index".to_string(), Value::Number(Number::from(1))),
                        ("output_index".to_string(), Value::Number(Number::from(2))),
                        ("percent".to_string(), Value::Number(Number::from(100))),
                        (
                            "swap".to_string(),
                            Value::Object(Map::from_iter([(
                                "MeteoraDlmm".to_string(),
                                Value::Null,
                            )])),
                        ),
                    ])),
                ]),
            ),
        ]);

        assert_eq!(value_map, expected_map);
    }

    // this test tests some of the actual values within the ape pro contract call provided
    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_ape_pro_call_values() {
        let ape_idl_file = "ape_pro.json";

        // Instruction #4 at this link -- https://solscan.io/tx/5Yifk8KmHjGWxLP3haiMsyzuUBT7EUoqzHphW5qQoazvAqhzREf3Lwu5mRFEK7o9xHMrLuM1UavDGmRetP8sDKrB
        // Instruction Name:  preFlashSwapApprove
        let value_map = get_idl_parsed_value_given_data(ape_idl_file, &hex::decode("315e88b6bffd5c280246218496010000cd6e86c4290c162621bd683dcf8de9da5aed6ffcbe1c3e0fb03581739510ba8137e3cba7093f1e888c7726a4f56809f90d38ed4cf1d85e2aae04b51016dee2ec01809698000000000000ca9a3b000000000000000000000000092d00000000ffffffffffffffff").unwrap()).unwrap();

        // initialize espected map
        let expected_map = HashMap::from_iter([
            ("useDurableNonce".to_string(), Value::Bool(false)),
            (
                "postSwapIxIndex".to_string(),
                Value::Number(Number::from(9)),
            ),
            ("recoveryId".to_string(), Value::Number(Number::from(1))),
            ("feeBps".to_string(), Value::Number(Number::from(45))),
            (
                "inputAmount".to_string(),
                Value::Number(Number::from(1000000000)),
            ),
            (
                "priorityFeeLamports".to_string(),
                Value::Number(Number::from(10000000)),
            ),
            (
                "maxOut".to_string(),
                Value::Number(Number::from(18446744073709551615u64)),
            ),
            ("minOut".to_string(), Value::Number(Number::from(0))),
            (
                "nonce".to_string(),
                Value::Number(Number::from(1745973495298u64)),
            ),
            (
                "referralShareBps".to_string(),
                Value::Number(Number::from(0)),
            ),
            (
                "signature".to_string(),
                Value::Array(vec![
                    Value::Number(Number::from(205)),
                    Value::Number(Number::from(110)),
                    Value::Number(Number::from(134)),
                    Value::Number(Number::from(196)),
                    Value::Number(Number::from(41)),
                    Value::Number(Number::from(12)),
                    Value::Number(Number::from(22)),
                    Value::Number(Number::from(38)),
                    Value::Number(Number::from(33)),
                    Value::Number(Number::from(189)),
                    Value::Number(Number::from(104)),
                    Value::Number(Number::from(61)),
                    Value::Number(Number::from(207)),
                    Value::Number(Number::from(141)),
                    Value::Number(Number::from(233)),
                    Value::Number(Number::from(218)),
                    Value::Number(Number::from(90)),
                    Value::Number(Number::from(237)),
                    Value::Number(Number::from(111)),
                    Value::Number(Number::from(252)),
                    Value::Number(Number::from(190)),
                    Value::Number(Number::from(28)),
                    Value::Number(Number::from(62)),
                    Value::Number(Number::from(15)),
                    Value::Number(Number::from(176)),
                    Value::Number(Number::from(53)),
                    Value::Number(Number::from(129)),
                    Value::Number(Number::from(115)),
                    Value::Number(Number::from(149)),
                    Value::Number(Number::from(16)),
                    Value::Number(Number::from(186)),
                    Value::Number(Number::from(129)),
                    Value::Number(Number::from(55)),
                    Value::Number(Number::from(227)),
                    Value::Number(Number::from(203)),
                    Value::Number(Number::from(167)),
                    Value::Number(Number::from(9)),
                    Value::Number(Number::from(63)),
                    Value::Number(Number::from(30)),
                    Value::Number(Number::from(136)),
                    Value::Number(Number::from(140)),
                    Value::Number(Number::from(119)),
                    Value::Number(Number::from(38)),
                    Value::Number(Number::from(164)),
                    Value::Number(Number::from(245)),
                    Value::Number(Number::from(104)),
                    Value::Number(Number::from(9)),
                    Value::Number(Number::from(249)),
                    Value::Number(Number::from(13)),
                    Value::Number(Number::from(56)),
                    Value::Number(Number::from(237)),
                    Value::Number(Number::from(76)),
                    Value::Number(Number::from(241)),
                    Value::Number(Number::from(216)),
                    Value::Number(Number::from(94)),
                    Value::Number(Number::from(42)),
                    Value::Number(Number::from(174)),
                    Value::Number(Number::from(4)),
                    Value::Number(Number::from(181)),
                    Value::Number(Number::from(16)),
                    Value::Number(Number::from(22)),
                    Value::Number(Number::from(222)),
                    Value::Number(Number::from(226)),
                    Value::Number(Number::from(236)),
                ]),
            ),
        ]);

        assert_eq!(value_map, expected_map);
    }

    #[allow(clippy::cast_possible_truncation)]
    // this test tests as many different arg types as possible
    #[test]
    fn test_different_arg_types() {
        let meteora_idl_file = "meteora.json";

        // Instruction #2 at this link -- https://solscan.io/tx/5N9UR8ojzPwhWWQCwcYhU2ayL4tSjBqHG7uNgbhERYL44TD9qr61uipsugiBZTKYeHEobqPnvqauCcck872hamMD
        // Instruction Name:  initializePosition
        let inst_map_1 = get_idl_parsed_value_given_data(
            meteora_idl_file,
            &hex::decode("dbc0ea47bebf6650cefbffff45000000").unwrap(),
        )
        .unwrap();

        /*
            testing the SIGNED INT types
        */
        let lower_bin_id = inst_map_1.get("lowerBinId").unwrap();
        let exp_val = -1_074_i32;
        assert_eq!(lower_bin_id.clone(), Value::Number(Number::from(exp_val)));

        // Instruction #3 at this link -- https://solscan.io/tx/2wSFx1JvMsdqXgbvKYJu8a1mRGdzBRjpeg7TSsFi5y1xxfEF7nTsc7Uj2ZEfrtdWCaz9ndZRK6NnrCERHtuf5QM1
        // Instruction Name: initializeBinArray
        let inst_map_2 = get_idl_parsed_value_given_data(
            meteora_idl_file,
            &hex::decode("235613b94ed44bd3f5ffffffffffffff").unwrap(),
        )
        .unwrap();

        let index = inst_map_2.get("index").unwrap();
        let exp_val = -11_i64;
        assert_eq!(index.clone(), Value::Number(Number::from(exp_val)));

        // Instruction #5 at this link -- https://solscan.io/tx/4852PmY5jz4XmqJVAEeF1kHMvkXYgGZqScMCCFDkH4tJAXt2cwh4k1rnvd5nGJFHyXToCMfyYFRc5C59zecewAdu
        // Instruction Name: swapWithPriceImpact2
        let inst_map_3 = get_idl_parsed_value_given_data(
            meteora_idl_file,
            &hex::decode("4a62c0d6b1334b338096980000000000018ffdffffe80300000000").unwrap(),
        )
        .unwrap();

        /*
            testing the OPTION type
        */
        // active id is an optional field -- it is populated here
        let active_id = inst_map_3.get("activeId").unwrap();
        let exp_val = -625_i64;
        assert_eq!(active_id.clone(), Value::Number(Number::from(exp_val)));

        /*
            testing the FLOAT type
        */
        let openb_idl_file = "openbook.json";

        // Instruction #4 at this link -- https://solscan.io/tx/sBXgLUYZaPLHLNPDPsakVcHKYEbmp9YU9WvK4HZ3fXQL8sy7tg3EZpGc6rpnyKY69uABjtutvQQhytmDFvAaWxh
        // Instruction Name: CreateMarket
        let inst_map_5 = get_idl_parsed_value_given_data(openb_idl_file, &hex::decode("67e261ebc8bcfbfe080000005749462d55534443cdcccc3d010a00000001000000000000001027000000000000000000000000000000000000000000000000000000000000").unwrap()).unwrap();

        let expected_map = HashMap::from_iter([
            ("takerFee".to_string(), Value::Number(Number::from(0))),
            ("timeExpiry".to_string(), Value::Number(Number::from(0))),
            (
                "oracleConfig".to_string(),
                Value::Object(Map::from_iter([
                    (
                        "confFilter".to_string(),
                        Value::Number(Number::from_f64(0.10000000149011612).unwrap()),
                    ),
                    (
                        "maxStalenessSlots".to_string(),
                        Value::Number(Number::from(10)),
                    ),
                ])),
            ),
            ("makerFee".to_string(), Value::Number(Number::from(0))),
            ("quoteLotSize".to_string(), Value::Number(Number::from(1))),
            (
                "baseLotSize".to_string(),
                Value::Number(Number::from(10000)),
            ),
            ("name".to_string(), Value::String("WIF-USDC".to_string())),
        ]);

        assert_eq!(inst_map_5, expected_map)

        // /*
        //     NOTE: Currently, call data instances of the instruction below break our parsing, because of a bug (or oversight) in the SDK used to create the instruction call data
        //         - Contract: Meteora DLMM
        //         - Instruction Name: initializeCustomizablePermissionlessLbPair2

        //     Specifically:
        //         - The IDL for this specifies a padding buffer of length 62 -- Reference: https://github.com/MeteoraAg/dlmm-sdk/blob/main/ts-client/src/dlmm/idl.ts#L5634
        //         - But the SDK creates one of length 63 -- Reference: https://github.com/MeteoraAg/dlmm-sdk/blob/main/ts-client/src/dlmm/index.ts#L1389

        //     - This leads to our parsing buffer having one extraneous byte left over after parsing all expected arguments.
        //     - NO TODO'S: At the current time I'm NOT going to add any exception cases here. I believe that block explorers can parse this
        //       because they have laxer requirements and don't hard fail if extraneous bytes are leftover in transaction call data
        //     - We should have tighter restrictions than block explorers given security implications and should fail in the case of one-off bugs like this and resort to us/our clients triaging
        //       the specifics of functions as they come up, with potential for reevaluation if it happens to frequently, which I do not anticipate given testing volume
        //     - Keeping this test case below commented out as a reference in case we ever need to use it.
        // */
        // // // ALT instance of instruction: https://solscan.io/tx/2hsUoPNtouChnkhDMUzYUQMqgyVA4zCuckZyCYwNi8SdAMx1LSE14t2QnSweU4P3GrimMxAaezeN54b2GQAeAmwA
        // // // Instruction #1 at this link -- https://solscan.io/tx/4pjxruibNd4apBm7JqzFq8yKXWTxQRTPXNaEJY5zxZtTSULqmi3yp8jmxbbqKEdaHFTRxRUAL3jivzybqsysJyDZ
        // // // Instruction Name: initializeCustomizablePermissionlessLbPair2
        // // let inst_map_4 = get_idl_parsed_value_given_data(
        // //     meteora_idl_file,
        // //     &hex::decode("f349817e3313f16b34fcffff6400204e01000144e73e68000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        // // )
        // // .unwrap();
    }

    // ******* IDL TESTING HELPER FUNCTION ****
    fn get_idl_parsed_value_given_data(
        idl_file_name: &str,
        data: &[u8],
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let idl_json_string =
            fs::read_to_string(TEST_IDL_DIRECTORY.to_string() + idl_file_name).unwrap();

        // Parse the IDL
        let idl = idl_parser::decode_idl_data(&idl_json_string).unwrap();

        // Find the correct instruction
        let instruction =
            idl_parser::find_instruction_by_discriminator(data, idl.instructions.clone()).unwrap();

        // parse instruction call data
        let parsed_args = idl_parser::parse_data_into_args(data, &instruction, &idl)?;

        // Create ArgMap from parsed args
        let mut arg_map: HashMap<String, Value> = HashMap::new();
        for a in parsed_args {
            arg_map.insert(a.0, a.1);
        }

        Ok(arg_map)
    }

    #[test]
    fn test_instruction_data_memory_allocation_guard_arrays() {
        // bug discovered with fuzz testing

        let idl_definition =
            "{\n  \"types\": [\n \n  ],\n  \"instructions\": [\n    {\n      \"name\": \"cyclciInstruction\",\n      \"accounts\": [],\n   \"args\": [    {\n           \"name\": \"vault\",     \"type\":{ \n          \"array\": [\"u16\", 20000001000]       }\n      }\n   ]\n    }\n ]}";
        let instruction_data = vec![199, 102, 212, 229, 0, 145, 80, 22, 0, 116, 0, 0, 0];

        let idl = idl_parser::decode_idl_data(idl_definition).unwrap();

        // will crash with "memory allocation of 640000032000 bytes failed"
        let instruction = idl_parser::find_instruction_by_discriminator(
            &instruction_data,
            idl.instructions.clone(),
        )
        .unwrap();
        let array_err = idl_parser::parse_data_into_args(&instruction_data, &instruction, &idl)
            .unwrap_err()
            .to_string();
        assert_eq!(array_err, "failed to parse IDL argument with error: memory allocation exceeded maximum allowed budget while parsing IDL call data -- check your uploaded IDL or call data");
    }

    #[test]
    fn test_instruction_data_memory_allocation_guard_string() {
        // bug discovered with fuzz testing

        let malicious_string_idl = r#"
                        {
                        "types": [],
                        "instructions": [
                            {
                            "name": "dangerousString",
                            "accounts": [],
                            "args": [
                                {
                                "name": "payload",
                                "type": "string"
                                }
                            ]
                            }
                        ]
                        }"#;

        // Instruction data: [discriminator] + [4-byte length] + [truncated data]
        let string_data = vec![
            77, 67, 244, 160, 33, 24, 166, 14, // Discriminator for 'dangerous_string'
            255, 255, 255, 127, // 2,147,483,647 length (i32::MAX)
            65,  // Single 'A' character (data truncated)
        ];

        let idl = idl_parser::decode_idl_data(malicious_string_idl).unwrap();

        // will crash with "memory allocation failed"
        let instruction =
            idl_parser::find_instruction_by_discriminator(&string_data, idl.instructions.clone())
                .unwrap();
        let string_err = idl_parser::parse_data_into_args(&string_data, &instruction, &idl)
            .unwrap_err()
            .to_string();
        assert_eq!(string_err, "failed to parse IDL argument with error: memory allocation exceeded maximum allowed budget while parsing IDL call data -- check your uploaded IDL or call data");
    }

    #[test]
    fn test_instruction_data_memory_allocation_guard_composite() {
        // bug discovered with fuzz testing

        let composite_idl = r#"
            {
            "types": [],
            "instructions": [
                {
                "name": "complexAttack",
                "accounts": [],
                "args": [
                    {
                    "name": "a",
                    "type": { "vec": "string" }
                    },
                    {
                    "name": "b",
                    "type": { "vec": { "vec": "bytes" } }
                    }
                ]
                }
            ]
            }"#;

        // Instruction data: [discriminator] + nested lengths
        let composite_data = vec![
            138, 103, 254, 47, 252, 195, 139, 174, // Discriminator
            4, 0, 0, 0, // 4 strings
            255, 255, 255, 127, // String 1 length (2,147,483,647)
            255, 255, 255, 127, // String 2 length
            255, 255, 255, 127, // String 3 length
            255, 255, 255, 127, // String 4 length
            4, 0, 0, 0, // 4 inner vectors
            8, 0, 0, 0, // Each inner vector has 8 elements
            255, 255, 255, 127, // Each element has 2GB size
        ];

        let idl = idl_parser::decode_idl_data(composite_idl).unwrap();

        // will crash with "memory allocation failed"
        let instruction = idl_parser::find_instruction_by_discriminator(
            &composite_data,
            idl.instructions.clone(),
        )
        .unwrap();
        let composite_err = idl_parser::parse_data_into_args(&composite_data, &instruction, &idl)
            .unwrap_err()
            .to_string();
        assert_eq!(composite_err, "failed to parse IDL argument with error: memory allocation exceeded maximum allowed budget while parsing IDL call data -- check your uploaded IDL or call data");
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_max_recursive_depth_of_idl() {
        // bug discovered with fuzz testing

        let deep_idl = r#"
                        {
                "types": [
                    {
                    "name": "A",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "b",
                            "type": { "defined": "B" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "B",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "c",
                            "type": { "defined": "C" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "C",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "d",
                            "type": { "defined": "D" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "D",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "e",
                            "type": { "defined": "E" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "E",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "f",
                            "type": { "defined": "F" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "F",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "g",
                            "type": { "defined": "G" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "G",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "h",
                            "type": { "defined": "H" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "H",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "i",
                            "type": { "defined": "I" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "I",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "j",
                            "type": { "defined": "J" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "J",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "k",
                            "type": { "defined": "K" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "K",
                    "type": {
                        "kind": "struct",
                        "fields": [
                        {
                            "name": "l",
                            "type": { "defined": "L" }
                        }
                        ]
                    }
                    },
                    {
                    "name": "L",
                    "type": {
                        "kind": "struct",
                        "fields": []
                    }
                    }
                ],
                "instructions": [
                    {
                    "name": "testMaxDepth",
                    "accounts": [],
                    "args": [
                        {
                        "name": "payload",
                        "type": { "defined": "A" }
                        }
                    ]
                    }
                ]
                }
                "#;

        let depth_err = idl_parser::decode_idl_data(deep_idl)
            .unwrap_err()
            .to_string();
        assert_eq!(
            depth_err,
            "defined types resolution max depth exceeded on type: L"
        );
    }
}

mod custom_idl_tests {
    use super::*;
    use crate::solana::parser::parse_transaction;
    use crate::solana::structs::ProgramType;

    #[test]
    fn test_idl_hash_computation() {
        // Test that whitespace is removed and hash is consistent
        let idl_with_spaces = r#"{ "instructions": [], "types": [] }"#;
        let idl_without_spaces = r#"{"instructions":[],"types":[]}"#;

        let hash1 = idl_parser::compute_idl_hash(idl_with_spaces);
        let hash2 = idl_parser::compute_idl_hash(idl_without_spaces);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_custom_idl_for_unknown_program() {
        // Create a minimal valid IDL
        let custom_idl = r#"{
                "instructions": [
                    {
                        "name": "testInstruction",
                        "accounts": [],
                        "args": []
                    }
                ],
                "types": []
            }"#;

        let unknown_program_id = "UnknownProgram11111111111111111111111111".to_string();
        let mut custom_idls = HashMap::new();
        custom_idls.insert(unknown_program_id.clone(), (custom_idl.to_string(), true));

        // This should not fail even though program is unknown
        let idl_map =
            idl_parser::construct_custom_idl_records_map_with_overrides(Some(custom_idls)).unwrap();

        assert!(idl_map.contains_key(&unknown_program_id));
        let record = &idl_map[&unknown_program_id];
        assert!(record.custom_idl_json.is_some());
        assert!(record.override_builtin);
    }

    #[test]
    fn test_builtin_idl_without_override() {
        // Use Jupiter program as test case
        let jupiter_program_id = "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB".to_string();

        let custom_idl = r#"{
                "instructions": [],
                "types": []
            }"#;

        let mut custom_idls = HashMap::new();
        custom_idls.insert(jupiter_program_id.clone(), (custom_idl.to_string(), false));

        let idl_map =
            idl_parser::construct_custom_idl_records_map_with_overrides(Some(custom_idls)).unwrap();

        let record = &idl_map[&jupiter_program_id];
        assert!(record.custom_idl_json.is_some());
        assert!(!record.override_builtin); // Should not override
        assert_eq!(record.program_type, Some(ProgramType::Jupiter)); // Built-in program type still there
    }

    #[test]
    fn test_builtin_idl_with_override() {
        let jupiter_program_id = "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB".to_string();

        let custom_idl = r#"{
                "instructions": [],
                "types": []
            }"#;

        let mut custom_idls = HashMap::new();
        custom_idls.insert(jupiter_program_id.clone(), (custom_idl.to_string(), true));

        let idl_map =
            idl_parser::construct_custom_idl_records_map_with_overrides(Some(custom_idls)).unwrap();

        let record = &idl_map[&jupiter_program_id];
        assert!(record.custom_idl_json.is_some());
        assert!(record.override_builtin); // Should override
    }

    #[test]
    fn test_program_type_from_program_id() {
        let jupiter_id = "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB";
        let orca_id = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

        let jupiter_type = ProgramType::from_program_id(jupiter_id);
        assert_eq!(jupiter_type, Some(ProgramType::Jupiter));

        let orca_type = ProgramType::from_program_id(orca_id);
        assert_eq!(orca_type, Some(ProgramType::Orca));

        let unknown = ProgramType::from_program_id("Unknown111111111111111111111111111111");
        assert_eq!(unknown, None);
    }

    #[test]
    fn test_program_type_methods() {
        let jupiter = ProgramType::Jupiter;

        assert_eq!(
            jupiter.program_id(),
            "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB"
        );
        assert_eq!(jupiter.file_path(), "jupiter.json");
        assert_eq!(jupiter.program_name(), "Jupiter Swap");
    }

    #[test]
    fn test_parse_transaction_without_custom_idl() {
        // Test that existing functionality still works
        let unsigned_payload = "010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000".to_string();

        let result = parse_transaction(unsigned_payload, false, None);
        assert!(result.is_ok());
    }
}