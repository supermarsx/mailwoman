//! Integration tests for the t10-e1 PIM/parity host surface: per-interface export
//! probing + the `calendar`/`tasks`/`bridge-parity` → `mw-engine` `Bridge*` adapters,
//! driven against REAL committed components:
//!
//! * `tests/fixtures/guest.wasm` — a `world plugin` (0.1.0) component. It exports NONE
//!   of the PIM interfaces, so it MUST load unchanged with no PIM adapter bound
//!   (probing proven) and its account-backend path is untouched.
//! * `tests/fixtures/pim-guest.wasm` — a `world plugin-pim` component (built from
//!   `tests/pim-guest-fixture/`). It exports calendar/tasks/bridge-parity, so the host
//!   probe binds them; reactions + recall + calendar are supported (round-trip proof),
//!   voting + focused are honestly unsupported.
//!
//! These prove the security core on the PIM seam: through-the-jail calls under the
//! account-backend resource limits, a trap → typed `LimitExceeded` (host survives, no
//! panic), and a deny-by-default capability gate → `CapabilityDenied`.

use mw_engine::{MessageRef, RecallOutcome};
use mw_plugin::{Capability, Grant, PluginError, PluginHost, PluginLimits, PluginManifest};

const GUEST: &[u8] = include_bytes!("fixtures/guest.wasm");
const PIM: &[u8] = include_bytes!("fixtures/pim-guest.wasm");

fn manifest(caps: Vec<Capability>, limits: PluginLimits) -> PluginManifest {
    PluginManifest {
        id: "pim-guest".into(),
        name: "PIM Guest".into(),
        version: "0".into(),
        signature: None,
        capabilities: caps,
        net_allowlist: vec![],
        limits,
    }
}

fn grant(caps: Vec<Capability>) -> Grant {
    Grant {
        plugin_id: "pim-guest".into(),
        capabilities: caps,
        granted_by: "admin@test".into(),
        allow_unsigned: true,
    }
}

fn fast_limits() -> PluginLimits {
    PluginLimits {
        memory_mb: 64,
        deadline_ms: 5_000,
        fuel: None,
    }
}

fn plugin_msg() -> MessageRef {
    MessageRef::Plugin {
        raw: "provider-native-id".into(),
    }
}

// ── (1) a `world plugin`-only component loads unchanged, binds NO PIM adapter ──────

#[tokio::test]
async fn plugin_only_component_binds_no_pim_adapter() {
    let host = PluginHost::new();
    let caps = vec![Capability::AccountBackend];
    let handle = host
        .load(GUEST, &manifest(caps.clone(), fast_limits()), &grant(caps))
        .unwrap();

    // The probe finds no PIM interfaces ⇒ nothing is advertised, every accessor None.
    assert!(
        !handle.advertises_pim(),
        "a world-plugin component exports no PIM interfaces"
    );
    assert!(handle.as_bridge_calendar().is_none());
    assert!(handle.as_bridge_tasks().is_none());
    assert!(handle.as_bridge_reactions().is_none());
    assert!(handle.as_bridge_voting().is_none());
    assert!(handle.as_bridge_recall().is_none());
    assert!(handle.as_bridge_focused_sync().is_none());

    // The account-backend path is byte-unchanged — still fully functional.
    let backend = handle
        .as_account_backend()
        .expect("account-backend granted");
    let c = backend.capabilities().await.unwrap();
    assert!(c.idle && c.r#move);
    assert_eq!(backend.list_mailboxes().await.unwrap().len(), 2);
}

// ── (2) a `plugin-pim` component round-trips reaction + recall + calendar delta ────

#[tokio::test]
async fn pim_component_round_trips_reaction_recall_calendar() {
    let host = PluginHost::new();
    let caps = vec![Capability::AccountBackend];
    let handle = host
        .load(PIM, &manifest(caps.clone(), fast_limits()), &grant(caps))
        .unwrap();

    assert!(handle.advertises_pim(), "PIM interfaces exported + granted");

    // Reaction round-trip THROUGH the engine trait object.
    let reactions = handle.as_bridge_reactions().expect("reactions bound");
    let msg = plugin_msg();
    reactions.set_reaction(&msg, "👍", true).await.unwrap();
    let rs = reactions.get_reactions(&msg).await.unwrap();
    assert_eq!(rs.len(), 1);
    assert_eq!(rs[0].actor, "alice@example.com");
    assert_eq!(rs[0].emoji, "👍");

    // Recall-outcome round-trip (honesty matrix): the guest returns `requested`.
    let recall = handle.as_bridge_recall().expect("recall bound");
    assert_eq!(recall.recall(&msg).await.unwrap(), RecallOutcome::Requested);

    // Calendar event-delta round-trip: one changed VEVENT + a removed id + a new
    // opaque cursor, all crossing the WIT boundary intact.
    let cal = handle.as_bridge_calendar().expect("calendar bound");
    let delta = cal.sync_events("cal-1", b"cursor-1").await.unwrap();
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].id, "evt-1");
    assert_eq!(delta.changed[0].calendar_id, "cal-1");
    assert!(delta.changed[0].ical.contains("VEVENT"));
    assert_eq!(delta.removed, vec!["evt-old".to_string()]);
    assert_eq!(delta.next_cursor, b"cursor-2".to_vec());
    assert_eq!(cal.list_calendars().await.unwrap()[0].id, "cal-1");
    assert_eq!(cal.find_rooms().await.unwrap()[0].capacity, Some(8));

    // Tasks are supported too.
    let tasks = handle.as_bridge_tasks().expect("tasks bound");
    assert_eq!(tasks.list_tasks().await.unwrap()[0].id, "task-1");

    // Honest `supports-*()`: reactions + recall + calendar + tasks true, voting +
    // focused false — the coarse interface presence does NOT overclaim.
    let parity = handle.bridge_parity_caps().await.unwrap();
    assert!(parity.reactions && parity.recall);
    assert!(!parity.voting && !parity.focused_sync);
    assert!(handle.bridge_supports_calendar().await.unwrap());
    assert!(handle.bridge_supports_tasks().await.unwrap());
}

// ── (3) an out-of-deadline PIM call trips LimitExceeded; the host SURVIVES ─────────

#[tokio::test]
async fn pim_out_of_deadline_trips_limit_exceeded_and_host_survives() {
    let host = PluginHost::new();
    let caps = vec![Capability::AccountBackend];
    let limits = PluginLimits {
        memory_mb: 64,
        deadline_ms: 100, // tight
        fuel: None,
    };
    let handle = host
        .load(PIM, &manifest(caps.clone(), limits), &grant(caps.clone()))
        .unwrap();

    // The guest busy-loops forever for the `"loop"` calendar id — the epoch deadline
    // must preempt it and the host must map the trap to a typed `LimitExceeded`.
    let err = handle.bridge_sync_events("loop", b"").await.unwrap_err();
    assert!(matches!(err, PluginError::LimitExceeded(_)), "got {err:?}");

    // Host survived: a fresh benign PIM call on a NEW handle still works.
    let h2 = host
        .load(PIM, &manifest(caps.clone(), fast_limits()), &grant(caps))
        .unwrap();
    assert_eq!(h2.bridge_list_calendars().await.unwrap().len(), 1);
}

// ── (4) a capability-denied PIM call returns CapabilityDenied (never a panic) ──────

#[tokio::test]
async fn pim_capability_denied_returns_capability_denied() {
    let host = PluginHost::new();
    // The PIM interfaces are EXPORTED, but `account-backend` is NOT granted.
    let caps = vec![Capability::DlpDetector];
    let handle = host
        .load(PIM, &manifest(caps.clone(), fast_limits()), &grant(caps))
        .unwrap();

    // No adapter is bound (deny-by-default) → the engine keeps its standards fallback.
    assert!(!handle.advertises_pim());
    assert!(handle.as_bridge_reactions().is_none());
    assert!(handle.as_bridge_calendar().is_none());

    // A direct host-level PIM call returns the typed CapabilityDenied — not a panic.
    let msg = plugin_msg();
    let err = handle.bridge_get_reactions(&msg).await.unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "got {err:?}"
    );
    let err = handle.bridge_recall(&msg).await.unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "got {err:?}"
    );
    let err = handle.bridge_sync_events("cal-1", b"").await.unwrap_err();
    assert!(
        matches!(err, PluginError::CapabilityDenied(_)),
        "got {err:?}"
    );
}
