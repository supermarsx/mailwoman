//! Host jail test for the spam-rspamd component (t10-e6). Drives the REAL committed
//! `wasm32-wasip2` component (`tests/fixtures/spam-rspamd.wasm`, built from `src/` via
//! `build.sh`) through the real `mw-plugin` wasmtime host — the same jail e13/e14 load.
//!
//! Proves the component is a valid, jail-loadable spam-action component: it loads under
//! its granted capabilities, and it is deny-by-default (loading is fine; the host only
//! calls the spam hook under the `spam-action` grant, which e13 wires — the live
//! classify-through-jail + out-of-allowlist E2E is e14's docker gate).
//!
//! NOTE for e13: `mw_plugin::PluginHandle` has no `call_spam_action` yet (only
//! `call_dlp_detect`/`call_addrbook_search`/`call_message_out`). The mount executor
//! should add one mirroring `call_dlp_detect` (require `Capability::SpamAction`, call
//! `mailwoman_plugin_spam_action().call_classify`) so the engine's §10.8 spam pipeline
//! can drive this component through the jail.

use mw_plugin::{Capability, Grant, PluginHost, PluginLimits, PluginManifest};

const COMPONENT: &[u8] = include_bytes!("fixtures/spam-rspamd.wasm");

fn manifest(caps: Vec<Capability>, allowlist: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "spam-rspamd".into(),
        name: "Rspamd spam classifier".into(),
        version: "0".into(),
        signature: None,
        capabilities: caps,
        net_allowlist: allowlist.iter().map(|s| (*s).to_string()).collect(),
        limits: PluginLimits {
            memory_mb: 32,
            deadline_ms: 10_000,
            fuel: None,
        },
    }
}

fn grant(caps: Vec<Capability>) -> Grant {
    Grant {
        plugin_id: "spam-rspamd".into(),
        capabilities: caps,
        granted_by: "admin@test".into(),
        allow_unsigned: true, // the committed fixture is unsigned
    }
}

#[test]
fn loads_capability_gated_in_the_jail() {
    let host = PluginHost::try_new(Default::default(), mw_plugin::TrustRoot::empty()).unwrap();
    let caps = vec![
        Capability::SpamAction,
        Capability::Net,
        Capability::StoreKvScoped,
    ];
    let m = manifest(caps.clone(), &["rspamd"]);
    let handle = host
        .load(COMPONENT, &m, &grant(caps))
        .expect("the real spam-rspamd component loads in the jail");

    // The spam-action capability is reflected on the loaded handle (deny-by-default:
    // only granted caps appear).
    assert!(handle.granted().contains(&Capability::SpamAction));
    assert!(!handle.granted().contains(&Capability::AccountBackend));
}

#[test]
fn loads_with_only_spam_action_and_net() {
    // The minimal viable grant (no KV config override) still loads.
    let host = PluginHost::try_new(Default::default(), mw_plugin::TrustRoot::empty()).unwrap();
    let caps = vec![Capability::SpamAction, Capability::Net];
    let m = manifest(caps.clone(), &["rspamd"]);
    host.load(COMPONENT, &m, &grant(caps))
        .expect("loads with the minimal spam-action + net grant");
}
