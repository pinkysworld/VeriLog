#[allow(dead_code)]
mod cli;
mod config;
#[allow(dead_code)]
mod enterprise_gate;
#[cfg(feature = "admin-console")]
mod research;

#[cfg(feature = "admin-console")]
mod admin;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::path::PathBuf;

use verilog_core::{
    entry::LogLevel, hash::hash32_to_hex, storage::LICENSE_FILE, LogEntry, LogStore, MerkleProof,
};
use verilog_enterprise_api::EnterpriseFeature;
use verilog_license::{
    generate_vendor_keypair_b64, issue_license, verify_license, DeviceId, LicensePayload,
    LicenseStore, LICENSE_VERSION,
};

use crate::cli::{CheckpointCommand, Cli, Command, LicenseCommand, ProofCommand};

const DEFAULT_VENDOR_PUBKEY_B64: &str = "REPLACE_ME_WITH_VENDOR_PUBKEY_B64";

fn parse_level(s: &str) -> Result<LogLevel> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(LogLevel::Trace),
        "debug" => Ok(LogLevel::Debug),
        "info" => Ok(LogLevel::Info),
        "warn" | "warning" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        other => Err(anyhow!("unknown level: {other}")),
    }
}

fn load_payload_arg(arg: &str) -> Result<Vec<u8>> {
    if let Some(rest) = arg.strip_prefix('@') {
        let bytes = std::fs::read(rest).with_context(|| format!("read payload file: {rest}"))?;
        Ok(bytes)
    } else {
        Ok(arg.as_bytes().to_vec())
    }
}

fn vendor_pubkey_from_args(arg: Option<String>) -> Result<String> {
    if let Some(v) = arg {
        return Ok(v);
    }
    if let Ok(v) = std::env::var("VERILOG_VENDOR_PUBKEY_B64") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Ok(v);
        }
    }
    if DEFAULT_VENDOR_PUBKEY_B64 != "REPLACE_ME_WITH_VENDOR_PUBKEY_B64" {
        return Ok(DEFAULT_VENDOR_PUBKEY_B64.to_string());
    }
    Err(anyhow!(
        "vendor public key not provided. Use --vendor-pubkey-b64 or set VERILOG_VENDOR_PUBKEY_B64 (or replace DEFAULT_VENDOR_PUBKEY_B64 in source)."
    ))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProofFile {
    pub proof: MerkleProof,
    pub entry: Option<LogEntry>,
}

fn parse_entitlement(s: &str) -> Result<EnterpriseFeature> {
    // Use serde JSON parsing to leverage rename rules.
    let v: EnterpriseFeature = serde_json::from_str(&format!("\"{}\"", s))
        .map_err(|_| anyhow!("unknown entitlement: {s}"))?;
    Ok(v)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let cli = Cli::parse();

    match cli.command {
        Command::Init { store, tree_height } => {
            verilog_core::storage::LogStore::init(&store, tree_height)
                .with_context(|| format!("init store at {}", store.display()))?;
            // Generate a default config.toml alongside the store.
            let config_path = store.join("config.toml");
            config::Config::init_default(&config_path)
                .with_context(|| "write default config.toml")?;
            println!("Initialized store at {}", store.display());
            Ok(())
        }

        Command::Append {
            store,
            kind,
            payload,
            level,
        } => {
            let level = parse_level(&level)?;
            let payload = load_payload_arg(&payload)?;
            let mut s = LogStore::open(&store)?;
            let res = s.append(kind, payload, level)?;
            println!("{}", serde_json::to_string_pretty(&res)?);
            Ok(())
        }

        Command::Verify { store } => {
            let s = LogStore::open(&store)?;
            let report = s.verify_store()?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }

        Command::Status { store } => {
            let s = LogStore::open(&store)?;
            let leaf_count = s.leaf_count();
            let current_root = s.current_root()?;
            let last_entry = s.last_entry()?;
            let last_entry_summary = last_entry.map(|entry| {
                serde_json::json!({
                    "index": entry.unsigned.index,
                    "ts_unix_ms": entry.unsigned.ts_unix_ms,
                    "kind": entry.unsigned.kind,
                    "level": entry.unsigned.level,
                    "payload_len": entry.unsigned.payload.len(),
                })
            });
            let license_path = PathBuf::from(&store).join(LICENSE_FILE);
            let license_installed = LicenseStore::new(license_path).load()?.is_some();

            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "store_dir": store.display().to_string(),
                    "tree_height": s.tree_height(),
                    "leaf_count": leaf_count,
                    "current_root_hex": hash32_to_hex(&current_root),
                    "verifying_key_b64": s.verifying_key_b64(),
                    "license_installed": license_installed,
                    "last_entry": last_entry_summary,
                }))?
            );
            Ok(())
        }

        Command::Export { store, out } => {
            let s = LogStore::open(&store)?;
            match out {
                None => {
                    let stdout = std::io::stdout();
                    let handle = stdout.lock();
                    s.export_json_lines(handle)?;
                }
                Some(path) => {
                    let f = std::fs::File::create(&path)?;
                    s.export_json_lines(f)?;
                    println!("Wrote export to {}", path.display());
                }
            }
            Ok(())
        }

        Command::Checkpoint { command } => match command {
            CheckpointCommand::Create { store, out, label } => {
                let s = LogStore::open(&store)?;
                let checkpoint = s.create_checkpoint(label)?;
                let bytes = serde_json::to_vec_pretty(&checkpoint)?;

                match out {
                    Some(path) => {
                        std::fs::write(&path, &bytes)?;
                        println!("Wrote checkpoint to {}", path.display());
                    }
                    None => {
                        println!("{}", String::from_utf8(bytes)?);
                    }
                }
                Ok(())
            }
            CheckpointCommand::Verify { checkpoint } => {
                let bytes = std::fs::read(&checkpoint)?;
                let checkpoint: verilog_core::SignedCheckpoint = serde_json::from_slice(&bytes)?;
                checkpoint.verify()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "version": checkpoint.checkpoint.version,
                        "tree_height": checkpoint.checkpoint.tree_height,
                        "leaf_count": checkpoint.checkpoint.leaf_count,
                        "current_root_hex": hash32_to_hex(&checkpoint.checkpoint.current_root),
                        "last_entry_hash_hex": hash32_to_hex(&checkpoint.checkpoint.last_entry_hash),
                        "created_at_unix_ms": checkpoint.checkpoint.created_at_unix_ms,
                        "label": checkpoint.checkpoint.label,
                    }))?
                );
                Ok(())
            }
        },

        Command::Proof { command } => match command {
            ProofCommand::Membership {
                store,
                index,
                out,
                include_entry,
            } => {
                let s = LogStore::open(&store)?;
                let proof = s.membership_proof(index)?;
                let entry = if include_entry {
                    s.get_entry(index)?
                } else {
                    None
                };
                let pf = ProofFile { proof, entry };
                let bytes = serde_json::to_vec_pretty(&pf)?;
                std::fs::write(&out, bytes)?;
                println!("Wrote proof to {}", out.display());
                Ok(())
            }
            ProofCommand::Verify { proof } => {
                let bytes = std::fs::read(&proof)?;
                let pf: ProofFile = serde_json::from_slice(&bytes)?;
                let ok = pf.proof.verify()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": ok,
                        "root_hex": verilog_core::hash::hash32_to_hex(&pf.proof.root),
                        "leaf_index": pf.proof.leaf_index,
                    }))?
                );
                Ok(())
            }
        },

        Command::License { command } => match command {
            LicenseCommand::VendorKeygen { out_dir } => {
                std::fs::create_dir_all(&out_dir)?;
                let (seed_b64, pub_b64) = generate_vendor_keypair_b64();
                std::fs::write(out_dir.join("vendor_seed.b64"), seed_b64.as_bytes())?;
                std::fs::write(out_dir.join("vendor_pubkey.b64"), pub_b64.as_bytes())?;
                println!(
                    "Wrote vendor_seed.b64 and vendor_pubkey.b64 to {}",
                    out_dir.display()
                );
                Ok(())
            }

            LicenseCommand::Issue {
                vendor_seed_b64,
                issued_to,
                org,
                not_before_unix_ms,
                not_after_unix_ms,
                device_id,
                entitle,
                out,
            } => {
                let mut ents = Vec::new();
                for e in entitle {
                    ents.push(parse_entitlement(&e)?);
                }

                let payload = LicensePayload {
                    version: LICENSE_VERSION,
                    license_id: format!("lic-{}", verilog_core::util::now_unix_ms()),
                    issued_to,
                    org,
                    not_before_unix_ms,
                    not_after_unix_ms,
                    device_id,
                    entitlements: ents,
                };

                let signed = issue_license(&vendor_seed_b64, payload)?;
                let bytes = serde_json::to_vec_pretty(&signed)?;
                std::fs::write(&out, bytes)?;
                println!("Wrote license to {}", out.display());
                Ok(())
            }

            LicenseCommand::Install {
                store,
                license,
                device_id,
                vendor_pubkey_b64,
            } => {
                let vendor_pubkey_b64 = vendor_pubkey_from_args(vendor_pubkey_b64)?;
                let lic_bytes = std::fs::read(&license)?;
                let lic: verilog_license::SignedLicense = serde_json::from_slice(&lic_bytes)?;

                let device = device_id
                    .or_else(DeviceId::detect_best_effort)
                    .unwrap_or_else(|| DeviceId::random_hex_128());

                let now = verilog_core::util::now_unix_ms();
                let verified = verify_license(&vendor_pubkey_b64, &lic, now, Some(&device))
                    .with_context(|| "license verification failed")?;

                let dst = PathBuf::from(&store).join(LICENSE_FILE);
                let store = LicenseStore::new(dst);
                store.save(&lic)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "installed": true,
                        "device_id": device,
                        "license_id": verified.payload.license_id,
                        "issued_to": verified.payload.issued_to,
                        "not_after_unix_ms": verified.payload.not_after_unix_ms,
                        "entitlements": verified.payload.entitlements,
                    }))?
                );
                Ok(())
            }

            LicenseCommand::Status {
                store,
                device_id,
                vendor_pubkey_b64,
            } => {
                let vendor_pubkey_b64 = vendor_pubkey_from_args(vendor_pubkey_b64)?;
                let device = device_id
                    .or_else(DeviceId::detect_best_effort)
                    .unwrap_or_else(|| DeviceId::random_hex_128());

                let lic_path = PathBuf::from(&store).join(LICENSE_FILE);
                let ls = LicenseStore::new(lic_path);
                let lic = ls.load()?;
                if lic.is_none() {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "installed": false,
                            "device_id": device,
                        }))?
                    );
                    return Ok(());
                }
                let lic = lic.unwrap();
                let now = verilog_core::util::now_unix_ms();
                let verified = verify_license(&vendor_pubkey_b64, &lic, now, Some(&device));

                match verified {
                    Ok(v) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "installed": true,
                                "valid": true,
                                "device_id": device,
                                "license_id": v.payload.license_id,
                                "issued_to": v.payload.issued_to,
                                "org": v.payload.org,
                                "not_before_unix_ms": v.payload.not_before_unix_ms,
                                "not_after_unix_ms": v.payload.not_after_unix_ms,
                                "entitlements": v.payload.entitlements,
                            }))?
                        );
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "installed": true,
                                "valid": false,
                                "device_id": device,
                                "error": e.to_string(),
                            }))?
                        );
                    }
                }

                Ok(())
            }
        },

        Command::Serve { store, bind } => {
            #[cfg(feature = "admin-console")]
            {
                admin::run(store, bind)?;
                Ok(())
            }

            #[cfg(not(feature = "admin-console"))]
            {
                let _ = (&store, &bind);
                Err(anyhow!(
                    "admin console not enabled. Rebuild with: cargo build -p verilogd --features admin-console"
                ))
            }
        }
    }
}
