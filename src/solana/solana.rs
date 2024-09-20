use std::error::Error;
use hex;
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{
        v0::{Message as VersionZeroMessage, MessageAddressTableLookup},
        Message as LegacyMessage, MessageHeader, VersionedMessage,
    },
    pubkey::Pubkey,
    system_instruction::SystemInstruction,
};

// Length of a solana signature in bytes (64 bytes long)
const LEN_SOL_SIG_BYTES: usize = 64;
// Length of a solana account key in bytes (32 bytes long)
const LEN_SOL_ACCOUNT_KEY_BYTES: usize = 32;
// This is the length of the header of a compact array -- a pattern used multiple times in solana transactions (length of header is 1 byte)
const LEN_ARRAY_HEADER_BYTES: usize = 1;
// Length of a solana message header in bytes (3 bytes long)
const LEN_MESSAGE_HEADER_BYTES: usize = 3;
// This is a string representation of the account address of the Solana System Program -- the main native program that "owns" user accounts and is in charge of facilitating basic SOL transfers among other things
const SOL_SYSTEM_PROGRAM_KEY: &str = "11111111111111111111111111111111";

// Entrypoint to parsing
pub fn parse_transaction(unsigned_tx: String) -> Result<SolanaParseResponse, Box<dyn Error>> {
    //let test_transaction = "010d31220a2a006ae386c11710f471e5b626fd356d86aad3cc3298482d7426f8ab45afdb4c95e18df8e3e8fb37d41871dbbc3bb2bd65692ece097c73d8e4d5b60f010001033576ba544d2d11541cfbce704ed3f5849f12792e1b8f4908794d2fd18742e2bfdecd7749fc61324afc2807e3d6e74461f3eea4c6d176deb4da60ae4ade29f9c60000000000000000000000000000000000000000000000000000000000000000f48a6f99c5aee4ca7e5aa1291270f79e13e28e7661d87033638f2ade245c6e0f01020200010c0200000000ca9a3b00000000";
    // let test_msg = "010001033576ba544d2d11541cfbce704ed3f5849f12792e1b8f4908794d2fd18742e2bfdecd7749fc61324afc2807e3d6e74461f3eea4c6d176deb4da60ae4ade29f9c60000000000000000000000000000000000000000000000000000000000000000f48a6f99c5aee4ca7e5aa1291270f79e13e28e7661d87033638f2ade245c6e0f01020200010c0200000000ca9a3b00000000";

    if unsigned_tx.is_empty() {
        return Err("Transaction is empty".into());
    }

    let tx = SolanaTransaction::new(&unsigned_tx).map_err(|e| {
        Box::<dyn std::error::Error>::from(format!("Unable to parse transaction: {}", e))
    })?;

    let payload = SolanaParsedTransactionPayload {
        transaction_metadata: Some(tx.transaction_metadata()?),
        unsigned_payload: unsigned_tx,
    };

    Ok(SolanaParseResponse {
        solana_parsed_transaction: SolanaParsedTransaction {
            payload: Some(payload),
        },
    })
}

/*
Parse Solana Transaction
- This function takes an unsigned solana transaction hex string and parses it either as a v0 transaction or as legacy transaction (v0 transactions include Address Lookup Tables which allow more addresses to be included in a transaction by only including references to the addresses instead of the whole string)
*/
fn parse_solana_transaction(
    unsigned_tx: &str,
) -> Result<VersionedMessage, Box<dyn std::error::Error>> {
    if unsigned_tx.len() % 2 != 0 {
        return Err("unsigned transaction provided is invalid when converted to bytes".into());
    }
    let unsigned_tx_bytes: &[u8] = &(0..unsigned_tx.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&unsigned_tx[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "unsigned transaction provided is invalid when converted to bytes")?;
    let tx_body = remove_signature_placeholder(unsigned_tx_bytes)?;
    let message = match tx_body[0] {
        0x80 => parse_solana_v0_transaction(&tx_body[LEN_ARRAY_HEADER_BYTES..tx_body.len()])?,
        _ => parse_solana_legacy_transaction(tx_body)?,
    };
    Ok(message)
}

/*
Parse Solana Legacy Transaction
- This function sequentially parses each separate section of a solana legacy transaction and constructs a Legacy message object as defined by the Solana SDK
*/
fn parse_solana_legacy_transaction(tx_body: &[u8]) -> Result<VersionedMessage, Box<dyn Error>> {
    let (header, tx_body_remainder) = parse_header(tx_body)?;
    let (account_keys, tx_body_remainder) = parse_accounts(tx_body_remainder)?;
    let (recent_blockhash, tx_body_remainder) = parse_block_hash(tx_body_remainder)?;
    let (instructions, tx_body_remainder) = parse_instructions(tx_body_remainder)?;
    if !tx_body_remainder.is_empty() {
        return Err(
            "Legacy Transaction formatted incorrectly contains extraneous bytes at the end".into(),
        );
    }
    let message = VersionedMessage::Legacy(LegacyMessage {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    });
    Ok(message)
}

/*
Parse Solana V0 Transaction
- This function sequentially parses each separate section of a solana v0 transaction and constructs a v0 message object as defined by the Solana SDK
*/
fn parse_solana_v0_transaction(tx_body: &[u8]) -> Result<VersionedMessage, Box<dyn Error>> {
    let (header, tx_body_remainder) = parse_header(tx_body)?;
    let (account_keys, tx_body_remainder) = parse_accounts(tx_body_remainder)?;
    let (recent_blockhash, tx_body_remainder) = parse_block_hash(tx_body_remainder)?;
    let (instructions, tx_body_remainder) = parse_instructions(tx_body_remainder)?;
    let (address_table_lookups, tx_body_remainder) =
        parse_address_table_lookups(tx_body_remainder)?;
    if !tx_body_remainder.is_empty() {
        return Err(
            "Solana V0 Transaction formatted incorrectly contains extraneous bytes at the end"
                .into(),
        );
    }

    let message = VersionedMessage::V0(VersionZeroMessage {
        header,
        account_keys,
        recent_blockhash,
        instructions,
        address_table_lookups,
    });
    Ok(message)
}

/*
Validate Length
- Context: Solana transactions must be parsed sequentially because it's formatting includes "Compact Arrays" who specify their length by their first byte, so the length of each section is not known beforehand
- This function validates the remaining bytes of a solana transaction to see whether the remaining bytes are greater than or equal to the calculated length of the next section, and errors with the section name if not

- Args:
    - Bytes -- this is the remainder bytes that you are checking for length
    - Length -- this is the length that you are checking for
    - Section -- this is the section of the solana transaction that you are currently parsing, used to surface for errors
*/
fn validate_length(
    bytes: &[u8],
    length: usize,
    section: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if bytes.len() < length {
        return Err(format!(
            "Unsigned transaction provided is incorrectly formatted, error while parsing {section}"
        )
        .into());
    }
    Ok(())
}

/*
Remove Signature Placeholder
- Context: Unsigned solana transactions contain a placeholder array of 0's at the beginning with enough space for all required signatures
- This function removes this section as it is not relevant for parsed unsigned transactions
*/
fn remove_signature_placeholder(
    unsigned_tx_bytes: &[u8],
) -> Result<&[u8], Box<dyn std::error::Error>> {
    validate_length(
        unsigned_tx_bytes,
        LEN_ARRAY_HEADER_BYTES,
        "Signature Array Header",
    )?;
    let sigs_num = unsigned_tx_bytes[0] as usize;
    let parse_len = 1 + (sigs_num * LEN_SOL_SIG_BYTES);
    validate_length(unsigned_tx_bytes, parse_len, "Signatures")?;
    Ok(&unsigned_tx_bytes[parse_len..unsigned_tx_bytes.len()])
}

/*
Parse Header
- This function is used to parse the three bytes at the beginning of a solana transaction message that comprise the header.
- The bytes specify the number of signatures required, the number of read only signing accounts, and the number of read only non signer accounts in the transaction
*/
fn parse_header(
    tx_body_remainder: &[u8],
) -> Result<(MessageHeader, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_MESSAGE_HEADER_BYTES,
        "Message Header",
    )?;
    let message_header = MessageHeader {
        num_required_signatures: tx_body_remainder[0],
        num_readonly_signed_accounts: tx_body_remainder[1],
        num_readonly_unsigned_accounts: tx_body_remainder[2],
    };
    Ok((
        message_header,
        &tx_body_remainder[LEN_MESSAGE_HEADER_BYTES..tx_body_remainder.len()],
    ))
}

/*
Parse Accounts
- This function parses the compact array of all static account keys (as opposed to address table lookups) included in this transaction
*/
fn parse_accounts(
    tx_body_remainder: &[u8],
) -> Result<(Vec<Pubkey>, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_ARRAY_HEADER_BYTES,
        "Accounts Array Header",
    )?;
    let accounts_num = tx_body_remainder[0] as usize;
    let len_accounts_array = (LEN_SOL_ACCOUNT_KEY_BYTES * accounts_num) + LEN_ARRAY_HEADER_BYTES;
    validate_length(tx_body_remainder, len_accounts_array, "Accounts")?;
    let mut pubkeys: Vec<Pubkey> = Vec::with_capacity(accounts_num);
    for i in 0..accounts_num {
        let mut pubkey_sized_bytes = [0u8; LEN_SOL_ACCOUNT_KEY_BYTES];
        pubkey_sized_bytes.copy_from_slice(
            &tx_body_remainder[((i * LEN_SOL_ACCOUNT_KEY_BYTES) + LEN_ARRAY_HEADER_BYTES)
                ..=((i + 1) * LEN_SOL_ACCOUNT_KEY_BYTES)],
        );
        pubkeys.push(Pubkey::new_from_array(pubkey_sized_bytes));
    }
    Ok((
        pubkeys,
        &tx_body_remainder[len_accounts_array..tx_body_remainder.len()],
    ))
}

/*
Parse Block Hash
- This function parses the recent block hash included in the transaction
*/
fn parse_block_hash(tx_body_remainder: &[u8]) -> Result<(Hash, &[u8]), Box<dyn std::error::Error>> {
    validate_length(tx_body_remainder, LEN_SOL_ACCOUNT_KEY_BYTES, "Block Hash")?;
    let hash_bytes: &[u8] = &tx_body_remainder[0..LEN_SOL_ACCOUNT_KEY_BYTES];
    let block_hash = Hash::new(hash_bytes);
    Ok((
        block_hash,
        &tx_body_remainder[LEN_SOL_ACCOUNT_KEY_BYTES..tx_body_remainder.len()],
    ))
}

/*
Parse Instructions
- This function parses all instructions included in the transaction and creates a vector of Compiled Instruction objects as specified by the Solana SDK
*/
fn parse_instructions(
    tx_body_remainder: &[u8],
) -> Result<(Vec<CompiledInstruction>, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_ARRAY_HEADER_BYTES,
        "Instructions Array Header",
    )?;
    let insts_num = tx_body_remainder[0] as usize;
    let mut compiled_insts: Vec<CompiledInstruction> = Vec::with_capacity(insts_num);
    let mut parsed_tx_body_remainder =
        &tx_body_remainder[LEN_ARRAY_HEADER_BYTES..tx_body_remainder.len()];
    for _ in 0..insts_num {
        let (new_inst, remainder_bytes) = parse_single_instruction(parsed_tx_body_remainder)?;
        parsed_tx_body_remainder = remainder_bytes;
        compiled_insts.push(new_inst);
    }
    Ok((compiled_insts, parsed_tx_body_remainder))
}

/*
Parse Single Instruction
- This function parses a single instruction in a solana transaction
*/
fn parse_single_instruction(
    tx_body_remainder: &[u8],
) -> Result<(CompiledInstruction, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_ARRAY_HEADER_BYTES,
        "Instruction Program Index",
    )?;
    let program_id_index = tx_body_remainder[0];
    let (accounts, tx_body_remainder) = parse_compact_array_of_bytes(
        &tx_body_remainder[LEN_ARRAY_HEADER_BYTES..tx_body_remainder.len()],
        "Instruction Account Indexes",
    )?;
    let (data, tx_body_remainder) =
        parse_compact_array_of_bytes(tx_body_remainder, "Instruction Data")?;
    let instruction = CompiledInstruction {
        program_id_index,
        accounts,
        data,
    };
    Ok((instruction, tx_body_remainder))
}

/*
Parse Address Table Lookups
- This function parses all address table lookups included in the transaction into a vector of MessageAddressTableLookup objects as described by the Solana SDK
*/
fn parse_address_table_lookups(
    tx_body_remainder: &[u8],
) -> Result<(Vec<MessageAddressTableLookup>, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_ARRAY_HEADER_BYTES,
        "Instructions Address Table Lookup Header",
    )?;
    let lookups_num = tx_body_remainder[0] as usize;
    let mut lookups: Vec<MessageAddressTableLookup> = Vec::with_capacity(lookups_num);
    let mut parsed_remainder = &tx_body_remainder[LEN_ARRAY_HEADER_BYTES..tx_body_remainder.len()];
    for _ in 0..lookups_num {
        let (new_lookup, remainder_bytes) = parse_single_address_table_lookup(parsed_remainder)?;
        parsed_remainder = remainder_bytes;
        lookups.push(new_lookup);
    }
    Ok((lookups, parsed_remainder))
}

/*
Parse Single Address Table Lookup
- This function parses a single adress table lookup into a MessageAddressTableLookup object from the Solana SDK
*/
fn parse_single_address_table_lookup(
    tx_body_remainder: &[u8],
) -> Result<(MessageAddressTableLookup, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_SOL_ACCOUNT_KEY_BYTES,
        "Address Table Lookup Program Account Key",
    )?;
    let mut pubkey_sized_bytes = [0u8; LEN_SOL_ACCOUNT_KEY_BYTES];
    pubkey_sized_bytes.copy_from_slice(&tx_body_remainder[0..LEN_SOL_ACCOUNT_KEY_BYTES]);
    let lookup_table_key = Pubkey::new_from_array(pubkey_sized_bytes);
    let (writable_indexes, remainder) = parse_compact_array_of_bytes(
        &tx_body_remainder[LEN_SOL_ACCOUNT_KEY_BYTES..tx_body_remainder.len()],
        "Address Table Lookup Writable Indexes",
    )?;
    let (readonly_indexes, tx_body_remainder) =
        parse_compact_array_of_bytes(remainder, "Address Table Lookup Read-Only Indexes")?;
    let lookup = MessageAddressTableLookup {
        account_key: lookup_table_key,
        writable_indexes,
        readonly_indexes,
    };
    Ok((lookup, tx_body_remainder))
}

/*
Parse Compact Array of Bytes
- Context: there are various cases in a solana transaction where a compact array of bytes is included with the first byte being how many bytes there are in the array. These byte arrays include Instruction account indexes and the instruction data
- This method parses a compact array of individual bytes
*/
fn parse_compact_array_of_bytes<'a>(
    tx_body_remainder: &'a [u8],
    section: &str,
) -> Result<(Vec<u8>, &'a [u8]), Box<dyn std::error::Error>> {
    validate_length(
        tx_body_remainder,
        LEN_ARRAY_HEADER_BYTES,
        &format!("{section} Array Header"),
    )?;
    let bytes_num = tx_body_remainder[0] as usize;
    let parse_len = (bytes_num + 1) * LEN_ARRAY_HEADER_BYTES;
    validate_length(tx_body_remainder, parse_len, &format!("{section} Array"))?;
    let bytes: Vec<u8> = tx_body_remainder[LEN_ARRAY_HEADER_BYTES..parse_len].to_vec();
    Ok((
        bytes,
        &tx_body_remainder[parse_len..tx_body_remainder.len()],
    ))
}

impl SolanaTransaction {
    pub fn new(hex_tx: &str) -> Result<Self, Box<dyn Error>> {
        let message = parse_solana_transaction(hex_tx)?;
        Ok(SolanaTransaction { message })
    }

    fn all_account_key_strings(&self) -> Vec<String> {
        self.message
            .static_account_keys()
            .to_vec()
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    fn all_invoked_programs(&self) -> Vec<String> {
        let mut invoked_programs: Vec<Pubkey> = vec![];
        let accts = self.message.static_account_keys();
        for (i, a) in accts.iter().enumerate() {
            if self.message.is_invoked(i) {
                invoked_programs.push(*a);
            }
        }
        invoked_programs
            .into_iter()
            .map(|i| i.to_string())
            .collect()
    }

    /*
    Context on Address Table Lookups
    - Address table lookups are included in Solana V0 transactions in the following way:
    ADDRESS TABLE LOOKUP OBJECT
    - account key: this is the FULL account key pointing to the address lookup table
    - writable_indexes: this is an array of all indexes (each index is represented by 1 byte) in the address lookup table that we want to include as WRITABLE addresses
    - readonly_indexes: this is an array of all indexes (each index is represented by 1 byte) in the address lookup table that we want to include as READ ONLY addresses

    Context on Address Table Lookup RESOLUTION PROCESS
    - In Solana transactions every single instruction contains an array of account INDEXES, and each index needs to be resolved to something
    - In Legacy transactions, that index is just used to index into the array of static accounts already fully included in the transaction -- plain and simple
    - However, in V0 transactions the index is supposed to index into an array that's actually a CONCATENATION of the static account keys included AND all address table lookups (concatenated in a particular order, explained next)
    - Specifically the concatenated array is --> [All Static Keys] + [All WRITABLE address table lookups] + [All READ ONLY address table lookups] IN THAT ORDER
    - IMPORTANT NOTE: if there are multiple address table lookup objects included in a transaction, the writable indexes from each lookup are concatenated IN ORDER of the address table lookups array, THEN we go back around and concatenate all the read only addresses, again in the same order

    EXAMPLE
    Lets create a transaction and name each portion of our accounts array as a variable. Lets say this transaction has:
    - 5 static addresses included --> (lets name this portion: a)
    - 2 address lookup table objects (lets call them ALT's)
        - ALT #1 includes:
            - 5 writable indexes (lets name this portion: b)
            - 5 read only indexes (lets name this portion: c)
        - ALT #2 includes:
            - 3 writable indexes (lets name this portion: d)
            - 3 read only indexes (lets name this portion: e)

    The concatenated array in order would be --> a + b + d + c + e

    lets say an instruction references address at index 16 (the 17th address in this array)
    - This would resolve to the 4th READ ONLY address lookup in ALT #1 (a=5, b=5, d=3, and then the 4th address in c would be at index 16)
    */
    fn resolve_address_table_lookup(
        &self,
        index: usize,
    ) -> Result<SolanaSingleAddressTableLookup, Box<dyn Error>> {
        match &self.message {
            VersionedMessage::Legacy(_) => {
                Err("Legacy transaction instruction account index out of bounds".into())
            }
            VersionedMessage::V0(message) => {
                let lookup_index = index - message.account_keys.len();
                let mut parsed_indexes = 0;

                // Go through writable indexes first
                for l in message.address_table_lookups.clone() {
                    if lookup_index < (parsed_indexes + l.writable_indexes.len()) {
                        return Ok(SolanaSingleAddressTableLookup {
                            address_table_key: l.account_key.to_string(),
                            index: i32::from(l.writable_indexes[lookup_index - parsed_indexes]),
                            writable: true,
                        });
                    }
                    parsed_indexes += l.writable_indexes.len();
                }

                // Go through readable indexes next
                for l in message.address_table_lookups.clone() {
                    if lookup_index < (parsed_indexes + l.readonly_indexes.len()) {
                        return Ok(SolanaSingleAddressTableLookup {
                            address_table_key: l.account_key.to_string(),
                            index: i32::from(l.readonly_indexes[lookup_index - parsed_indexes]),
                            writable: false,
                        });
                    }
                    parsed_indexes += l.writable_indexes.len();
                }
                Err("Versioned transaction instruction account index out of bounds".into())
            }
        }
    }

    fn all_instructions_and_transfers(
        &self,
    ) -> Result<(Vec<SolanaInstruction>, Vec<SolTransfer>), Box<dyn std::error::Error>> {
        let mut instructions: Vec<SolanaInstruction> = vec![];
        let mut transfers: Vec<SolTransfer> = vec![];
        for i in self.message.instructions() {
            let mut accounts: Vec<SolanaAccount> = vec![];
            let mut address_table_lookups: Vec<SolanaSingleAddressTableLookup> = vec![];
            for a in i.accounts.clone() {
                // if the index is out of bounds of the static account keys array it is an address lookup table (only for versioned transactions)
                if a as usize >= self.message.static_account_keys().len() {
                    address_table_lookups.push(self.resolve_address_table_lookup(a as usize)?);
                    continue;
                }
                let account_key = self
                    .message
                    .static_account_keys()
                    .get(a as usize)
                    .ok_or("Instruction account index out of bounds for account keys array")?
                    .to_string();
                let acct = SolanaAccount {
                    account_key,
                    signer: self.message.is_signer(a as usize),
                    writable: self.message.is_maybe_writable(a as usize, None),
                };
                accounts.push(acct);
            }
            let program_key = i.program_id(self.message.static_account_keys()).to_string();
            if program_key == *SOL_SYSTEM_PROGRAM_KEY {
                let system_instruction: SystemInstruction = bincode::deserialize(&i.data)
                    .map_err(|_| "Could not parse system instruction")?;
                if let SystemInstruction::Transfer { lamports } = system_instruction {
                    let transfer = SolTransfer {
                        amount: lamports.to_string(),
                        to: accounts[1].account_key.clone(),
                        from: accounts[0].account_key.clone(),
                    };
                    transfers.push(transfer);
                }
            }

            // TODO: verify this. unsure if this is correct
            let instruction_data_hex: String = hex::encode(&i.data);
            let inst = SolanaInstruction {
                program_key,
                accounts,
                instruction_data_hex,
                address_table_lookups,
            };
            instructions.push(inst);
        }
        Ok((instructions, transfers))
    }

    fn recent_blockhash(&self) -> String {
        self.message.recent_blockhash().to_owned().to_string()
    }

    fn address_table_lookups(&self) -> Vec<SolanaAddressTableLookup> {
        match self.message.address_table_lookups() {
            Some(address_table_lookups) => address_table_lookups
                .to_vec()
                .iter()
                .map(|a| SolanaAddressTableLookup {
                    address_table_key: a.account_key.to_string(),
                    writable_indexes: a
                        .writable_indexes
                        .iter()
                        .map(|a| i32::from(a.to_owned()))
                        .collect(),
                    readonly_indexes: a
                        .readonly_indexes
                        .iter()
                        .map(|a| i32::from(a.to_owned()))
                        .collect(),
                })
                .collect(),
            None => vec![],
        }
    }

    pub fn transaction_metadata(&self) -> Result<SolanaMetadata, Box<dyn Error>> {
        let (instructions, transfers) = self.all_instructions_and_transfers()?;
        Ok(SolanaMetadata {
            account_keys: self.all_account_key_strings(),
            address_table_lookups: self.address_table_lookups(),
            recent_blockhash: self.recent_blockhash(),
            program_keys: self.all_invoked_programs(),
            instructions,
            transfers,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SolanaTransaction {
    message: VersionedMessage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaMetadata {
    pub account_keys: Vec<String>,
    pub program_keys: Vec<String>,
    pub instructions: Vec<SolanaInstruction>,
    pub transfers: Vec<SolTransfer>,
    pub recent_blockhash: String,
    pub address_table_lookups: Vec<SolanaAddressTableLookup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaInstruction {
    pub program_key: String,
    pub accounts: Vec<SolanaAccount>,
    pub instruction_data_hex: String,
    pub address_table_lookups: Vec<SolanaSingleAddressTableLookup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaAccount {
    pub account_key: String,
    pub signer: bool,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaSingleAddressTableLookup {
    pub address_table_key: String,
    pub index: i32,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolTransfer {
    pub from: String,
    pub to: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaAddressTableLookup {
    pub address_table_key: String,
    pub writable_indexes: Vec<i32>,
    pub readonly_indexes: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaParsedTransactionPayload {
    pub transaction_metadata: Option<SolanaMetadata>,
    pub unsigned_payload: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaParsedTransaction {
    pub payload: Option<SolanaParsedTransactionPayload>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaParseResponse {
    pub solana_parsed_transaction: SolanaParsedTransaction,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parses_valid_legacy_transactions() {
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f00000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload).unwrap();
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
        let parsed_tx = SolanaTransaction::new(&unsigned_payload);

        let inst_error_message = parsed_tx.unwrap_err().to_string(); // Unwrap the error
        assert_eq!(
            inst_error_message,
            "unsigned transaction provided is invalid when converted to bytes"
        );

        // Invalid length for Instruction Data Array
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001032b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload);

        let inst_error_message = parsed_tx.unwrap_err().to_string(); // Convert to String
        assert_eq!(
    inst_error_message,
    "Unsigned transaction provided is incorrectly formatted, error while parsing Instruction Data Array"
);

        // Invalid length for Accounts Array
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010001192b162ad640a79029d57fbe5dad39d5741066c4c65b22bd248c8677174c28a4630d42099a5e0aaeaad1d4ede263662787cb3f6291a6ede340c4aa7ca26249dbe3000000000000000000000000000000000000000000000000000000000000000021d594adba2b7fbd34a0383ded05e2ba526e907270d8394b47886805b880e73201020200010c020000006f000000000000".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload);

        let inst_error_message = parsed_tx.unwrap_err().to_string(); // Unwrap the error
        assert_eq!(
            inst_error_message,
            "Unsigned transaction provided is incorrectly formatted, error while parsing Accounts"
        );
    }
    #[test]
    #[allow(clippy::too_many_lines)]
    fn parses_v0_transactions() {
        // You can also ensure that the output of this transaction makes sense yourself using the below references
        // Transaction reference: https://solscan.io/tx/4tkFaZQPGNYTBag6sNTawpBnAodqiBNF494y86s2qBLohQucW1AHRaq9Mm3vWTSxFRaUTmtdYp67pbBRz5RDoAdr
        // Address Lookup Table Account key: https://explorer.solana.com/address/6yJwigBRYdkrpfDEsCRj7H5rrzdnAYv8LHzYbb5jRFKy/entries

        // Invalid bytes, odd number length string
        let unsigned_payload = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800100070ae05271368f77a2c5fefe77ce50e2b2f93ceb671eee8b172734c8d4df9d9eddc186a35856664b03306690c1c0fbd4b5821aea1c64ffb8c368a0422e47ae0d2895de288ba87b903021e6c8c2abf12c2484e98b040792b1fbb87091bc8e0dd76b6600000000000000000000000000000000000000000000000000000000000000000306466fe5211732ffecadba72c39be7bc8ce5bbc5f7126b2c439b3a400000000479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e8c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d616419cee70b839eb4eadd1411aa73eea6fd8700da5f0ea730136db1dd6fb2de660804000502c05c150004000903caa200000000000007060002000e03060101030200020c0200000080f0fa02000000000601020111070600010009030601010515060002010509050805100f0a0d01020b0c0011060524e517cb977ae3ad2a01000000120064000180f0fa02000000005d34700000000000320000060302000001090158b73fa66d1fb4a0562610136ebc84c7729542a8d792cb9bd2ad1bf75c30d5a404bdc2c1ba0497bcbbbf".to_string();
        let parsed_tx = SolanaTransaction::new(&unsigned_payload).unwrap();
        let transaction_metadata = parsed_tx.transaction_metadata().unwrap();

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
}
