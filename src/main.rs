use alloy_primitives::{Address, U256, B256, hex};
use std::error::Error;
use std::fs;
use alloy_json_abi::JsonAbi;
use serde_json::from_str;

// Home-cooked type definitions for decoded parameter types
#[derive(Debug, Clone)]
pub enum SolType {
    Address(Address),
    Uint(U256),
    Bool(bool),
    Bytes32(B256),
    String(String),
    Bytes(Vec<u8>),
    DynamicArray(usize), // length
    Unsupported(String),  // everything else
}

// Type definition representing an individual parameter, corresponding to ABI types
#[derive(Debug, Clone)]
pub struct DecodedParameter {
    pub name: String, // the variable name
    pub param_type: String, // the Solidity type
    pub value: SolType, // actual value
}

// Type definition representing a decoded transaction
#[derive(Debug, Clone)]
pub struct DecodedTransaction {
    pub function_name: String,
    pub function_signature: String,
    pub parameters: Vec<DecodedParameter>,
}

fn decode_transaction(tx_data: &[u8], abi_json: &str) -> Result<DecodedTransaction, Box<dyn Error>> {
    // Parse the ABI
    let abi: JsonAbi = from_str(abi_json)?;
    
    // Ensure we have enough data for at least a function selector
    if tx_data.len() < 4 {
        return Err("Transaction data too short".into());
    }
    
    // Extract the function selector (first 4 bytes)
    let selector = &tx_data[0..4];
    
    // Find the function in the ABI by its selector
    let function = abi.functions()
        .find(|f| f.selector() == selector)
        .ok_or("Function not found in ABI for this selector")?;
    
    println!("Decoded function: {} ({})", function.name, function.signature());
    
    // Extract the input data (after the already-processed function selector)
    let input_data = &tx_data[4..];
    
    // Decode each parameter based on its type
    let mut offset = 0;
    let mut parameters = Vec::new();
    
    for (i, param) in function.inputs.iter().enumerate() {
        // Fixed types take 32 bytes (including padding)
        // For dynamic types, they would have an offset pointer here
        // More info: https://docs.soliditylang.org/en/latest/abi-spec.html#formal-specification-of-the-encoding
        let is_dynamic = is_dynamic_type(&param.ty);
        
        let decoded_param = if is_dynamic {
            // For dynamic types, this 32-byte slot contains an offset to the actual data
            let dynamic_offset_bytes = &input_data[offset..offset + 32];
            let dynamic_offset = U256::from_be_slice(dynamic_offset_bytes).as_limbs()[0] as usize;
            
            // The actual data starts at this offset from the beginning of the input data
            decode_parameter(i, param, &input_data[dynamic_offset..], true)?
        } else {
            // For fixed types, the data is right here
            let param_data = &input_data[offset..offset + 32];
            decode_parameter(i, param, param_data, false)?
        };
        
        parameters.push(decoded_param);
        offset += 32; // Move to the next parameter slot
    }
    
    Ok(DecodedTransaction {
        function_name: function.name.clone(),
        function_signature: function.signature(),
        parameters,
    })
}

// Helper function to determine if a type is dynamic (like string, bytes, arrays)
// See https://docs.soliditylang.org/en/latest/abi-spec.html#use-of-dynamic-types for more
fn is_dynamic_type(type_str: &str) -> bool {
    type_str == "string" || 
    type_str == "bytes" || 
    type_str.ends_with("[]") || // Dynamic arrays
    type_str.contains("[]") // Multi-dimensional arrays have at least one dynamic dimension
}

// Helper function to decode a single parameter
fn decode_parameter(index: usize, param: &alloy_json_abi::Param, data: &[u8], is_dynamic: bool) -> Result<DecodedParameter, Box<dyn Error>> {
    println!("  Parameter {}: {} ({})", index, param.name, param.ty);
    
    let value = if is_dynamic {
        // For dynamic types, the first 32 bytes contain the length
        if data.len() < 32 {
            return Err("Not enough data for dynamic type length".into());
        }
        
        let length_bytes = &data[0..32];
        let length = U256::from_be_slice(length_bytes).as_limbs()[0] as usize;
        
        match param.ty.as_str() {
            "string" | "bytes" => {
                if data.len() < 32 + length {
                    return Err("Not enough data for dynamic type content".into());
                }
                
                let content = &data[32..32 + length];
                if param.ty == "string" {
                    // Try to decode as UTF-8 string. Otherwise, hex-encode
                    match std::str::from_utf8(content) {
                        Ok(s) => {
                            println!("    Value (string): \"{}\"", s);
                            SolType::String(s.to_string())
                        },
                        Err(_) => {
                            println!("    Value (bytes): 0x{}", hex::encode(content));
                            SolType::Bytes(content.to_vec())
                        },
                    }
                } else {
                    // Just show as hex
                    println!("    Value (bytes): 0x{}", hex::encode(content));
                    SolType::Bytes(content.to_vec())
                }
            },
            _ if param.ty.ends_with("[]") => {
                println!("    Dynamic array with {} elements", length);
                // Decoding array elements would require recursive handling based on the element type
                // This is a simplified example
                SolType::DynamicArray(length)
            },
            _ => {
                println!("    Unsupported dynamic type: {}", param.ty);
                SolType::Unsupported(param.ty.clone())
            },
        }
    } else {
        // Handle fixed types, which are much more straight-forward
        match param.ty.as_str() {
            "address" => {
                // Addresses take up the right-most 20 bytes (first 12 bytes are for padding).
                // This translates to 42 hex characters (including the 0x prefix)
                let address = Address::from_slice(&data[12..32]);
                println!("    Value: {}", address);
                SolType::Address(address)
            },
            "uint256" | "uint" => {
                let value = U256::from_be_slice(data);
                println!("    Value: {}", value);
                SolType::Uint(value)
            },
            t if t.starts_with("uint") => {
                let value = U256::from_be_slice(data);
                println!("    Value: {}", value);
                SolType::Uint(value)
            },
            "bool" => {
                let value = U256::from_be_slice(data);
                let bool_value = value != U256::ZERO;
                println!("    Value: {}", bool_value);
                SolType::Bool(bool_value)
            },
            "bytes32" => {
                let value = B256::from_slice(data);
                println!("    Value: 0x{}", hex::encode(value.as_slice()));
                SolType::Bytes32(value)
            },
            _ => {
                println!("    Unsupported fixed type: {}", param.ty);
                SolType::Unsupported(param.ty.clone())
            },
        }
    };
    
    Ok(DecodedParameter {
        name: param.name.clone(),
        param_type: param.ty.clone(),
        value,
    })
}

// Helper function to pretty-print a decoded transaction
fn print_decoded_transaction(decoded: &DecodedTransaction) {
    println!("Decoded function: {} ({})", decoded.function_name, decoded.function_signature);
    
    for (i, param) in decoded.parameters.iter().enumerate() {
        println!("  Parameter {}: {} ({})", i, param.name, param.param_type);
        
        match &param.value {
            SolType::Address(addr) => println!("    Value: {}", addr),
            SolType::Uint(val) => println!("    Value: {}", val),
            SolType::Bool(val) => println!("    Value: {}", val),
            SolType::Bytes32(val) => println!("    Value: 0x{}", hex::encode(val.as_slice())),
            SolType::String(val) => println!("    Value (string): \"{}\"", val),
            SolType::Bytes(val) => println!("    Value (bytes): 0x{}", hex::encode(val)),
            SolType::DynamicArray(len) => println!("    Dynamic array with {} elements", len),
            SolType::Unsupported(ty) => println!("    Unsupported type: {}", ty),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Pass in a locally stored ABI.
    // Alternatively, you can define your own in-line ABI via:
    // let abi_json = r#"<your JSON here>"#;
    let abi_json = fs::read_to_string("src/usdc_abi.json")?;
    
    println!("Decoding transfer:");
    let transfer_data = hex!("a9059cbb0000000000000000000000008bc47be1e3abbaba182069c89d08a61fa6c2b2920000000000000000000000000000000000000000000000000000000253c51700");
    let decoded = decode_transaction(&transfer_data, &abi_json)?;
    print_decoded_transaction(&decoded);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to load the ABI for tests
    fn load_abi() -> String {
        fs::read_to_string("src/usdc_abi.json").expect("Failed to read ABI file")
    }
    
    #[test]
    fn test_decode_transfer() -> Result<(), Box<dyn Error>> {
        let abi_json = load_abi();
        
        // `transfer`` transaction:
        // https://etherscan.io/tx/0x7b46ff4bf830a03fdbe90e1eee8da5400af4900e43c4b7fee1f883bb1b89eaf8
        let transfer_data = hex!("0xa9059cbb00000000000000000000000041fc802e01bcf85d91e5708b42d41c2eaf01f37500000000000000000000000000000000000000000000000000000000036400cc");
        
        let decoded = decode_transaction(&transfer_data, &abi_json)?;
        
        // Check function
        assert_eq!(decoded.function_name, "transfer");
        assert_eq!(decoded.function_signature, "transfer(address,uint256)");

        // Check parameters
        assert_eq!(decoded.parameters.len(), 2);
        assert_eq!(decoded.parameters[0].name, "to");
        assert_eq!(decoded.parameters[0].param_type, "address");
        if let SolType::Address(addr) = &decoded.parameters[0].value {
            assert_eq!(addr.to_string(), "0x41Fc802E01Bcf85D91E5708B42d41C2EAf01f375");
        }
        assert_eq!(decoded.parameters[1].name, "value");
        assert_eq!(decoded.parameters[1].param_type, "uint256");
        match decoded.parameters[1].value {
            SolType::Uint(amount) => assert_eq!(amount.to_string(), "56885452"),
            _ => panic!("unexpected value")
        }
        
        Ok(())
    }
    
    #[test]
    fn test_decode_approve() -> Result<(), Box<dyn Error>> {
        let abi_json = load_abi();
        
        // `approve` transaction:
        // https://etherscan.io/tx/0x7c01ea5ec0862474f15f1fcf1ed4b0fa2f2c6fb4160be4723c8791e1caef2186
        let approve_data = hex!("0x095ea7b30000000000000000000000001715a3e4a142d8b698131108995174f37aeba10d0000000000000000000000000000000000000000000000000000000006b5b54f");
        
        let decoded = decode_transaction(&approve_data, &abi_json)?;
        
        // Check function
        assert_eq!(decoded.function_name, "approve");
        assert_eq!(decoded.function_signature, "approve(address,uint256)");

        // Check parameters
        assert_eq!(decoded.parameters.len(), 2);
        assert_eq!(decoded.parameters[0].name, "spender");
        assert_eq!(decoded.parameters[0].param_type, "address");
        if let SolType::Address(addr) = &decoded.parameters[0].value {
            assert_eq!(addr.to_string(), "0x1715a3E4A142d8b698131108995174F37aEBA10D");
        }
        assert_eq!(decoded.parameters[1].name, "value");
        assert_eq!(decoded.parameters[1].param_type, "uint256");
        match decoded.parameters[1].value {
            SolType::Uint(amount) => assert_eq!(amount.to_string(), "112571727"),
            _ => panic!("unexpected value")
        }
        
        Ok(())
    }
    
    #[test]
    fn test_decode_transfer_from() -> Result<(), Box<dyn Error>> {
        let abi_json = load_abi();
        
        // `transferFrom` transaction:
        // https://etherscan.io/tx/0x6634538955136ae956fcb635dda5bb66af479be8cff19a12cd0128e010d4e3a3
        let transfer_from_data = hex!("0x23b872dd0000000000000000000000004e95ac442c32fe2b4fb0b6e6944489fb6aef0c5b00000000000000000000000063c79fccd0a21e4a4d87056a0efe3b85d8c373d40000000000000000000000000000000000000000000000000000000015de6254");
        
        let decoded = decode_transaction(&transfer_from_data, &abi_json)?;
        
        // Check function
        assert_eq!(decoded.function_name, "transferFrom");
        assert_eq!(decoded.function_signature, "transferFrom(address,address,uint256)");

        // Check parameters
        assert_eq!(decoded.parameters.len(), 3);
        assert_eq!(decoded.parameters[0].name, "from");
        assert_eq!(decoded.parameters[0].param_type, "address");
        if let SolType::Address(addr) = &decoded.parameters[0].value {
            assert_eq!(addr.to_string(), "0x4e95ac442C32FE2B4fb0b6E6944489fB6AEF0C5B");
        }
        assert_eq!(decoded.parameters[1].name, "to");
        assert_eq!(decoded.parameters[1].param_type, "address");
        if let SolType::Address(addr) = &decoded.parameters[1].value {
            assert_eq!(addr.to_string(), "0x63c79FcCd0a21e4a4D87056A0Efe3B85d8c373d4");
        }
        assert_eq!(decoded.parameters[2].name, "value");
        assert_eq!(decoded.parameters[2].param_type, "uint256");
        match decoded.parameters[2].value {
            SolType::Uint(amount) => assert_eq!(amount.to_string(), "366895700"),
            _ => panic!("unexpected value")
        }
        
        Ok(())
    }
}
