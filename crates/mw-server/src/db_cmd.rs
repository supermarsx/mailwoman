//! `mailwoman db ...`: whole-database encryption tooling for the SQLite store.
//!
//! Keys come from the environment only (never from arguments, which land in process
//! listings and shell history):
//!
//! * current key: `MW_DB_KEY`, `MW_DB_KEY_FILE` or `MW_DB_KEY_COMMAND`
//! * new key (`encrypt`, `rekey`): `MW_DB_NEW_KEY`, `MW_DB_NEW_KEY_FILE` or `MW_DB_NEW_KEY_COMMAND`
//!   (`encrypt` falls back to the current-key variables when no new key is given)
//! * backup key (`backup`, `restore`): `MW_DB_BACKUP_KEY`, `MW_DB_BACKUP_KEY_FILE` or
//!   `MW_DB_BACKUP_KEY_COMMAND`; the current key is used when none is set

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use mw_store::encryption::{self, DbKey, TransformOptions};
use std::path::PathBuf;

#[derive(Parser)]
pub struct DbArgs {
    /// SQLite database file (env: MW_DB_PATH).
    #[arg(long, env = "MW_DB_PATH", default_value = "mailwoman.db")]
    db_path: String,
    #[command(subcommand)]
    command: DbCommand,
}

#[derive(Subcommand)]
pub enum DbCommand {
    /// Print the file's state and the configured key source. Changes nothing.
    Status,
    /// Print a new random 256-bit key as 64 hex characters.
    Keygen,
    /// Encrypt a plaintext database in place (copy, verify, swap). Stop the server first.
    Encrypt {
        /// Do everything except the swap and report what would happen.
        #[arg(long)]
        dry_run: bool,
        /// Delete the plaintext original after a successful swap (overwritten first).
        #[arg(long)]
        no_rollback_copy: bool,
    },
    /// Re-encrypt under a new key (key rotation). Stop the server first.
    Rekey {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_rollback_copy: bool,
    },
    /// Turn an encrypted database back into a plaintext one. Stop the server first.
    Decrypt {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_rollback_copy: bool,
    },
    /// Prove the database is encrypted: header, wrong key refused, right key opens,
    /// a marker row round-trips and is absent from the file bytes.
    Verify,
    /// Write an encrypted copy of the database.
    Backup {
        /// Destination file (must not exist).
        #[arg(long)]
        to: PathBuf,
        /// Allow a plaintext copy (refused by default for an encrypted database).
        #[arg(long)]
        plaintext: bool,
    },
    /// Restore a backup to a new file, re-keyed to the current key.
    Restore {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
    },
    /// Put a rollback copy written by encrypt/rekey/decrypt back in place.
    Rollback {
        #[arg(long)]
        copy: PathBuf,
    },
    /// Overwrite and delete a rollback copy once the result has been checked.
    PurgeRollback { copy: PathBuf },
}

fn key_from(prefix: &str) -> anyhow::Result<Option<DbKey>> {
    let (key, _) = encryption::resolve_key_from_env(
        &format!("{prefix}_KEY"),
        &format!("{prefix}_KEY_FILE"),
        &format!("{prefix}_KEY_COMMAND"),
        None,
    )?;
    Ok(key)
}

fn current_key() -> anyhow::Result<Option<DbKey>> {
    Ok(encryption::EncryptionConfig::from_env()?.key)
}

fn report(r: &encryption::TransformReport) {
    println!("{}{}", r.operation, if r.dry_run { " (dry run, nothing changed)" } else { "" });
    println!("  {} -> {}", r.from_state, r.to_state);
    println!("  tables: {}   rows: {}   integrity_check: {}", r.tables.len(), r.rows_total, r.integrity);
    for (t, n) in &r.tables {
        println!("    {t}: {n}");
    }
    if let Some(fp) = &r.new_key_fingerprint {
        println!("  new key fingerprint: {fp}");
    }
    if let Some(p) = &r.rollback_copy {
        println!("  rollback copy kept at {p}");
        println!("  Delete it with `mailwoman db purge-rollback` once you have checked the result;");
        println!("  until then it holds the data under the OLD protection (plaintext after `encrypt`).");
    }
}

pub async fn run(args: DbArgs) -> anyhow::Result<()> {
    let Some(path) = encryption::sqlite_file_from_dsn(&args.db_path) else {
        bail!("`mailwoman db` works on a SQLite file; for Postgres see docs/admin/encryption.md");
    };
    match args.command {
        DbCommand::Keygen => {
            println!("{}", DbKey::generate().to_hex().as_str());
        }
        DbCommand::Status => {
            let cfg = encryption::EncryptionConfig::from_env()?;
            println!("file:        {}", path.display());
            println!("file state:  {:?}", encryption::probe_file(&path)?);
            println!("mode:        {}", cfg.mode.as_str());
            match (&cfg.key, &cfg.source) {
                (Some(k), Some(s)) => println!("key:         {} ({}: {}), fingerprint {}", "configured", s.kind, s.detail, k.fingerprint()),
                (Some(k), None) => println!("key:         configured, fingerprint {}", k.fingerprint()),
                _ => println!("key:         not configured"),
            }
            if let Some(m) = encryption::read_meta(&path) {
                println!("last rotated: {}", m.last_rotated_at.unwrap_or_else(|| "never".into()));
            }
        }
        DbCommand::Encrypt { dry_run, no_rollback_copy } => {
            let new = match key_from("MW_DB_NEW")? {
                Some(k) => k,
                None => current_key()?.context("no key: set MW_DB_KEY, MW_DB_KEY_FILE or MW_DB_KEY_COMMAND (or the MW_DB_NEW_* equivalents)")?,
            };
            let r = encryption::transform(&path, None, Some(&new), &TransformOptions { dry_run, keep_rollback_copy: !no_rollback_copy }).await?;
            report(&r);
            if !dry_run {
                println!("Next: set MW_DB_ENCRYPTION=required and start the server with the same key.");
            }
        }
        DbCommand::Rekey { dry_run, no_rollback_copy } => {
            let old = current_key()?.context("the current key is not configured (MW_DB_KEY*)")?;
            let new = key_from("MW_DB_NEW")?.context("the new key is not configured (MW_DB_NEW_KEY*)")?;
            if old.fingerprint() == new.fingerprint() {
                bail!("the new key is the same as the current key");
            }
            let r = encryption::transform(&path, Some(&old), Some(&new), &TransformOptions { dry_run, keep_rollback_copy: !no_rollback_copy }).await?;
            report(&r);
            if !dry_run {
                println!("Next: point MW_DB_KEY* at the new key and start the server. Keep the old key until the rollback copy is purged.");
            }
        }
        DbCommand::Decrypt { dry_run, no_rollback_copy } => {
            let old = current_key()?.context("the current key is not configured (MW_DB_KEY*)")?;
            let r = encryption::transform(&path, Some(&old), None, &TransformOptions { dry_run, keep_rollback_copy: !no_rollback_copy }).await?;
            report(&r);
            if !dry_run {
                println!("The database is now plaintext. Set MW_DB_ENCRYPTION=off (or optional) before starting the server.");
            }
        }
        DbCommand::Verify => {
            let key = current_key()?;
            let r = encryption::verify(&path, key.as_ref()).await;
            for c in &r.checks {
                println!("[{}] {}: {}", if c.passed { "pass" } else { "FAIL" }, c.name, c.detail);
            }
            if !r.passed {
                bail!("the database is not verified as encrypted");
            }
            println!("verified");
        }
        DbCommand::Backup { to, plaintext } => {
            let key = current_key()?;
            let backup_key = key_from("MW_DB_BACKUP")?.or_else(|| key.clone());
            let counts = encryption::backup(&path, key.as_ref(), &to, backup_key.as_ref(), plaintext).await?;
            println!("backup written to {} ({} tables, {} rows)", to.display(), counts.len(), counts.iter().map(|(_, n)| n).sum::<i64>());
            println!("backup key: {}", backup_key.map_or("none (plaintext)".to_string(), |k| format!("fingerprint {}", k.fingerprint())));
        }
        DbCommand::Restore { from, to } => {
            let backup_key = key_from("MW_DB_BACKUP")?.or(current_key()?);
            let live = current_key()?;
            let counts = encryption::restore(&from, backup_key.as_ref(), &to, live.as_ref()).await?;
            println!("restored to {} ({} tables, {} rows)", to.display(), counts.len(), counts.iter().map(|(_, n)| n).sum::<i64>());
        }
        DbCommand::Rollback { copy } => {
            encryption::rollback(&path, &copy)?;
            println!("restored {} over {}", copy.display(), path.display());
        }
        DbCommand::PurgeRollback { copy } => {
            encryption::purge_file(&copy)?;
            println!("removed {}", copy.display());
        }
    }
    Ok(())
}
