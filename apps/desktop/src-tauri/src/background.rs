//! OS jobs contain only the executable and a fixed switch. The hidden WebView
//! opens its usual profile and runs the shared encrypted queue replay. No token,
//! message, account identifier or encryption key is written into an OS job.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
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

fn job_executable(
    current: std::path::PathBuf,
    launcher: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, String> {
    // AppImage mounts its inner executable at a temporary path that disappears
    // when the process exits. Only the persistent outer launcher can wake it.
    let executable = launcher.unwrap_or(current);
    if !executable.is_absolute() || !executable.is_file() {
        return Err("Background executable unavailable".into());
    }
    Ok(executable)
}

fn session_missing(key: &str) -> Result<bool, keyring::Error> {
    match keyring::Entry::new(crate::keychain::SERVICE_SESSION, key)?.get_password() {
        Ok(_) => Ok(false),
        Err(keyring::Error::NoEntry) => Ok(true),
        Err(error) => Err(error),
    }
}

fn run(command: &mut Command) -> Result<(), String> {
    run_bounded(command, Duration::from_secs(10))
}

fn run_bounded(command: &mut Command, limit: Duration) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Background scheduler unavailable".to_string())?;
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err("Background scheduler refused the job".into())
                };
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Background scheduler timed out".into());
            }
        }
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
        let Ok(current) = std::env::current_exe() else {
            return false;
        };
        #[cfg(target_os = "linux")]
        let launcher = std::env::var_os("APPIMAGE").map(std::path::PathBuf::from);
        #[cfg(not(target_os = "linux"))]
        let launcher = None;
        let Ok(executable) = job_executable(current, launcher) else {
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
    // Re-read the OS store, not the WebView's memoized token. An older hidden
    // process must preserve a newer foreground sign-in. The JS scheduling lock
    // serializes this cancellation with any racing foreground registration.
    let missing = if complete {
        false
    } else {
        tauri::async_runtime::spawn_blocking(|| session_missing("token").unwrap_or(false))
            .await
            .unwrap_or(false)
    };
    if complete || missing {
        let _ = mw_background_replay_schedule(app.clone(), false).await;
    }
    app.exit(0);
    Support { supported: true }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scheduler_child_fixture() {
        if std::env::var("MW_BACKGROUND_TEST_WAIT").as_deref() == Ok("1") {
            std::fs::write(
                std::env::var_os("MW_BACKGROUND_TEST_MARKER").unwrap(),
                b"waiting",
            )
            .unwrap();
            std::thread::sleep(Duration::from_secs(60));
        }
    }
    #[test]
    fn scheduler_wait_is_bounded_and_reaps_a_stalled_child() {
        let executable = std::env::current_exe().unwrap();
        let make = || {
            let mut command = Command::new(&executable);
            command.args([
                "--exact",
                "background::tests::scheduler_child_fixture",
                "--test-threads=1",
            ]);
            command.env_remove("MW_BACKGROUND_TEST_WAIT");
            command
        };
        assert!(
            run_bounded(&mut make(), Duration::from_secs(5)).is_ok(),
            "completed scheduler command must be accepted"
        );
        let mut random = [0; 8];
        getrandom::fill(&mut random).unwrap();
        let marker = std::env::temp_dir().join(format!(
            "mw-background-child-{}.started",
            random
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ));
        let result = run_bounded(
            make()
                .env("MW_BACKGROUND_TEST_WAIT", "1")
                .env("MW_BACKGROUND_TEST_MARKER", &marker),
            Duration::from_secs(1),
        );
        let entered = std::fs::read(&marker);
        let _ = std::fs::remove_file(&marker);
        assert_eq!(
            entered.unwrap(),
            b"waiting",
            "prove the fixture entered its blocking body before the timeout"
        );
        assert_eq!(
            result.unwrap_err(),
            "Background scheduler timed out",
            "the same child really waits unless terminated"
        );
    }
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
    #[test]
    fn packaged_launcher_is_used_instead_of_the_temporary_inner_executable() {
        let launcher = std::env::current_exe().unwrap();
        let inner = std::env::temp_dir()
            .join("gone-appimage-mount")
            .join("mailwoman");
        assert_eq!(
            job_executable(inner, Some(launcher.clone())).unwrap(),
            launcher
        );
        assert!(
            job_executable(
                launcher.clone(),
                Some(std::path::PathBuf::from("relative.AppImage"))
            )
            .is_err()
        );
        assert_eq!(job_executable(launcher.clone(), None).unwrap(), launcher);
    }
    #[test]
    fn missing_session_cleanup_preserves_a_present_native_session() {
        let mut random = [0; 8];
        getrandom::fill(&mut random).unwrap();
        let key = format!(
            "background-test-{}",
            random
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let entry = keyring::Entry::new(crate::keychain::SERVICE_SESSION, &key).unwrap();
        assert!(session_missing(&key).unwrap());
        entry.set_password("fixture-only-native-session").unwrap();
        let present = session_missing(&key);
        entry.delete_credential().unwrap();
        assert!(!present.unwrap());
        assert!(session_missing(&key).unwrap());
    }
}
