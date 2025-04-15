// Import the required traits and types from Alloy RS.
// (Make sure your Cargo.toml lists the proper Alloy RS dependencies.)
// use alloy::abi::AbiDecode;  // Trait for ABI decoding
use alloy_primitives::{Address, U256}; // Common Ethereum primitives (addresses & uint256)
use alloy_sol_types::{sol_data::*, SolType, SolValue};

// Define a struct representing the ERC‑20 transfer call parameters.
// The derive macro (AbiDecode) instructs Alloy RS to implement the decoder.
// #[derive(Debug, AbiDecode)]
// The #[abi(...)] attribute tells Alloy RS:
//   - that this is a function call (rather than just parameters)
//   - which function signature to expect; here we supply the full signature for clarity.
// #[abi(function, name = "transfer", inputs = "address,uint256")]
// struct Transfer {
//     /// The recipient address.
//     pub to: Address,
//     /// The amount of tokens to transfer.
//     pub amount: U256,
// }

// Represent a Solidity type in rust
type MySolType = FixedArray<Bool, 2>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = [true, false];
    let validate = true;

    // SolTypes expose their Solidity name :)
    assert_eq!(&MySolType::sol_type_name(), "bool[2]");

    // SolTypes are used to transform Rust into ABI blobs, and back.
    let encoded: Vec<u8> = MySolType::abi_encode(&data);
    let decoded: [bool; 2] = MySolType::abi_decode(&encoded)?;
    assert_eq!(data, decoded);

    // This is more easily done with the `SolValue` trait:
    let encoded: Vec<u8> = data.abi_encode();
    let decoded: [bool; 2] = <[bool; 2]>::abi_decode(&encoded)?;
    assert_eq!(data, decoded);
        
    Ok(())
}
