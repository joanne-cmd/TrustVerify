//! REST API for verification, history, and registry.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use trustverify::history::HistoryStore;
use trustverify::registry::{verify_entry_signature, Registry, RegistryEntry};
use trustverify::verifier::{verify_with_registry, VerificationResult};

/// Shared app state. HistoryStore is behind Mutex because rusqlite::Connection is !Sync.
pub struct AppState {
    pub registry: Arc<Registry>,
    pub history: Option<Arc<Mutex<HistoryStore>>>,
}

#[derive(Deserialize)]
struct VerifyRequest {
    quote: String,
    #[serde(default)]
    #[allow(dead_code)]
    format: String,
}

#[derive(Deserialize)]
struct HistoryQuery {
    ppid: String,
}

async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> impl IntoResponse {
    let history_guard = state.history.as_ref().and_then(|m| m.lock().ok());
    let history = history_guard.as_ref().map(|g| &**g);
    let result: VerificationResult =
        verify_with_registry(&req.quote, &state.registry, history);
    (StatusCode::OK, Json(result))
}

/// GET /api/history?ppid=<hex> — list stored quotes for this PPID and any regression/migration.
async fn history_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let Some(mutex) = &state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "History store not configured" })),
        )
            .into_response();
    };
    let Ok(guard) = mutex.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "History store lock failed" })),
        )
            .into_response();
    };
    match guard.list_by_ppid(&q.ppid) {
        Ok(records) => {
            let tcb_svn = records.first().map(|r| r.tcb_svn.clone()).unwrap_or_default();
            let regression = guard.detect_regression(&q.ppid, &tcb_svn).ok().flatten();
            let migration = guard.detect_migration(&q.ppid);
            let body = serde_json::json!({
                "ppid": q.ppid,
                "records": records,
                "regression": regression,
                "migration": migration,
            });
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(serde::Serialize)]
struct RegistryEntryWithSig {
    #[serde(flatten)]
    entry: RegistryEntry,
    signature_valid: bool,
}

#[derive(serde::Serialize)]
struct RegistryResponse {
    version: String,
    description: Option<String>,
    entries: Vec<RegistryEntryWithSig>,
}

/// GET /api/registry — full registry with signature validity per entry.
async fn registry_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let r = &*state.registry;
    let entries: Vec<RegistryEntryWithSig> = r
        .entries
        .iter()
        .map(|e| RegistryEntryWithSig {
            entry: e.clone(),
            signature_valid: verify_entry_signature(e),
        })
        .collect();
    let response = RegistryResponse {
        version: r.version.clone(),
        description: r.description.clone(),
        entries,
    };
    (StatusCode::OK, Json(response))
}

pub async fn serve(
    host: String,
    port: u16,
    registry: Arc<Registry>,
    history: Option<Arc<Mutex<HistoryStore>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = Arc::new(AppState { registry, history });

    let app = Router::new()
        .route("/api/verify", post(verify_handler))
        .route("/api/history", get(history_handler))
        .route("/api/registry", get(registry_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("TrustVerify API listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
