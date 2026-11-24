//! Load-fixture-only process control. Never an authentication or policy switch.
use serde::Serialize;
use std::sync::{OnceLock, atomic::{AtomicU64, Ordering}};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Original, Published }
impl Mode {
    fn parse(raw: Option<&str>) -> Result<Self, &'static str> {
        match raw {
            None | Some("published") => Ok(Self::Published),
            Some("original") => Ok(Self::Original),
            _ => Err("MW_NATIVE_LOAD_REQUEST_INPUTS_MODE must be original or published"),
        }
    }
    fn label(self) -> &'static str {
        match self {Self::Original=>"original",Self::Published=>"published"}
    }
}
static MODE: OnceLock<Mode> = OnceLock::new();
fn initialize(cell: &OnceLock<Mode>, raw: Option<&str>) -> Result<(), &'static str> {
    let wanted=Mode::parse(raw)?;
    let actual=cell.get_or_init(||wanted);
    if *actual==wanted {Ok(())} else {Err("request-input fixture mode already initialized differently")}
}

/// Call before fixture database creation, seeding or AppState construction.
/// No process environment reads occur at request/hint/build boundaries.
pub fn initialize_from_environment() -> Result<(), &'static str> {
    if std::env::var_os("MW_NATIVE_LOAD_ORDINARY_AUTH_MODE").is_some() {
        return Err("retired ordinary-auth mode does not control request inputs; use MW_NATIVE_LOAD_REQUEST_INPUTS_MODE");
    }
    let raw=std::env::var("MW_NATIVE_LOAD_REQUEST_INPUTS_MODE");
    match raw {
        Ok(value)=>initialize(&MODE,Some(&value)),
        Err(std::env::VarError::NotPresent)=>initialize(&MODE,None),
        Err(_)=>Err("request-input fixture mode must be Unicode"),
    }
}
pub(crate) fn enabled() -> bool { *MODE.get_or_init(||Mode::Published)==Mode::Published }

pub(crate) enum Event { Context, Ready, Consumed, Hint, Build, Built, FirstFence, RefreshFence }
static COUNTS: [AtomicU64;8]=[const { AtomicU64::new(0) };8];
pub(crate) fn record(event: Event) { COUNTS[event as usize].fetch_add(1,Ordering::Relaxed); }

/// Independently sampled monotonic process counters, not an atomic snapshot.
/// Sample before/after a workload; startup/control
/// work is included in lifetime totals and is not evidence of workload use.
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status {
    pub mode: &'static str,
    pub process_immutable: bool,
    pub ordinary_contexts: u64,
    pub ready_frames: u64,
    pub consumed_frames: u64,
    pub hint_attempts: u64,
    pub build_attempts: u64,
    pub completed_builds: u64,
    pub first_fence_attempts: u64,
    pub refresh_fence_attempts: u64,
}
/// t34 guard snapshot (`MW_GUARD_SNAPSHOT`) activity witness for the load
/// fixture: process-global counts only (see `request_inputs::snapshot::witness`).
pub fn guard_snapshot() -> serde_json::Value {
    let mut witness = crate::request_inputs::snapshot::witness();
    witness["postgresStartup"] = crate::guard_snapshot_startup::witness();
    witness
}
pub fn snapshot() -> Status {
    let counts=COUNTS.each_ref().map(|v|v.load(Ordering::Relaxed));
    Status {mode:MODE.get_or_init(||Mode::Published).label(),process_immutable:true,
        ordinary_contexts:counts[0],ready_frames:counts[1],consumed_frames:counts[2],
        hint_attempts:counts[3],build_attempts:counts[4],completed_builds:counts[5],
        first_fence_attempts:counts[6],refresh_fence_attempts:counts[7]}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_parser_is_strict_and_defaults_to_published() {
        assert!(Mode::parse(None)==Ok(Mode::Published));
        assert!(Mode::parse(Some("published"))==Ok(Mode::Published));
        assert!(Mode::parse(Some("original"))==Ok(Mode::Original));
        for value in ["","on","off","composite","Original"," published","published "] {
            assert!(Mode::parse(Some(value)).is_err());
        }
    }
    #[test]
    fn initialization_is_one_way_and_invalid_input_does_not_initialize() {
        let cell=OnceLock::new();
        assert!(initialize(&cell,Some("invalid")).is_err());
        assert!(cell.get().is_none());
        initialize(&cell,Some("original")).unwrap();
        initialize(&cell,Some("original")).unwrap();
        assert!(initialize(&cell,Some("published")).is_err());
        assert!(cell.get()==Some(&Mode::Original));
    }
}
