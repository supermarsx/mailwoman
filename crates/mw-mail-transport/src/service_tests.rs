use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
struct Memory(Mutex<HashMap<String, CachedSts>>);
impl StsCache for Memory {
    fn load<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<CachedSts>, PolicyError>> {
        Box::pin(async move { Ok(self.0.lock().unwrap().get(key).cloned()) })
    }
    fn store<'a>(
        &'a self,
        key: &'a str,
        value: &'a CachedSts,
    ) -> BoxFuture<'a, Result<(), PolicyError>> {
        Box::pin(async move {
            self.0.lock().unwrap().insert(key.into(), value.clone());
            Ok(())
        })
    }
}
struct Io {
    txt: Mutex<Result<Vec<String>, PolicyError>>,
    policy: Mutex<Result<StsPolicy, PolicyError>>,
    fetches: AtomicUsize,
    dane: Mutex<Result<Vec<TLSA>, PolicyError>>,
}
impl PolicyIo for Io {
    fn txt<'a>(
        &'a self,
        _: &'a PolicyContext,
        _: &'a str,
    ) -> BoxFuture<'a, Result<Vec<String>, PolicyError>> {
        Box::pin(async { self.txt.lock().unwrap().clone() })
    }
    fn fetch<'a>(
        &'a self,
        _: &'a PolicyContext,
        _: &'a str,
    ) -> BoxFuture<'a, Result<StsPolicy, PolicyError>> {
        Box::pin(async {
            self.fetches.fetch_add(1, Ordering::Relaxed);
            self.policy.lock().unwrap().clone()
        })
    }
    fn dane<'a>(
        &'a self,
        _: &'a PolicyContext,
        _: &'a str,
        _: u16,
    ) -> BoxFuture<'a, Result<crate::dns::DaneRecords, PolicyError>> {
        Box::pin(async {
            self.dane
                .lock()
                .unwrap()
                .clone()
                .map(|records| crate::dns::DaneRecords {
                    records,
                    base: Some("smtp.example.org".into()),
                })
        })
    }
}
fn fixture(label: &str) -> (TransportPolicy, Arc<Memory>, Arc<Io>) {
    let cache = Arc::new(Memory::default());
    let io = Arc::new(Io {
        txt: Mutex::new(Ok(vec!["v=STSv1; id=one".into()])),
        policy: Mutex::new(Ok(StsPolicy::parse(
            b"version: STSv1\nmode: enforce\nmx: smtp.example.org\nmax_age: 86400",
        )
        .unwrap())),
        fetches: AtomicUsize::new(0),
        dane: Mutex::new(Ok(vec![])),
    });
    let mut policy = TransportPolicy::new(
        TransportPolicyConfig {
            dane: DaneMode::Off,
            mta_sts: StsMode::Auto,
            policy_domain: Some("example.org".into()),
        },
        PolicyContext {
            doh_url: "https://dns.example/dns-query".into(),
            egress_domain: None,
            cache_namespace: label.into(),
        },
        cache.clone(),
    )
    .unwrap();
    policy.io = io.clone();
    (policy, cache, io)
}
#[tokio::test]
async fn known_sts_domain_refreshes_https_daily_despite_missing_or_failed_txt() {
    for (label, txt) in [
        ("stripped-txt", Ok(vec![])),
        ("failed-txt", Err(PolicyError::Dns)),
    ] {
        let (policy, cache, io) = fixture(label);
        *io.policy.lock().unwrap() = Ok(StsPolicy::parse(
            b"version: STSv1\nmode: enforce\nmx: smtp.example.org\nmax_age: 200000",
        )
        .unwrap());
        assert!(
            policy
                .prepare_inner("smtp.example.org", 587, 1000)
                .await
                .unwrap()
                .requires_tls()
        );
        *io.txt.lock().unwrap() = txt;
        assert!(
            policy
                .prepare_inner("smtp.example.org", 587, 87399)
                .await
                .unwrap()
                .requires_tls()
        );
        assert_eq!(
            io.fetches.load(Ordering::Relaxed),
            1,
            "no premature HTTPS refresh"
        );
        assert!(
            policy
                .prepare_inner("smtp.example.org", 587, 87400)
                .await
                .unwrap()
                .requires_tls()
        );
        assert_eq!(
            io.fetches.load(Ordering::Relaxed),
            2,
            "known domain refresh must not depend on attacker-controlled TXT availability"
        );
        let saved = cache.0.lock().unwrap().values().next().unwrap().clone();
        assert_eq!(saved.id, "one");
        assert_eq!(saved.fetched_at, 87400);
        assert_eq!(saved.expires_at, 287400);
    }
}
#[tokio::test]
async fn failed_daily_refresh_backs_off_and_never_extends_cached_lifetime() {
    let (policy, cache, io) = fixture("daily-refresh-failure");
    *io.policy.lock().unwrap() = Ok(StsPolicy::parse(
        b"version: STSv1\nmode: enforce\nmx: smtp.example.org\nmax_age: 200000",
    )
    .unwrap());
    policy
        .prepare_inner("smtp.example.org", 587, 1000)
        .await
        .unwrap();
    *io.txt.lock().unwrap() = Ok(vec![]);
    *io.policy.lock().unwrap() = Err(PolicyError::Sts);
    for now in [87400, 87401, 87699] {
        assert!(
            policy
                .prepare_inner("smtp.example.org", 587, now)
                .await
                .unwrap()
                .requires_tls()
        );
    }
    assert_eq!(io.fetches.load(Ordering::Relaxed), 2);
    assert!(
        policy
            .prepare_inner("smtp.example.org", 587, 87700)
            .await
            .unwrap()
            .requires_tls()
    );
    assert_eq!(io.fetches.load(Ordering::Relaxed), 3);
    let saved = cache.0.lock().unwrap().values().next().unwrap().clone();
    assert_eq!(saved.fetched_at, 1000);
    assert_eq!(saved.expires_at, 201000);
    assert!(
        !policy
            .prepare_inner("smtp.example.org", 587, 201000)
            .await
            .unwrap()
            .requires_tls(),
        "failed refresh cannot extend expired protection"
    );
}
#[tokio::test]
async fn unexpired_enforcement_survives_restart_txt_loss_fetch_failure_and_refuses_wrong_host() {
    let (policy, cache, io) = fixture("retention");
    assert!(
        policy
            .prepare_inner("smtp.example.org", 587, 1000)
            .await
            .unwrap()
            .requires_tls()
    );
    let mut restarted =
        TransportPolicy::new(policy.config.clone(), policy.context.clone(), cache.clone()).unwrap();
    restarted.io = io.clone();
    *io.txt.lock().unwrap() = Err(PolicyError::Dns);
    assert!(
        restarted
            .prepare_inner("smtp.example.org", 587, 2000)
            .await
            .unwrap()
            .requires_tls()
    );
    assert_eq!(
        restarted
            .prepare_inner("evil.example.org", 587, 2000)
            .await
            .unwrap_err(),
        PolicyError::MxMismatch
    );
    *io.txt.lock().unwrap() = Ok(vec!["v=STSv1; id=two".into()]);
    *io.policy.lock().unwrap() = Err(PolicyError::Sts);
    assert!(
        restarted
            .prepare_inner("smtp.example.org", 587, 2001)
            .await
            .unwrap()
            .requires_tls()
    );
    assert!(
        restarted
            .prepare_inner("smtp.example.org", 587, 2002)
            .await
            .unwrap()
            .requires_tls()
    );
    assert_eq!(
        io.fetches.load(Ordering::Relaxed),
        2,
        "failed version is backed off"
    );
    assert!(
        !restarted
            .prepare_inner("smtp.example.org", 587, 90000)
            .await
            .unwrap()
            .requires_tls(),
        "expired cache is not silently extended"
    );
    restarted.config.mta_sts = StsMode::Required;
    assert_eq!(
        restarted
            .prepare_inner("smtp.example.org", 587, 90001)
            .await
            .unwrap_err(),
        PolicyError::StsRequired
    );
}
#[tokio::test]
async fn only_fresh_valid_https_none_policy_removes_cached_enforcement() {
    let (policy, _, io) = fixture("removal");
    policy
        .prepare_inner("smtp.example.org", 25, 1000)
        .await
        .unwrap();
    *io.txt.lock().unwrap() = Ok(vec![]);
    assert!(
        policy
            .prepare_inner("smtp.example.org", 25, 1001)
            .await
            .unwrap()
            .requires_tls()
    );
    *io.txt.lock().unwrap() = Ok(vec!["v=STSv1; id=two".into()]);
    *io.policy.lock().unwrap() =
        Ok(StsPolicy::parse(b"version: STSv1\nmode: none\nmax_age: 1000").unwrap());
    assert!(
        !policy
            .prepare_inner("smtp.example.org", 25, 1002)
            .await
            .unwrap()
            .requires_tls()
    );
}
#[tokio::test]
async fn secure_dane_has_precedence_bogus_cannot_fall_back_and_unusable_still_requires_tls() {
    use hickory_net::proto::rr::rdata::tlsa::{CertUsage, Matching, Selector};
    let (mut policy, _, io) = fixture("dane");
    policy.config.dane = DaneMode::Opportunistic;
    policy.config.mta_sts = StsMode::Required;
    let usable = TLSA::new(
        CertUsage::DaneEe,
        Selector::Spki,
        Matching::Sha256,
        vec![1; 32],
    );
    *io.dane.lock().unwrap() = Ok(vec![usable]);
    let decision = policy
        .prepare_inner("smtp.example.org", 25, 1000)
        .await
        .unwrap();
    assert!(decision.has_dane());
    assert!(decision.requires_tls());
    assert_eq!(io.fetches.load(Ordering::Relaxed), 0);
    *io.dane.lock().unwrap() = Err(PolicyError::Dnssec);
    assert_eq!(
        policy
            .prepare_inner("smtp.example.org", 25, 1000)
            .await
            .unwrap_err(),
        PolicyError::Dnssec
    );
    policy.config.mta_sts = StsMode::Off;
    *io.dane.lock().unwrap() = Ok(vec![TLSA::new(
        CertUsage::PkixEe,
        Selector::Spki,
        Matching::Sha256,
        vec![1; 32],
    )]);
    let decision = policy
        .prepare_inner("smtp.example.org", 25, 1000)
        .await
        .unwrap();
    assert!(decision.requires_tls());
    assert!(!decision.has_dane());
    policy.config.dane = DaneMode::Required;
    assert_eq!(
        policy
            .prepare_inner("smtp.example.org", 25, 1000)
            .await
            .unwrap_err(),
        PolicyError::DaneRequired
    );
}
#[tokio::test]
async fn authority_namespaces_do_not_share_cache_and_corrupt_future_cache_fails_closed() {
    let (policy, cache, io) = fixture("isolation1");
    policy
        .prepare_inner("smtp.example.org", 25, 1000)
        .await
        .unwrap();
    let mut isolated = TransportPolicy::new(
        policy.config.clone(),
        PolicyContext {
            cache_namespace: "isolation2".into(),
            ..policy.context.clone()
        },
        cache.clone(),
    )
    .unwrap();
    isolated.io = io.clone();
    *io.txt.lock().unwrap() = Err(PolicyError::Dns);
    assert!(
        !isolated
            .prepare_inner("smtp.example.org", 25, 1001)
            .await
            .unwrap()
            .requires_tls()
    );
    cache
        .0
        .lock()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()
        .expires_at = u64::MAX;
    assert_eq!(
        policy
            .prepare_inner("smtp.example.org", 25, 1001)
            .await
            .unwrap_err(),
        PolicyError::Cache
    );
}

#[tokio::test]
async fn rejected_old_none_fetch_cannot_override_a_concurrent_enforcing_policy() {
    struct ConcurrentCache {
        winner: CachedSts,
        stored: std::sync::atomic::AtomicBool,
    }
    impl StsCache for ConcurrentCache {
        fn load<'a>(&'a self, _: &'a str) -> BoxFuture<'a, Result<Option<CachedSts>, PolicyError>> {
            Box::pin(async {
                Ok(self
                    .stored
                    .load(Ordering::Relaxed)
                    .then(|| self.winner.clone()))
            })
        }
        fn store<'a>(
            &'a self,
            _: &'a str,
            incoming: &'a CachedSts,
        ) -> BoxFuture<'a, Result<(), PolicyError>> {
            Box::pin(async move {
                assert_eq!(incoming.policy.mode, StsPolicyMode::None);
                self.stored.store(true, Ordering::Relaxed);
                Ok(())
            })
        }
    }
    for timestamp in [1000, 1001] {
        let (mut policy, _, io) = fixture(&format!("concurrent-{timestamp}"));
        *io.policy.lock().unwrap() =
            Ok(StsPolicy::parse(b"version: STSv1\nmode: none\nmax_age: 600").unwrap());
        policy.cache = Arc::new(ConcurrentCache {
            winner: CachedSts {
                id: "newenforce".into(),
                policy: StsPolicy::parse(
                    b"version: STSv1\nmode: enforce\nmx: smtp.example.org\nmax_age: 600",
                )
                .unwrap(),
                fetched_at: timestamp,
                expires_at: timestamp + 600,
            },
            stored: std::sync::atomic::AtomicBool::new(false),
        });
        match policy.prepare_inner("smtp.example.org", 25, 1000).await {
            Ok(decision) => assert!(
                decision.requires_tls(),
                "same-second concurrent winner must still enforce"
            ),
            Err(error) => assert_eq!(
                error,
                PolicyError::Cache,
                "later clock winner fails closed rather than applying old none"
            ),
        }
    }
}
#[tokio::test]
async fn real_dns_and_sts_requests_use_scoped_proxy_without_direct_fallback_and_refuse_private_dns()
{
    use mw_egress::{
        InternalAllowList,
        plan::{EgressPlan, PlanRoute, RouteScope},
        proxy::{ProxyRoute, ProxyScheme},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    mw_egress::plan::install(
        EgressPlan::new(
            vec![PlanRoute {
                scope: RouteScope::Domain("account.example".into()),
                classes: vec![TrafficClass::DnsAuth],
                route: ProxyRoute {
                    id: "policy-fixture".into(),
                    scheme: ProxyScheme::Socks5h,
                    host: addr.ip().to_string(),
                    port: addr.port(),
                    auth: None,
                    allow_plaintext: false,
                },
            }],
            InternalAllowList::default(),
        )
        .unwrap(),
    );
    let proxy = tokio::spawn(async move {
        let mut names = vec![];
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut greeting = [0; 3];
            stream.read_exact(&mut greeting).await.unwrap();
            assert_eq!(greeting, [5, 1, 0]);
            stream.write_all(&[5, 0]).await.unwrap();
            let mut head = [0; 5];
            stream.read_exact(&mut head).await.unwrap();
            assert_eq!(&head[..4], &[5, 1, 0, 3]);
            let mut host = vec![0; head[4] as usize];
            stream.read_exact(&mut host).await.unwrap();
            let mut port = [0; 2];
            stream.read_exact(&mut port).await.unwrap();
            assert_eq!(u16::from_be_bytes(port), 443);
            names.push(String::from_utf8(host).unwrap());
            stream
                .write_all(&[5, 2, 0, 1, 0, 0, 0, 0, 0, 0])
                .await
                .unwrap();
        }
        assert!(
            tokio::time::timeout(Duration::from_millis(100), listener.accept())
                .await
                .is_err(),
            "no retry or alternate request after proxy refusal"
        );
        names
    });
    let context = PolicyContext {
        doh_url: "https://dns.example.org/dns-query".into(),
        egress_domain: Some("account.example".into()),
        cache_namespace: "proxy-test".into(),
    };
    assert!(
        crate::dns::txt(&context, "_mta-sts.example.org")
            .await
            .is_err()
    );
    assert!(RoutedIo.fetch(&context, "example.org").await.is_err());
    let private = PolicyContext {
        doh_url: "https://127.0.0.1/dns-query".into(),
        ..context
    };
    assert!(
        crate::dns::txt(&private, "_mta-sts.example.org")
            .await
            .is_err()
    );
    assert_eq!(
        proxy.await.unwrap(),
        ["dns.example.org", "mta-sts.example.org"]
    );
}
