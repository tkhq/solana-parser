# solana-parser

This is a library containing useful tools to parse and inspect Solana transactions at the most primitive level. 

## Build

```sh
cargo build
```

## Usage

The solana transaction parsers accepts hex string inputs of two types: 
- Messages: the "message" portion of the solana transaction, which is the payload that gets signed by signers, NOT including the compact array of Signatures that comes at the beginning of a full solana transaction
- Full transactions: this includes both the message, as well as the compact array of signatures (or a placeholder array with zero-signatures) at the beginning

To use the parser include the hex string of EITHER a solana transaction message or a full solana transaction. You MUST also include the corresponding flag: --message or --transaction. You must include exactly one of these flags.

```sh
# Example for parsing a solana transaction message
cargo run parse <your unsigned Solana transaction message> --message
```

```sh
# Example for parsing a full solana transaction
cargo run parse <your unsigned Solana transaction> --transaction
```

## Examples

See tests for examples using transactions from various scenarios. Here are the corresponding outputs:

Command to parse simple legacy transaction messages: 

```
cargo run parse 010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000 --message
```

Output: 

```
Solana Parsed Transaction Payload:
  Unsigned Payload: 010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000
  Transaction Metadata:
    Signatures: []
    Account Keys: ["3uC8tBZQQA1RCKv9htCngTfYm4JK4ezuYx4M4nFsZQVp", "tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C", "11111111111111111111111111111111"]
    Program Keys: ["11111111111111111111111111111111"]
    Recent Blockhash: 3H5M4mR53HeFhi1FG5UxPeHgx7gGqb8Q2uM4Z4CWkLnm
    Instructions:
      Instruction 1:
        Program Key: 11111111111111111111111111111111
        Accounts: [SolanaAccount { account_key: "3uC8tBZQQA1RCKv9htCngTfYm4JK4ezuYx4M4nFsZQVp", signer: true, writable: true }, SolanaAccount { account_key: "tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C", signer: false, writable: true }]
        Instruction Data (hex): 020000006f00000000000000
        Address Table Lookups: []
    Transfers:
      Transfer 1:
        From: 3uC8tBZQQA1RCKv9htCngTfYm4JK4ezuYx4M4nFsZQVp
        To: tkhqC9QX2gkqJtUFk2QKhBmQfFyyqZXSpr73VFRi35C
        Amount: 111
    Address Table Lookups: []
```

Example Command for a complex Jupiter Trade (full transaction): 

```
cargo run parse 0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100070ae05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc186a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b6600000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d616419cee70b839eb4eadd1411aa73eea6fd8700da5f0ea730136db1dd6fb2de660804000502c05c150004000903caa200000000000007060002000e03060101030200020c0200000080f0fa02000000000601020111070600010009030601010515060002010509050805100f0a0d01020b0c0011060524e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000060302000001090158b73fa66d1fb4a0562610136ebc84c7729542a8d792cb9bd2ad1bf75c30d5a404bdc2c1ba0497bcbbbf --transaction
```

Output:

```
Solana Parsed Transaction Payload:
  Unsigned Payload: 0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100070ae05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc186a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b6600000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d616419cee70b839eb4eadd1411aa73eea6fd8700da5f0ea730136db1dd6fb2de660804000502c05c150004000903caa200000000000007060002000e03060101030200020c0200000080f0fa02000000000601020111070600010009030601010515060002010509050805100f0a0d01020b0c0011060524e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000060302000001090158b73fa66d1fb4a0562610136ebc84c7729542a8d792cb9bd2ad1bf75c30d5a404bdc2c1ba0497bcbbbf
  Transaction Metadata:
    Signatures: ["00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"]
    Account Keys: ["G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8", "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", "11111111111111111111111111111111", "ComputeBudget111111111111111111111111111111", "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]
    Program Keys: ["11111111111111111111111111111111", "ComputeBudget111111111111111111111111111111", "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]
    Recent Blockhash: 7jkbVYFiE9wY2Ghuk8e99nZmfnN1R4gUVbJAgPeQSirH
    Instructions:
      Instruction 1:
        Program Key: ComputeBudget111111111111111111111111111111
        Accounts: []
        Instruction Data (hex): 02c05c1500
        Address Table Lookups: []
      Instruction 2:
        Program Key: ComputeBudget111111111111111111111111111111
        Accounts: []
        Instruction Data (hex): 03caa2000000000000
        Address Table Lookups: []
      Instruction 3:
        Program Key: ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
        Accounts: [SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "11111111111111111111111111111111", signer: false, writable: false }, SolanaAccount { account_key: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", signer: false, writable: false }]
        Instruction Data (hex): 01
        Address Table Lookups: [SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 151, writable: false }]
      Instruction 4:
        Program Key: 11111111111111111111111111111111
        Accounts: [SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }]
        Instruction Data (hex): 0200000080f0fa0200000000
        Address Table Lookups: []
      Instruction 5:
        Program Key: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        Accounts: [SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }]
        Instruction Data (hex): 11
        Address Table Lookups: []
      Instruction 6:
        Program Key: ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
        Accounts: [SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8", signer: false, writable: true }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", signer: false, writable: false }, SolanaAccount { account_key: "11111111111111111111111111111111", signer: false, writable: false }, SolanaAccount { account_key: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", signer: false, writable: false }]
        Instruction Data (hex): 01
        Address Table Lookups: []
      Instruction 7:
        Program Key: JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4
        Accounts: [SolanaAccount { account_key: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", signer: false, writable: false }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }, SolanaAccount { account_key: "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8", signer: false, writable: true }, SolanaAccount { account_key: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", signer: false, writable: false }, SolanaAccount { account_key: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", signer: false, writable: false }, SolanaAccount { account_key: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", signer: false, writable: false }, SolanaAccount { account_key: "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf", signer: false, writable: false }, SolanaAccount { account_key: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", signer: false, writable: false }, SolanaAccount { account_key: "A4a6VbNvKA58AGpXBEMhp7bPNN9bDCFS9qze4qWDBBQ8", signer: false, writable: true }, SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", signer: false, writable: false }, SolanaAccount { account_key: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", signer: false, writable: false }]
        Instruction Data (hex): e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000
        Address Table Lookups: [SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 187, writable: false }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 188, writable: false }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 189, writable: true }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 186, writable: true }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 194, writable: true }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 193, writable: true }, SolanaSingleAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", index: 191, writable: false }]
      Instruction 8:
        Program Key: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        Accounts: [SolanaAccount { account_key: "FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw", signer: false, writable: true }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }, SolanaAccount { account_key: "G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp", signer: true, writable: true }]
        Instruction Data (hex): 09
        Address Table Lookups: []
    Transfers:
      Transfer 1:
        From: G6fEj2pt4YYAxLS8JAsY5BL6hea7Fpe8Xyqscg2e7pgp
        To: FxDNKZ14p3W7o1tpinH935oiwUo3YiZowzP1hUcUzUFw
        Amount: 50000000
    Address Table Lookups: [SolanaAddressTableLookup { address_table_key: "6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy", writable_indexes: [189, 194, 193, 186], readonly_indexes: [151, 188, 187, 191] }]
```

## Future considerations

TODO
