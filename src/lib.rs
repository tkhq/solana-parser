pub mod solana;

// Re-export commonly used types and functions for convenience
pub use solana::idl_parser::{
    compute_idl_hash, construct_custom_idl_records_map,
    construct_custom_idl_records_map_with_overrides, construct_idl_records_map, decode_idl_data,
    find_instruction_by_discriminator, parse_instruction_with_idl,
};
pub use solana::parser::{parse_transaction, parse_transaction_with_idls};
pub use solana::structs::{
    CustomIdl, CustomIdlConfig, Idl, IdlInstruction, IdlParseError, IdlSource, ProgramType,
    SolanaInstruction, SolanaMetadata, SolanaParseResponse, SolanaParsedInstructionData,
    SolanaParsedTransaction, SolanaParsedTransactionPayload,
};
