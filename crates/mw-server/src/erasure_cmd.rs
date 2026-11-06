//! `mailwoman erasure ...`: backup, restore with erasure replay, tombstone
//! hand-over and log scrubbing, for the SQLite store.
//!
//! A backup written by `erasure backup` records the erasure tombstone watermark in
//! its manifest. `erasure restore` copies a backup to a new database file and, when
//! `backup.propagateErasure` is on (and `--no-propagate` is not given), replays the
//! tombstones newer than that watermark from the install being replaced, so people
//! erased after the backup are erased again. The tombstones come from the live
//! database (`--live-db`) or from a file written by `erasure export-tombstones`.
//!
//! The server key must be the one the databases were written with (`MW_SERVER_KEY`).

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use mw_store::{ServerKey, Store, erasure::Tombstone, scrub::KeyRow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
pub struct ErasureArgs {
    /// SQLite database file (env: MW_DB_PATH).
    #[arg(long, env = "MW_DB_PATH", default_value = "mailwoman.db")]
    db_path: String,
    #[command(subcommand)]
    command: ErasureCommand,
}

#[derive(Subcommand)]
pub enum ErasureCommand {
    /// Print the erasure tombstone watermark of the database.
    Watermark,
    /// Write a consistent copy of the database plus a manifest recording the watermark.
    Backup {
        /// Destination file (must not exist); the manifest is `<out>.manifest.json`.
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "backup")]
        label: String,
    },
    /// Write the tombstones (and the key generations they need) newer than a watermark.
    ExportTombstones {
        #[arg(long)]
        since: i64,
        #[arg(long)]
        out: PathBuf,
    },
    /// Restore a backup to a new database file and replay newer erasures.
    Restore {
        /// The backup file written by `erasure backup`.
        #[arg(long)]
        backup: PathBuf,
        /// The database file to create.
        #[arg(long)]
        to: PathBuf,
        /// The live database being replaced: its newer tombstones are replayed.
        #[arg(long)]
        live_db: Option<String>,
        /// A file written by `erasure export-tombstones` (instead of --live-db).
        #[arg(long)]
        tombstones: Option<PathBuf>,
        /// Restore only; do not replay erasures.
        #[arg(long)]
        no_propagate: bool,
    },
    /// Replace one address in the rotated log files of a directory.
    ScrubLogs {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long)]
        address: String,
        /// Report what would change.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct Handover {
    tombstones: Vec<Tombstone>,
    keys: Vec<HandoverKey>,
}

#[derive(Serialize, Deserialize)]
struct HandoverKey {
    generation: i64,
    sealed_key_hex: String,
    created_at: i64,
    retired_at: Option<i64>,
    destroy_after: Option<i64>,
}

fn server_key() -> anyhow::Result<ServerKey> {
    let hex = std::env::var("MW_SERVER_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .context("MW_SERVER_KEY must be set")?;
    ServerKey::from_hex(&hex).map_err(|_| anyhow::anyhow!("MW_SERVER_KEY is not valid hex"))
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> anyhow::Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        bail!("odd-length hex");
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| anyhow::anyhow!("hex: {e}")))
        .collect()
}

pub async fn run(args: ErasureArgs) -> anyhow::Result<()> {
    match args.command {
        ErasureCommand::ScrubLogs { dir, address, dry_run } => {
            let person = scrub_pseudonym(&args.db_path, &address).await?;
            let r = crate::scrub_admin::scrub_log_dir(&dir, &[(address, person)], dry_run);
            println!("{} line(s) in {} file(s){}", r.lines, r.files, if dry_run { " would change" } else { " changed" });
            for e in &r.errors {
                eprintln!("error: {e}");
            }
            if !r.errors.is_empty() {
                bail!("{} file(s) could not be rewritten", r.errors.len());
            }
            Ok(())
        }
        ErasureCommand::Watermark => {
            let store = Store::open(&args.db_path, server_key()?).await?;
            println!("{}", store.erasure_watermark().await?);
            Ok(())
        }
        ErasureCommand::Backup { out, label } => {
            let store = Store::open(&args.db_path, server_key()?).await?;
            let manifest = store.backup_to_file(&out, &label, now()).await?;
            println!("{}", serde_json::to_string_pretty(&manifest)?);
            Ok(())
        }
        ErasureCommand::ExportTombstones { since, out } => {
            let store = Store::open(&args.db_path, server_key()?).await?;
            let h = handover(&store, since).await?;
            std::fs::write(&out, serde_json::to_vec_pretty(&h)?)?;
            println!("{} tombstone(s) written to {}", h.tombstones.len(), out.display());
            Ok(())
        }
        ErasureCommand::Restore { backup, to, live_db, tombstones, no_propagate } => {
            if to.exists() {
                bail!("{} already exists", to.display());
            }
            let manifest_path = {
                let mut m = backup.as_os_str().to_owned();
                m.push(".manifest.json");
                PathBuf::from(m)
            };
            std::fs::copy(&backup, &to).with_context(|| format!("copy {}", backup.display()))?;
            let key = server_key()?;
            let restored = Store::open(&to.to_string_lossy(), key.clone()).await?;
            let watermark = match std::fs::read(&manifest_path)
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                .and_then(|v| v["tombstoneWatermark"].as_i64())
            {
                Some(w) => w,
                None => restored.erasure_watermark().await?,
            };
            let live = match &live_db {
                Some(p) => Some(Store::open(p, key.clone()).await?),
                None => None,
            };
            let propagate = !no_propagate && propagate_enabled(live.as_ref().unwrap_or(&restored)).await;
            if !propagate {
                println!("restored; erasure replay is off");
                return Ok(());
            }
            let h = match (&live, &tombstones) {
                (Some(l), _) => handover(l, watermark).await?,
                (None, Some(f)) => serde_json::from_slice(&std::fs::read(f)?)?,
                (None, None) => {
                    println!("restored; no live database or tombstone file given, so nothing was replayed");
                    return Ok(());
                }
            };
            let keys: Vec<KeyRow> = h
                .keys
                .iter()
                .map(|k| {
                    Ok(KeyRow {
                        generation: k.generation,
                        sealed_key: unhex(&k.sealed_key_hex)?,
                        created_at: k.created_at,
                        retired_at: k.retired_at,
                        destroy_after: k.destroy_after,
                    })
                })
                .collect::<anyhow::Result<_>>()?;
            let newer: Vec<Tombstone> = h.tombstones.into_iter().filter(|t| t.seq > watermark).collect();
            let report = restored.erasure_replay(&newer, &keys, now()).await;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.errors.is_empty() {
                bail!("{} tombstone(s) could not be replayed", report.errors.len());
            }
            Ok(())
        }
    }
}

async fn handover(store: &Store, since: i64) -> anyhow::Result<Handover> {
    Ok(Handover {
        tombstones: store.erasure_tombstones_since(since).await?,
        keys: store
            .scrub_key_rows()
            .await?
            .into_iter()
            .map(|k| HandoverKey {
                generation: k.generation,
                sealed_key_hex: hex(&k.sealed_key),
                created_at: k.created_at,
                retired_at: k.retired_at,
                destroy_after: k.destroy_after,
            })
            .collect(),
    })
}

async fn propagate_enabled(store: &Store) -> bool {
    crate::settings_registry::effective_for(store, "backup.propagateErasure", &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|e| e.value.as_bool())
        .unwrap_or(true)
}

async fn scrub_pseudonym(db_path: &str, address: &str) -> anyhow::Result<String> {
    let store = Store::open(db_path, server_key()?).await?;
    let (generation, key) = store.scrub_key_current(now()).await?;
    Ok(mw_store::scrub::pseudonym(
        &key,
        generation,
        "person",
        address,
        mw_store::scrub::DEFAULT_ALGORITHM,
    ))
}
