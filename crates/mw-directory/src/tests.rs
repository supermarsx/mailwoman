//! Unit tests for `mw-directory`. Everything here runs against an **in-memory
//! mock LDAP backend** (no live server): the mock parses the LDAP filter strings
//! this crate emits and matches them against canned entries, so search / paging /
//! priority-merge / group-expand / cert / photo / bind / attr-mapping and the GAL
//! cache are all exercised for real. StartTLS/LDAPS *negotiation* and the raw
//! `ldap3` wire path are deferred to the live OpenLDAP run in e16; the TLS *config
//! mapping* (`tls_plan`) and filter escaping are unit-tested here.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use mw_cache::Cache;

use super::*;
use crate::ldap_backend::{LdapBackend, RawEntry, SearchReq, tls_plan};

// ── A minimal LDAP filter evaluator so the mock matches real emitted filters ──

enum F {
    And(Vec<F>),
    Or(Vec<F>),
    Not(Box<F>),
    Eq(String, String),
    Present(String),
}

struct Parser {
    s: Vec<char>,
    i: usize,
}

impl Parser {
    fn parse(s: &str) -> F {
        let mut p = Parser {
            s: s.chars().collect(),
            i: 0,
        };
        p.filter()
    }
    fn filter(&mut self) -> F {
        assert_eq!(self.s[self.i], '(', "filter must start with '('");
        self.i += 1;
        let f = match self.s[self.i] {
            '&' => {
                self.i += 1;
                F::And(self.list())
            }
            '|' => {
                self.i += 1;
                F::Or(self.list())
            }
            '!' => {
                self.i += 1;
                F::Not(Box::new(self.filter()))
            }
            _ => self.item(),
        };
        assert_eq!(self.s[self.i], ')', "filter must end with ')'");
        self.i += 1;
        f
    }
    fn list(&mut self) -> Vec<F> {
        let mut v = Vec::new();
        while self.s[self.i] == '(' {
            v.push(self.filter());
        }
        v
    }
    fn item(&mut self) -> F {
        let mut attr = String::new();
        while self.s[self.i] != '=' {
            attr.push(self.s[self.i]);
            self.i += 1;
        }
        self.i += 1; // consume '='
        let mut val = String::new();
        while self.s[self.i] != ')' {
            let c = self.s[self.i];
            if c == '\\' {
                let h: String = [self.s[self.i + 1], self.s[self.i + 2]].iter().collect();
                let byte = u8::from_str_radix(&h, 16).expect("hex escape");
                val.push(byte as char);
                self.i += 3;
            } else {
                val.push(c);
                self.i += 1;
            }
        }
        if val == "*" {
            F::Present(attr)
        } else {
            F::Eq(attr, val)
        }
    }
}

fn wildcard(pattern: &str, value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    if !pattern.contains('*') {
        return value == pattern;
    }
    let segs: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;
    for (idx, seg) in segs.iter().enumerate() {
        if seg.is_empty() {
            continue;
        }
        if idx == 0 {
            if !value[pos..].starts_with(seg) {
                return false;
            }
            pos += seg.len();
        } else if idx == segs.len() - 1 {
            if !value.ends_with(seg) || value.len() < pos + seg.len() {
                return false;
            }
        } else if let Some(found) = value[pos..].find(seg) {
            pos += found + seg.len();
        } else {
            return false;
        }
    }
    true
}

// ── The mock backend ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockEntry {
    dn: String,
    attrs: Vec<(String, Vec<String>)>,
    bin: Vec<(String, Vec<Vec<u8>>)>,
    password: Option<String>,
}

impl MockEntry {
    fn get(&self, attr: &str) -> Option<&Vec<String>> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(attr))
            .map(|(_, v)| v)
    }
    fn eval(&self, f: &F) -> bool {
        match f {
            F::And(v) => v.iter().all(|x| self.eval(x)),
            F::Or(v) => v.iter().any(|x| self.eval(x)),
            F::Not(x) => !self.eval(x),
            F::Present(a) => self.get(a).map(|v| !v.is_empty()).unwrap_or(false),
            F::Eq(a, pat) => self
                .get(a)
                .map(|vals| vals.iter().any(|val| wildcard(pat, val)))
                .unwrap_or(false),
        }
    }
    fn to_raw(&self) -> RawEntry {
        let mut attrs: HashMap<String, Vec<String>> = HashMap::new();
        for (k, v) in &self.attrs {
            attrs.insert(k.clone(), v.clone());
        }
        let mut bin_attrs: HashMap<String, Vec<Vec<u8>>> = HashMap::new();
        for (k, v) in &self.bin {
            bin_attrs.insert(k.clone(), v.clone());
        }
        RawEntry {
            dn: self.dn.clone(),
            attrs,
            bin_attrs,
        }
    }
}

struct MockBackend {
    entries: Vec<MockEntry>,
    searches: AtomicUsize,
}

impl MockBackend {
    fn new(entries: Vec<MockEntry>) -> Arc<Self> {
        Arc::new(Self {
            entries,
            searches: AtomicUsize::new(0),
        })
    }
    fn search_count(&self) -> usize {
        self.searches.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl LdapBackend for MockBackend {
    async fn search(&self, req: SearchReq<'_>) -> Result<Vec<RawEntry>> {
        self.searches.fetch_add(1, Ordering::SeqCst);
        let base = req.base.to_ascii_lowercase();
        let mut out = Vec::new();
        if !req.subtree {
            // Base-object scope: match the exact DN, ignore the filter.
            for e in &self.entries {
                if e.dn.eq_ignore_ascii_case(req.base) {
                    out.push(e.to_raw());
                    break;
                }
            }
            return Ok(out);
        }
        let filter = Parser::parse(req.filter);
        for e in &self.entries {
            if out.len() >= req.limit {
                break;
            }
            if !e.dn.to_ascii_lowercase().ends_with(&base) {
                continue; // outside the subtree
            }
            if e.eval(&filter) {
                out.push(e.to_raw());
            }
        }
        Ok(out)
    }

    async fn bind(&self, dn: &str, password: &str) -> Result<bool> {
        for e in &self.entries {
            if e.dn.eq_ignore_ascii_case(dn) {
                return Ok(e.password.as_deref() == Some(password));
            }
        }
        Ok(false)
    }
}

// ── Fixture builders ─────────────────────────────────────────────────────────

fn a(k: &str, v: &str) -> (String, Vec<String>) {
    (k.to_string(), vec![v.to_string()])
}

fn person(cn: &str, mail: &str, display: &str) -> MockEntry {
    MockEntry {
        dn: format!("cn={cn},dc=example,dc=com"),
        attrs: vec![
            a("cn", cn),
            a("mail", mail),
            a("displayName", display),
            a("objectClass", "inetOrgPerson"),
        ],
        bin: vec![],
        password: None,
    }
}

fn ep(url: &str, priority: i32) -> LdapEndpoint {
    LdapEndpoint {
        url: url.to_string(),
        base_dn: "dc=example,dc=com".to_string(),
        bind_dn: None,
        tls: LdapTls::None,
        priority,
        attr_map: AttrMap::default(),
    }
}

fn mk(
    parts: Vec<(LdapEndpoint, Arc<MockBackend>)>,
    cache: Option<Cache>,
    options: DirectoryOptions,
) -> Directory {
    // Mirror `Directory::with_options`: endpoints are sorted by ascending priority
    // (lower number queried first) so the priority-merge dedup is faithful.
    let mut parts = parts;
    parts.sort_by_key(|(e, _)| e.priority);
    let config = DirectoryConfig {
        endpoints: parts.iter().map(|(e, _)| e.clone()).collect(),
    };
    let runtimes = parts
        .into_iter()
        .map(|(endpoint, backend)| EndpointRuntime {
            endpoint,
            backend: backend as Arc<dyn LdapBackend>,
            service_bind_pw: None,
        })
        .collect();
    Directory {
        config,
        options,
        cache,
        runtimes,
    }
}

// ── Frozen-shape / pure-helper tests ─────────────────────────────────────────

#[test]
fn config_round_trips() {
    let cfg = DirectoryConfig {
        endpoints: vec![LdapEndpoint {
            url: "ldaps://dc.example.com".into(),
            base_dn: "dc=example,dc=com".into(),
            bind_dn: Some("cn=svc,dc=example,dc=com".into()),
            tls: LdapTls::Ldaps,
            priority: 0,
            attr_map: AttrMap::default(),
        }],
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: DirectoryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn unconfigured_directory_errors() {
    let d = Directory::new(DirectoryConfig::default());
    // A directory with no endpoints reports NotConfigured, not an empty result.
    let out = futures_block(d.search_gal("smith", 0));
    assert!(matches!(out, Err(DirectoryError::NotConfigured)));
}

#[test]
fn tls_plan_maps_negotiation() {
    // StartTLS on a plain port ⇒ negotiate StartTLS; LDAPS ⇒ implicit TLS.
    let start = tls_plan(&LdapEndpoint {
        tls: LdapTls::StartTls,
        ..ep("ldap://dc:389", 0)
    });
    assert!(start.starttls && !start.ldaps);
    let ldaps = tls_plan(&ep("ldaps://dc:636", 0));
    // scheme is ldaps:// but tls=None → plan still recognises implicit TLS scheme.
    assert!(ldaps.ldaps);
    let explicit = tls_plan(&LdapEndpoint {
        tls: LdapTls::Ldaps,
        ..ep("ldap://dc:636", 0)
    });
    assert!(explicit.ldaps && !explicit.starttls);
    let none = tls_plan(&ep("ldap://dc:389", 0));
    assert!(!none.starttls && !none.ldaps);
}

#[test]
fn filter_escaping_neutralises_injection() {
    // A query trying to break out of the filter is neutralised.
    let esc = crate::ldap_backend::escape_filter_value("a)(uid=*");
    assert_eq!(esc, "a\\29\\28uid=\\2a");
    assert!(!esc.contains('*') && !esc.contains(')') && !esc.contains('('));
}

// ── Search / paging / merge ──────────────────────────────────────────────────

#[tokio::test]
async fn search_matches_across_recipient_fields() {
    let backend = MockBackend::new(vec![
        person("Alice Smith", "alice@example.com", "Alice Smith"),
        person("Bob Jones", "bob@example.com", "Bob Jones"),
    ]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );
    // Matches on cn substring.
    let r = d.search_gal("smith", 0).await.unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].mail, "alice@example.com");
    // Matches on mail substring too.
    let r = d.search_gal("bob@", 0).await.unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].display_name, "Bob Jones");
}

#[tokio::test]
async fn paging_slices_pages() {
    let people: Vec<MockEntry> = (0..5)
        .map(|i| {
            person(
                &format!("User {i}"),
                &format!("u{i}@example.com"),
                &format!("User {i}"),
            )
        })
        .collect();
    let backend = MockBackend::new(people);
    let opts = DirectoryOptions {
        page_size: 2,
        ..DirectoryOptions::default()
    };
    let d = mk(vec![(ep("ldap://x", 0), backend)], None, opts);
    assert_eq!(d.search_gal("user", 0).await.unwrap().len(), 2);
    assert_eq!(d.search_gal("user", 1).await.unwrap().len(), 2);
    assert_eq!(d.search_gal("user", 2).await.unwrap().len(), 1);
    assert_eq!(d.search_gal("user", 3).await.unwrap().len(), 0);
    // Pages are disjoint.
    let p0 = d.search_gal("user", 0).await.unwrap();
    let p1 = d.search_gal("user", 1).await.unwrap();
    for e in &p0 {
        assert!(!p1.iter().any(|x| x.mail == e.mail));
    }
}

#[tokio::test]
async fn priority_merge_dedupes_lower_number_wins() {
    // Two directories both hold alice; endpoint priority 0 must win the merge.
    let mut alice_primary = person("Alice", "alice@example.com", "Alice PRIMARY");
    alice_primary.dn = "cn=alice,ou=primary,dc=example,dc=com".into();
    let mut alice_secondary = person("Alice", "alice@example.com", "Alice SECONDARY");
    alice_secondary.dn = "cn=alice,ou=secondary,dc=example,dc=com".into();

    let primary = MockBackend::new(vec![alice_primary]);
    let secondary = MockBackend::new(vec![alice_secondary]);
    // Deliberately pass them out of priority order to prove sorting.
    let d = mk(
        vec![
            (ep("ldap://sec", 10), secondary),
            (ep("ldap://pri", 0), primary),
        ],
        None,
        DirectoryOptions::default(),
    );
    let r = d.search_gal("alice", 0).await.unwrap();
    assert_eq!(r.len(), 1, "duplicate mail merged to one entry");
    assert_eq!(r[0].display_name, "Alice PRIMARY", "priority 0 wins");
}

// ── Group expand-before-send ─────────────────────────────────────────────────

#[tokio::test]
async fn expand_group_flattens_nested() {
    let alice = person("alice", "alice@example.com", "Alice");
    let bob = person("bob", "bob@example.com", "Bob");
    let carol = person("carol", "carol@example.com", "Carol");
    let nested = MockEntry {
        dn: "cn=nested,dc=example,dc=com".into(),
        attrs: vec![
            a("cn", "nested"),
            a("objectClass", "groupOfNames"),
            ("member".into(), vec![carol.dn.clone()]),
        ],
        bin: vec![],
        password: None,
    };
    let team = MockEntry {
        dn: "cn=team,dc=example,dc=com".into(),
        attrs: vec![
            a("cn", "team"),
            a("mail", "team@example.com"),
            a("objectClass", "groupOfNames"),
            (
                "member".into(),
                vec![alice.dn.clone(), bob.dn.clone(), nested.dn.clone()],
            ),
        ],
        bin: vec![],
        password: None,
    };
    let backend = MockBackend::new(vec![alice, bob, carol, nested, team]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );

    let mut members = d.expand_group("cn=team,dc=example,dc=com").await.unwrap();
    members.sort_by(|x, y| x.mail.cmp(&y.mail));
    let mails: Vec<&str> = members.iter().map(|m| m.mail.as_str()).collect();
    // Nested group flattened to its leaf; groups themselves excluded from leaves.
    assert_eq!(
        mails,
        vec!["alice@example.com", "bob@example.com", "carol@example.com"]
    );
    assert!(members.iter().all(|m| !m.is_group));
}

// ── Cert / photo lookup ──────────────────────────────────────────────────────

#[tokio::test]
async fn lookup_cert_returns_der_bytes() {
    let mut alice = person("alice", "alice@example.com", "Alice");
    alice.bin = vec![(
        "userCertificate;binary".into(),
        vec![vec![0x30, 0x82, 0x01, 0x02], vec![0x30, 0x82, 0x03, 0x04]],
    )];
    let backend = MockBackend::new(vec![alice]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );
    let certs = d.lookup_cert("alice@example.com").await.unwrap();
    assert_eq!(certs.len(), 2);
    assert_eq!(certs[0], vec![0x30, 0x82, 0x01, 0x02]);
    // Unknown recipient → empty (not an error).
    assert!(
        d.lookup_cert("nobody@example.com")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn lookup_photo_returns_first_blob() {
    let mut alice = person("alice", "alice@example.com", "Alice");
    alice.bin = vec![("jpegPhoto".into(), vec![vec![0xFF, 0xD8, 0xFF]])];
    let backend = MockBackend::new(vec![alice]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );
    let photo = d.lookup_photo("alice@example.com").await.unwrap();
    assert_eq!(photo, Some(vec![0xFF, 0xD8, 0xFF]));
    assert_eq!(d.lookup_photo("nobody@example.com").await.unwrap(), None);
}

// ── Attribute mapping ────────────────────────────────────────────────────────

#[tokio::test]
async fn attr_map_remaps_schema() {
    // A deployment that uses `rfc822Mailbox`/`fullName` instead of mail/displayName.
    let entry = MockEntry {
        dn: "cn=x,dc=example,dc=com".into(),
        attrs: vec![
            a("cn", "x"),
            a("rfc822Mailbox", "x@example.com"),
            a("fullName", "Person X"),
            a("objectClass", "person"),
        ],
        bin: vec![],
        password: None,
    };
    let mut endpoint = ep("ldap://x", 0);
    endpoint.attr_map = AttrMap {
        display_name: Some("fullName".into()),
        mail: Some("rfc822Mailbox".into()),
        ..AttrMap::default()
    };
    let backend = MockBackend::new(vec![entry]);
    let d = mk(vec![(endpoint, backend)], None, DirectoryOptions::default());
    let r = d.search_gal("person", 0).await.unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].mail, "x@example.com");
    assert_eq!(r[0].display_name, "Person X");
}

// ── Bind auth ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn bind_auth_accepts_and_rejects() {
    let mut alice = person("alice", "alice@example.com", "Alice");
    alice.password = Some("s3cret".into());
    let backend = MockBackend::new(vec![alice]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );

    // Correct password by mail lookup → Ok with resolved DN.
    match d.bind_auth("alice@example.com", "s3cret").await.unwrap() {
        BindOutcome::Ok { dn } => assert_eq!(dn, "cn=alice,dc=example,dc=com"),
        BindOutcome::Denied => panic!("should authenticate"),
    }
    // Wrong password → Denied.
    assert_eq!(
        d.bind_auth("alice@example.com", "wrong").await.unwrap(),
        BindOutcome::Denied
    );
    // Unknown user → Denied.
    assert_eq!(
        d.bind_auth("ghost@example.com", "s3cret").await.unwrap(),
        BindOutcome::Denied
    );
}

#[tokio::test]
async fn bind_auth_rejects_empty_password() {
    // SECURITY: empty password must never authenticate (LDAP unauthenticated-bind
    // pitfall). Denied even though the DN exists.
    let mut alice = person("alice", "alice@example.com", "Alice");
    alice.password = Some(String::new()); // even if the entry stored an empty pw
    let backend = MockBackend::new(vec![alice]);
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        None,
        DirectoryOptions::default(),
    );
    assert_eq!(
        d.bind_auth("alice@example.com", "").await.unwrap(),
        BindOutcome::Denied
    );
}

#[tokio::test]
async fn bind_auth_falls_through_priority() {
    // alice lives only on the secondary directory; primary has no such user.
    let bob = person("bob", "bob@example.com", "Bob");
    let mut alice = person("alice", "alice@example.com", "Alice");
    alice.password = Some("pw".into());
    let primary = MockBackend::new(vec![bob]);
    let secondary = MockBackend::new(vec![alice]);
    let d = mk(
        vec![
            (ep("ldap://pri", 0), primary),
            (ep("ldap://sec", 1), secondary),
        ],
        None,
        DirectoryOptions::default(),
    );
    assert!(matches!(
        d.bind_auth("alice@example.com", "pw").await.unwrap(),
        BindOutcome::Ok { .. }
    ));
}

// ── GAL cache ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn gal_cache_serves_repeat_search() {
    let backend = MockBackend::new(vec![person("Alice", "alice@example.com", "Alice")]);
    let counter = backend.clone();
    // Memory-only cache with the spec GalDirectory matrix (memory+store; no store
    // attached ⇒ memory only).
    let cache = Cache::default();
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        Some(cache),
        DirectoryOptions::default(),
    );
    let r1 = d.search_gal("alice", 0).await.unwrap();
    let r2 = d.search_gal("alice", 0).await.unwrap();
    assert_eq!(r1, r2);
    assert_eq!(counter.search_count(), 1, "second search served from cache");
}

#[tokio::test]
async fn cache_disabled_when_ttl_zero() {
    let backend = MockBackend::new(vec![person("Alice", "alice@example.com", "Alice")]);
    let counter = backend.clone();
    let opts = DirectoryOptions {
        cache_ttl: Duration::from_secs(0), // forces bypass
        ..DirectoryOptions::default()
    };
    let d = mk(
        vec![(ep("ldap://x", 0), backend)],
        Some(Cache::default()),
        opts,
    );
    let _ = d.search_gal("alice", 0).await.unwrap();
    let _ = d.search_gal("alice", 0).await.unwrap();
    assert_eq!(counter.search_count(), 2, "ttl=0 disables the cache");
}

// ── Hard gate: no openssl / native-tls anywhere in the resolved tree ─────────

#[test]
fn no_openssl_in_tree() {
    // ldap3 defaults to native-tls (→ openssl), which is BANNED. We force
    // `default-features=false, features=["tls-rustls"]`. This test proves the
    // guarantee STRUCTURALLY from the workspace Cargo.lock: the `ldap3` package's
    // transitive dependency closure must contain no openssl/native-tls, AND those
    // C-TLS packages must not appear anywhere in the workspace at all.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lock = std::path::Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("Cargo.lock");
    let text =
        std::fs::read_to_string(&lock).unwrap_or_else(|e| panic!("read {}: {e}", lock.display()));

    // Parse [[package]] blocks into name -> transitive dep names.
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let mut names: HashSet<String> = HashSet::new();
    for block in text.split("[[package]]") {
        let mut name = None;
        let mut block_deps: Vec<String> = Vec::new();
        let mut in_deps = false;
        for line in block.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("name = \"") {
                if let Some(n) = rest.strip_suffix('"') {
                    name = Some(n.to_string());
                }
            } else if t.starts_with("dependencies = [") {
                in_deps = true;
                if t.contains(']') {
                    in_deps = false;
                }
            } else if in_deps {
                if t.starts_with(']') {
                    in_deps = false;
                } else if let Some(dep) = t.trim_matches(|c| c == '"' || c == ',').split(' ').next()
                    && !dep.is_empty()
                {
                    block_deps.push(dep.trim_matches('"').to_string());
                }
            }
        }
        if let Some(n) = name {
            names.insert(n.clone());
            deps.insert(n, block_deps);
        }
    }

    const BANNED: [&str; 3] = ["openssl", "openssl-sys", "native-tls"];

    // (1) Absent from the whole workspace (the deny.toml `openssl` ban, re-proven
    // here at the crate level).
    for b in BANNED {
        assert!(
            !names.contains(b),
            "BANNED C-TLS crate `{b}` present in Cargo.lock — ldap3 must be rustls-only"
        );
    }

    // (2) ldap3 must actually be resolved (the feature we rely on is live) and its
    // transitive closure must be clean.
    assert!(
        names.contains("ldap3"),
        "ldap3 not resolved into Cargo.lock"
    );
    let mut stack = vec!["ldap3".to_string()];
    let mut visited: HashSet<String> = HashSet::new();
    while let Some(cur) = stack.pop() {
        if !visited.insert(cur.clone()) {
            continue;
        }
        assert!(
            !BANNED.contains(&cur.as_str()),
            "ldap3 transitively depends on BANNED `{cur}` — rustls feature not effective"
        );
        if let Some(children) = deps.get(&cur) {
            for c in children {
                stack.push(c.clone());
            }
        }
    }
}

// Tiny blocking helper for the one sync test that calls an async fn.
fn futures_block<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(fut)
}
