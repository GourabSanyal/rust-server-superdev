use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signer, Signature},
    pubkey::Pubkey,
};
use super::response_types::{SuccessResponse, ErrorResponse};
use bs58;
use base64;

#[derive(Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub secret: String,
}

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
    pub public_key: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct VerifyMessageResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

fn validate_secret_key(secret: &str) -> Result<Keypair, ErrorResponse> {
    let secret_bytes = bs58::decode(secret)
        .into_vec()
        .map_err(|_| ErrorResponse::new("Invalid base58 encoding for secret key"))?;

    Keypair::from_bytes(&secret_bytes)
        .map_err(|_| ErrorResponse::new("Invalid secret key format"))
}

fn validate_message(message: &str) -> Result<(), ErrorResponse> {
    if message.is_empty() {
        return Err(ErrorResponse::new("Message cannot be empty"));
    }
    Ok(())
}

fn validate_signature(signature: &str) -> Result<Signature, ErrorResponse> {
    let sig_bytes = base64::decode(signature)
        .map_err(|_| ErrorResponse::new("Invalid base64 encoding for signature"))?;
    
    Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| ErrorResponse::new("Invalid signature format"))
}

fn validate_pubkey(pubkey: &str) -> Result<Pubkey, ErrorResponse> {
    let pubkey_bytes = bs58::decode(pubkey)
        .into_vec()
        .map_err(|_| ErrorResponse::new("Invalid base58 encoding for public key"))?;

    Pubkey::try_from(pubkey_bytes.as_slice())
        .map_err(|_| ErrorResponse::new("Invalid public key format"))
}

pub fn sign_message(request: SignMessageRequest) -> Result<SuccessResponse<SignMessageResponse>, ErrorResponse> {
    // Validate inputs
    if request.message.is_empty() || request.secret.is_empty() {
        return Err(ErrorResponse::new("Missing required fields"));
    }

    validate_message(&request.message)?;
    let keypair = validate_secret_key(&request.secret)?;

    // Sign the message
    let message_bytes = request.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    // Format the response
    let response = SignMessageResponse {
        signature: base64::encode(signature.as_ref()),
        public_key: bs58::encode(keypair.pubkey().to_bytes()).into_string(),
        message: request.message,
    };

    Ok(SuccessResponse::new(response))
}

pub fn verify_message(request: VerifyMessageRequest) -> Result<SuccessResponse<VerifyMessageResponse>, ErrorResponse> {
    // Validate inputs
    if request.message.is_empty() || request.signature.is_empty() || request.pubkey.is_empty() {
        return Err(ErrorResponse::new("Missing required fields"));
    }

    validate_message(&request.message)?;
    let signature = validate_signature(&request.signature)?;
    let pubkey = validate_pubkey(&request.pubkey)?;

    // Verify the signature
    let message_bytes = request.message.as_bytes();
    let valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    // Format the response
    let response = VerifyMessageResponse {
        valid,
        message: request.message,
        pubkey: request.pubkey,
    };

    Ok(SuccessResponse::new(response))
} 