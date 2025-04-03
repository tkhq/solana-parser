use std::fmt;

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