use std::fmt;
use serde::{Deserialize, Serialize};

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

// Contains a reference to "uploaded" IDL's 
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdlRecord {
    pub program_id: String,
    pub program_name: String,
    pub file_path: String,
}

/// IDL that is compatible with what anchor and shank extract from a solana program.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Idl {
    /// This is the program ID of the 
    pub program_id: String,

    /// This is the name of the program
    pub name: String,

    /// Instructions that are handled by the program.
    pub instructions: Vec<IdlInstruction>,

    /// Types defined in the program that are used by account structs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub types: Vec<IdlTypeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlInstruction {
    /// Name of the instruction.
    pub name: String,

    /// Name of the instruction.
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

fn is_false(x: &bool) -> bool {
    !x
}

/// Custom type definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlTypeDefinition {
    /// Name of the struct or enum.
    pub name: String,

    /// Underlying type description.
    /// Note: This is named ty and not type because type is a type is a reserved name in rust
    #[serde(rename = "type")]
    pub ty: IdlTypeDefinitionTy,
}

/// A field in a struct, enum variant or [IdlInstruction] args.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlField {
    /// Name of the field.
    pub name: String,

    /// Type of the field.
    #[serde(rename = "type")]
    pub ty: IdlType,
}

/// Underlying fields of a tuple or struct [IdlEnumVariant].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EnumFields {
    Named(Vec<IdlField>),
    Tuple(Vec<IdlType>),
}

/// An enum variant which could be scalar (withouth fields) or tuple/struct (with fields).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlEnumVariant {
    /// Name of the variant.
    pub name: String,

    /// Optional fields of the variant, only present when it is a tuple or a struct variant.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fields: Option<EnumFields>,
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum IdlTypeDefinitionTy {
    Struct { fields: Vec<IdlField> },
    Enum { variants: Vec<IdlEnumVariant> },
    Alias { value: IdlType },
}

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
    #[serde(rename = "coption")]
    COption(Box<IdlType>),
    Tuple(Vec<IdlType>),
    #[serde(alias = "pubkey")]
    PublicKey,
    String,
    U128,
    U16,
    U32,
    U64,
    U8,
    Vec(Box<IdlType>),
    HashMap(Box<IdlType>, Box<IdlType>),
    BTreeMap(Box<IdlType>, Box<IdlType>),
    HashSet(Box<IdlType>),
    BTreeSet(Box<IdlType>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Defined {
    String(String),
    Object { name: String },
}

// First 4 bytes -- discriminator 
// Arg 1 -- F32
// Arg 2 -- String 
// Arg 3 - I128
// Arg 2 -- String
// Arg 3 - I128

