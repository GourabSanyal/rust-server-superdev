use super::response_types::{ErrorResponse, SuccessResponse};
use bs58;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};

#[derive(Serialize, Deserialize)]
pub struct KeypairResponse {
    pub pubkey: String,
    pub secret: String,
}

pub fn generate_keypair() -> Result<SuccessResponse<KeypairResponse>, ErrorResponse> {
    let keypair = Keypair::new();

    let pubkey = bs58::encode(keypair.pubkey().to_bytes()).into_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Ok(SuccessResponse::new(KeypairResponse { pubkey, secret }))
}
