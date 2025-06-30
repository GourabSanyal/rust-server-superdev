use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
    system_program,
};
use super::response_types::{SuccessResponse, ErrorResponse};
use bs58;
use base64;

#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Serialize)]
pub struct AccountInfo {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
pub struct SendSolResponse {
    pub program_id: String,
    pub accounts: Vec<AccountInfo>,
    pub instruction_data: String,
}

fn validate_pubkey(key: &str, field_name: &str) -> Result<Pubkey, ErrorResponse> {
    bs58::decode(key)
        .into_vec()
        .map_err(|_| ErrorResponse::new(format!("Invalid base58 encoding for {}", field_name)))
        .and_then(|bytes| 
            Pubkey::try_from(bytes.as_slice())
                .map_err(|_| ErrorResponse::new(format!("Invalid public key format for {}", field_name)))
        )
}

fn validate_lamports(lamports: u64) -> Result<(), ErrorResponse> {
    if lamports == 0 {
        return Err(ErrorResponse::new("Amount must be greater than 0 lamports"));
    }
    
    // Check if amount is reasonable (less than total supply)
    // Solana total supply is ~500M SOL = 500M * 10^9 lamports
    const MAX_REASONABLE_LAMPORTS: u64 = 500_000_000 * 1_000_000_000;
    if lamports > MAX_REASONABLE_LAMPORTS {
        return Err(ErrorResponse::new("Amount exceeds maximum reasonable transfer"));
    }
    
    Ok(())
}

pub fn send_sol(request: SendSolRequest) -> Result<SuccessResponse<SendSolResponse>, ErrorResponse> {
    // Validate inputs
    if request.from.is_empty() || request.to.is_empty() {
        return Err(ErrorResponse::new("Missing required fields"));
    }

    let from_pubkey = validate_pubkey(&request.from, "sender address")?;
    let to_pubkey = validate_pubkey(&request.to, "recipient address")?;
    validate_lamports(request.lamports)?;

    // Prevent sending to the same address
    if from_pubkey == to_pubkey {
        return Err(ErrorResponse::new("Sender and recipient addresses cannot be the same"));
    }

    // Create the transfer instruction
    let instruction = system_instruction::transfer(
        &from_pubkey,
        &to_pubkey,
        request.lamports
    );

    // Required accounts for the transfer
    let accounts = vec![
        AccountMeta::new(from_pubkey, true),  // from account (writable and signer)
        AccountMeta::new(to_pubkey, false),   // to account (writable)
    ];

    // Format the response
    let response = SendSolResponse {
        program_id: system_program::id().to_string(),
        accounts: accounts.iter().map(|account| AccountInfo {
            pubkey: bs58::encode(account.pubkey.to_bytes()).into_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        }).collect(),
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(SuccessResponse::new(response))
} 