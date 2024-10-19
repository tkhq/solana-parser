use std::vec;

use super::*;
use parser::SOL_SYSTEM_PROGRAM_KEY;
use structs::SolanaMetadata;
use crate::solana::structs::{SolanaInstruction, SolanaAccount, SolanaAddressTableLookup, SolanaSingleAddressTableLookup, SolTransfer};
use crate::solana::parser::SolanaTransaction;


    #[test]
    fn parses_valid_legacy_transactions() {
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, true).unwrap();
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
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, false).unwrap(); // check that a message is parsed correctly
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
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, true);

        let inst_error_message = parsed_tx.unwrap_err().to_string(); // Unwrap the error
        assert_eq!(
            inst_error_message,
            "unsigned transaction provided is invalid when converted to bytes"
        );

        // Invalid length for Instruction Data Array
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, true);

        let inst_error_message = parsed_tx.unwrap_err().to_string(); // Convert to String
        assert_eq!(
    inst_error_message,
    "Error parsing full transaction. If this is just a message instead of a full transaction, parse using the --message flag. Parsing Error: \"Unsigned transaction provided is incorrectly formatted, error while parsing Instruction Data Array\""
);

        // Invalid length for Accounts Array
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001192b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, true);

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
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, true).unwrap();
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
        let parsed_tx = SolanaTransaction::new(&unsigned_payload, false).unwrap();
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
        };
        assert_eq!(exp_instruction_1, transaction_metadata.instructions[0]);

        // Instruction 2 -- SetComputeUnitPrice
        let exp_instruction_2 = SolanaInstruction {
            program_key: compute_budget_acct_key.to_string(),
            accounts: vec![],
            address_table_lookups: vec![],
            instruction_data_hex: "03caa2000000000000".to_string(),
        };
        assert_eq!(exp_instruction_2, transaction_metadata.instructions[1]);

        // Instruction 3 - CreateIdempotent
        let exp_instruction_3 = SolanaInstruction {
            program_key: assoc_token_acct_key.to_string(),
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
        };
        assert_eq!(exp_instruction_4, transaction_metadata.instructions[3]);

        // Instruction 5 -- SyncNative
        let exp_instruction_5 = SolanaInstruction {
            program_key: token_acct_key.to_string(),
            accounts: vec![receiving_acct.clone()],
            address_table_lookups: vec![],
            instruction_data_hex: "11".to_string(),
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
        let exp_instruction_7 = SolanaInstruction {
            program_key: jupiter_program_acct_key.to_string(),
            accounts: vec![
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
            ],
            address_table_lookups: lookups_7,
            instruction_data_hex:
                "e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000"
                    .to_string(),
        };
        assert_eq!(exp_instruction_7, transaction_metadata.instructions[6]);

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
        let parsed_tx = SolanaTransaction::new(&unsigned_transaction, true).unwrap();
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
        };
        assert_eq!(exp_instruction_1, transaction_metadata.instructions[0]);

        // Instruction 2 -- SetComputeUnitPrice
        let exp_instruction_2 = SolanaInstruction {
            program_key: compute_budget_acct_key.to_string(),
            accounts: vec![],
            address_table_lookups: vec![],
            instruction_data_hex: "032753050000000000".to_string(),
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
        };
        assert_eq!(exp_instruction_3, transaction_metadata.instructions[2]);

        // Instruction 4 -- This is a basic SOL transfer
        let exp_instruction_4 = SolanaInstruction {
            program_key: SOL_SYSTEM_PROGRAM_KEY.to_string(),
            accounts: vec![signer_acct.clone(), wsol_mint_acct.clone()],
            address_table_lookups: vec![],
            instruction_data_hex: "020000008096980000000000".to_string(),
        };
        assert_eq!(exp_instruction_4, transaction_metadata.instructions[3]);

        // Instruction 5 -- SyncNative
        let exp_instruction_5 = SolanaInstruction {
            program_key: token_acct_key.to_string(),
            accounts: vec![wsol_mint_acct.clone()],
            address_table_lookups: vec![],
            instruction_data_hex: "11".to_string(),
        };
        assert_eq!(exp_instruction_5, transaction_metadata.instructions[4]);

        // Instruction 6 -- Jupiter Aggregator v6: sharedAccountsRoute
        let exp_instruction_6 = SolanaInstruction {
            program_key: jupiter_program_acct_key.to_string(),
            accounts: vec![
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
            ],
            instruction_data_hex: "c1209b3341d69c810502000000136400011c016401028096980000000000b2a3170000000000640000"
                .to_string(),
            address_table_lookups: vec![
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
            ],
        };
        assert_eq!(exp_instruction_6, transaction_metadata.instructions[5]);

        // Instruction 7 -- Close Account
        let exp_instruction_7: SolanaInstruction = SolanaInstruction {
            program_key: token_acct_key.to_string(),
            accounts: vec![wsol_mint_acct.clone(), signer_acct.clone(), signer_acct.clone()],
            address_table_lookups: vec![],
            instruction_data_hex: "09".to_string(),
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
