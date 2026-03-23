//! Embedded IDL files - all built-in IDLs are compiled into the binary
//! This eliminates the need for runtime file access and ensures the library
//! works regardless of the working directory.

// Embed all built-in IDL files at compile time
pub const APE_PRO_IDL: &str = include_str!("idls/ape_pro.json");
pub const CANDY_MACHINE_IDL: &str = include_str!("idls/cndy.json");
pub const DRIFT_IDL: &str = include_str!("idls/drift.json");
pub const JUPITER_LIMIT_IDL: &str = include_str!("idls/jupiter_limit.json");
pub const JUPITER_IDL: &str = include_str!("idls/jupiter.json");
pub const KAMINO_IDL: &str = include_str!("idls/kamino.json");
pub const LIFINITY_IDL: &str = include_str!("idls/lifinity.json");
pub const METEORA_IDL: &str = include_str!("idls/meteora.json");
pub const OPENBOOK_IDL: &str = include_str!("idls/openbook.json");
pub const ORCA_IDL: &str = include_str!("idls/orca.json");
pub const RAYDIUM_IDL: &str = include_str!("idls/raydium.json");
pub const STABBLE_IDL: &str = include_str!("idls/stabble.json");
pub const JUPITER_AGG_V6_IDL: &str = include_str!("idls/jupiter_agg_v6.json");
