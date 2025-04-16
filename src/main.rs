use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall, SolInterface};
use std::error::Error;

// Define the interface using the sol! macro
sol! {
    #[derive(Debug, PartialEq)]
    interface USDC {
        function transfer(address to, uint256 value) external returns (bool);
        function approve(address spender, uint256 value) external returns (bool);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Example 1: USDC transfer
    let transfer_data = alloy_primitives::hex!("a9059cbb0000000000000000000000008bc47be1e3abbaba182069c89d08a61fa6c2b2920000000000000000000000000000000000000000000000000000000253c51700");

    let expected_transfer = USDC::transferCall {
        to: Address::from(alloy_primitives::hex!("8bc47be1e3abbaba182069c89d08a61fa6c2b292")),
        value: U256::from(9995360000_u64),
    };

    assert_eq!(transfer_data[..4], USDC::transferCall::SELECTOR);
    let decoded_transfer = USDC::USDCCalls::abi_decode(&transfer_data, true).unwrap();

    println!("DECODED TRANSFER: {:?}", decoded_transfer);

    // Example 2: USDC approval
    let approve_data = alloy_primitives::hex!("095ea7b30000000000000000000000006a000f20005980200259b80c510200304000106800000000000000000000000000000000000000000000000000000000000f4240");

    let expected_approve = USDC::approveCall {
        spender: Address::from(alloy_primitives::hex!("6A000F20005980200259B80c5102003040001068")),
        value: U256::from(1000000_u64),
    };

    assert_eq!(approve_data[..4], USDC::approveCall::SELECTOR);
    let decoded_approve = USDC::USDCCalls::abi_decode(&approve_data, true).unwrap();

    println!("DECODED APPROVE: {:?}", decoded_approve);

    Ok(())
}
