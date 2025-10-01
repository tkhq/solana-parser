use std::{fmt, collections::HashMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ProgramType represents the built-in IDL types supported by the library
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramType {
    ApePro,
    CandyMachine,
    Drift,
    JupiterLimit,
    Jupiter,
    Kamino,
    Lifinity,
    Meteora,
    Openbook,
    Orca,
    Raydium,
    Stabble,
    JupiterAggregatorV6,
}

impl ProgramType {
    /// Returns the program ID associated with this ProgramType
    pub fn program_id(&self) -> &str {
        match self {
            ProgramType::ApePro => "JSW99DKmxNyREQM14SQLDykeBvEUG63TeohrvmofEiw",
            ProgramType::CandyMachine => "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ",
            ProgramType::Drift => "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH",
            ProgramType::JupiterLimit => "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X",
            ProgramType::Jupiter => "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB",
            ProgramType::Kamino => "6LtLpnUFNByNXLyCoK9wA2MykKAmQNZKBdY8s47dehDc",
            ProgramType::Lifinity => "2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c",
            ProgramType::Meteora => "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
            ProgramType::Openbook => "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb",
            ProgramType::Orca => "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
            ProgramType::Raydium => "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
            ProgramType::Stabble => "swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ",
            ProgramType::JupiterAggregatorV6 => "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
        }
    }

    /// Returns the file path for this ProgramType
    pub fn file_path(&self) -> &str {
        match self {
            ProgramType::ApePro => "ape_pro.json",
            ProgramType::CandyMachine => "cndy.json",
            ProgramType::Drift => "drift.json",
            ProgramType::JupiterLimit => "jupiter_limit.json",
            ProgramType::Jupiter => "jupiter.json",
            ProgramType::Kamino => "kamino.json",
            ProgramType::Lifinity => "lifinity.json",
            ProgramType::Meteora => "meteora.json",
            ProgramType::Openbook => "openbook.json",
            ProgramType::Orca => "orca.json",
            ProgramType::Raydium => "raydium.json",
            ProgramType::Stabble => "stabble.json",
            ProgramType::JupiterAggregatorV6 => "jupiter_agg_v6.json",
        }
    }

    /// Returns the program name for this ProgramType
    pub fn program_name(&self) -> &str {
        match self {
            ProgramType::ApePro => "Ape Pro",
            ProgramType::CandyMachine => "Metaplex Candy Machine",
            ProgramType::Drift => "Drift Protocol V2",
            ProgramType::JupiterLimit => "Jupiter Limit",
            ProgramType::Jupiter => "Jupiter Swap",
            ProgramType::Kamino => "Kamino",
            ProgramType::Lifinity => "Lifinity Swap V2",
            ProgramType::Meteora => "Meteora",
            ProgramType::Openbook => "Openbook",
            ProgramType::Orca => "Orca Whirlpool",
            ProgramType::Raydium => "Raydium",
            ProgramType::Stabble => "Stabble",
            ProgramType::JupiterAggregatorV6 => "Jupiter Aggregator V6",
        }
    }

    /// Looks up a ProgramType by program_id
    pub fn from_program_id(program_id: &str) -> Option<ProgramType> {
        match program_id {
            "JSW99DKmxNyREQM14SQLDykeBvEUG63TeohrvmofEiw" => Some(ProgramType::ApePro),
            "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ" => Some(ProgramType::CandyMachine),
            "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH" => Some(ProgramType::Drift),
            "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X" => Some(ProgramType::JupiterLimit),
            "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB" => Some(ProgramType::Jupiter),
            "6LtLpnUFNByNXLyCoK9wA2MykKAmQNZKBdY8s47dehDc" => Some(ProgramType::Kamino),
            "2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c" => Some(ProgramType::Lifinity),
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo" => Some(ProgramType::Meteora),
            "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb" => Some(ProgramType::Openbook),
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => Some(ProgramType::Orca),
            "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C" => Some(ProgramType::Raydium),
            "swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ" => Some(ProgramType::Stabble),
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => Some(ProgramType::JupiterAggregatorV6),
            _ => None,
        }
    }
}

/// IdlSource indicates whether a built-in or custom IDL was used for parsing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdlSource {
    BuiltIn(ProgramType),
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaMetadata {
    pub signatures: Vec<String>,
    pub account_keys: Vec<String>,
    pub program_keys: Vec<String>,
    pub instructions: Vec<SolanaInstruction>,
    pub transfers: Vec<SolTransfer>,
    pub spl_transfers: Vec<SplTransfer>,
    pub recent_blockhash: String,
    pub address_table_lookups: Vec<SolanaAddressTableLookup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaInstruction {
    pub program_key: String,
    pub accounts: Vec<SolanaAccount>,
    pub instruction_data_hex: String,
    pub address_table_lookups: Vec<SolanaSingleAddressTableLookup>,
    pub parsed_instruction: Option<SolanaParsedInstructionData>,
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
pub struct SplTransfer {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub owner: String,
    pub signers: Vec<String>, // This is an empty array if ths is not a multisig account with multiple signers 
    pub token_mint: Option<String>, 
    pub decimals: Option<String>,
    pub fee: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolanaParsedInstructionData {
    pub instruction_name: String,
    pub discriminator: String,
    pub named_accounts: HashMap<String, String>,
    pub program_call_args: serde_json::Map<std::string::String, Value>,
    /// Indicates whether a built-in or custom IDL was used
    pub idl_source: IdlSource,
    /// SHA256 hash of the compressed (whitespace removed) IDL JSON string
    pub idl_hash: String,
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

#[derive(Debug, Clone, PartialEq)]
pub enum AccountAddress {
    /// Static Account Addresses refer to addresses whose string/hex representations have been entirely included in the serialized transaction
    Static(SolanaAccount),
    /// `AddressTableLookUp` Addresses refer to addresses whose string/hex representation have NOT been included
    /// Rather, only a reference has been included using the concept of Address Lookup Tables -- <https://solana.com/developers/guides/advanced/lookup-tables>
    /// NOTE the difference between Address Lookup Tables and Address Table Lookups
    /// Address Lookup Tables -- the on chain tables where addresses are stored
    /// Address Table Lookups -- the struct that gets serialized with transactions that is used to POINT to an address in a lookup table --> <https://github.com/solana-labs/solana-web3.js/blob/4e9988cfc561f3ed11f4c5016a29090a61d129a8/src/message/index.ts#L27-L30>
    AddressTableLookUp(SolanaSingleAddressTableLookup),
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountAddress::Static(account) => write!(f, "{}", account.account_key),
            AccountAddress::AddressTableLookUp(_) => write!(f, "ADDRESS_TABLE_LOOKUP"),
        }
    }
}

/*
    SOLANA IDL TYPE DEFINITIONS
    - All types below are used in Solana IDL data parsing for custom uploaded IDL's
    - Some structs are vendored from an IDL parsing library with some modifications
    - vendor docs reference: <https://github.com/metaplex-foundation/shank/blob/9a8f2a77f6000d2d00e04f5aaa8c36a36765f567/shank-idl/src/idl_type.rs>
*/

// Contains a reference to "uploaded" IDL's
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdlRecord {
    pub program_id: String,
    pub program_name: String,
    pub file_path: String,
    /// Custom IDL JSON string, if provided by the caller
    pub custom_idl_json: Option<String>,
    /// Whether to override built-in IDL with custom one (if both exist)
    pub override_builtin: bool,
}

/// IDL that is compatible with what anchor and shank extract from a solana program.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Idl {
    /// Instructions that are handled by the program defined by this IDL
    pub instructions: Vec<IdlInstruction>,

    /// Types defined in the program defined by this IDL that are used by account structs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub types: Vec<IdlTypeDefinition>,
}

/// `IdlInstruction` outlines all information required to parse data into a particular instruction to a program
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlInstruction {
    /// Name of the instruction.
    pub name: String,

    /*
       ABI/IDL Discriminators
       - While some IDL's explicitly provide each instruction's discriminator, many do not. This is because Anchor has a standard way of calculating instruction discriminators.
       - Reference for calculating the default discriminator - https://www.anchor-lang.com/docs/basics/idl#discriminators
    */
    /// discriminator that denotes the unique instruction
    pub discriminator: Option<Vec<u8>>,

    /// Accounts that need to be supplied in order to process the instruction.
    pub accounts: Vec<IdlAccount>,

    /// Instruction args.
    pub args: Vec<IdlField>,
}

/// Metadata of an account that is provided when calling an instruction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IdlAccount {
    /// Name of the account used for documentation and by code generators.
    pub name: String,

    /// Whether the account is writable.
    #[serde(alias = "writable", skip_serializing_if = "is_false", default)]
    pub is_mut: bool,

    /// Whether the account is signer.
    #[serde(alias = "signer", skip_serializing_if = "is_false", default)]
    pub is_signer: bool,

    /// Whether the account is optional or not.
    #[serde(alias = "optional", skip_serializing_if = "is_false", default)]
    pub is_optional: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(x: &bool) -> bool {
    !x
}

/// Custom type definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlTypeDefinition {
    /// Name of the struct or enum.
    pub name: String,

    /// Underlying type description.
    #[serde(rename = "type")]
    pub r#type: IdlTypeDefinitionType,
}

/// A field in a struct, enum variant or [`IdlInstruction`] args.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlField {
    /// Name of the field.
    pub name: String,

    /// Type of the field.
    #[serde(rename = "type")]
    pub r#type: IdlType,
}

/// Underlying fields of a tuple or struct [`IdlEnumVariant`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EnumFields {
    Named(Vec<IdlField>),
    Tuple(Vec<IdlType>),
}

impl EnumFields {
    pub fn types(&self) -> Vec<IdlType> {
        match self {
            EnumFields::Named(fields) => fields.iter().map(|f| f.r#type.clone()).collect(),
            EnumFields::Tuple(types) => types.clone(),
        }
    }
}

/// An enum variant which could be scalar (without fields) or tuple/struct (with fields).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlEnumVariant {
    /// Name of the variant.
    pub name: String,

    /// Optional fields of the variant, only present when it is a tuple or a struct variant.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fields: Option<EnumFields>,
}

/// `IdlTypeDefinitionType` defines the different forms in which a type definition can come in
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum IdlTypeDefinitionType {
    /// The Struct variant parses IDL Defined types that are formatted as structs
    Struct { fields: Vec<IdlField> },
    /// The Enum variant parses IDL Defined types that are formatted as enums
    Enum { variants: Vec<IdlEnumVariant> },
    /// The Alias variant parses IDL Defined types that are formatted as aliases
    Alias { value: IdlType },
}

/// NOTE: The below vendored type has been modified to support ONLY the types supported by Anchor IDL's
/// Reference to Anchor types: <https://www.anchor-lang.com/docs/references/type-conversion>
/// Reference commit for type: <https://github.com/metaplex-foundation/shank/blob/9a8f2a77f6000d2d00e04f5aaa8c36a36765f567/shank-idl/src/idl_type.rs>
/// Types that can be included in accounts or user defined structs or instruction args of an IDL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IdlType {
    Array(Box<IdlType>, usize),
    Bool,
    Bytes,
    Defined(Defined),
    F32,
    F64,
    I128,
    I16,
    I32,
    I64,
    I8,
    Option(Box<IdlType>),
    #[serde(alias = "pubkey")]
    PublicKey,
    String,
    U128,
    U16,
    U32,
    U64,
    U8,
    Vec(Box<IdlType>),
}

/// The Defined type enum outlines the different formats in which Defined types can be referred to in the arguments to instructions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Defined {
    /// The String variant parses defined types that are formatted as strings
    String(String),
    /// The Object variant parses defined types that are formatted as objects with the defined type name under the key 'name'
    Object { name: String },
}

impl fmt::Display for Defined {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Defined::String(s) => write!(f, "{s}"),
            Defined::Object { name } => write!(f, "{name}"),
        }
    }
}

