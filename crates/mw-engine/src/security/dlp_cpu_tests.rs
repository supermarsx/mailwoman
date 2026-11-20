//! Actual policy/owned-buffer behavior; the executor is not a parser sandbox.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn rule(action: &str) -> DlpRule {
    serde_json::from_value(json!({"id":"cpu-pan","name":"PAN","enabled":true,"priority":10,"conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],"attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],"recipientDomainMode":null,"classification":null},"action":action,"message":"blocked"})).unwrap()
}
async fn policy(store: &Store, action: &str) {
    store
        .set_setting(
            SECURITY_POLICY_SETTINGS_KEY,
            &json!({"dlp_rules_json":serde_json::to_string(&[rule(action)]).unwrap()}).to_string(),
        )
        .await
        .unwrap();
}
fn outgoing() -> mw_smtp::Outgoing {
    mw_smtp::Outgoing{mail_from:"owner@example.invalid".into(),rcpt_to:vec!["outside@example.invalid".into()],raw:b"From: owner@example.invalid\r\nTo: display@example.invalid\r\nSubject: Test\r\nContent-Type: text/plain\r\n\r\n4111 1111 1111 1111".to_vec()}
}
struct HookGuard(String);
impl Drop for HookGuard {
    fn drop(&mut self) {
        INSPECTION_HOOK.lock().unwrap().remove(&self.0);
    }
}
#[tokio::test]
async fn owned_outgoing_preserves_allocation_exact_bytes_recipients_and_detector_parity() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    policy(&store, "block").await;
    let value = outgoing();
    let pointer = value.raw.as_ptr() as usize;
    let bytes = value.raw.clone();
    let recipients = value.rcpt_to.clone();
    let oracle = scan(
        &vec![rule("block")],
        &input_from_raw(&bytes, Some(&recipients)).unwrap(),
    );
    let (WorkInput::Outgoing(value), verdicts) = inspect_owned(&store, WorkInput::Outgoing(value))
        .await
        .unwrap()
    else {
        panic!("owned outgoing changed type")
    };
    assert_eq!(
        value.raw.as_ptr() as usize,
        pointer,
        "production boundary must move, not clone raw buffers"
    );
    assert_eq!(value.raw, bytes);
    assert_eq!(value.rcpt_to, recipients);
    assert_eq!(verdicts, oracle);
    assert!(verdicts[0].blocked);
    store.close().await;
}
#[tokio::test]
async fn changed_late_policy_is_reinspected_before_an_allow_can_escape() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    policy(&store, "warn").await;
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let writer = store.clone();
    INSPECTION_HOOK.lock().unwrap().insert(
        store.instance_id().to_string(),
        Arc::new(move || {
            let writer = writer.clone();
            let count = count.clone();
            Box::pin(async move {
                if count.fetch_add(1, Ordering::SeqCst) == 0 {
                    policy(&writer, "block").await;
                }
            })
        }),
    );
    let _guard = HookGuard(store.instance_id().to_string());
    let (_, verdicts) = inspect_owned(&store, WorkInput::Outgoing(outgoing()))
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(verdicts.iter().any(|value| value.blocked));
    store.close().await;
}
#[tokio::test]
async fn continuously_changing_policy_and_malformed_policy_fail_closed() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    policy(&store, "warn").await;
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let writer = store.clone();
    INSPECTION_HOOK.lock().unwrap().insert(
        store.instance_id().to_string(),
        Arc::new(move || {
            let writer = writer.clone();
            let count = count.clone();
            Box::pin(async move {
                let n = count.fetch_add(1, Ordering::SeqCst);
                policy(&writer, if n % 2 == 0 { "block" } else { "warn" }).await;
            })
        }),
    );
    let guard = HookGuard(store.instance_id().to_string());
    assert!(matches!(
        inspect_owned(&store, WorkInput::Outgoing(outgoing())).await,
        Err(DlpError::PolicyUnavailable)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    drop(guard);
    store
        .set_setting(SECURITY_POLICY_SETTINGS_KEY, "not-json")
        .await
        .unwrap();
    assert!(matches!(
        inspect_owned(&store, WorkInput::Outgoing(outgoing())).await,
        Err(DlpError::PolicyUnavailable)
    ));
    store.close().await;
}
#[tokio::test]
async fn reserved_capacity_is_bounded_even_when_visible_content_is_empty() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    let mut input = DlpInput::default();
    input.body = String::with_capacity(MAX_INSPECT_BYTES + 2 * 1024 * 1024);
    assert!(matches!(
        inspect_owned(&store, WorkInput::Inline(input)).await,
        Err(DlpError::InputUnavailable)
    ));
    store.close().await;
}

#[tokio::test]
async fn absent_policy_cannot_escape_as_stale_empty_allow_after_cpu_queue_delay() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let writer = store.clone();
    INSPECTION_HOOK.lock().unwrap().insert(
        store.instance_id().to_string(),
        Arc::new(move || {
            let writer = writer.clone();
            let count = count.clone();
            Box::pin(async move {
                if count.fetch_add(1, Ordering::SeqCst) == 0 {
                    policy(&writer, "block").await;
                }
            })
        }),
    );
    let _guard = HookGuard(store.instance_id().to_string());
    let rules = load_rules(&store).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(
        rules
            .iter()
            .any(|rule| rule.enabled && rule.action == "block")
    );
    let (_, verdicts) = inspect_owned(&store, WorkInput::Outgoing(outgoing()))
        .await
        .unwrap();
    assert!(verdicts.iter().any(|verdict| verdict.blocked));
    store.close().await;
}

#[tokio::test]
async fn disabled_policy_preserves_overallocated_outgoing_but_active_policy_enforces_budget() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    let engine = Engine::new(store.clone());
    let mut original = outgoing();
    original
        .raw
        .reserve_exact(MAX_INSPECT_BYTES + 2 * 1024 * 1024);
    assert!(original.raw.capacity() > MAX_INSPECT_BYTES + 1024 * 1024);
    let pointer = original.raw.as_ptr() as usize;
    let capacity = original.raw.capacity();
    let recipients = original.rcpt_to.clone();
    let (returned, verdicts) = evaluate_outgoing_owned(&engine, "no-policy-account", original)
        .await
        .unwrap();
    assert!(verdicts.is_empty());
    assert_eq!(returned.raw.as_ptr() as usize, pointer);
    assert_eq!(returned.raw.capacity(), capacity);
    assert_eq!(returned.rcpt_to, recipients);
    // Reuse the spare allocation to exercise visible length above the former
    // inspection cap as well, without a second large buffer allocation.
    let mut returned = returned;
    returned.raw.resize(MAX_INSPECT_BYTES + 1, b'x');
    let (returned, verdicts) = evaluate_outgoing_owned(&engine, "no-policy-account", returned)
        .await
        .unwrap();
    assert!(verdicts.is_empty());
    assert_eq!(returned.raw.as_ptr() as usize, pointer);
    assert_eq!(returned.raw.len(), MAX_INSPECT_BYTES + 1);
    assert_eq!(returned.rcpt_to, recipients);
    policy(&store, "block").await;
    assert!(matches!(
        evaluate_outgoing_owned(&engine, "no-policy-account", returned).await,
        Err(DlpError::InputUnavailable)
    ));
    store.close().await;
}

#[tokio::test]
async fn stored_rule_view_preserves_duplicates_while_enforcement_keeps_first_id() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    store.set_setting(SECURITY_POLICY_SETTINGS_KEY,&json!({"dlp_rules_json":serde_json::to_string(&[rule("warn"),rule("block")]).unwrap()}).to_string()).await.unwrap();
    let stored = stored_rules(&store).await.unwrap();
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].action, "warn");
    assert_eq!(stored[1].action, "block");
    let enforced = load_rules(&store).await.unwrap();
    assert_eq!(enforced.len(), 1);
    assert_eq!(enforced[0].action, "warn");
    store.close().await;
}
