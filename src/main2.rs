use alloy_json_abi::{Function, JsonAbi};
// Import SolValue from alloy_sol_types
use alloy_sol_types::SolValue;
use hex::FromHex;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Replace this with your actual contract ABI in JSON format.
    let abi_json = r#"
    [
        {
            "type": "function",
            "name": "transfer",
            "inputs": [
                { "name": "to", "type": "address" },
                { "name": "amount", "type": "uint256" }
            ],
            "outputs": []
        }
    ]
    "#;

    // Parse the ABI JSON into an Abi object.
    let abi: JsonAbi = serde_json::from_str(abi_json)?;

    // Example unsigned transaction input data (hex string).
    // For instance, "a9059cbb" is the selector for transfer(address,uint256).
    let tx_data_hex = "a9059cbb0000000000000000000000000123456789abcdef0123456789abcdef012345670000000000000000000000000000000000000000000000000000000000000001";

    // Convert the hex string into bytes.
    let tx_data = Vec::from_hex(tx_data_hex)?;
    if tx_data.len() < 4 {
        return Err("Transaction data too short".into());
    }

    // Extract the first 4 bytes as the function selector.
    let selector = &tx_data[..4];

    // Find the matching function in the ABI using the selector.
    let function: &Function = abi
        .functions()
        .find(|f| f.selector() == selector)
        .ok_or("Function not found for the given selector")?; // Use ok_or for better error handling

    println!("MATCHING FUNCTION: {:?}", function.name); // Print just the name for clarity

    // --- DECODING STEP ---
    // Get the encoded parameters data (skip the 4-byte selector)
    let encoded_params = &tx_data[4..];

    // Use the function's decode_input method
    let decoded_tokens: Vec<dyn SolValue> = function.decode_input(encoded_params)?;

    println!("\nDecoded Parameters:");

    // Iterate over the inputs defined in the ABI and the decoded tokens
    for (input, token) in function.inputs.iter().zip(decoded_tokens.iter()) {
        // The `token` is a SolValue enum. You can match on it or use its methods
        // to get the specific Rust type if needed.
        // For simple printing, the default Debug impl is often helpful.
        println!("  {}: {} = {:?}", input.name, input.ty, token);

        // Example of extracting specific types (optional):
        match token {
            dyn SolValue::Address(addr) => {
                println!("    (Decoded as Address: {})", addr);
            }
            dyn SolValue::Uint(uint, size) => {
                // `uint` is ruint::Uint<BITS, LIMBS>
                println!("    (Decoded as Uint<{}, {}>: {})", size, uint.limb_count(), uint);
            }
            _ => {
                // Handle other types as needed
            }
        }
    }

    Ok(())
}