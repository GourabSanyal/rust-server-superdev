use axum::{
    http::Method,
    routing::{get, post},
    Router,
    Json,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod utils;
use utils::{
    generate_keypair, create_token, mint_token, sign_message, verify_message, send_sol, send_token,
    CreateTokenRequest, MintTokenRequest, SignMessageRequest, VerifyMessageRequest, SendSolRequest,
    SendTokenRequest, ErrorResponse
};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/generateKeypair", get(handle_generate_keypair))
        .route("/token/create", post(handle_create_token))
        .route("/token/mint", post(handle_mint_token))
        .route("/message/sign", post(handle_sign_message))
        .route("/message/verify", post(handle_verify_message))
        .route("/send/sol", post(handle_send_sol))
        .route("/send/token", post(handle_send_token))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server is running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn handle_generate_keypair() -> Json<serde_json::Value> {
    match generate_keypair() {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_create_token(
    Json(request): Json<CreateTokenRequest>,
) -> Json<serde_json::Value> {
    match create_token(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_mint_token(
    Json(request): Json<MintTokenRequest>,
) -> Json<serde_json::Value> {
    match mint_token(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_sign_message(
    Json(request): Json<SignMessageRequest>,
) -> Json<serde_json::Value> {
    match sign_message(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_verify_message(
    Json(request): Json<VerifyMessageRequest>,
) -> Json<serde_json::Value> {
    match verify_message(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_send_sol(
    Json(request): Json<SendSolRequest>,
) -> Json<serde_json::Value> {
    match send_sol(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}

async fn handle_send_token(
    Json(request): Json<SendTokenRequest>,
) -> Json<serde_json::Value> {
    match send_token(request) {
        Ok(response) => Json(serde_json::to_value(response).unwrap()),
        Err(err) => Json(serde_json::to_value(err).unwrap()),
    }
}
