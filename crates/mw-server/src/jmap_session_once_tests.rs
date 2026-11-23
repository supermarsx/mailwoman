//! t34-e11: `GET /jmap/session` on published inputs and ONE authentication per
//! request in `MW_GUARD_SNAPSHOT=bounded`; off mode unchanged. Actual router,
//! real Store and EpochMirror (the shared `Fixture` and `guard_snapshot_cases`).
use super::*;
use super::guard_snapshot_cases as snapcase;
use crate::feature_gate::auth_boundary::{PauseAt, count_auth, pause_at};
use crate::request_inputs::snapshot::Mode;

fn session_document(f: &Fixture) -> Request<Body> {
    let mut request = snapcase::me(f);
    *request.uri_mut() = "/jmap/session".parse().unwrap();
    request
}

struct Counted {
    sent: snapcase::Sent,
    auths: u64,
}
async fn counted(f: &Fixture, request: Request<Body>, mode: Mode) -> Counted {
    let (sent, auths) = count_auth(snapcase::send(f, request, mode)).await;
    Counted { sent, auths }
}
/// Bounded session documents until one ran entirely on certified DATA.
async fn reusing_document(f: &Fixture, mode: Mode) -> Counted {
    for _ in 0..40 {
        let reply = counted(f, session_document(f), mode).await;
        assert_eq!(reply.sent.status, StatusCode::OK, "{}", reply.sent.body);
        if reply.sent.snap.reused > 0 && reply.sent.snap.fresh == 0 { return reply; }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("bounded /jmap/session never reused a certified fence");
}

#[test]
fn jmap_session_bounded_authenticates_once_on_published_inputs_and_off_mode_is_unchanged() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        // Non-default Sieve policy, so the document proves which values resolved.
        policy(&f.writer, "sieve.maxScripts", json!(3)).await;
        policy(&f.writer, "cap.sieve.raw_edit", json!(true)).await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        // The first document of the process also warms route memos; compare warm.
        let cold = counted(&f, session_document(&f), snapcase::OFF).await;
        assert_eq!(cold.sent.status, StatusCode::OK, "{}", cold.sent.body);
        let off = counted(&f, session_document(&f), snapcase::OFF).await;
        assert_eq!(off.sent.status, StatusCode::OK, "{}", off.sent.body);
        // Today's path: the feature guard authenticates, then the handler again,
        // and the route is not a published request-input route.
        assert_eq!(off.auths, 2);
        let sieve = &off.sent.body["accounts"][&f.account]["accountCapabilities"][mw_engine::sieve_jmap::CAPABILITY];
        assert_eq!(sieve["maxNumberScripts"], 3, "{}", off.sent.body);
        assert_eq!(sieve["urn:mailwoman:policy"]["rawEdit"], true);
        assert_eq!(off.sent.inputs.first_fences + off.sent.inputs.refresh_fences, 0);
        assert!(off.sent.inputs.legacy_session_reads >= 2);
        snapcase::started(&f, snapcase::SHORT).await;
        let hit = reusing_document(&f, snapcase::SHORT).await;
        assert_eq!(hit.sent.body, off.sent.body, "same document from certified DATA");
        assert_eq!(hit.auths, 1, "one full authentication per request");
        assert_eq!(hit.sent.inputs.legacy_session_reads, 0);
        assert_eq!((hit.sent.inputs.first_fences, hit.sent.inputs.refresh_fences), (0, 0));
        // Regression ceilings measured at t34-e11 (this fixture, warm): off 60
        // reads; bounded 6 = 1 protection subject read (e5) + 5 handler DATA
        // reads (Sieve domain family 2, DSN policy 3); 0 guard/auth/feature.
        assert!(off.sent.sql.read.calls <= 60, "off={:?}", off.sent.sql);
        assert!(hit.sent.sql.read.calls <= 6, "bounded={:?}", hit.sent.sql);
        assert!(hit.sent.sql.read.calls < off.sent.sql.read.calls,
            "bounded={:?} off={:?}", hit.sent.sql, off.sent.sql);
        assert_eq!(hit.sent.sql.write.calls, off.sent.sql.write.calls);
        // Off after bounded in the same process: today's path and counts again.
        let again = counted(&f, session_document(&f), snapcase::OFF).await;
        assert_eq!(again.auths, off.auths);
        assert_eq!(again.sent.sql.read.calls, off.sent.sql.read.calls);
        assert_eq!(again.sent.snap.reused, 0);
        assert_eq!(again.sent.body, off.sent.body);
        eprintln!("{}", json!({"kind":"t34-e11-jmap-session-router-counts",
            "offReads":off.sent.sql.read.calls,"offWrites":off.sent.sql.write.calls,"offAuths":off.auths,
            "boundedReads":hit.sent.sql.read.calls,"boundedWrites":hit.sent.sql.write.calls,"boundedAuths":hit.auths,
            "realPeerGuardsIncluded":true,"workerQueriesIncluded":false}));
    });
}

#[test]
fn jmap_session_bounded_feature_policy_change_applies_within_the_bound_and_at_once_in_process() {
    run_fixture(|| async {
        // Cross-connection: a second Store on the same file writes the policy and
        // nothing expedites; within one bound (100 ms, no manual poll) it refuses.
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        snapcase::started(&f, snapcase::SHORT).await;
        reusing_document(&f, snapcase::SHORT).await;
        policy(&f.writer, "access.protocols", json!(["pop3"])).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let denied = counted(&f, session_document(&f), snapcase::SHORT).await;
        assert_eq!(denied.sent.status, StatusCode::FORBIDDEN, "{}", denied.sent.body);
        assert_eq!(denied.sent.body["code"], "feature_disabled");
        assert_eq!(denied.sent.body["feature"], "access.protocols");
        assert_eq!(denied.sent.snap.reused, 0);
        policy(&f.writer, "access.protocols", json!(["imap"])).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let allowed = counted(&f, session_document(&f), snapcase::SHORT).await;
        assert_eq!(allowed.sent.status, StatusCode::OK, "{}", allowed.sent.body);

        // In-process: the router's own Store writes and the write site expedites
        // (request_inputs::layer does this after every state-changing request).
        // A 5 s bound polls every 1.25 s, so only the expedite can refuse at once.
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let mirror = snapcase::started(&f, snapcase::LONG).await;
        reusing_document(&f, snapcase::LONG).await;
        policy(&f.measured, "access.protocols", json!(["pop3"])).await;
        mirror.expedite();
        let denied = counted(&f, session_document(&f), snapcase::LONG).await;
        assert_eq!(denied.sent.status, StatusCode::FORBIDDEN, "{}", denied.sent.body);
        assert_eq!(denied.sent.body["feature"], "access.protocols");
        assert_eq!(denied.sent.snap.reused, 0);
    });
}

#[test]
fn jmap_session_bounded_revocation_after_the_feature_decision_is_refused() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let mirror = snapcase::started(&f, snapcase::SHORT).await;
        reusing_document(&f, snapcase::SHORT).await;
        // Pause the deferred path after its feature decision, before the one full
        // authentication; revoke the session from another connection and let one
        // poll apply it, then resume.
        let pause = pause_at(PauseAt::AfterFeatures, &f.session);
        let request = counted(&f, session_document(&f), snapcase::SHORT);
        tokio::pin!(request);
        tokio::select! {
            _ = pause.entered.notified() => {},
            _ = &mut request => panic!("deferred path did not reach its feature decision pause"),
        }
        assert_eq!(snapcase::raw(&f, "DELETE FROM sessions WHERE id=?1", &f.session).await, 1);
        mirror.poll_now().await.unwrap();
        pause.release.notify_one();
        let denied = request.await;
        assert_ne!(denied.sent.status, StatusCode::OK, "{}", denied.sent.body);
        assert!(denied.sent.body.get("apiUrl").is_none(), "no session document after revocation");
    });
}

#[test]
fn jmap_session_bounded_keeps_the_password_change_gate() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let mirror = snapcase::started(&f, snapcase::SHORT).await;
        reusing_document(&f, snapcase::SHORT).await;
        crate::account_gate::update_flags(&f.writer, &f.account, |flags| flags.force_password_change = true)
            .await.unwrap();
        mirror.poll_now().await.unwrap();
        for mode in [snapcase::SHORT, snapcase::OFF] {
            let held = counted(&f, session_document(&f), mode).await;
            assert_eq!(held.sent.status, StatusCode::FORBIDDEN, "{mode:?}: {}", held.sent.body);
            assert_eq!(held.sent.body["passwordChangeRequired"], true, "{mode:?}");
        }
        // `/api/me` is the one place a held account still reaches.
        let me = snapcase::send(&f, snapcase::me(&f), snapcase::SHORT).await;
        assert_eq!(me.status, StatusCode::OK, "{}", me.body);
        assert_eq!(me.body["passwordChangeRequired"], true);
    });
}
