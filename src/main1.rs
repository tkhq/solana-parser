use alloy_primitives::{Address, U256};

/// A very simple “dynamic” decoder for fixed-size parameters.
fn decode_fixed_param(param_type: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    match param_type {
        "address" => {
            // An address is always encoded as 32 bytes with 12 zero-padding bytes
            if data.len() != 32 {
                return Err("Invalid data length for address".into());
            }
            // The last 20 bytes represent the address.
            let addr = Address::from_slice(&data[12..]);
            println!("Decoded address: {:?}", addr);
            Ok(())
        },
        "uint256" => {
            if data.len() != 32 {
                return Err("Invalid data length for uint256".into());
            }
            let value = U256::from_big_endian(data);
            println!("Decoded uint256: {:?}", value);
            Ok(())
        },
        _ => Err(format!("Unsupported parameter type: {}", param_type).into()),
    }
}

/// Decodes the function inputs given the parsed ABI `function` and the full tx data.
fn decode_function_inputs(function: &alloy_json_abi::Function, tx_data: &[u8])
    -> Result<(), Box<dyn std::error::Error>> 
{
    // The first 4 bytes are the function selector.
    let mut offset = 4;
    let input_data = &tx_data[4..];
    
    // Iterate through each input, decoding each fixed-size parameter.
    // (This simple example only works for fixed-size types.)
    for param in function.inputs.clone() {
        // Each parameter is assumed to occupy 32 bytes.
        let data_slice = &input_data[offset - 4..offset - 4 + 32];
        offset += 32;

        // Here, `param.ty` is assumed to be a string such as "address" or "uint256".
        decode_fixed_param(&param.ty, data_slice)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assume you have already parsed your JSON ABI to get a Function.
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
        },
        {
            "type": "function",
            "name": "transferFrom",
            "inputs": [
                { "name": "to", "type": "address" },
                { "name": "amount", "type": "uint256" }
            ],
            "outputs": []
        },
        // pretend this is gone
        // {
        //     "type": "function",
        //     "name": "approve",
        //     "inputs": [
        //         { "name": "to", "type": "address" },
        //         { "name": "amount", "type": "uint256" }
        //     ],
        //     "outputs": []
        // }
    ]
    "#;

    let abi: alloy_json_abi::JsonAbi = serde_json::from_str(abi_json)?;
    let function = abi
        .functions()
        .find(|f| f.selector() == &hex::decode("a9059cbb")?[..])
        .ok_or("Matching function not found")?;

    let tx_data_hex = "a9059cbb0000000000000000000000000123456789abcdef0123456789abcdef012345670000000000000000000000000000000000000000000000000000000000000001";
    let tx_data = hex::decode(tx_data_hex)?;

    decode_function_inputs(&function, &tx_data)?;

    Ok(())
}