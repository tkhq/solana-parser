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
