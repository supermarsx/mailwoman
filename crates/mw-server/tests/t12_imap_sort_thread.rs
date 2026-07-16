//! t12-e-e2e-backend — IMAP SORT + THREAD (RFC 5256) live-E2E (audit #8).
//!
//! Drives mw-imap's new `Session::uid_sort` / `Session::uid_thread` (session.rs)
//! against a REAL Dovecot over a KNOWN seeded mailbox, proving the command render +
//! reply parse (incl. the hand-rolled `* THREAD (…)` recursive-descent parser, which
//! `imap-proto` does not model) match a real server's orderings.
//!
//! The seeded INBOX (scripts/dovecot-sasl/seed) is, in delivery/UID order:
//!   UID 1  <t1-root>     "Project kickoff"      Date 2026-01-01
//!   UID 2  <t1-r1>       "Re: Project kickoff"  Date 2026-01-03  (refs t1-root)
//!   UID 3  <t2-lunch>    "Lunch tomorrow"       Date 2026-01-02  (standalone)
//!   UID 4  <t1-r2>       "Re: Project kickoff"  Date 2026-01-04  (refs t1-root t1-r1)
//!
//! So SORT by ARRIVAL differs from SORT by DATE, and THREAD=REFERENCES yields two
//! root threads: the kickoff chain {1,2,4} and the standalone {3}.
//!
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-sasl
//!   MW_IMAP_LIVE=1 cargo test -p mw-server --test t12_imap_sort_thread -- --nocapture

use mw_imap::session::{
    Credentials, SelectMode, Session, SortCriterion, SortKey, ThreadAlgorithm, ThreadNode,
};
use mw_imap::transport::TlsMode;

fn live() -> bool {
    std::env::var("MW_IMAP_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_IMAP_LIVE_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}
fn port() -> u16 {
    std::env::var("MW_IMAP_LIVE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3243)
}

/// Log in over SCRAM + SELECT INBOX. `None` after a loud skip banner.
async fn logged_in(scenario: &str) -> Option<Session> {
    let mut session = match Session::connect(&host(), port(), TlsMode::Plaintext).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "\n[t12 SORT/THREAD SKIP] {scenario}: Dovecot-SASL unreachable at {}:{} ({e}). \
                 up -d --wait dovecot-sasl ; MW_IMAP_LIVE=1.\n",
                host(),
                port()
            );
            return None;
        }
    };
    session
        .login(&Credentials::Password {
            username: "testuser".into(),
            password: "testpass".into(),
        })
        .await
        .expect("SCRAM login");
    session
        .select("INBOX", SelectMode::Plain)
        .await
        .expect("SELECT INBOX");
    Some(session)
}

/// Depth-first collect of every UID in a thread subtree.
fn flatten(nodes: &[ThreadNode], out: &mut Vec<u32>) {
    for n in nodes {
        out.push(n.id);
        flatten(&n.children, out);
    }
}

#[tokio::test]
async fn imap_uid_sort_live() {
    if !live() {
        eprintln!("\n[t12 SORT/THREAD SKIP] MW_IMAP_LIVE!=1 — see module doc.\n");
        return;
    }
    let Some(mut s) = logged_in("imap_uid_sort_live").await else {
        return;
    };
    if !s.backend_caps().sort {
        eprintln!("\n[t12 SORT SKIP] server does not advertise SORT.\n");
        return;
    }

    // ARRIVAL == delivery/UID order. (The four messages are delivered in the same
    // second, so their INTERNALDATE keys tie and Dovecot breaks the tie by UID —
    // hence ARRIVAL is the deterministic delivery order. REVERSE is asserted on
    // DATE below, which has DISTINCT keys and so genuinely reverses.)
    let by_arrival = s
        .uid_sort(&[SortCriterion::asc(SortKey::Arrival)], "ALL")
        .await
        .expect("UID SORT ARRIVAL");
    assert_eq!(
        by_arrival,
        vec![1, 2, 3, 4],
        "SORT ARRIVAL is delivery order"
    );

    // DATE (the Date: header / sent date) orders differently from arrival: the
    // headers are Jan1(uid1) Jan2(uid3) Jan3(uid2) Jan4(uid4).
    let by_date = s
        .uid_sort(&[SortCriterion::asc(SortKey::Date)], "ALL")
        .await
        .expect("UID SORT DATE");
    assert_eq!(
        by_date,
        vec![1, 3, 2, 4],
        "SORT DATE follows the Date: headers, not arrival"
    );

    // REVERSE DATE genuinely reverses (distinct keys, no ties).
    let rev_date = s
        .uid_sort(&[SortCriterion::desc(SortKey::Date)], "ALL")
        .await
        .expect("UID SORT REVERSE DATE");
    assert_eq!(
        rev_date,
        vec![4, 2, 3, 1],
        "SORT REVERSE DATE reverses the sent-date order"
    );
    let _ = s.logout().await;
}

#[tokio::test]
async fn imap_uid_thread_references_live() {
    if !live() {
        return;
    }
    let Some(mut s) = logged_in("imap_uid_thread_references_live").await else {
        return;
    };
    if !s.backend_caps().thread_references {
        eprintln!("\n[t12 THREAD SKIP] server does not advertise THREAD=REFERENCES.\n");
        return;
    }

    let roots = s
        .uid_thread(ThreadAlgorithm::References, "ALL")
        .await
        .expect("UID THREAD REFERENCES");

    // Two root threads: the kickoff chain and the standalone Lunch.
    assert_eq!(
        roots.len(),
        2,
        "two root threads (kickoff + standalone): {roots:?}"
    );

    let mut thread_sets: Vec<Vec<u32>> = roots
        .iter()
        .map(|r| {
            let mut v = Vec::new();
            flatten(std::slice::from_ref(r), &mut v);
            v.sort_unstable();
            v
        })
        .collect();
    thread_sets.sort();

    assert_eq!(
        thread_sets,
        vec![vec![1, 2, 4], vec![3]],
        "REFERENCES threads the kickoff chain {{1,2,4}} together and Lunch {{3}} alone: {roots:?}"
    );
    let _ = s.logout().await;
}
