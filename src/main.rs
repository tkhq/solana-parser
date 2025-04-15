// use alloy_abi::{Abi, Function};
use alloy_json_abi::{Function, JsonAbi};
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

    print!("ABI: {:?}\n", abi);

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

    print!("ABI FUNCTIONS: {:?}\n", abi.functions);

    // Find the matching function in the ABI using the selector.
    let found = abi.functions().find(|f| {
        print!("FOUND MATCHING SELECTOR: {:?}\n", f.selector());
        f.selector() == selector
    });

    let function = found.unwrap();

    print!("MATCHING FUNCTION: {:?}\n", function);

    // Now, we have the matching function. We just need to decode the parameters encoded within the transaction data, given the function signature / interface

    // Decode the input parameters (skipping the first 4 bytes for the selector).
    // let tokens = function.&tx_data[4..])?;

    // // Print the function name and its parameters.
    // println!("Function: {}", function.name);
    // for (input, token) in function.inputs.iter().zip(tokens) {
    //     println!("Parameter {}: {:?}", input.name, token);
    // }

    Ok(())
}
