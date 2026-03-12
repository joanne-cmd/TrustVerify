//! REST API for verification.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use trustverify::registry::Registry;
use trustverify::verifier::{verify_with_registry, VerificationResult};

#[derive(Deserialize)]
struct VerifyRequest {
    quote: String,
    #[serde(default)]
    #[allow(dead_code)]
    format: String,
}

async fn verify_handler(
    State(registry): State<Arc<Registry>>,
    Json(req): Json<VerifyRequest>,
) -> impl IntoResponse {
    let result: VerificationResult = verify_with_registry(&req.quote, &registry);
    (StatusCode::OK, Json(result))
}

pub async fn serve(
    host: String,
    port: u16,
    registry: Arc<Registry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/verify", post(verify_handler))
        .layer(cors)
        .with_state(registry);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("TrustVerify API listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
