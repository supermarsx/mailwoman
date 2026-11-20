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
    /// Ask the running server to convert SQLite without stopping the service.
    Online {
        /// Read the running server's latest conversion report without opening SQLite.
        #[arg(long,conflicts_with="cancel")]
        status: bool,
        /// Request cancellation before the non-interruptible publication stage.
        #[arg(long)]
        cancel: bool,
        #[arg(long,value_parser=clap::value_parser!(u32).range(1..=128))]
        batch_rows: Option<u32>,
        #[arg(long,value_parser=clap::value_parser!(u32).range(1..=256))]
        batch_mib: Option<u32>,
        #[arg(long,value_parser=clap::value_parser!(u64).range(1024..=2_000_000))]
        journal_limit: Option<u64>,
        #[arg(long,value_parser=clap::value_parser!(u64).range(0..=1000))]
        pause_ms: Option<u64>,
        #[arg(long,value_parser=clap::value_parser!(u64).range(32..=4096))]
        max_wal_mib: Option<u64>,
        #[arg(long,value_parser=clap::value_parser!(u64).range(100..=10_000))]
        max_cutover_ms: Option<u64>,
    },
    /// Preview or apply offline cleanup. Stop ALL writers before --apply.
    Cleanup {
        #[arg(long)]
        apply: bool,
        /// Confirm that the server and every database writer have been stopped.
        #[arg(long, requires = "apply")]
        offline: bool,
        /// Remove cached bodies with no message reference (never live mail).
        #[arg(long)]
        orphan_bodies: bool,
        /// Purge the default sibling search index; rebuild it after restarting.
        #[arg(long)]
        purge_search_index: bool,
        /// Purge operational diagnostics older than N days (0 means all).
        #[arg(long)]
        logs_older_than_days: Option<u32>,
        /// Rebuild SQLite to reclaim free pages. May need twice the DB size free.
        #[arg(long)]
        compact: bool,
        /// Enable incremental auto-vacuum; NONE-mode databases need one offline rebuild.
        #[arg(long)]
        enable_incremental_vacuum: bool,
    },
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
    // Online control writes a sealed local request to the existing server. It
    // must run before any offline lease or raw SQLite connection is opened.
    if let DbCommand::Online{status,cancel,batch_rows,batch_mib,journal_limit,pause_ms,max_wal_mib,max_cutover_ms}=&args.command {
        let path=encryption::sqlite_file_from_dsn(&args.db_path).context("online conversion requires an on-disk SQLite installation")?;
        if *status||*cancel {
            if batch_rows.is_some()||batch_mib.is_some()||journal_limit.is_some()||pause_ms.is_some()||max_wal_mib.is_some()||max_cutover_ms.is_some(){bail!("tuning options apply only when starting a conversion");}
        }
        if *status {
            println!("{}",serde_json::to_string_pretty(&mw_store::online_control::online_conversion_status_path(&path))?);
            return Ok(());
        }
        let explicit=std::env::var("MW_SERVER_KEY").ok();
        let key=mw_server::server_key::resolve(&args.db_path,explicit.as_deref())?;
        let id=if *cancel {
            mw_store::online_control::submit_online_cancellation(&path,&key)
        }else{
            let mut options=mw_store::online_maintenance::ConversionOptions::default();
            if let Some(value)=batch_rows{options.batch_rows=*value;}
            if let Some(value)=batch_mib{options.batch_mib=*value;}
            if let Some(value)=journal_limit{options.journal_limit=*value;}
            if let Some(value)=pause_ms{options.pause_ms=*value;}
            if let Some(value)=max_wal_mib{options.max_wal_mib=*value;}
            if let Some(value)=max_cutover_ms{options.max_cutover_ms=*value;}
            mw_store::online_control::submit_online_conversion(&path,&key,options)
        }.map_err(anyhow::Error::msg)?;
        println!("Queued online maintenance command: {id}");
        let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {
            if let Some(ack)=mw_store::online_control::online_conversion_acknowledgement(&path,&id) {
                if !ack.accepted { bail!("Online maintenance command rejected: {}",ack.error.as_deref().unwrap_or("request was not accepted")); }
                println!("The running server accepted this command.");
                break;
            }
            if tokio::time::Instant::now()>=deadline { bail!("The running server has not acknowledged command {id}; check server status before retrying."); }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!("The server remains running. Check progress with mailwoman db online --status, or in Observability > Database.");
        return Ok(());
    }
    if matches!(&args.command, DbCommand::Keygen) {
        println!("{}", DbKey::generate().to_hex().as_str());
        return Ok(());
    }
    let Some(logical_path) = encryption::sqlite_file_from_dsn(&args.db_path) else {
        bail!("`mailwoman db` works on a SQLite file; for Postgres see docs/admin/encryption.md");
    };
    // Read-only inspection can coexist with the running server. Both lease
    // types exclude conversion and keep CURRENT stable through raw-handle close.
    let read_only=matches!(&args.command,DbCommand::Status|DbCommand::Cleanup{apply:false,..});
    let inspection_lease=read_only.then(||mw_store::generation_maintenance::DatabaseInspectionLease::acquire(&logical_path)).transpose()?;
    let offline_lease=(!read_only).then(||mw_store::generation_maintenance::OfflineDatabaseLease::acquire(&logical_path)).transpose()?;
    let path=inspection_lease.as_ref().map(|lease|lease.active_path()).or_else(||offline_lease.as_ref().map(|lease|lease.active_path())).expect("one maintenance lease").to_path_buf();
    let logical_path=inspection_lease.as_ref().map(|lease|lease.logical_path()).or_else(||offline_lease.as_ref().map(|lease|lease.logical_path())).expect("one maintenance lease").to_path_buf();
    match args.command {
        DbCommand::Online{..}=>unreachable!("online control returned before offline commands"),
        DbCommand::Cleanup {apply,offline,orphan_bodies,purge_search_index,logs_older_than_days,compact,enable_incremental_vacuum} => {
            if apply&&!offline {bail!("stop every writer first, then acknowledge with --offline; omit --apply for a read-only preview");}
            if !orphan_bodies&&!purge_search_index&&logs_older_than_days.is_none()&&!compact&&!enable_incremental_vacuum {bail!("select --orphan-bodies, --purge-search-index, --logs-older-than-days N, --compact or --enable-incremental-vacuum");}
            if purge_search_index {
                if std::env::var("MW_SEARCH_DIR").ok().is_some_and(|dir|!dir.is_empty()) {bail!("index purge supports the default sibling directory only; use Maintenance > Rebuild search index for a custom MW_SEARCH_DIR");}
                let result=mw_store::database_maintenance::purge_default_search_index(&logical_path,apply).map_err(|e|anyhow::anyhow!(e.to_string()))?;
                println!("Search index: {}",serde_json::to_string_pretty(&result)?);
            }
            let cancel=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let requested=cancel.clone();
            #[cfg(unix)]
            let mut terminate=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            let signals=tokio::spawn(async move {
                #[cfg(unix)]
                tokio::select! {_=tokio::signal::ctrl_c()=>{},_=terminate.recv()=>{}}
                #[cfg(not(unix))]
                let _=tokio::signal::ctrl_c().await;
                requested.store(true,std::sync::atomic::Ordering::Relaxed);
                eprintln!("[maintenance] cancellation requested; waiting for SQLite to interrupt, roll back the active statement and close. Keep the app stopped until this command exits.");
            });
            // Keep polling the operation after a signal. Dropping a VACUUM
            // future would not prove that its worker has released the file.
            let result=mw_store::database_maintenance::sqlite_maintenance_with_control(&path,apply,orphan_bodies,logs_older_than_days,compact,enable_incremental_vacuum,cancel).await;
            signals.abort();let _=signals.await;
            let result=result.map_err(|e|anyhow::anyhow!(e.to_string()))?;
            println!("{}",serde_json::to_string_pretty(&result)?);
        }
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
            let destination_lease = mw_store::generation_maintenance::OfflineDatabaseLease::acquire(&to)?;
            let backup_key = key_from("MW_DB_BACKUP")?.or(current_key()?);
            let live = current_key()?;
            let counts = encryption::restore(&from, backup_key.as_ref(), destination_lease.active_path(), live.as_ref()).await?;
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


#[cfg(test)]
mod online_cli_tests {
    use super::*;
    #[test]
    fn online_cli_modes_and_tuning_are_explicit_and_bounded() {
        let status=DbArgs::try_parse_from(["mailwoman db","online","--status"]).unwrap();
        assert!(matches!(status.command,DbCommand::Online{status:true,cancel:false,..}));
        assert!(DbArgs::try_parse_from(["mailwoman db","online","--status","--cancel"]).is_err());
        for value in ["0","129"] {assert!(DbArgs::try_parse_from(["mailwoman db","online","--batch-rows",value]).is_err());}
        assert!(DbArgs::try_parse_from(["mailwoman db","online","--batch-mib","32","--max-cutover-ms","2000"]).is_ok());
    }
    #[tokio::test]
    async fn online_status_does_not_open_or_modify_sqlite_or_create_a_server_key() {
        let dir=std::env::temp_dir().join(format!("mw-db-online-cli-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&dir).unwrap();let path=dir.join("not-a-database.db");
        std::fs::write(&path,b"sentinel - not a sqlite database").unwrap();
        let args=DbArgs::try_parse_from(["mailwoman db","--db-path",path.to_str().unwrap(),"online","--status"]).unwrap();
        run(args).await.unwrap();
        assert_eq!(std::fs::read(&path).unwrap(),b"sentinel - not a sqlite database");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(),1);
        std::fs::remove_file(path).unwrap();std::fs::remove_dir(dir).unwrap();
    }
}
