pub mod solana {
    pub mod parser;
    pub mod structs;
    pub mod idl_parser;
    pub mod idl_db;
}

// Re-export commonly used types and functions for convenience
pub use solana::parser::parse_transaction;
pub use solana::structs::{
    SolanaParseResponse, SolanaParsedTransaction, SolanaParsedTransactionPayload,
    SolanaMetadata, SolanaInstruction, SolanaParsedInstructionData,
    ProgramType, IdlSource,
};
pub use solana::idl_parser::compute_idl_hash;