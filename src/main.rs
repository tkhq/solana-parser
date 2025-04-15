use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use std::error::Error;
use alloy_sol_types::SolInterface;

// Need to be able to dynamically generate these interfaces
sol! {
    #[derive(Debug, PartialEq)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
    }

    #[derive(Debug, PartialEq)]
    interface IUSDC {
        function approve(address spender, uint256 value) external returns (bool);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // random mainnet ERC20 transfer
    // https://etherscan.io/tx/0x947332ff624b5092fb92e8f02cdbb8a50314e861a4b39c29a286b3b75432165e
    let data = alloy_primitives::hex!("a9059cbb0000000000000000000000008bc47be1e3abbaba182069c89d08a61fa6c2b2920000000000000000000000000000000000000000000000000000000253c51700");

    let expected = IERC20::transferCall {
        to: Address::from(alloy_primitives::hex!("8bc47be1e3abbaba182069c89d08a61fa6c2b292")),
        amount: U256::from(9995360000_u64),
    };

    assert_eq!(data[..4], IERC20::transferCall::SELECTOR);
    let decoded = IERC20::IERC20Calls::abi_decode(&data, true).unwrap();

    print!("DECODED: {:?}\n", decoded);

    assert_eq!(decoded, IERC20::IERC20Calls::transfer(expected));
    assert_eq!(decoded.abi_encode(), data);

    // random mainnet USDC approval
    // https://etherscan.io/tx/0xed07bce2d76cabed2de738b5c184543db1206bba8a97833ab7835bc234e337e7
    let data = alloy_primitives::hex!("095ea7b30000000000000000000000006a000f20005980200259b80c510200304000106800000000000000000000000000000000000000000000000000000000000f4240");

    let expected = IUSDC::approveCall {
        spender: Address::from(alloy_primitives::hex!("6A000F20005980200259B80c5102003040001068")),
        value: U256::from(1000000_u64),
    };

    assert_eq!(data[..4], IUSDC::approveCall::SELECTOR);
    let decoded = IUSDC::IUSDCCalls::abi_decode(&data, true).unwrap();

    print!("DECODED: {:?}\n", decoded);

    assert_eq!(decoded, IUSDC::IUSDCCalls::approve(expected));
    assert_eq!(decoded.abi_encode(), data);

    Ok(())
}
