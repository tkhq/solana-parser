use serde_json::{from_str, from_value, Value, Map};
use std::error::Error;
use crate::solana::structs::{Idl, IdlTypeDefinition, IdlInstruction, IdlRecord, IdlType, Defined, EnumFields, IdlTypeDefinitionTy};
use crate::solana::idl_db::IDL_DB;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use bs58;

use solana_sdk::instruction::CompiledInstruction;

// Top Level Required Fields 
const IDL_INSTRUCTIONS_KEY: &str = "instructions";
const IDL_TYPES_KEY: &str = "types";

const IDL_INST_DEFAULT_DISC_LEN: usize = 8;

pub fn construct_custom_idl_records_map() -> Result<HashMap<String, IdlRecord>, Box<dyn Error>> {
    let mut idl_map = HashMap::new();
    
    for entry in IDL_DB {
        let program_id = entry.1.to_string(); 
        let idl_record = IdlRecord {
            program_name: entry.0.to_string(),
            program_id: entry.1.to_string(),
            file_path: entry.2.to_string(),
        };
        
        // Use insert() instead of indexing syntax
        idl_map.insert(program_id, idl_record);
    }
    
    Ok(idl_map)
}

pub fn decode_idl_data (idl_json: &str, program_id: &str, program_name: &str) -> Result<Idl, Box<dyn Error>> {
    // Parse IDL from JSON string into Maps
    let idl_map: Map<String, Value> = from_str(idl_json).map_err(|_| {
        Box::<dyn std::error::Error>::from("Unable to parse IDL: Invalid JSON")
    })?;

    // Parse instructions array
    let instructions = validate_idl_array(&idl_map, IDL_INSTRUCTIONS_KEY)?;
    let mut parsed_instructions: Vec<IdlInstruction> = vec![];
    for i in instructions {
        let parsed_i: IdlInstruction = from_value(i).map_err(|e| Box::<dyn Error>::from(format!("Failed to parse instructions array in uploaded IDL with error: {}", e)))?;
        parsed_instructions.push(parsed_i)
    }

    // Create discriminators using default anchor for all instructions without explicitly included discriminators
    for i in 0..parsed_instructions.len() {
        if parsed_instructions[i].discriminator.is_none() {
            parsed_instructions[i].discriminator = Some(compute_discriminator(&parsed_instructions[i].name)?);
        }
    }

    // Parse types array
    let types = validate_idl_array(&idl_map, IDL_TYPES_KEY)?;
    let mut parsed_types: Vec<IdlTypeDefinition> = vec![];
    for t in types {
        let parsed_t: IdlTypeDefinition = from_value(t).map_err(|e| Box::<dyn Error>::from(format!("Failed to parse types array in uploaded IDL with error: {}", e)))?;
        parsed_types.push(parsed_t)
    }
    
    Ok(Idl { program_id: program_id.to_string(), name: program_name.to_string(), instructions: parsed_instructions, types: parsed_types })
}

fn validate_idl_array(idl_map: &Map<String, Value>, key: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let value = idl_map.get(key).ok_or_else(|| format!("Key '{}' not found in uploaded IDL", key))?;
    let checked_value = value.as_array()
        .ok_or_else(|| format!("Value for Key '{}' must be a JSON array", key))?;
    Ok(checked_value.clone())
}

fn compute_discriminator(instruction_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let seed = format!("global:{}", instruction_name);
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();
    result[..IDL_INST_DEFAULT_DISC_LEN].try_into().map_err(|_| Box::<dyn Error>::from(format!("Failed to compute instruction byte discriminator for instruction: {}", instruction_name)))
}

// TODO TESTS 
// TEST correct parsing of isMut/writable, isSigner/signer, isOptional/optional
// TEST discriminators
// Add comments 

pub fn process_instruction_data(
    instruction_data: Vec<u8>,
    idl: Idl,
) -> Option<serde_json::Value> {
    // TODO ADD SAFETY CHECK
    // 2. Extract discriminator 
    let discriminator = &instruction_data[..8];
    
    // 3. Find matching instruction
    let instruction = idl.instructions.iter().find(|i| {
        i.discriminator.as_ref().map(|d| &d[..]) == Some(discriminator)
    })?;

    // 4. Parse data
    parse_data(&instruction_data, instruction, &idl).ok()
}

fn parse_data(
    data: &[u8],
    idl_instruction: &IdlInstruction,
    idl: &Idl,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut cursor = Cursor::new(data);
    let resolver = TypeResolver::new(idl);
    
    // Skip discriminator (first 8 bytes)
    cursor.set_position(8);
    
    let mut args = serde_json::Map::new();
    for arg in &idl_instruction.args {
        args.insert(arg.name.clone(), parse_type(&mut cursor, &arg.ty, &resolver)?);
    }
    
    Ok(serde_json::Value::Object(args))
}

fn parse_type<R: Read>(
    reader: &mut R,
    ty: &IdlType,
    resolver: &TypeResolver,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match ty {
        // Primitive types
        IdlType::Bool => Ok(reader.read_u8()?.into()),
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
        },
        IdlType::String => {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            Ok(String::from_utf8(buf)?.into())
        },
        IdlType::Bytes => {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            Ok(serde_json::Value::String(hex::encode(&buf)))
        },
        
        // Container types
        IdlType::Array(ty, size) => {
            let mut arr = Vec::with_capacity(*size);
            for _ in 0..*size {
                arr.push(parse_type(reader, ty, resolver)?);
            }
            Ok(arr.into())
        },
        IdlType::Vec(ty) => {
            let len = reader.read_u32::<LittleEndian>()?;
            let mut vec = Vec::with_capacity(len as usize);
            for _ in 0..len {
                vec.push(parse_type(reader, ty, resolver)?);
            }
            Ok(vec.into())
        },
        IdlType::Option(ty) => {
            let flag = reader.read_u8()?;
            Ok(if flag == 0 {
                serde_json::Value::Null
            } else {
                parse_type(reader, ty, resolver)?
            })
        },
        IdlType::COption(ty) => {
            let flag = reader.read_u32::<LittleEndian>()?;
            Ok(if flag == 0 {
                serde_json::Value::Null
            } else {
                parse_type(reader, ty, resolver)?
            })
        },
        IdlType::Tuple(tys) => {
            let mut values = Vec::with_capacity(tys.len());
            for ty in tys {
                values.push(parse_type(reader, ty, resolver)?);
            }
            Ok(values.into())
        },
        
        // Collection types
        IdlType::HashMap(k_ty, v_ty) | IdlType::BTreeMap(k_ty, v_ty) => {
            let len = reader.read_u32::<LittleEndian>()?;
            let mut entries = Vec::with_capacity(len as usize);
            for _ in 0..len {
                entries.push(serde_json::json!({
                    "key": parse_type(reader, k_ty, resolver)?,
                    "value": parse_type(reader, v_ty, resolver)?
                }));
            }
            Ok(entries.into())
        },
        IdlType::HashSet(ty) | IdlType::BTreeSet(ty) => {
            let len = reader.read_u32::<LittleEndian>()?;
            let mut items = Vec::with_capacity(len as usize);
            for _ in 0..len {
                items.push(parse_type(reader, ty, resolver)?);
            }
            Ok(items.into())
        },
        // Custom types
        IdlType::Defined(defined) => {
            let type_name = match defined {
                Defined::String(s) => s,
                Defined::Object { name } => name,
            };
            parse_defined_type(reader, type_name, resolver)
        },
    }
}

fn parse_defined_type<R: Read>(
    reader: &mut R,
    type_name: &str,
    resolver: &TypeResolver,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let ty_def = resolver.resolve(type_name)
        .ok_or_else(|| format!("Type {} not found in IDL", type_name))?;

    match &ty_def.ty {
        IdlTypeDefinitionTy::Struct { fields } => {
            let mut map = serde_json::Map::new();
            for field in fields {
                map.insert(
                    field.name.clone(),
                    parse_type(reader, &field.ty, resolver)?
                );
            }
            Ok(map.into())
        }
        IdlTypeDefinitionTy::Enum { variants } => {
            let variant_index = reader.read_u8()?;
            let variant = variants.get(variant_index as usize)
                .ok_or("Invalid variant index")?;
            
            let value = match &variant.fields {
                Some(EnumFields::Tuple(types)) => {
                    let mut vec = Vec::new();
                    for ty in types {
                        vec.push(parse_type(reader, ty, resolver)?);
                    }
                    serde_json::Value::Array(vec)
                }
                Some(EnumFields::Named(fields)) => {
                    let mut map = serde_json::Map::new();
                    for field in fields {
                        map.insert(
                            field.name.clone(),
                            parse_type(reader, &field.ty, resolver)?
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
        IdlTypeDefinitionTy::Alias { value } => {
            parse_type(reader, value, resolver)
        }
    }
}

struct TypeResolver<'a> {
    idl: &'a Idl,
    type_cache: HashMap<String, &'a IdlTypeDefinition>,
}

impl<'a> TypeResolver<'a> {
    fn new(idl: &'a Idl) -> Self {
        let mut type_cache = HashMap::new();
        for ty in &idl.types {
            type_cache.insert(ty.name.clone(), ty);
        }
        Self { idl, type_cache }
    }

    fn resolve(&self, name: &str) -> Option<&IdlTypeDefinition> {
        self.type_cache.get(name).copied()
    }
}