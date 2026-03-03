#![cfg(feature = "admin-console")]

use anyhow::{Context, Result};
use axum::{
    extract::{Path as RoutePath, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};
use verilog_core::{entry::LogLevel, hash::hash32_to_hex, LogStore};
use verilog_license::{verify_license, DeviceId, LicenseStore};

use crate::research;

#[derive(Clone)]
struct AppState {
    store_dir: PathBuf,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    store_dir: String,
    tree_height: usize,
    leaf_count: u64,
    current_root_hex: String,
    verifying_key_b64: String,
    last_entry: Option<LastEntrySummary>,
    license: LicenseStatus,
}

#[derive(Debug, Serialize)]
struct LastEntrySummary {
    index: u64,
    ts_unix_ms: u64,
    kind: String,
    level: LogLevel,
}

#[derive(Debug, Serialize)]
struct LicenseStatus {
    installed: bool,
    valid: Option<bool>,
    details: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct MembershipQuery {
    index: u64,
}

async fn index() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

async fn status(State(st): State<AppState>) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let store = LogStore::open(&st.store_dir).map_err(internal_err)?;
    let leaf_count = store.leaf_count();
    let root = store.current_root().map_err(internal_err)?;
    let verifying_key_b64 = store.verifying_key_b64();
    let last_entry = store
        .last_entry()
        .map_err(internal_err)?
        .map(|entry| LastEntrySummary {
            index: entry.unsigned.index,
            ts_unix_ms: entry.unsigned.ts_unix_ms,
            kind: entry.unsigned.kind,
            level: entry.unsigned.level,
        });

    // License status best-effort
    let lic_path = st.store_dir.join(verilog_core::storage::LICENSE_FILE);
    let ls = LicenseStore::new(lic_path);
    let installed_license = ls.load().map_err(internal_err)?;
    let installed = installed_license.is_some();

    let device = DeviceId::detect_best_effort().unwrap_or_else(|| DeviceId::random_hex_128());

    let vendor_pub = std::env::var("VERILOG_VENDOR_PUBKEY_B64").ok();
    let (valid, details) = if let (Some(vendor_pubkey_b64), Some(lic)) =
        (vendor_pub, installed_license)
    {
        let now = verilog_core::util::now_unix_ms();
        match verify_license(&vendor_pubkey_b64, &lic, now, Some(&device)) {
            Ok(v) => (
                Some(true),
                serde_json::json!({"license_id": v.payload.license_id, "entitlements": v.payload.entitlements, "device_id": device}),
            ),
            Err(e) => (
                Some(false),
                serde_json::json!({"error": e.to_string(), "device_id": device}),
            ),
        }
    } else {
        (
            None,
            serde_json::json!({"note": "set VERILOG_VENDOR_PUBKEY_B64 to validate licenses"}),
        )
    };

    Ok(Json(StatusResponse {
        store_dir: st.store_dir.display().to_string(),
        tree_height: store.tree_height(),
        leaf_count,
        current_root_hex: hash32_to_hex(&root),
        verifying_key_b64,
        last_entry,
        license: LicenseStatus {
            installed,
            valid,
            details,
        },
    }))
}

async fn verify(
    State(st): State<AppState>,
) -> Result<Json<verilog_core::VerifyReport>, (StatusCode, String)> {
    let store = LogStore::open(&st.store_dir).map_err(internal_err)?;
    let report = store.verify_store().map_err(internal_err)?;
    Ok(Json(report))
}

async fn membership_proof(
    State(st): State<AppState>,
    Query(q): Query<MembershipQuery>,
) -> Result<Json<verilog_core::MerkleProof>, (StatusCode, String)> {
    let store = LogStore::open(&st.store_dir).map_err(internal_err)?;
    let proof = store.membership_proof(q.index).map_err(internal_err)?;
    Ok(Json(proof))
}

async fn checkpoint(
    State(st): State<AppState>,
) -> Result<Json<verilog_core::SignedCheckpoint>, (StatusCode, String)> {
    let store = LogStore::open(&st.store_dir).map_err(internal_err)?;
    let checkpoint = store
        .create_checkpoint(Some("admin-console".into()))
        .map_err(internal_err)?;
    Ok(Json(checkpoint))
}

async fn research_tracks() -> Json<&'static [research::ResearchTrack]> {
    Json(research::tracks())
}

async fn research_track(
    RoutePath(id): RoutePath<String>,
) -> Result<Json<&'static research::ResearchTrack>, (StatusCode, String)> {
    let track = research::track(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("unknown track: {id}")))?;
    Ok(Json(track))
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub fn run(store_dir: PathBuf, bind: String) -> Result<()> {
    let addr: SocketAddr = bind.parse().context("parse bind addr")?;
    let state = AppState { store_dir };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/status", get(status))
        .route("/api/verify", post(verify))
        .route("/api/checkpoint", get(checkpoint))
        .route("/api/proofs/membership", get(membership_proof))
        .route("/api/research/tracks", get(research_tracks))
        .route("/api/research/track/:id", get(research_track))
        .with_state(state);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        tracing::info!("admin console listening on http://{}", addr);
        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
