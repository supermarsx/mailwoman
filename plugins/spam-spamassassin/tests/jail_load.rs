//! Host jail test for the spam-spamassassin component (t10-e6). Drives the REAL committed
//! `wasm32-wasip2` component (`tests/fixtures/spam-spamassassin.wasm`, built from `src/`
//! via `build.sh`) through the real `mw-plugin` wasmtime host — the same jail e13/e14
//! load.
//!
//! Proves the component is a valid, jail-loadable spam-action component under its granted
//! capabilities. The live classify-through-jail + out-of-allowlist E2E is e14's docker
//! gate (`spamd` service). See `spam-rspamd/tests/jail_load.rs` for the e13 note on
//! adding `PluginHandle::call_spam_action`.

use mw_plugin::{Capability, Grant, PluginHost, PluginLimits, PluginManifest};

const COMPONENT: &[u8] = include_bytes!("fixtures/spam-spamassassin.wasm");

fn manifest(caps: Vec<Capability>, allowlist: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "spam-spamassassin".into(),
        name: "SpamAssassin spam classifier".into(),
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
        plugin_id: "spam-spamassassin".into(),
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
    let m = manifest(caps.clone(), &["spamassassin"]);
    let handle = host
        .load(COMPONENT, &m, &grant(caps))
        .expect("the real spam-spamassassin component loads in the jail");

    assert!(handle.granted().contains(&Capability::SpamAction));
    assert!(!handle.granted().contains(&Capability::AccountBackend));
}

#[test]
fn loads_with_only_spam_action_and_net() {
    let host = PluginHost::try_new(Default::default(), mw_plugin::TrustRoot::empty()).unwrap();
    let caps = vec![Capability::SpamAction, Capability::Net];
    let m = manifest(caps.clone(), &["spamassassin"]);
    host.load(COMPONENT, &m, &grant(caps))
        .expect("loads with the minimal spam-action + net grant");
}
