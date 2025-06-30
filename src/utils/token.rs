use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar::rent,
};
use spl_token::{instruction as token_instruction, state::Account as TokenAccount};
use spl_associated_token_account::instruction as ata_instruction;
use super::response_types::{SuccessResponse, ErrorResponse};
use bs58;
use base64;

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Deserialize)]
pub struct MintTokenRequest {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Serialize)]
pub struct AccountInfo {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountInfo>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct MintTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountInfo>,
    pub instruction_data: String,
}

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct SendTokenResponse {
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

fn validate_decimals(decimals: u8) -> Result<(), ErrorResponse> {
    if decimals > 9 {
        return Err(ErrorResponse::new("Decimals must be between 0 and 9"));
    }
    Ok(())
}

fn validate_amount(amount: u64) -> Result<(), ErrorResponse> {
    if amount == 0 {
        return Err(ErrorResponse::new("Amount must be greater than 0"));
    }
    Ok(())
}

pub fn create_token(request: CreateTokenRequest) -> Result<SuccessResponse<CreateTokenResponse>, ErrorResponse> {
    // Validate inputs
    let mint_authority = validate_pubkey(&request.mint_authority, "mint_authority")?;
    let mint = validate_pubkey(&request.mint, "mint")?;
    validate_decimals(request.decimals)?;

    // Get the token program ID
    let token_program_id = spl_token::id();

    // Required accounts for token initialization
    let accounts = vec![
        AccountMeta::new(mint, true),                    // mint account (writable, signer)
        AccountMeta::new_readonly(mint_authority, true), // mint authority (readonly, signer)
        AccountMeta::new_readonly(rent::id(), false),    // rent sysvar (readonly)
        AccountMeta::new_readonly(system_program::id(), false), // system program (readonly)
    ];

    // Create the initialize mint instruction
    let instruction = token_instruction::initialize_mint(
        &token_program_id,
        &mint,
        &mint_authority,
        None, // freeze_authority
        request.decimals,
    ).map_err(|e| ErrorResponse::new(format!("Failed to create token instruction: {}", e)))?;

    // Format the response
    let response = CreateTokenResponse {
        program_id: token_program_id.to_string(),
        accounts: accounts.iter().map(|account| AccountInfo {
            pubkey: bs58::encode(account.pubkey.to_bytes()).into_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        }).collect(),
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(SuccessResponse::new(response))
}

pub fn mint_token(request: MintTokenRequest) -> Result<SuccessResponse<MintTokenResponse>, ErrorResponse> {
    // Validate inputs
    let mint_authority = validate_pubkey(&request.mint_authority, "mintAuthority")?;
    let mint = validate_pubkey(&request.mint, "mint")?;
    validate_decimals(request.decimals)?;

    // Get the token program ID
    let token_program_id = spl_token::id();

    // Required accounts for minting tokens
    let accounts = vec![
        AccountMeta::new(mint, true),                    // mint account (writable)
        AccountMeta::new_readonly(mint_authority, true), // mint authority (signer)
        AccountMeta::new_readonly(system_program::id(), false), // system program
    ];

    // Create the mint instruction
    let amount = 1_000_000_000; // Amount to mint (adjust based on decimals)
    let instruction = token_instruction::mint_to(
        &token_program_id,
        &mint,
        &mint,  // token account (same as mint for simplicity)
        &mint_authority,
        &[],    // signer seeds
        amount,
    ).map_err(|e| ErrorResponse::new(format!("Failed to create mint instruction: {}", e)))?;

    // Format the response
    let response = MintTokenResponse {
        program_id: token_program_id.to_string(),
        accounts: accounts.iter().map(|account| AccountInfo {
            pubkey: bs58::encode(account.pubkey.to_bytes()).into_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        }).collect(),
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(SuccessResponse::new(response))
}

pub fn send_token(request: SendTokenRequest) -> Result<SuccessResponse<SendTokenResponse>, ErrorResponse> {
    // Validate inputs
    if request.destination.is_empty() || request.mint.is_empty() || request.owner.is_empty() {
        return Err(ErrorResponse::new("Missing required fields"));
    }

    let destination = validate_pubkey(&request.destination, "destination address")?;
    let mint = validate_pubkey(&request.mint, "mint address")?;
    let owner = validate_pubkey(&request.owner, "owner address")?;
    validate_amount(request.amount)?;

    // Prevent sending to the same address
    if owner == destination {
        return Err(ErrorResponse::new("Owner and destination addresses cannot be the same"));
    }

    // Get token program ID
    let token_program_id = spl_token::id();

    // Derive Associated Token Accounts (ATAs) for both owner and destination
    let owner_ata = spl_associated_token_account::get_associated_token_address(
        &owner,
        &mint
    );
    
    let destination_ata = spl_associated_token_account::get_associated_token_address(
        &destination,
        &mint
    );

    // Create instructions
    let mut accounts = vec![
        AccountMeta::new(owner_ata, false),        // Source ATA (writable)
        AccountMeta::new(destination_ata, false),  // Destination ATA (writable)
        AccountMeta::new_readonly(owner, true),    // Owner (signer)
        AccountMeta::new_readonly(mint, false),    // Mint account
        AccountMeta::new_readonly(token_program_id, false), // Token program
    ];

    // Create the transfer instruction
    let instruction = token_instruction::transfer_checked(
        &token_program_id,
        &owner_ata,
        &mint,
        &destination_ata,
        &owner,
        &[],
        request.amount,
        9, // Decimals - typically 9 for most tokens
    ).map_err(|e| ErrorResponse::new(format!("Failed to create transfer instruction: {}", e)))?;

    // Format the response
    let response = SendTokenResponse {
        program_id: token_program_id.to_string(),
        accounts: accounts.iter().map(|account| AccountInfo {
            pubkey: bs58::encode(account.pubkey.to_bytes()).into_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        }).collect(),
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(SuccessResponse::new(response))
} 