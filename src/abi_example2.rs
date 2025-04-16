use alloy_primitives::{Address, U256, Bytes};
use alloy_sol_types::{SolType, SolValue};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the ABI file
    let abi_json = fs::read_to_string("src/abi.json")?;
    
    // Parse the ABI into a JsonAbi object
    let abi: alloy_json_abi::JsonAbi = serde_json::from_str(&abi_json)?;
    
    // Example transaction data for a transfer call
    let transfer_data = alloy_primitives::hex!("a9059cbb0000000000000000000000008bc47be1e3abbaba182069c89d08a61fa6c2b2920000000000000000000000000000000000000000000000000000000253c51700");
    
    // Extract the function selector (first 4 bytes)
    let selector = &transfer_data[0..4];
    
    // Find the function in the ABI by its selector
    let function = abi.functions()
        .find(|f| f.selector() == selector)
        .expect("Function not found for selector");
    
    println!("Found function by selector: {}", function.signature());
    
    // Extract the input data (skip the 4-byte selector)
    let input_data = &transfer_data[4..];
    
    // Manually decode the parameters based on the function's input types
    // For transfer(address to, uint256 value), we need to decode an address and a uint256
    
    // The first parameter is an address (20 bytes, but padded to 32 bytes in the ABI)
    let to_bytes = &input_data[0..32];
    // Address is right-aligned in the 32-byte word, so we skip the first 12 bytes
    let to = Address::from_slice(&to_bytes[12..32]);
    
    // The second parameter is a uint256 (32 bytes)
    let value_bytes = &input_data[32..64];
    let value = U256::from_be_slice(value_bytes);
    
    println!("Decoded parameters using JsonAbi:");
    println!("  to: {}", to);
    println!("  value: {}", value);
    
    // Example transaction data for an approve call
    let approve_data = alloy_primitives::hex!("095ea7b30000000000000000000000006a000f20005980200259b80c510200304000106800000000000000000000000000000000000000000000000000000000000f4240");
    
    // Extract the function selector
    let selector = &approve_data[0..4];
    
    // Find the function in the ABI
    let function = abi.functions()
        .find(|f| f.selector() == selector)
        .expect("Function not found for selector");
    
    println!("\nFound function by selector: {}", function.signature());
    
    // Extract the input data
    let input_data = &approve_data[4..];
    
    // Decode the parameters
    // For approve(address spender, uint256 value), we need to decode an address and a uint256
    
    // The first parameter is an address
    let spender_bytes = &input_data[0..32];
    let spender = Address::from_slice(&spender_bytes[12..32]);
    
    // The second parameter is a uint256
    let value_bytes = &input_data[32..64];
    let value = U256::from_be_slice(value_bytes);
    
    println!("Decoded parameters using JsonAbi:");
    println!("  spender: {}", spender);
    println!("  value: {}", value);
    
    // A more generic approach for decoding parameters
    println!("\nGeneric decoding approach:");
    decode_function_inputs(function, &approve_data)?;
    
    Ok(())
}

// A more generic function to decode function inputs
fn decode_function_inputs(function: &alloy_json_abi::Function, tx_data: &[u8]) -> Result<(), Box<dyn Error>> {
    println!("Decoding function: {}", function.signature());
    
    // The first 4 bytes are the function selector
    let input_data = &tx_data[4..];
    let mut offset = 0;
    
    // Iterate through each input parameter
    for (i, param) in function.inputs.iter().enumerate() {
        let param_data = &input_data[offset..offset + 32];
        offset += 32; // Each parameter takes 32 bytes in the ABI encoding
        
        println!("  Parameter {}: {} ({})", i, param.name, param.ty);
        
        // Decode based on the parameter type
        match param.ty.as_str() {
            "address" => {
                // Address is right-aligned in the 32-byte word
                let address = Address::from_slice(&param_data[12..32]);
                println!("    Value: {}", address);
            },
            "uint256" => {
                let value = U256::from_be_slice(param_data);
                println!("    Value: {}", value);
            },
            // Add more types as needed
            _ => println!("    Unsupported type: {}", param.ty),
        }
    }
    
    Ok(())
}
