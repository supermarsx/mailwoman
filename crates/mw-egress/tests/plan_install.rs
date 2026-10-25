//! The process-wide plan (t32 e31b): a process that has not installed one refuses
//! every class request, and the counters record each result by class.
//!
//! One test, in its own binary, because it reads and writes process-global state.
//!
//! Run:
//!   cargo test -p mw-egress --test plan_install -- --test-threads=1

use mw_egress::plan::{self, EgressPlan, Request};
use mw_egress::{Refusal, TrafficClass};

#[tokio::test]
async fn before_a_plan_is_installed_every_class_request_is_refused() {
    assert!(
        plan::current().is_none(),
        "nothing has installed a plan yet"
    );
    let before = plan::counts();

    // Loopback is refused by the address gate too, so the two answers differ only
    // in WHICH check said no: the error text tells them apart.
    let url = reqwest::Url::parse("http://127.0.0.1:9/feed.ics").unwrap();
    let unloaded = plan::send(url.clone(), &Request::get(TrafficClass::IcsFeeds)).await;
    assert!(!unloaded.traversed_proxy);
    assert_eq!(
        unloaded.outcome.unwrap_err(),
        Refusal::BadRequest("egress configuration is not loaded"),
        "with no plan installed the request must be refused, not sent direct"
    );

    plan::install(EgressPlan::direct());
    assert!(plan::current().is_some());
    let loaded = plan::fetch_class(TrafficClass::IcsFeeds, None, url, "text/calendar").await;
    assert_eq!(
        loaded.outcome.unwrap_err(),
        Refusal::Blocked,
        "once a plan is installed the request reaches the address gate"
    );

    let after = plan::counts();
    let ics = |v: &[(TrafficClass, plan::ClassCounts)]| {
        v.iter()
            .find(|(c, _)| *c == TrafficClass::IcsFeeds)
            .map(|(_, n)| *n)
            .unwrap()
    };
    assert_eq!(ics(&after).refused, ics(&before).refused + 2);
    assert_eq!(ics(&after).allowed, ics(&before).allowed);
    for (c, n) in &after {
        if *c != TrafficClass::IcsFeeds {
            let b = before.iter().find(|(x, _)| x == c).unwrap().1;
            assert_eq!(*n, b, "{c} must be untouched");
        }
    }
}
