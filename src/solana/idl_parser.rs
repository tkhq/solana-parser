use crate::solana::structs::{
    AccountAddress, CustomIdl, CustomIdlConfig, Defined, EnumFields, Idl, IdlInstruction,
    IdlRecord, IdlType, IdlTypeDefinition, IdlTypeDefinitionType, ProgramType,
};
use bs58;
use byteorder::{LittleEndian, ReadBytesExt};
use serde_json::{from_str, from_value, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

/// Compresses an IDL JSON string by removing all whitespace
fn compress_idl_json(idl_json: &str) -> String {
    idl_json.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Computes SHA256 hash of the compressed IDL JSON string
/// this doesn't canonicalize the JSON by sorting it or anything, so if your JSON isn't exactly same, you will get some inconsistencies
/// we can resolve this by implementing a custom serializer for IDLs
pub fn compute_idl_hash(idl_json: &str) -> String {
    let compressed = compress_idl_json(idl_json);
    let mut hasher = Sha256::new();
    hasher.update(compressed.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

// Required Top-Level fields in Solana IDL JSON's
/*
   The fields that are NEEDED in an uploaded Solana IDL, for the purposes of parsing instruction call data are:
   - instructions: This field is an array containing information related to all instructions on this IDL, including the name of the instruction,
                   the byte discriminator that is used at the beginning of call data corresponding to this instruction, and the arguments
   - types: This field is an array containing all custom defined types (structs) that arguments to this IDL's instruction use for formatting
*/
const IDL_INSTRUCTIONS_KEY: &str = "instructions";
const IDL_TYPES_KEY: &str = "types";

// The default instruction Discriminator (DISC) length is used in the process of calculating discriminators that are not explicitly included in IDL's, according to Anchor's protocol for calculating discriminators
// Reference for calculating the default discriminator - https://www.anchor-lang.com/docs/basics/idl#discriminators
const IDL_INST_DEFAULT_DISC_LEN: usize = 8;

/*
    SIZE CONSTRAINTS: The below constants are configurable value thresholds to guard against maliciously formatted IDL's
    - MAX_DEFINED_TYPE_DEPTH: For solana IDL defined types, this value defines the max depth for defined types within defined types during the type resolution process
    - MAX_CURSOR_LENGTH: This is the max length of a cursor that's allowed for IDL parsing (currently set to 1232 bytes which is the max size of a serialized solana transaction)
    - MAX_ALLOC_PER_CURSOR_LENGTH: This is the max memory allocation per byte in the cursor, which is currently set at 24 bytes which is the typical heap allocation overhead for pointers
*/
const MAX_DEFINED_TYPE_DEPTH: usize = 10; // Max depth for defined types
const MAX_CURSOR_LENGTH: usize = 1232; // Max size in bytes of a serialized Solana transaction
const MAX_ALLOC_PER_CURSOR_LENGTH: usize = 24; // Typical heap allocation overhead for pointers

/// Constructs a mapping from program_id to IdlRecord for all built-in IDLs.
/// IDLs are embedded at compile time and do not require file system access.
#[allow(dead_code)] // Public API - exported from lib.rs
pub fn construct_custom_idl_records_map(
) -> Result<HashMap<String, IdlRecord>, Box<dyn std::error::Error>> {
    let mut idl_map = HashMap::new();

    for program_type in ProgramType::all() {
        let program_id = program_type.program_id().to_string();
        let idl_record = IdlRecord {
            program_name: program_type.program_name().to_string(),
            program_id: program_id.clone(),
            program_type: Some(program_type.clone()),
            custom_idl: None,
            custom_idl_json: None,
            override_builtin: false,
        };

        idl_map.insert(program_id, idl_record);
    }

    Ok(idl_map)
}

/// Constructs custom IDL records map with optional custom IDLs (JSON string version).
/// This is the legacy API that accepts JSON strings.
#[allow(dead_code)] // Public API - exported from lib.rs
pub fn construct_custom_idl_records_map_with_overrides(
    custom_idls: Option<HashMap<String, (String, bool)>>, // program_id -> (idl_json, override_builtin)
) -> Result<HashMap<String, IdlRecord>, Box<dyn std::error::Error>> {
    // Convert old API to new API
    let custom_configs = custom_idls.map(|idls| {
        idls.into_iter()
            .map(|(program_id, (json, override_builtin))| {
                (
                    program_id,
                    CustomIdlConfig::from_json(json, override_builtin),
                )
            })
            .collect()
    });
    construct_idl_records_map(custom_configs)
}

/// Constructs IDL records map with optional custom IDLs.
/// This is the new API that supports both pre-parsed Idl structs and JSON strings.
///
/// # Arguments
/// * `custom_idls` - Optional map of program_id -> CustomIdlConfig
pub fn construct_idl_records_map(
    custom_idls: Option<HashMap<String, CustomIdlConfig>>,
) -> Result<HashMap<String, IdlRecord>, Box<dyn std::error::Error>> {
    let mut idl_map = HashMap::new();

    // First, load all built-in IDLs (using embedded IDL data)
    for program_type in ProgramType::all() {
        let program_id = program_type.program_id().to_string();
        let idl_record = IdlRecord {
            program_name: program_type.program_name().to_string(),
            program_id: program_id.clone(),
            program_type: Some(program_type.clone()),
            custom_idl: None,
            custom_idl_json: None,
            override_builtin: false,
        };

        idl_map.insert(program_id, idl_record);
    }

    // Then, add or override with custom IDLs if provided
    if let Some(custom_idls) = custom_idls {
        for (program_id, config) in custom_idls {
            // Parse the custom IDL if it's JSON, otherwise use the pre-parsed version
            let (custom_idl, custom_idl_json) = match config.idl {
                CustomIdl::Parsed(idl) => {
                    // For pre-parsed IDLs, we serialize to JSON for hash computation
                    let json = serde_json::to_string(&idl)?;
                    (idl, json)
                }
                CustomIdl::Json(json) => {
                    let idl = decode_idl_data(&json)?;
                    (idl, json)
                }
            };

            if let Some(existing_record) = idl_map.get_mut(&program_id) {
                // Update existing record with custom IDL
                existing_record.custom_idl = Some(custom_idl);
                existing_record.custom_idl_json = Some(custom_idl_json);
                existing_record.override_builtin = config.override_builtin;
            } else {
                // Create new record for unknown program
                let short_id = if program_id.len() >= 8 {
                    &program_id[..8]
                } else {
                    &program_id
                };
                let idl_record = IdlRecord {
                    program_name: format!("Custom Program {short_id}"),
                    program_id: program_id.clone(),
                    program_type: None,
                    custom_idl: Some(custom_idl),
                    custom_idl_json: Some(custom_idl_json),
                    override_builtin: true, // Always override for unknown programs
                };
                idl_map.insert(program_id, idl_record);
            }
        }
    }

    Ok(idl_map)
}

/// Get the resolved IDL and its JSON string for an IdlRecord.
/// Returns (Idl, idl_json_str, IdlSource)
pub fn resolve_idl_for_record(
    idl_record: &IdlRecord,
    program_key: &str,
) -> Result<(Idl, String, crate::solana::structs::IdlSource), Box<dyn std::error::Error>> {
    use crate::solana::structs::IdlSource;

    // Determine which IDL to use
    if let Some(ref custom_idl) = idl_record.custom_idl {
        // Custom IDL provided
        if idl_record.override_builtin || idl_record.program_type.is_none() {
            // Use custom IDL (either override is set or no built-in exists)
            let json = idl_record
                .custom_idl_json
                .clone()
                .ok_or("Custom IDL present but JSON string missing")?;
            return Ok((custom_idl.clone(), json, IdlSource::Custom));
        }
    }

    // Use built-in IDL
    if let Some(ref program_type) = idl_record.program_type {
        let builtin_json = program_type.idl_json();
        let builtin_idl = decode_idl_data(builtin_json)?;
        Ok((
            builtin_idl,
            builtin_json.to_string(),
            IdlSource::BuiltIn(program_type.clone()),
        ))
    } else if let Some(ref custom_idl) = idl_record.custom_idl {
        // Fallback to custom IDL if no built-in
        let json = idl_record
            .custom_idl_json
            .clone()
            .ok_or("Custom IDL present but JSON string missing")?;
        Ok((custom_idl.clone(), json, IdlSource::Custom))
    } else {
        Err(format!("No IDL available for program: {program_key}").into())
    }
}

// The TypeResolver struct helps resolved defined types within an IDL during the parsing of instruction call data
struct TypeResolver<'a> {
    type_cache: HashMap<String, &'a IdlTypeDefinition>,
}

impl<'a> TypeResolver<'a> {
    fn new(idl: &'a Idl) -> Result<Self, Box<dyn std::error::Error>> {
        let mut type_cache = HashMap::new();
        for ty in &idl.types {
            if type_cache.contains_key(&ty.name) {
                return Err(
                    format!("multiple types with the same name detected: {}", &ty.name).into(),
                );
            }
            type_cache.insert(ty.name.clone(), ty);
        }
        check_idl_for_cycles(idl.clone(), &type_cache)?;
        Ok(Self { type_cache })
    }

    fn resolve(&self, name: &str) -> Option<&IdlTypeDefinition> {
        self.type_cache.get(name).copied()
    }
}

/*
   SizeGuard allocates memory for performance while making sure that a parsing procedure does not exceed the budget
*/
struct SizeGuard {
    remaining_budget: usize,
}

impl SizeGuard {
    fn new(total_budget: usize) -> Self {
        Self {
            remaining_budget: total_budget,
        }
    }

    // This method checks to see whether there is enough memory left in the budget to be allocated, and if so, creates a byte vector of the correct length
    fn create_allocated_buffer(
        &mut self,
        len: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if len > self.remaining_budget {
            return Err(
                "memory allocation exceeded maximum allowed budget while parsing IDL call data -- check your uploaded IDL or call data".into(),
            );
        }
        self.remaining_budget -= len;

        Ok(vec![0u8; len])
    }

    // This method checks to see whether there is enough memory left in the budget to be allocated, and if so, creates a vector of serde json value type objects of the correct length
    fn create_allocated_arg_vector(
        &mut self,
        len: usize,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let amount = len * size_of::<serde_json::Value>();

        if amount > self.remaining_budget {
            return Err(
                "memory allocation exceeded maximum allowed budget while parsing IDL call data -- check your uploaded IDL or call data".into());
        }
        self.remaining_budget -= amount;

        Ok(Vec::with_capacity(len))
    }
}

/*
    PARSING FUNCTIONS BEGIN HERE
*/

// the Decode IDL Data method takes an IDL json string and parses it into IDL rust structs to be used to parse passed in instruction data
pub fn decode_idl_data(idl_json: &str) -> Result<Idl, Box<dyn std::error::Error>> {
    // Parse IDL from JSON string into Maps
    let idl_map: Map<String, Value> =
        from_str(idl_json).map_err(|e| -> Box<dyn std::error::Error> {
            format!("unable to parse IDL: Invalid JSON with error: {e}").into()
        })?;

    // Parse instructions array
    let instructions = validate_idl_array(&idl_map, IDL_INSTRUCTIONS_KEY)?;
    let mut parsed_instructions: Vec<IdlInstruction> = vec![];
    for i in instructions {
        let parsed_i: IdlInstruction =
            from_value(i).map_err(|e| -> Box<dyn std::error::Error> {
                format!("failed to parse instructions array in uploaded IDL with error: {e}").into()
            })?;
        parsed_instructions.push(parsed_i);
    }

    // Create discriminators using default anchor discriminator for all instructions without explicitly included discriminators
    for i in &mut parsed_instructions {
        if i.discriminator.is_none() {
            i.discriminator = Some(compute_default_anchor_discriminator(&i.name)?);
        }
    }

    // Parse defined types array
    let types = validate_idl_array(&idl_map, IDL_TYPES_KEY)?;
    let mut parsed_types: Vec<IdlTypeDefinition> = vec![];
    for t in types {
        let parsed_t: IdlTypeDefinition =
            from_value(t).map_err(|e| -> Box<dyn std::error::Error> {
                format!("failed to parse types array in uploaded IDL with error: {e}").into()
            })?;
        parsed_types.push(parsed_t);
    }

    let parsed_idl = Idl {
        instructions: parsed_instructions,
        types: parsed_types,
    };

    // Validate IDL by checking for type cycles by creating a type resolver -- which checks for cycles during initialization
    TypeResolver::new(&parsed_idl)?;

    Ok(parsed_idl)
}

// This method takes in a json object, and validates the existence of an ARRAY at a particular key (for example the top level instructions array within all IDL Json's)
// It then returns the value at the provided key, cast as an array type
fn validate_idl_array(
    idl_map: &Map<String, Value>,
    key: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let value = idl_map
        .get(key)
        .ok_or_else(|| -> Box<dyn std::error::Error> {
            format!("key '{key}' not found in uploaded IDL").into()
        })?;
    let checked_value = value
        .as_array()
        .ok_or_else(|| -> Box<dyn std::error::Error> {
            format!("value for key '{key}' must be a JSON array").into()
        })?;
    Ok(checked_value.clone())
}

// This method computes the default anchor discriminator for an instruction using it's instruction name (only computes if the instruction discriminators are not EXPLICITLY provided)
// Reference for calculating the default discriminator - https://www.anchor-lang.com/docs/basics/idl#discriminators
pub fn compute_default_anchor_discriminator(
    instruction_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if instruction_name.is_empty() {
        return Err("attempted to compute the default anchor instruction discriminator for an instruction with no name".into());
    }

    // All anchor generated IDL's use the snake_case representation of instruction names to generate the default function discriminator (not officially documented)
    let snake_case = to_snake_case(instruction_name);
    let seed = format!("global:{snake_case}");
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();
    if result.len() < IDL_INST_DEFAULT_DISC_LEN {
        return Err(
            format!(
                "error calculating default anchor instruction discriminator for instruction with name: {snake_case}",
            ).into());
    }

    Ok(result[..IDL_INST_DEFAULT_DISC_LEN].into())
}

// helper method for discriminator calculation -- converts all instruction names into snake_case
fn to_snake_case(target: &str) -> String {
    let mut new_str = String::new();
    for (i, c) in target.chars().enumerate() {
        // add _ if we encounter a uppercase letter
        if c.is_uppercase() && i != 0 {
            new_str.push('_');
        }
        new_str.push(c.to_ascii_lowercase());
    }
    new_str
}

// Create Accounts list takes in all accounts provided to this instruction, as parsed by our transaction parser (both static and look ups) and creates a map of the names of addresses (as specified by the IDL) to the address public keys (if they are statically included) or ADDRESS-TABLE-LOOKUP if not
pub fn create_accounts_map(
    accounts: &[AccountAddress],
    instruction_spec: &IdlInstruction,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if accounts.len() < instruction_spec.accounts.len() {
        return Err(format!(
            "too few accounts provided in transaction payload for instruction {}",
            instruction_spec.name
        )
        .into());
    }

    let mut acct_map: HashMap<String, String> = HashMap::new();
    for (account_spec, account) in instruction_spec.accounts.iter().zip(accounts.iter()) {
        acct_map.insert(account_spec.name.clone(), account.to_string());
    }
    Ok(acct_map)
}

// This method compares all instructions in the chosen IDL against the instruction call data to be parsed to find the correct instruction being called, handling error cases appropriately
pub fn find_instruction_by_discriminator(
    instruction_data: &[u8],
    instructions: Vec<IdlInstruction>,
) -> Result<IdlInstruction, Box<dyn std::error::Error>> {
    if instruction_data.len() > MAX_CURSOR_LENGTH {
        return Err("instruction call data exceeded max cursor length".into());
    }

    for i in instructions {
        let disc = i
            .clone()
            .discriminator
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!(
                    "no discriminator found for instruction {} found in IDL",
                    i.name
                )
                .into()
            })?;

        // Validate length of instruction data, to make sure it has enough bytes for the discriminator
        if instruction_data.len() < disc.len() {
            continue;
        }

        // Check for matching instruction
        if instruction_data[..disc.len()] == disc {
            return Ok(i);
        }
    }
    let inst_data_string = hex::encode(instruction_data);
    Err(format!(
        "no matching instruction discriminator found for instruction data: {inst_data_string:?}"
    )
    .into())
}

// Parse data into args -- takes in the IDL instruction object corresponding to the instruction call data, as well as the instruction call data and parses the data into a vector of arguments
pub fn parse_data_into_args(
    data: &[u8],
    idl_instruction: &IdlInstruction,
    idl: &Idl,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let mut data_cursor = Cursor::new(data);
    let resolver = TypeResolver::new(idl)?;

    // Validate discriminator length and set cursor to correct position
    let disc =
        idl_instruction
            .clone()
            .discriminator
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!(
                    "no discriminator found for instruction {} not found in IDL",
                    idl_instruction.name
                )
                .into()
            })?;
    if data.len() < disc.len() {
        // we should not get here since we've checked the length of the discriminator
        return Err(
            format!(
                "error while parsing data into instruction with name: {}. Discriminator longer than data bytes",
                idl_instruction.name).into());
    }

    // set cursor being used to parse the instruction data to the correct position
    data_cursor.set_position(disc.len() as u64);

    // Initialize size guard
    let mut size_guard = SizeGuard::new(MAX_CURSOR_LENGTH * MAX_ALLOC_PER_CURSOR_LENGTH);

    // parse all arguments
    let mut args = serde_json::Map::new();
    for arg in &idl_instruction.args {
        let parsed_arg = parse_type(&mut data_cursor, &arg.r#type, &resolver, &mut size_guard)
            .map_err(|e| -> Box<dyn std::error::Error> {
                format!("failed to parse IDL argument with error: {}", e).into()
            })?;
        args.insert(arg.name.clone(), parsed_arg);
    }

    // Error if data bytes still remaining after parsing all expected arguments
    let cursor_position =
        usize::try_from(data_cursor.position()).map_err(|e| -> Box<dyn std::error::Error> {
            format!(
                "invalid cursor position while parsing solana IDL data {}",
                e
            )
            .into()
        })?;
    if cursor_position != data.len() {
        if cursor_position < data.len() {
            return Err(
                "extra unexpected bytes remaining at the end of instruction call data".into(),
            );
        }
        return Err("cursor out of bounds".into());
    }

    Ok(args)
}

#[allow(clippy::too_many_lines)]
// Parse Type -- given a type, this method attempts to parse the next part of the intruction call data (as tracked by the cursor) into that type
fn parse_type<R: Read>(
    reader: &mut R,
    ty: &IdlType,
    resolver: &TypeResolver,
    size_guard: &mut SizeGuard,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match ty {
        // Primitive types
        IdlType::Bool => {
            let bool_val = reader.read_u8()?;
            Ok(if bool_val == 0 {
                serde_json::Value::Bool(false)
            } else {
                serde_json::Value::Bool(true)
            })
        }
        IdlType::I8 => Ok(reader.read_i8()?.into()),
        IdlType::I16 => Ok(reader.read_i16::<LittleEndian>()?.into()),
        IdlType::I32 => Ok(reader.read_i32::<LittleEndian>()?.into()),
        IdlType::I64 => Ok(reader.read_i64::<LittleEndian>()?.into()),
        IdlType::I128 => {
            let mut buf = [0u8; 16];
            reader.read_exact(&mut buf)?;
            Ok(i128::from_le_bytes(buf).to_string().into())
        }
        IdlType::U8 => Ok(reader.read_u8()?.into()),
        IdlType::U16 => Ok(reader.read_u16::<LittleEndian>()?.into()),
        IdlType::U32 => Ok(reader.read_u32::<LittleEndian>()?.into()),
        IdlType::U64 => Ok(reader.read_u64::<LittleEndian>()?.into()),
        IdlType::U128 => {
            let mut buf = [0u8; 16];
            reader.read_exact(&mut buf)?;
            Ok(u128::from_le_bytes(buf).to_string().into())
        }
        IdlType::F32 => Ok(reader.read_f32::<LittleEndian>()?.into()),
        IdlType::F64 => Ok(reader.read_f64::<LittleEndian>()?.into()),

        // Composite types
        IdlType::PublicKey => {
            let mut buf = [0u8; 32];
            reader.read_exact(&mut buf)?;
            Ok(bs58::encode(buf).into_string().into())
        }
        IdlType::String => {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            // Check size guard & allocate memory
            let mut buf = size_guard.create_allocated_buffer(len)?;

            reader.read_exact(&mut buf)?;
            Ok(String::from_utf8(buf)?.into())
        }
        IdlType::Bytes => {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            // Check size guard & allocate memory
            let mut buf = size_guard.create_allocated_buffer(len)?;

            reader.read_exact(&mut buf)?;
            Ok(serde_json::Value::String(hex::encode(&buf)))
        }

        // Container types
        IdlType::Array(ty, size) => {
            // Check size guard & allocate memory
            let mut arr = size_guard.create_allocated_arg_vector(*size)?;

            for _ in 0..*size {
                arr.push(parse_type(reader, ty, resolver, size_guard)?);
            }
            Ok(arr.into())
        }
        IdlType::Vec(ty) => {
            let len =
                reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| -> Box<dyn std::error::Error> {
                        format!("failed while parsing length header of argument of type vec: {e}")
                            .into()
                    })?;

            // Check size guard & allocate memory
            let mut vec = size_guard.create_allocated_arg_vector(len as usize)?;

            for _ in 0..len {
                vec.push(parse_type(reader, ty, resolver, size_guard)?);
            }
            Ok(vec.into())
        }
        IdlType::Option(ty) => {
            let flag = reader.read_u8()?;
            Ok(if flag == 0 {
                serde_json::Value::Null
            } else {
                parse_type(reader, ty, resolver, size_guard)?
            })
        }
        // Custom types
        IdlType::Defined(defined) => {
            let type_name = match defined {
                Defined::String(s) => s,
                Defined::Object { name } => name,
            };
            parse_defined_type(reader, type_name, resolver, size_guard)
        }
    }
}

// Parse Defined Type -- if the type being parsed is a defined type (it should have been defined in the IDL), this method resolves it recursively using parse_type
fn parse_defined_type<R: Read>(
    reader: &mut R,
    type_name: &str,
    resolver: &TypeResolver,
    size_guard: &mut SizeGuard,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let ty_def = resolver
        .resolve(type_name)
        .ok_or_else(|| format!("type {} not found in IDL", type_name))?;

    match &ty_def.r#type {
        IdlTypeDefinitionType::Struct { fields } => {
            let mut map = serde_json::Map::new();
            for field in fields {
                map.insert(
                    field.name.clone(),
                    parse_type(reader, &field.r#type, resolver, size_guard)?,
                );
            }
            Ok(map.into())
        }
        IdlTypeDefinitionType::Enum { variants } => {
            let variant_index = reader.read_u8()?;
            let variant = variants
                .get(variant_index as usize)
                .ok_or("invalid variant index")?;

            let value = match &variant.fields {
                Some(EnumFields::Tuple(types)) => {
                    let mut vec = Vec::new();
                    for ty in types {
                        vec.push(parse_type(reader, ty, resolver, size_guard)?);
                    }
                    serde_json::Value::Array(vec)
                }
                Some(EnumFields::Named(fields)) => {
                    let mut map = serde_json::Map::new();
                    for field in fields {
                        map.insert(
                            field.name.clone(),
                            parse_type(reader, &field.r#type, resolver, size_guard)?,
                        );
                    }
                    serde_json::Value::Object(map)
                }
                None => serde_json::Value::Null,
            };

            Ok(serde_json::json!({
                variant.name.clone(): value
            }))
        }
        IdlTypeDefinitionType::Alias { value } => parse_type(reader, value, resolver, size_guard),
    }
}

/*
    Cycle Checking in Defined types
*/

// Check IDL for Cycles -- takes in an IDL and checks to see whether the defined types contains any cycles (invalid case)
fn check_idl_for_cycles(
    idl: Idl,
    type_cache: &HashMap<String, &IdlTypeDefinition>,
) -> Result<(), Box<dyn std::error::Error>> {
    for t in idl.types {
        cycle_recursive_check(type_cache.clone(), &t.name, HashSet::new())?;
    }
    Ok(())
}

// Cycle Recursive Check -- is a recursive helper function that checks a parsed IDL for cycles
fn cycle_recursive_check(
    type_cache: HashMap<String, &IdlTypeDefinition>,
    type_name: &str,
    mut path: HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check to see whether max depth has been exceeded
    if path.len() > MAX_DEFINED_TYPE_DEPTH {
        return Err(
            format!("defined types resolution max depth exceeded on type: {type_name}").into(),
        );
    }

    // Check to see whether a cycle has been detected
    if path.contains(type_name) {
        return Err(
            format!("defined types cycle check failed. Recursive type found: {type_name}").into(),
        );
    }
    path.insert(type_name.to_string());

    let ty_def =
        type_cache
            .get(type_name)
            .copied()
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!("type {type_name} not found in IDL").into()
            })?;

    match &ty_def.r#type {
        IdlTypeDefinitionType::Struct { fields } => {
            for field in fields {
                if let IdlType::Defined(defined) = &field.r#type {
                    let type_name = defined.to_string();
                    cycle_recursive_check(type_cache.clone(), &type_name, path.clone())?;
                }
            }
            Ok(())
        }
        IdlTypeDefinitionType::Enum { variants } => {
            for variant in variants {
                if let Some(fields) = &variant.fields {
                    for ty in fields.types() {
                        if let IdlType::Defined(defined) = ty {
                            let type_name = defined.to_string();
                            cycle_recursive_check(type_cache.clone(), &type_name, path.clone())?;
                        }
                    }
                }
            }
            Ok(())
        }
        IdlTypeDefinitionType::Alias { value } => {
            if let IdlType::Defined(defined) = value {
                let type_name = defined.to_string();
                cycle_recursive_check(type_cache, &type_name, path.clone())?;
            }
            Ok(())
        }
    }
}
