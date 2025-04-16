use alloy_primitives::{Address, U256, B256, Bytes};
use std::error::Error;
use std::fs;

// A generic function to decode any transaction using only the ABI
fn decode_transaction(tx_data: &[u8], abi_json: &str) -> Result<(), Box<dyn Error>> {
    // Parse the ABI
    let abi: alloy_json_abi::JsonAbi = serde_json::from_str(abi_json)?;
    
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
    
    // Extract the input data (skip the 4-byte selector)
    let input_data = &tx_data[4..];
    
    // Decode each parameter based on its type
    let mut offset = 0;
    
    for (i, param) in function.inputs.iter().enumerate() {
        // For fixed-size types, they take up 32 bytes each
        // For dynamic types, they would have an offset pointer here
        let is_dynamic = is_dynamic_type(&param.ty);
        
        if is_dynamic {
            // For dynamic types, this 32-byte slot contains an offset to the actual data
            let dynamic_offset_bytes = &input_data[offset..offset + 32];
            let dynamic_offset = U256::from_be_slice(dynamic_offset_bytes).as_limbs()[0] as usize;
            
            // The actual data starts at this offset from the beginning of the input data
            decode_parameter(i, param, &input_data[dynamic_offset..], true)?;
        } else {
            // For fixed types, the data is right here
            let param_data = &input_data[offset..offset + 32];
            decode_parameter(i, param, param_data, false)?;
        }
        
        offset += 32; // Move to the next parameter slot
    }
    
    Ok(())
}

// Helper function to determine if a type is dynamic (like string, bytes, arrays)
fn is_dynamic_type(type_str: &str) -> bool {
    type_str == "string" || 
    type_str == "bytes" || 
    type_str.ends_with("[]") || // Dynamic arrays
    type_str.contains("[]") // Multi-dimensional arrays have at least one dynamic dimension
}

// Helper function to decode a single parameter
fn decode_parameter(index: usize, param: &alloy_json_abi::Param, data: &[u8], is_dynamic: bool) -> Result<(), Box<dyn Error>> {
    println!("  Parameter {}: {} ({})", index, param.name, param.ty);
    
    if is_dynamic {
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
                    // Try to decode as UTF-8 string
                    match std::str::from_utf8(content) {
                        Ok(s) => println!("    Value (string): \"{}\"", s),
                        Err(_) => println!("    Value (bytes): 0x{}", hex::encode(content)),
                    }
                } else {
                    // Just show as hex
                    println!("    Value (bytes): 0x{}", hex::encode(content));
                }
            },
            _ if param.ty.ends_with("[]") => {
                println!("    Dynamic array with {} elements", length);
                // Decoding array elements would require recursive handling based on the element type
                // This is a simplified example
            },
            _ => println!("    Unsupported dynamic type: {}", param.ty),
        }
    } else {
        // Fixed types are easier to decode
        match param.ty.as_str() {
            "address" => {
                // Address is right-aligned in the 32-byte word
                let address = Address::from_slice(&data[12..32]);
                println!("    Value: {}", address);
            },
            "uint256" | "uint" => {
                let value = U256::from_be_slice(data);
                println!("    Value: {}", value);
            },
            t if t.starts_with("uint") => {
                // Handle other uint types (uint8, uint16, etc.)
                let value = U256::from_be_slice(data);
                println!("    Value: {}", value);
            },
            "bool" => {
                let value = U256::from_be_slice(data);
                println!("    Value: {}", value != U256::ZERO);
            },
            "bytes32" => {
                let value = B256::from_slice(data);
                println!("    Value: 0x{}", hex::encode(value.as_slice()));
            },
            _ => println!("    Unsupported fixed type: {}", param.ty),
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read the ABI file
    let abi_json = fs::read_to_string("src/abi.json")?;
    
    println!("=== Example 1: Decoding a transfer transaction ===");
    // Example transaction data for a transfer call
    let transfer_data = alloy_primitives::hex!("a9059cbb0000000000000000000000008bc47be1e3abbaba182069c89d08a61fa6c2b2920000000000000000000000000000000000000000000000000000000253c51700");
    decode_transaction(&transfer_data, &abi_json)?;
    
    println!("\n=== Example 2: Decoding an approve transaction ===");
    // Example transaction data for an approve call
    let approve_data = alloy_primitives::hex!("095ea7b30000000000000000000000006a000f20005980200259b80c510200304000106800000000000000000000000000000000000000000000000000000000000f4240");
    decode_transaction(&approve_data, &abi_json)?;
    
    println!("\n=== Example 3: Decoding a transferFrom transaction ===");
    // Example transaction data for an approve call
    let approve_data = alloy_primitives::hex!("0x23b872dd000000000000000000000000b9991669f54a19d822c614769f6a863f807971cd000000000000000000000000ae2d4617c862309a3d75a0ffb358c7a5009c673f0000000000000000000000000000000000000000000000000000000005651e68");
    decode_transaction(&approve_data, &abi_json)?;
    
    // You could also take transaction data from user input, a file, or an API
    // This makes the decoder completely generic - it can handle any transaction
    // as long as the function is defined in the ABI
    
    Ok(())
}
