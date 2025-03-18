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
use super::structs::{AddressAccount, SolTransfer, SolanaAccount, SolanaAddressTableLookup, SolanaInstruction, SolanaMetadata, SolanaParseResponse, SolanaParsedTransaction, SolanaParsedTransactionPayload, SolanaSingleAddressTableLookup, SplTransfer};

// Length of a solana signature in bytes (64 bytes long)
pub const LEN_SOL_SIGNATURE_BYTES: usize = 64;
// Length of a solana account key in bytes (32 bytes long)
pub const LEN_SOL_ACCOUNT_KEY_BYTES: usize = 32;
// This is the length of the header of a compact array -- a pattern used multiple times in solana transactions (length of header is 1 byte)
pub const LEN_ARRAY_HEADER_BYTES: usize = 1;
// Length of a solana message header in bytes (3 bytes long)
pub const LEN_MESSAGE_HEADER_BYTES: usize = 3;
// This is a string representation of the account address of the Solana System Program -- the main native program that "owns" user accounts and is in charge of facilitating basic SOL transfers among other things
pub const SOL_SYSTEM_PROGRAM_KEY: &str = "11111111111111111111111111111111";
// This is a string representation of the account address of the Token Program -- Used for transferring SPL tokens 
pub const TOKEN_PROGRAM_KEY: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
// This is a string representation of the account address for the Token 2022 Program which is a strict superset of the old Token Program, used to add extra functionality -- Used for transferring SPL tokens
pub const TOKEN_2022_PROGRAM_KEY: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
// Versioned transactions have a prefix of 0x80
const V0_TRANSACTION_INDICATOR: u8 = 0x80;

// Entrypoint to parsing
pub fn parse_transaction(unsigned_tx: String, full_transaction: bool) -> Result<SolanaParseResponse, Box<dyn Error>> {
    if unsigned_tx.is_empty() {
        return Err("Transaction is empty".into());
    }

    let tx = SolanaTransaction::new(&unsigned_tx, full_transaction).map_err(|e| {
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
    full_transaction: bool,
) -> Result<SolanaTransaction, Box<dyn std::error::Error>> {
    if unsigned_tx.len() % 2 != 0 {
        return Err("unsigned transaction provided is invalid when converted to bytes".into());
    }
    let unsigned_tx_bytes: &[u8] = &(0..unsigned_tx.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&unsigned_tx[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "unsigned transaction provided is invalid when converted to bytes")?;

    if full_transaction {
        let (signatures, tx_body) = parse_signatures(unsigned_tx_bytes)?;
        let message = match tx_body[0] {
            V0_TRANSACTION_INDICATOR => parse_solana_v0_transaction(&tx_body[LEN_ARRAY_HEADER_BYTES..tx_body.len()]).map_err(|e| format!("Error parsing full transaction. If this is just a message instead of a full transaction, parse using the --message flag. Parsing Error: {:#?}", e))?,
            _ => parse_solana_legacy_transaction(tx_body).map_err(|e| format!("Error parsing full transaction. If this is just a message instead of a full transaction, parse using the --message flag. Parsing Error: {:#?}", e))?,
        };
        return Ok(SolanaTransaction{ message, signatures });
    }
    let message = match unsigned_tx_bytes[0] {
        V0_TRANSACTION_INDICATOR => parse_solana_v0_transaction(&unsigned_tx_bytes[LEN_ARRAY_HEADER_BYTES..unsigned_tx_bytes.len()]).map_err(|e| format!("Error parsing message. If this is a serialized Solana transaction with signatures, parse using the --transaction flag. Parsing error: {:#?}", e))?,
        _ => parse_solana_legacy_transaction(unsigned_tx_bytes).map_err(|e| format!("Error parsing message. If this is a full solana transaction with signatures or signature placeholders, parse using the --transaction flag. Parsing Error: {:#?}", e))?,
    };
    return Ok(SolanaTransaction{ message, signatures: vec![] }); // Signatures array is empty when we are parsing a message (using --message) as opposed to a full transaction

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
Parse Signatures
- Context: Solana transactions contain a compact array of signatures at the beginning of a transaction 
- This function parses these signatures.
- NOTE: This is only relevant for when we are parsing FULL TRANSACTIONS (using the flag --transasction) not when we are parsing only the message (using --message)
*/
fn parse_signatures(
    unsigned_tx_bytes: &[u8],
) -> Result<(Vec<Signature>, &[u8]), Box<dyn std::error::Error>> {
    validate_length(
        unsigned_tx_bytes,
        LEN_ARRAY_HEADER_BYTES,
        "Signature Array Header",
    )?;
    let num_signatures = unsigned_tx_bytes[0] as usize;
    let parse_len = 1 + (num_signatures * LEN_SOL_SIGNATURE_BYTES);
    validate_length(unsigned_tx_bytes, parse_len, "Signatures")?;
    let signatures: Vec<Signature> = unsigned_tx_bytes[1..]
        .chunks_exact(LEN_SOL_SIGNATURE_BYTES)
        .take(num_signatures)
        .map(<[u8]>::to_vec)
        .collect();
    Ok((signatures, &unsigned_tx_bytes[parse_len..unsigned_tx_bytes.len()]))
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
    let (length, tx_body_remainder) = read_compact_u16(&tx_body_remainder)?;
    let parse_len = length * LEN_ARRAY_HEADER_BYTES;
    validate_length(tx_body_remainder, parse_len, &format!("{section} Array"))?;
    let bytes: Vec<u8> = tx_body_remainder[0..parse_len].to_vec();
    Ok((
        bytes,
        &tx_body_remainder[parse_len..tx_body_remainder.len()],
    ))
}

/*
Read Compact u16
- Context: compact arrays in Solana begin with a compact-u16 multi-byte header
- A compact-u16 multibyte header consists of 1-3 bytes.
- Each byte in compact-u16 (except the 3rd byte, if there is one) has its MOST SIGNIFICANT BIT set as a "continuation bit" where 1 = continue (as in there will be another byte) and 0 = don't continue
- The remaining 7 bits are the data bits to be included in the bit representation of the final u16 number (except for the case in which there is a 3rd byte, in which case there is NO continuation bit and only 2 data bits, because 7 + 7 + 2 = 16)
- In other words, while parsing a byte that is part of a compact-u16, if the first bit is 1, then there will be a next byte (unless it is the 3rd bit, then the first bit being a 1 is erroneous and malformatted)
- FINALLY -- if the final format is zzyyyyyyyxxxxxxx it is represented in compactu16 as 1xxxxxxx1yyyyyyy000000zz
- In a 3 byte representation, the first byte has the 7 least significant digits, and the 3rd byte has the two most significant digits of the final u16
*/
fn read_compact_u16(tx_body_remainder: &[u8]) -> Result<(usize, &[u8]), Box<dyn std::error::Error>> {
    let mut value = 0u16;
    let mut shift = 0;
    let mut bytes_read = 0;

    loop {
        // Check if there are enough bytes
        if bytes_read >= tx_body_remainder.len() {
            return Err(format!(
                "error parsing unsigned transaction: unable to parse compact array header, not enough bytes"
            )
            .into());
        }

        let byte = tx_body_remainder[bytes_read];
        bytes_read += 1;

        // if we're on the 3rd byte, and the any bits other than the two least significant bits are set, this is an error
        if shift == 14 && (byte & 0xfc) != 0 {
            return Err(format!(
                "error parsing unsigned transaction: third byte in compact u16 has more than it's 2 least significant bits set",
            )
            .into());
        }

        // remove the continuation bit from the new byte and shift it over by the correct amount to put it in the final bit representation of the u16 number
        value |= u16::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok((value as usize, &tx_body_remainder[bytes_read..tx_body_remainder.len()]));
        }

        // increasing the shift of the digits in the next byte by 7 as in the description above this method
        shift += 7;
    }
}

// SplInstructionData represents parsed instruction data of an instruction to the solana token program or the token 2022 program
enum SplInstructionData {
    Transfer{ amount: u64 },
    TransferChecked{ amount: u64, decimals: u8 },
    TransferCheckedWithFee{ amount: u64, decimals: u8, fee: u64 },
    Unsupported,
}

impl SplInstructionData {
    // Reference for instruction data parsing code: https://docs.rs/spl-token/latest/src/spl_token/instruction.rs.html#476
    fn parse_spl_transfer_data(instruction_data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let (&tag, rest) = instruction_data.split_first().ok_or("Error while parsing spl instruction data header")?;
        Ok(match tag {
            3 => {
                let (amount, _rest) = unpack_u64(rest).map_err(|_| format!("Error while parsing spl instruction Transfer -- amount"))?;
                Self::Transfer { amount }
            }
            12 => {
                let (amount, rest) = unpack_u64(rest).map_err(|_| format!("Error while parsing spl instruction TransferChecked -- amount"))?;
                let (&decimals, _rest) = rest.split_first().ok_or("Error while parsing spl instruction TransferChecked -- decimals")?;
                Self::TransferChecked { amount, decimals }
            }
            26 => {
                let (&transfer_fee_tag, rest) = rest.split_first().ok_or("Error while parsing spl instruction TransferCheckedWithFee -- instruction index")?;
                // Given the extension prefix of 26 ensure that we're calling instruciton 01, which is TransferCheckedWithFee
                let (amount, decimals, fee) = match transfer_fee_tag {
                    1 => {
                        let (amount, rest) = unpack_u64(rest).map_err(|_| format!("Error while parsing spl instruction TransferCheckedWithFee -- amount"))?;
                        let (&decimals, rest) = rest.split_first().ok_or("Error while parsing spl instruction TransferCheckedWithFee -- decimals")?;
                        let (fee, _rest) = unpack_u64(rest).map_err(|_| format!("Error while parsing spl instruction TransferCheckedWithFee -- fee"))?;
                        (amount, decimals, fee)
                    }
                    _ => return Ok(Self::Unsupported),
                };
                Self::TransferCheckedWithFee { amount, decimals, fee }
            }
            _ => Self::Unsupported,
        })
    }
}

fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), Box<dyn std::error::Error>> {
    let value = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or("Error parsing u64 in spl instruction")?;
    Ok((value, &input[8..]))
}

// Each signature is a Vec<u8> of 64 bytes
pub type Signature = Vec<u8>;

#[derive(Debug, PartialEq, Eq)]
pub struct SolanaTransaction {
    message: VersionedMessage,
    signatures: Vec<Signature>,
}
impl SolanaTransaction {
    pub fn new(hex_tx: &str, full_transaction: bool) -> Result<Self, Box<dyn Error>> {
        parse_solana_transaction(hex_tx, full_transaction)
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
                let mut num_parsed_indexes = 0;

                // Go through writable indexes first
                for l in message.address_table_lookups.clone() {
                    if lookup_index < (num_parsed_indexes + l.writable_indexes.len()) {
                        return Ok(SolanaSingleAddressTableLookup {
                            address_table_key: l.account_key.to_string(),
                            index: i32::from(l.writable_indexes[lookup_index - num_parsed_indexes]),
                            writable: true,
                        });
                    }
                    num_parsed_indexes += l.writable_indexes.len();
                }

                // Go through readable indexes next
                for l in message.address_table_lookups.clone() {
                    if lookup_index < (num_parsed_indexes + l.readonly_indexes.len()) {
                        return Ok(SolanaSingleAddressTableLookup {
                            address_table_key: l.account_key.to_string(),
                            index: i32::from(l.readonly_indexes[lookup_index - num_parsed_indexes]),
                            writable: false,
                        });
                    }
                    num_parsed_indexes += l.readonly_indexes.len();
                }
                Err("Versioned transaction instruction account index out of bounds".into())
            }
        }
    }

    fn all_instructions_and_transfers(
        &self,
    ) -> Result<(Vec<SolanaInstruction>, Vec<SolTransfer>, Vec<SplTransfer>), Box<dyn std::error::Error>> {
        let mut instructions: Vec<SolanaInstruction> = vec![];
        let mut transfers: Vec<SolTransfer> = vec![];
        let mut spl_transfers: Vec<SplTransfer> = vec![];
        for i in self.message.instructions() {
            let mut account_addresses: Vec<AddressAccount> = vec![];
            let mut accounts: Vec<SolanaAccount> = vec![];
            let mut address_table_lookups: Vec<SolanaSingleAddressTableLookup> = vec![];
            for a in i.accounts.clone() {
                // if the index is out of bounds of the static account keys array it is an address lookup table (only for versioned transactions)
                if a as usize >= self.message.static_account_keys().len() {
                    let atlu = self.resolve_address_table_lookup(a as usize)?;
                    address_table_lookups.push(atlu.clone());
                    account_addresses.push(AddressAccount::AddressTableLookUp(atlu.clone()));
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
                accounts.push(acct.clone());
                account_addresses.push(AddressAccount::Static(acct.clone()));
            }
            let program_key = i.program_id(self.message.static_account_keys()).to_string();
            match program_key.as_str() {
                SOL_SYSTEM_PROGRAM_KEY => {
                    let system_instruction: SystemInstruction = bincode::deserialize(&i.data)
                    .map_err(|_| "Could not parse system instruction")?;
                    if let SystemInstruction::Transfer { lamports } = system_instruction {
                        if account_addresses.len() != 2 {
                            return Err("System Program Transfer Instruction should have exactly 2 arguments".into())
                        }
                        let transfer = SolTransfer {
                            amount: lamports.to_string(),
                            to: account_addresses[1].to_string(),
                            from: account_addresses[0].to_string(),
                        };
                        transfers.push(transfer);
                    }
                }
                TOKEN_PROGRAM_KEY => {
                    let token_program_instruction: SplInstructionData = SplInstructionData::parse_spl_transfer_data(&i.data)?;
                    let spl_tranfer_opt = self.parse_spl_instruction_data(token_program_instruction, account_addresses.clone())?;
                    match spl_tranfer_opt {
                        Some(spl_transfer) => spl_transfers.push(spl_transfer),
                        None => (),
                    }
                }
                TOKEN_2022_PROGRAM_KEY => {
                    let token_program_22_instruction: SplInstructionData = SplInstructionData::parse_spl_transfer_data(&i.data)?;
                    let spl_tranfer_opt = self.parse_spl_instruction_data(token_program_22_instruction, account_addresses.clone())?;
                    match spl_tranfer_opt {
                        Some(spl_transfer) => spl_transfers.push(spl_transfer),
                        None => (),
                    }
                }
                _ => {}
            }
            let instruction_data_hex: String = hex::encode(&i.data);
            let inst = SolanaInstruction {
                program_key,
                accounts,
                instruction_data_hex,
                address_table_lookups,
            };
            instructions.push(inst);
        }
        Ok((instructions, transfers, spl_transfers))
    }

    // Parse Instruction to Solana Token Program OR Solana Token Program 2022 and return something if it is an SPL transfer
    fn parse_spl_instruction_data(&self, token_instruction: SplInstructionData, accounts: Vec<AddressAccount>) -> Result<Option<SplTransfer>, Box<dyn std::error::Error>> {
        if let SplInstructionData::Transfer { amount } = token_instruction {
            let signers = self.get_spl_multisig_signers_if_exist(&accounts, 3)?;
            let spl_transfer = SplTransfer {
                amount: amount.to_string(),
                to: accounts[1].to_string(),
                from: accounts[0].to_string(),
                owner: accounts[2].to_string(),
                signers,
                decimals: None, 
                fee: None,
                token_mint: None,
            };
            return Ok(Some(spl_transfer))
        } else if let SplInstructionData::TransferChecked{ amount, decimals } = token_instruction {
            let signers = self.get_spl_multisig_signers_if_exist(&accounts, 4)?;
            let spl_transfer = SplTransfer {
                amount: amount.to_string(),
                to: accounts[2].to_string(),
                from: accounts[0].to_string(),
                token_mint: Some(accounts[1].to_string()),
                owner: accounts[3].to_string(),
                signers,
                decimals: Some(decimals.to_string()),
                fee: None,
            };
            return Ok(Some(spl_transfer))
        } else if let SplInstructionData::TransferCheckedWithFee { amount, decimals, fee } = token_instruction {
            let signers = self.get_spl_multisig_signers_if_exist(&accounts, 4)?;
            let spl_transfer = SplTransfer {
                amount: amount.to_string(),
                to: accounts[2].to_string(),
                from: accounts[0].to_string(),
                token_mint: Some(accounts[1].to_string()),
                signers,
                owner: accounts[3].to_string(),
                decimals: Some(decimals.to_string()),
                fee: Some(fee.to_string()),
            };
            return Ok(Some(spl_transfer))
        }
        return Ok(None)
    }

    fn get_spl_multisig_signers_if_exist(&self, accounts: &Vec<AddressAccount>, num_accts_before_signer: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if accounts.len() < num_accts_before_signer {
            return Err(format!("Invalid number of accounts provided for spl token transfer instruction").into())
        }
        Ok(accounts[num_accts_before_signer..accounts.len()].to_vec().into_iter().map(|a| a.to_string()).collect())
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

    fn signatures(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.signatures
            .iter()
            .map(|sig| sig.iter().map(|b| format!("{:02x}", b)).collect::<String>())
            .collect())
    }

    pub fn transaction_metadata(&self) -> Result<SolanaMetadata, Box<dyn Error>> {
        let (instructions, transfers, spl_transfers) = self.all_instructions_and_transfers()?;
        let signatures = self.signatures()?;
        Ok(SolanaMetadata {
            signatures,
            account_keys: self.all_account_key_strings(),
            address_table_lookups: self.address_table_lookups(),
            recent_blockhash: self.recent_blockhash(),
            program_keys: self.all_invoked_programs(),
            instructions,
            transfers,
            spl_transfers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_to_vec(inp: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let unsigned_tx_bytes: Vec<u8> = (0..inp.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&inp[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| "input provided is invalid when converted to bytes")?;
        Ok(unsigned_tx_bytes)
    }

#[test]
    fn test_read_compact_u16() {
        // Test compact header with a single byte
        let test_input_1 = "05FFFFFFFFFF";
        let test_bytes_1 = hex_to_vec(test_input_1).unwrap();
        let (len_1, rem_1) = read_compact_u16(&test_bytes_1).unwrap();
        assert_eq!(len_1, 5);
        assert_eq!(rem_1, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        // Test compact header with two bytes
        let test_input_2 = "8001FFFFFFFFFF";
        let test_bytes_2 = hex_to_vec(test_input_2).unwrap();
        let (len_2, rem_2) = read_compact_u16(&test_bytes_2).unwrap();
        assert_eq!(len_2, 128);
        assert_eq!(rem_2, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        // Test compact header with 3 bytes (VALID)
        let test_input_3 = "808003FFFFFFFFFF";
        let test_bytes_3 = hex_to_vec(test_input_3).unwrap();
        let (len_3, rem_3) = read_compact_u16(&test_bytes_3).unwrap();
        assert_eq!(len_3, 49152);
        assert_eq!(rem_3, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        // Test compact header with 3 bytes (INVALID, last byte has more than the 2 least significant digits populated since 0x04 == 0000 0100)
        let test_input_4 = "808004FFFFFFFFFF";
        let test_bytes_4 = hex_to_vec(test_input_4).unwrap();
        let decoding_err_1 = read_compact_u16(&test_bytes_4).unwrap_err();
        assert_eq!(
            decoding_err_1.to_string(),
            "error parsing unsigned transaction: third byte in compact u16 has more than it's 2 least significant bits set",
        );

        // Test compact header with 2 bytes (INVALID, 2 continuation bytes but then nothing after)
        let test_input_5 = "8080";
        let test_bytes_5 = hex_to_vec(test_input_5).unwrap();
        let decoding_err_2 = read_compact_u16(&test_bytes_5).unwrap_err();
        assert_eq!(
            decoding_err_2.to_string(),
            "error parsing unsigned transaction: unable to parse compact array header, not enough bytes",
        );
    }
}