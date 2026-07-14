//! The wasmtime `Engine` + `Config` for the plugin jail (plan §2.1, SPEC §7.5).
//!
//! Config posture, security-first:
//! - **Component model on**, **async** (host drives an async loop; guest calls are
//!   preemptible), **epoch interruption** (wall-clock deadline), **fuel** metering
//!   (optional, per-store).
//! - **Cranelift JIT** by default; the `pulley` crate feature selects the **Pulley
//!   pure-interpreter** target for `MemoryDenyWriteExecute` hosts (§7.5 systemd)
//!   that forbid the JIT's W^X page.
//! - No ambient authority is configured here — capabilities are wired per-store in
//!   [`crate::host_state`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use wasmtime::{Config, Engine, Strategy};

use crate::{PluginError, Result};

/// The epoch ticker cadence. Coarse enough to be cheap, fine enough that a tight
/// `deadline_ms` resolves promptly. (Windows sleep granularity is ~15 ms; a busy
/// loop never terminates, so any positive tick rate trips the deadline.)
pub(crate) const EPOCH_TICK: Duration = Duration::from_millis(5);

/// Build the jail engine. `pulley` (crate feature) ⇒ interpreter target.
pub(crate) fn build_engine() -> Result<Engine> {
    let mut cfg = Config::new();
    cfg.wasm_component_model(true);
    // NB: async support is always available in this wasmtime line (the old
    // `async_support` toggle is a no-op); host/guest calls use `*_async`.
    cfg.epoch_interruption(true);
    cfg.consume_fuel(true);
    // wasm backtraces are on by default (≤20 frames) — kept for actionable
    // `Runtime`/`LimitExceeded` diagnostics.

    #[cfg(feature = "pulley")]
    {
        // Pulley pure-interpreter: no JIT, no W^X page — safe under systemd
        // MemoryDenyWriteExecute (§7.5). Selected by compilation target, not by
        // `Strategy` (wasmtime 38 exposes Pulley only as a target triple).
        cfg.target("pulley64")
            .map_err(|e| PluginError::Load(format!("pulley target unavailable: {e}")))?;
    }
    #[cfg(not(feature = "pulley"))]
    {
        // Cranelift JIT (W^X): the default high-performance backend.
        cfg.strategy(Strategy::Cranelift);
    }

    Engine::new(&cfg).map_err(|e| PluginError::Load(format!("wasmtime engine init: {e}")))
}

/// A background thread that advances the engine epoch so per-store wall-clock
/// deadlines (epoch-interruption) actually fire. One per [`crate::PluginHost`];
/// stopped + joined on drop so no thread leaks.
pub(crate) struct EpochTicker {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl EpochTicker {
    pub(crate) fn spawn(engine: Engine) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let handle = std::thread::Builder::new()
            .name("mw-plugin-epoch".into())
            .spawn(move || {
                while !stop_thread.load(Ordering::Relaxed) {
                    std::thread::sleep(EPOCH_TICK);
                    engine.increment_epoch();
                }
            })
            .expect("spawn epoch ticker");
        Self {
            stop,
            handle: Some(handle),
        }
    }

    /// Epoch ticks that correspond to a wall-clock budget of `deadline_ms`.
    pub(crate) fn ticks_for(deadline_ms: u64) -> u64 {
        let tick_ms = EPOCH_TICK.as_millis() as u64;
        (deadline_ms / tick_ms).max(1)
    }
}

impl Drop for EpochTicker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
