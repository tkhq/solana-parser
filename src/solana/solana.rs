use std::error::Error;
use solana_sdk::{hash::Hash, instruction::CompiledInstruction, message, pubkey::Pubkey};
// use solana_events_parser::*;

const LEN_SOL_STRING: usize = 32;
const BYTE: usize = 1;
const LEN_MESSAGE_HEADER_BYTES: usize = 3;

// Entrypoint to parsing
pub fn parse_transaction() -> Result<(), Box<dyn std::error::Error>> {
    //let test_transaction = "010d31220a2a006ae386c11710f471e5b626fd356d86aad3cc3298482d7426f8ab45afdb4c95e18df8e3e8fb37d41871dbbc3bb2bd65692ece097c73d8e4d5b60f010001033576ba544d2d11541cfbce704ed3f5849f12792e1b8f4908794d2fd18742e2bfdecd7749fc61324afc2807e3d6e74461f3eea4c6d176deb4da60ae4ade29f9c60000000000000000000000000000000000000000000000000000000000000000f48a6f99c5aee4ca7e5aa1291270f79e13e28e7661d87033638f2ade245c6e0f01020200010c0200000000ca9a3b00000000";
    let test_msg = "010001033576ba544d2d11541cfbce704ed3f5849f12792e1b8f4908794d2fd18742e2bfdecd7749fc61324afc2807e3d6e74461f3eea4c6d176deb4da60ae4ade29f9c60000000000000000000000000000000000000000000000000000000000000000f48a6f99c5aee4ca7e5aa1291270f79e13e28e7661d87033638f2ade245c6e0f01020200010c0200000000ca9a3b00000000";

    let test_msg_bytes_res: Result<Vec<u8>, _> = (0..test_msg.len()).step_by(2).map(|i| u8::from_str_radix(&test_msg[i..i+2], 16)).collect();
    let test_msg_bytes: &[u8] = &test_msg_bytes_res?; 

    let (header, remainder) = parse_header(test_msg_bytes)?;
    let (account_keys, remainder) = parse_accounts(remainder)?;
    let (recent_blockhash, remainder) = parse_block_hash(remainder)?;
    let instructions = parse_instructions(remainder)?;
    
    let message = message::Message{
        account_keys,
        header,
        instructions,
        recent_blockhash,
    };

    print!("{:#?}", message);
    return Ok(())
}

// fn parse_solana_transaction(unsigned )

fn parse_header(unsigned_tx_bytes: &[u8]) -> Result<(message::MessageHeader, &[u8]), Box<dyn Error>> {
    let message_header = message::MessageHeader{
        num_required_signatures: unsigned_tx_bytes[0],  
        num_readonly_signed_accounts: unsigned_tx_bytes[1],
        num_readonly_unsigned_accounts: unsigned_tx_bytes[2], 
    };
    return Ok((message_header, &unsigned_tx_bytes[LEN_MESSAGE_HEADER_BYTES..unsigned_tx_bytes.len()]))
}

fn parse_accounts(remainder: &[u8]) -> Result<(Vec<Pubkey>, &[u8]), Box<dyn Error>> {
    let accounts_num = remainder[0] as usize;
    let len_accounts_array = (LEN_SOL_STRING * accounts_num) + BYTE;
    let mut pubkeys: Vec<Pubkey> = Vec::with_capacity(accounts_num);
    for i in 0..accounts_num {
        let mut pubkey_sized_bytes = [0u8; 32];
        pubkey_sized_bytes.copy_from_slice(&remainder[((i * LEN_SOL_STRING) + BYTE)..(((i+1) * LEN_SOL_STRING) + BYTE)]);
        pubkeys.push(Pubkey::new_from_array(pubkey_sized_bytes));
    }
    return Ok((pubkeys, &remainder[len_accounts_array..remainder.len()]))
}

fn parse_block_hash(remainder: &[u8]) -> Result<(Hash, &[u8]), Box<dyn Error>> {
    let hash_bytes: &[u8] = &remainder[0..LEN_SOL_STRING];
    let block_hash = Hash::new(hash_bytes);
    return Ok((block_hash, &remainder[LEN_SOL_STRING..remainder.len()]))
}

fn parse_instructions(insts_str: &[u8]) -> Result<Vec<CompiledInstruction>, Box<dyn Error>> {
    let insts_num = insts_str[0] as usize;
    let mut compiled_insts: Vec<CompiledInstruction> = Vec::with_capacity(insts_num);
    let mut remainder = &insts_str[BYTE..insts_str.len()];
    for _ in 0..insts_num {
        let (new_inst, remainder_bytes) = parse_single_instruction(remainder)?;
        remainder = remainder_bytes;
        compiled_insts.push(new_inst);
    }
    Ok(compiled_insts)
}

fn parse_single_instruction(remainder: &[u8]) -> Result<(CompiledInstruction, &[u8]), Box<dyn Error>> {
    let program_id_index = remainder[0];
    let (accounts, remainder) = parse_compact_array_of_bytes(&remainder[BYTE..remainder.len()])?;
    let (data, remainder) = parse_compact_array_of_bytes(remainder)?;
    let instruction = CompiledInstruction{
        program_id_index, 
        accounts,
        data,
    };
    Ok((instruction, remainder))
}

fn parse_compact_array_of_bytes(remainder: &[u8]) -> Result<(Vec<u8>, &[u8]), Box<dyn Error>> {
    let bytes_num = remainder[0] as usize;
    let bytes: Vec<u8> = remainder[BYTE..(bytes_num+1)*BYTE].to_vec();
    Ok((bytes, &remainder[BYTE*(bytes_num+1)..remainder.len()]))
}
