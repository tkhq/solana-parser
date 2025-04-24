use serde_json::{from_str, from_value, Value, Map};
use std::error::Error;
use crate::solana::structs::{Idl, IdlAccount, IdlTypeDefinition, IdlInstruction, IdlType, IdlField, IdlTypeDefinitionTy};
use sha2::{Sha256, Digest};

// Top Level Required Fields 
const IDL_INSTRUCTIONS_KEY: &str = "instructions";
const IDL_TYPES_KEY: &str = "types";

// // Instructions Required Fields
// const IDL_INST_NAME_KEY: &str = "name";
// const IDL_INST_ACCTS_KEY: &str = "accounts";
// const IDL_INST_ARGS_KEY: &str = "args";

// // Instructions NON-Required Fields
// const IDL_INST_DISC_KEY: &str = "discriminator";
const IDL_INST_DEFAULT_DISC_LEN: usize = 8;

// // Accounts Fields
// const IDL_ACCT_NAME_KEY: &str = "name";
// static IDL_ACCOUNT_MUTABLE_KEYS: &[&str] = &["isMut", "writable"];
// static IDL_ACCOUNT_SIGNER_KEYS: &[&str] = &["isSigner", "signer"];
// static IDL_ACCOUNT_OPTIONAL_KEYS: &[&str] = &["isOptional", "optional"];

// // Args Fields 
// const IDL_ARG_NAME_KEY: &str = "name";
// const IDL_ARG_TYPE_KEY: &str = "type";

// // Types fields 
// const IDL_TYPES_NAME_KEY: &str = "name";
// const IDL_TYPES_TYPE_KEY: &str = "type";

// // Defined TypeFields
// const IDL_DEF_TYPES_KIND_KEY: &str = "kind";
// const IDL_DEF_TYPES_ENUM_KIND: &str = "enum";
// const IDL_DEF_TYPES_ALIAS_KIND: &str = "alias";
// const IDL_DEF_TYPES_STRUCT_KIND: &str = "struct";

// const IDL_INNER_TYPES_ENUM_VAR: &str = "variants";


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





// fn validate_idl_string(idl_map: &Map<String, Value>, key: &str) -> Result<String, Box<dyn Error>> {
//     let value = idl_map.get(key).ok_or_else(|| format!("Key '{}' not found in expected place IDL", key))?;
//     let checked_value = value.as_str()
//         .ok_or_else(|| format!("Value for Key '{}' must be a JSON string", key))?;
//     Ok(checked_value.to_string())
// }

// fn validate_idl_obj(idl_map: &Map<String, Value>, key: &str) -> Result<Map<String, Value>, Box<dyn Error>> {
//     let value = idl_map.get(key).ok_or_else(|| format!("Key '{}' not found in expected place IDL", key))?;
//     let checked_value = value.as_object()
//         .ok_or_else(|| format!("Value for Key '{}' must be a JSON object", key))?;
//     Ok(checked_value.to_owned())
// }

// fn check_for_idl_bools_or_false(idl_map: &Map<String, Value>, keys: &[&str]) -> bool {
//     keys.iter()
//     .filter_map(|&key| idl_map.get(key))
//     .find_map(|val| match val {
//         Value::Bool(b) => Some(*b),
//         _ => None,
//     })
//     .unwrap_or(false)
// }

// fn validate_or_calculate_discriminator_bytes(idl_map: &Map<String, Value>, name: String) -> Result<Vec<u8>, Box<dyn Error>> {
//     let explicit_discriminator = idl_map.get(IDL_INST_DISC_KEY);
//     match explicit_discriminator {
//         Some(value) => {
//             // Validate array format
//             let arr = value.as_array().ok_or("Discriminator must be an array of bytes")?;
            
//             // Convert each value to u8 with range checking
//             let mut bytes = Vec::with_capacity(arr.len());
//             for (idx, v) in arr.iter().enumerate() {
//                 let n = v.as_u64()
//                     .ok_or_else(|| format!("Invalid byte at position {}: not a number", idx))?;
        
//                 if n > 255 {
//                     return Err(format!("Value {} at position {} exceeds byte range", n, idx).into());
//                 }

//                 bytes.push(n as u8);
//             }
            
//             Ok(bytes)
//         }
//         None => compute_discriminator(&name)
//     }
// }

// fn parse_single_defined_type(idl_type: Value) -> Result<IdlTypeDefinition, Box<dyn Error>> {
//     let type_map = idl_type.as_object().ok_or_else(|| "Each Type within the IDL must be a JSON object")?;

//     // Parse type name
//     let name = validate_idl_string(type_map, IDL_TYPES_NAME_KEY)?;

//     // Parse type kind
//     let kind = validate_idl_string(&type_map, IDL_TYPES_NAME_KEY)?;

//     // // Parse defined type inner object
//     // let parse_single_defined_type(type_map, kind)?;
    




//     return Err("Parsing all types succeeded (so farrrr)".into())
// }

// // fn parse_single_defined_type(type_map: &Map<String, Value>, kind: String) -> Result<IdlTypeDefinitionTy, Box<dyn Error>> {
// //     // validate the inner type key
// //     let inner_type_obj = validate_idl_obj(type_map, IDL_TYPES_TYPE_KEY)?;
// //     let name = validate_idl_string(type_map, IDL_TYPES_NAME_KEY)?;
// //     match kind.as_str() {
// //         IDL_DEF_TYPES_ENUM_KIND => {
// //             let variants = validate_idl_array(&inner_type_obj, )
// //         }
// //         IDL_DEF_TYPES_STRUCT_KIND => Err("fake".into()),
// //         IDL_DEF_TYPES_ALIAS_KIND => Err("fake".into()),
// //         _ => Err("fake".into()),
// //     }
// // }


// fn parse_single_instruction(instruction: Value) -> Result<IdlInstruction, Box<dyn Error>> {
//     let inst_map = instruction.as_object().ok_or_else(|| "Each Instruction within the IDL must be a JSON object")?;

//     // Parse instruction name
//     let name = validate_idl_string(inst_map, IDL_INST_NAME_KEY)?;

//     // Parse instruction discriminator
//     let discriminator = validate_or_calculate_discriminator_bytes(&inst_map, name.clone())?;

//     // Parse all instruction accounts
//     let accounts = validate_idl_array(inst_map, IDL_INST_ACCTS_KEY)?;
//     let parsed_accounts: Vec<IdlAccount> = accounts
//     .iter()
//     .map(|account_value| parse_single_account(account_value.clone()))
//     .collect::<Result<Vec<_>, _>>()?;

//     // Parse all instruction args
//     let args = validate_idl_array(inst_map, IDL_INST_ARGS_KEY)?;
//     let parsed_args: Vec<IdlField> = args
//     .iter()
//     .map(|a| parse_single_arg(a.clone()))
//     .collect::<Result<Vec<_>, _>>()?;

//     Ok(IdlInstruction {
//         name: name.clone(),
//         discriminator,
//         accounts: parsed_accounts,
//         args: parsed_args,
//     })
// }

// fn parse_single_account(account: Value) -> Result<IdlAccount, Box<dyn Error>> {
//     let acct_map = account.as_object().ok_or_else(|| "Each Instruction within the IDL must be a JSON object")?;
//     let name = validate_idl_string(acct_map, IDL_ACCT_NAME_KEY)?;
//     let is_mut = check_for_idl_bools_or_false(acct_map, IDL_ACCOUNT_MUTABLE_KEYS);
//     let is_signer = check_for_idl_bools_or_false(acct_map, IDL_ACCOUNT_SIGNER_KEYS);
//     let is_optional = check_for_idl_bools_or_false(acct_map, IDL_ACCOUNT_OPTIONAL_KEYS);
//     Ok(IdlAccount{
//         name,
//         is_mut,
//         is_signer,
//         is_optional
//     })
// }

// fn parse_single_arg(arg: Value) -> Result<IdlField, Box<dyn Error>> {
//     let acct_map = arg.as_object().ok_or_else(|| "Each Arg within the IDL must be a JSON object")?;
//     let name = validate_idl_string(acct_map, IDL_ARG_NAME_KEY)?;
//     let acct_type = validate_and_parse_idl_type(acct_map)?;
//     Ok(IdlField {
//         name,
//         ty: acct_type
//     })
// }

// fn validate_and_parse_idl_type(idl_map: &Map<String, Value>) -> Result<IdlType, Box<dyn std::error::Error>> {
//     let value = idl_map.get(IDL_ARG_TYPE_KEY).ok_or_else(|| format!("Key '{}' not found in IDL ", IDL_ARG_TYPE_KEY))?;
//     parse_idl_type(value)
// }

// // fn parse_idl_type(value: &Value) -> Result<IdlType, Box<dyn Error>> {
// //     match value {
// //         Value::String(s) => match s.as_str() {
// //             "bool" => Ok(IdlType::Bool),
// //             "bytes" => Ok(IdlType::Bytes),
// //             "f32" => Ok(IdlType::F32),
// //             "f64" => Ok(IdlType::F64),
// //             "i128" => Ok(IdlType::I128),
// //             "i16" => Ok(IdlType::I16),
// //             "i32" => Ok(IdlType::I32),
// //             "i64" => Ok(IdlType::I64),
// //             "i8" => Ok(IdlType::I8),
// //             "publicKey" | "pubkey" => Ok(IdlType::PublicKey),
// //             "string" => Ok(IdlType::String),
// //             "u128" => Ok(IdlType::U128),
// //             "u16" => Ok(IdlType::U16),
// //             "u32" => Ok(IdlType::U32),
// //             "u64" => Ok(IdlType::U64),
// //             "u8" => Ok(IdlType::U8),
// //             _ => Err(format!("Invalid IDL argument type: {}", s).into()),
// //         },
// //         Value::Object(obj) => {
// //             if let Some(defined) = obj.get("defined") {
// //                 match defined {
// //                     Value::String(s) => {
// //                         return Ok(IdlType::Defined(s.to_string()))
// //                     }
// //                     Value::Object(o) => {
// //                         if let Some(s) = o.get("name") {
// //                             return Ok(IdlType::Defined(s.to_string()))
// //                         } else {
// //                             return Err("Invalid Defined type in IDL".into())
// //                         }
// //                     }
// //                     _ => return Err("Invalid Defined type in IDL".into())
// //                 }
// //             }
// //             if let Some(option) = obj.get("option") {
// //                 return Ok(IdlType::Option(Box::new(parse_idl_type(option)?)));
// //             }
// //             if let Some(coption) = obj.get("coption") {
// //                 return Ok(IdlType::COption(Box::new(parse_idl_type(coption)?)));
// //             }
// //             if let Some(vec) = obj.get("vec") {
// //                 return Ok(IdlType::Vec(Box::new(parse_idl_type(vec)?)));
// //             }
// //             if let Some(array) = obj.get("array") {
// //                 let arr = array.as_array()
// //                     .ok_or_else(|| "Invalid array format")?;
                
// //                 let inner = parse_idl_type(&arr[0])?;
// //                 let len = arr[1].as_u64()
// //                     .ok_or_else(|| "Invalid array length")?
// //                     as usize;
                
// //                 return Ok(IdlType::Array(Box::new(inner), len));
// //             }
// //             if let Some(tuple) = obj.get("tuple") {
// //                 let types = tuple.as_array()
// //                     .ok_or_else(|| "Tuple must be an array")?
// //                     .iter()
// //                     .map(parse_idl_type)
// //                     .collect::<Result<Vec<_>, _>>()?;
                
// //                 return Ok(IdlType::Tuple(types));
// //             }
// //             if let Some(map) = obj.get("hashMap") {
// //                 let map_arr = map.as_array()
// //                     .ok_or_else(|| "HashMap requires [key, value]")?;
                
// //                 let key = parse_idl_type(&map_arr[0])?;
// //                 let value = parse_idl_type(&map_arr[1])?;
// //                 return Ok(IdlType::HashMap(Box::new(key), Box::new(value)));
// //             }
// //             Err(format!("Unsupported type object: {:?}", obj).into())
// //         }
// //         _ => Err("Invalid IDL type format".into()),
// //     }
// // }
