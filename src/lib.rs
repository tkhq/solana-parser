pub mod solana;

// Re-export commonly used types and functions for convenience
pub use solana::idl_parser::{
    compute_idl_hash, construct_custom_idl_records_map,
    construct_custom_idl_records_map_with_overrides, construct_idl_records_map, decode_idl_data,
};
pub use solana::parser::{parse_transaction, parse_transaction_with_idls};
pub use solana::structs::{
    CustomIdl, CustomIdlConfig, Idl, IdlSource, ProgramType, SolanaInstruction, SolanaMetadata,
    SolanaParseResponse, SolanaParsedInstructionData, SolanaParsedTransaction,
    SolanaParsedTransactionPayload,
};
