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
