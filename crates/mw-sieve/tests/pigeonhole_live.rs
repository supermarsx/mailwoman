//! The ManageSieve client and the Sieve validator against a real Dovecot
//! Pigeonhole (t32 f18). Skipped unless the servers are named:
//!
//! - `MW_SIEVE_LIVE=host:port` — the stock dev/CI config
//!   (`scripts/dovecot-pigeonhole/zz-pigeonhole.conf`);
//! - `MW_SIEVE_LIVE_VARIANT=host:port` — the same plus
//!   `scripts/dovecot-pigeonhole/zz-variant.conf` (2-script quota, 2 KiB size
//!   limit, `vacation` switched off).
//!
//! Both use the dev account testuser/testpass (DEV/CI ONLY). Run with
//! `--test-threads=1`: the tests share the account's script store.

use mw_sieve::managesieve::Checked;
use mw_sieve::policy::{EditKind, Policy, ViolationKind, enforce};
use mw_sieve::{
    Action, Condition, Credentials, ManageSieveClient, MatchOp, Rule, SieveError, StringTest,
    TlsMode, check, generate, managed,
};

fn target(var: &str) -> Option<(String, u16)> {
    let v = std::env::var(var).ok()?;
    let (host, port) = v.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

async fn login(host: &str, port: u16) -> ManageSieveClient {
    let creds = Credentials::Plain {
        username: "testuser".into(),
        password: "testpass".into(),
    };
    ManageSieveClient::connect(host, port, TlsMode::Plaintext, creds)
        .await
        .expect("connect + authenticate")
}

/// Leave the account with no scripts.
async fn clean(c: &mut ManageSieveClient) {
    c.set_active("").await.expect("deactivate");
    for s in c.list_scripts().await.expect("list") {
        c.delete_script(&s.name).await.expect("delete");
    }
}

fn st(op: MatchOp, v: &str) -> StringTest {
    StringTest {
        op,
        value: v.into(),
    }
}

fn sample_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "news".into(),
            name: "News".into(),
            match_all: true,
            conditions: vec![Condition::From(st(MatchOp::Contains, "news@example.com"))],
            actions: vec![
                Action::Move {
                    mailbox: "INBOX/News".into(),
                },
                Action::Stop,
            ],
            enabled: true,
        },
        Rule {
            id: "big".into(),
            name: "Big \"mail\"".into(),
            match_all: false,
            conditions: vec![
                Condition::HasAttachment,
                Condition::Size {
                    over: true,
                    bytes: 5_000_000,
                },
                Condition::Body(st(MatchOp::Matches, "*invoice*")),
                Condition::Keyword(st(MatchOp::Is, "$label1")),
            ],
            actions: vec![
                Action::Copy {
                    mailbox: "Archive".into(),
                },
                Action::Tag {
                    keyword: "\\Flagged".into(),
                },
                Action::Forward {
                    address: "me@example.com".into(),
                },
            ],
            enabled: true,
        },
    ]
}

fn refusal_code(e: SieveError) -> String {
    match e {
        SieveError::Refused(r) => r.code.map(|c| c.name).unwrap_or_default(),
        other => panic!("expected a refusal, got {other}"),
    }
}

#[tokio::test]
async fn live_script_lifecycle() {
    let Some((host, port)) = target("MW_SIEVE_LIVE") else {
        eprintln!("MW_SIEVE_LIVE not set; skipping");
        return;
    };
    let mut c = login(&host, port).await;
    clean(&mut c).await;
    assert!(c.capabilities().is_rfc5804());

    let body = generate(&sample_rules()).unwrap();
    check(&body).expect("our validator accepts the generated script");

    c.have_space("mw-live", body.len() as u64).await.expect("havespace");
    assert_eq!(
        c.check_script(&body).await.unwrap(),
        Checked::Valid { warnings: None }
    );
    match c.check_script("keep;\r\nfrob;\r\n").await.unwrap() {
        Checked::Invalid { problems, text } => {
            assert_eq!(problems[0].line, Some(2), "{text}");
            assert!(text.contains("frob"), "{text}");
        }
        other => panic!("expected Invalid, got {other:?}"),
    }

    assert_eq!(c.put_script("mw-live", &body).await.unwrap(), None);
    let list = c.list_scripts().await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(!list[0].active);
    c.set_active("mw-live").await.unwrap();
    assert!(c.list_scripts().await.unwrap()[0].active);
    assert_eq!(c.get_script("mw-live").await.unwrap(), body);

    c.rename_script("mw-live", "mw live \"2\"").await.unwrap();
    let list = c.list_scripts().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "mw live \"2\"", "quoted name round-trips");
    assert!(list[0].active, "rename keeps the active flag");

    let code = refusal_code(c.delete_script("mw live \"2\"").await.unwrap_err());
    assert_eq!(code, "ACTIVE", "an active script cannot be deleted");
    let code = refusal_code(c.get_script("missing").await.unwrap_err());
    assert_eq!(code, "NONEXISTENT");

    clean(&mut c).await;
    assert!(c.list_scripts().await.unwrap().is_empty());
    c.logout().await.unwrap();
}

/// What the builder writes, merged into hand-written text, compiles on
/// Pigeonhole; scripts our validator refuses, Pigeonhole refuses too.
#[tokio::test]
async fn live_validator_agrees_with_pigeonhole() {
    let Some((host, port)) = target("MW_SIEVE_LIVE") else {
        eprintln!("MW_SIEVE_LIVE not set; skipping");
        return;
    };
    let mut c = login(&host, port).await;

    let hand = "# mine\r\nrequire \"vacation\";\r\nif header :is \"x-a\" \"1\" {\r\n  vacation :days 3 text:\r\nAway.\r\n.\r\n;\r\n}\r\n";
    let merged = managed::merge(hand, &sample_rules()).unwrap();
    check(&merged).expect("validator");
    assert_eq!(
        c.check_script(&merged).await.unwrap(),
        Checked::Valid { warnings: None },
        "{merged}"
    );

    let valid = [
        "require [\"variables\", \"fileinto\"];\r\nif header :matches \"List-ID\" \"*<*@*\" { fileinto \"lists.${2}\"; }\r\n",
        "require [\"date\", \"relational\"];\r\nif currentdate :value \"ge\" \"hour\" \"09\" { keep; }\r\n",
        "require [\"duplicate\", \"fileinto\", \"mailbox\"];\r\nif duplicate :seconds 60 { fileinto :create \"Dup\"; }\r\n",
        "require [\"enotify\"];\r\nnotify :message \"hi\" \"mailto:x@example.com\";\r\n",
        "require \"ihave\";\r\nif ihave \"fileinto\" { keep; } else { error \"no\"; }\r\n",
    ];
    for s in valid {
        check(s).unwrap_or_else(|e| panic!("validator refused {s:?}: {e}"));
        assert_eq!(
            c.check_script(s).await.unwrap(),
            Checked::Valid { warnings: None },
            "{s}"
        );
    }
    let invalid = [
        "fileinto \"x\";\r\n",
        "keep;\r\nrequire \"fileinto\";\r\n",
        "elsif true { keep; }\r\n",
        "if true keep;\r\n",
        "require \"fileinto\";\r\nfileinto :copy \"x\";\r\n",
        "keep\r\n",
        "frob;\r\n",
    ];
    for s in invalid {
        assert!(check(s).is_err(), "validator accepted {s:?}");
        assert!(
            matches!(c.check_script(s).await.unwrap(), Checked::Invalid { .. }),
            "Pigeonhole accepted {s:?}"
        );
    }
    c.logout().await.unwrap();
}

/// Quota refusals surface with their codes, and an extension the server does
/// not advertise is refused by the policy before anything is uploaded.
#[tokio::test]
async fn live_variant_quota_and_unadvertised_extension() {
    let Some((host, port)) = target("MW_SIEVE_LIVE_VARIANT") else {
        eprintln!("MW_SIEVE_LIVE_VARIANT not set; skipping");
        return;
    };
    let mut c = login(&host, port).await;
    clean(&mut c).await;

    assert!(!c.capabilities().supports("vacation"), "variant hides vacation");
    let advertised = c
        .capabilities()
        .sieve
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let policy = Policy {
        vacation_allowed: true,
        raw_edit: true,
        advertised: Some(advertised),
        ..Policy::default()
    };
    let vacation = "require \"vacation\";\r\nvacation \"away\";\r\n";
    let err = enforce(vacation, &policy, EditKind::Raw).unwrap_err();
    assert_eq!(err[0].kind, ViolationKind::ExtensionNotAdvertised);
    assert!(matches!(
        c.check_script(vacation).await.unwrap(),
        Checked::Invalid { .. }
    ));

    let code = refusal_code(c.have_space("big", 5000).await.unwrap_err());
    assert!(code.starts_with("QUOTA"), "{code}");
    c.put_script("one", "keep;\r\n").await.unwrap();
    c.put_script("two", "keep;\r\n").await.unwrap();
    let code = refusal_code(c.put_script("three", "keep;\r\n").await.unwrap_err());
    assert_eq!(code, "QUOTA/MAXSCRIPTS");
    // Replacing an existing script is not a new one.
    c.put_script("two", "stop;\r\n").await.unwrap();

    clean(&mut c).await;
    c.logout().await.unwrap();
}
