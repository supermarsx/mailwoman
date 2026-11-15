//! OS jobs contain only the executable and a fixed switch. The hidden WebView
//! opens its usual profile and runs the shared encrypted queue replay. No token,
//! message, account identifier or encryption key is written into an OS job.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};
use tauri::Manager;

#[derive(Serialize)]
pub struct Support {
    pub supported: bool,
}

pub fn is_background() -> bool {
    std::env::args_os().any(|arg| arg == "--background-replay")
}

fn job_name(executable: &Path) -> String {
    let digest = Sha256::digest(executable.as_os_str().to_string_lossy().as_bytes());
    format!(
        "mailwoman-replay-{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}

fn run(command: &mut Command) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command
        .output()
        .map_err(|_| "Background scheduler unavailable".to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err("Background scheduler refused the job".into())
    }
}

#[cfg(windows)]
fn schedule(executable: &Path, _data: &Path, enabled: bool) -> Result<(), String> {
    let name = job_name(executable);
    if !enabled {
        // A missing job is already disabled. Do not delete another app's jobs.
        if run(Command::new("schtasks.exe").args(["/Query", "/TN", &name])).is_err() {
            return Ok(());
        }
        return run(Command::new("schtasks.exe").args(["/Delete", "/TN", &name, "/F"]));
    }
    let path = executable
        .to_str()
        .ok_or("Executable path is not Unicode")?;
    if path.contains(['"', '\r', '\n']) {
        return Err("Executable path is invalid".into());
    }
    // Interactive current-user token: no password, elevation or service account.
    // MINUTE jobs persist over process exits/reboots. The OS may defer them.
    run(Command::new("schtasks.exe").args([
        "/Create",
        "/TN",
        &name,
        "/TR",
        &format!("\"{path}\" --background-replay"),
        "/SC",
        "MINUTE",
        "/MO",
        "5",
        "/IT",
        "/RL",
        "LIMITED",
        "/F",
    ]))
}

#[cfg(target_os = "linux")]
fn schedule(executable: &Path, _data: &Path, enabled: bool) -> Result<(), String> {
    let name = job_name(executable);
    let home = std::env::var_os("HOME").ok_or("User home unavailable")?;
    let dir = std::path::PathBuf::from(home).join(".config/systemd/user");
    let timer = format!("{name}.timer");
    if !enabled {
        let _ = run(Command::new("systemctl").args(["--user", "disable", "--now", &timer]));
        for suffix in ["service", "timer"] {
            let _ = std::fs::remove_file(dir.join(format!("{name}.{suffix}")));
        }
        return run(Command::new("systemctl").args(["--user", "daemon-reload"]));
    }
    let path = executable
        .to_str()
        .ok_or("Executable path is not Unicode")?
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%");
    if path.contains(['\r', '\n']) {
        return Err("Executable path is invalid".into());
    }
    std::fs::create_dir_all(&dir).map_err(|_| "Cannot create user scheduler directory")?;
    std::fs::write(dir.join(format!("{name}.service")), format!("[Unit]\nDescription=Mailwoman pending changes\n[Service]\nType=oneshot\nExecStart=\"{path}\" --background-replay\nTimeoutStartSec=150\n")).map_err(|_| "Cannot write background service")?;
    std::fs::write(dir.join(&timer), format!("[Unit]\nDescription=Mailwoman pending changes\n[Timer]\nOnStartupSec=30\nOnUnitActiveSec=300\nPersistent=true\nUnit={name}.service\n[Install]\nWantedBy=timers.target\n")).map_err(|_| "Cannot write background timer")?;
    run(Command::new("systemctl").args(["--user", "daemon-reload"]))?;
    run(Command::new("systemctl").args(["--user", "enable", "--now", &timer]))
}

#[cfg(target_os = "macos")]
fn schedule(executable: &Path, _data: &Path, enabled: bool) -> Result<(), String> {
    let name = job_name(executable);
    let home = std::env::var_os("HOME").ok_or("User home unavailable")?;
    let dir = std::path::PathBuf::from(home).join("Library/LaunchAgents");
    let file = dir.join(format!("{name}.plist"));
    if file.exists() {
        let _ = run(Command::new("launchctl").arg("unload").arg(&file));
    }
    if !enabled {
        if file.exists() {
            std::fs::remove_file(file).map_err(|_| "Cannot remove background job")?;
        }
        return Ok(());
    }
    let path = executable
        .to_str()
        .ok_or("Executable path is not Unicode")?
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    std::fs::create_dir_all(dir).map_err(|_| "Cannot create user scheduler directory")?;
    std::fs::write(&file,format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>Label</key><string>{name}</string><key>ProgramArguments</key><array><string>{path}</string><string>--background-replay</string></array><key>StartInterval</key><integer>300</integer><key>RunAtLoad</key><true/><key>ProcessType</key><string>Background</string></dict></plist>")).map_err(|_| "Cannot write background job")?;
    run(Command::new("launchctl").arg("load").arg(file))
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn schedule(_executable: &Path, _data: &Path, _enabled: bool) -> Result<(), String> {
    Err("Background scheduling unavailable".into())
}

#[tauri::command]
pub async fn mw_background_replay_schedule(app: tauri::AppHandle, enabled: bool) -> Support {
    let app2 = app.clone();
    let supported = tauri::async_runtime::spawn_blocking(move || {
        let Ok(executable) = std::env::current_exe() else {
            return false;
        };
        let Ok(data) = app2.path().app_local_data_dir() else {
            return false;
        };
        // Never cache success in this process: a completed background process
        // may have removed the same job while an ordinary window stayed open.
        schedule(&executable, &data, enabled).is_ok()
    })
    .await
    .unwrap_or(false);
    Support { supported }
}

#[tauri::command]
pub async fn mw_background_replay_complete(app: tauri::AppHandle, complete: bool) -> Support {
    if !is_background() {
        return Support { supported: false };
    }
    if complete {
        let _ = mw_background_replay_schedule(app.clone(), false).await;
    }
    app.exit(0);
    Support { supported: true }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_are_stable_bounded_and_install_specific() {
        let first = job_name(Path::new("C:/Apps/Mailwoman.exe"));
        assert_eq!(first, job_name(Path::new("C:/Apps/Mailwoman.exe")));
        assert_ne!(first, job_name(Path::new("C:/Other/Mailwoman.exe")));
        assert!(first.len() < 64);
        assert!(
            first
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        );
    }
}
