//! t12-e-e2e-backend — S/MIME GAL cert lookup live-E2E (audit #12), the real
//! external-service leg of the DLP/SPF/S-MIME conformance row.
//!
//! Proves the NEW S/MIME recipient-cert wiring (#12): `mw-engine/security/keyring.rs`
//! now resolves recipient certs from the GAL via `Engine::gal_lookup_cert`, which
//! calls the attached `mw_directory::Directory::lookup_cert`. THIS leg drives that
//! exact seam against a REAL OpenLDAP seeded with an S/MIME `userCertificate;binary`
//! for alice (and no cert for bob) — the live half of the keyring change.
//!
//!   docker compose -f docker-compose.ci.yml up -d --wait openldap
//!   MW_LDAP_LIVE=1 cargo test -p mw-server --test t12_dlp_spf_smime -- --nocapture
//!
//! NOTE on DLP + SPF (audit #14/#13): the DLP dictionary/classification + `notify`
//! action and SPF pass/fail evaluation are pure engine logic with no external
//! dependency and are proven in mw-engine's own unit legs (security::dlp /
//! security::verdict, incl. seeded SPF pass/`-all` fail records). They are not
//! re-driven from the mw-server boundary because the DLP/SPF handlers require a
//! full account runtime that only the delivery/JMAP pipeline constructs; the
//! external-dependency piece of this row — the S/MIME GAL cert lookup vs a real
//! directory — is what this file exercises live.

use std::sync::Arc;

use ldap3::LdapConnAsync;

use mw_directory::{AttrMap, Directory, DirectoryConfig, DirectorySource, LdapEndpoint, LdapTls};
use mw_engine::{Engine, V7Hooks};
use mw_store::{ServerKey, Store};

const LDAP_ADMIN_DN: &str = "cn=admin,dc=example,dc=com";
const LDAP_ADMIN_PW: &str = "adminpassword";
const LDAP_BASE_DN: &str = "dc=example,dc=com";

fn live() -> bool {
    std::env::var("MW_LDAP_LIVE").ok().as_deref() == Some("1")
}
fn ldap_url() -> String {
    std::env::var("MW_LDAP_LIVE_URL")
        .or_else(|_| std::env::var("MW_E16_LDAP_URL"))
        .unwrap_or_else(|_| "ldap://127.0.0.1:1389".into())
}

/// Probe LDAP with the admin bind; LOUD skip on failure (never silent).
async fn ldap_reachable(scenario: &str) -> bool {
    match LdapConnAsync::new(&ldap_url()).await {
        Ok((conn, mut ldap)) => {
            tokio::spawn(async move {
                let _ = conn.drive().await;
            });
            let ok = ldap.simple_bind(LDAP_ADMIN_DN, LDAP_ADMIN_PW).await.is_ok();
            let _ = ldap.unbind().await;
            if !ok {
                eprintln!(
                    "\n[t12 S/MIME SKIP] {scenario}: LDAP admin bind rejected at {}.",
                    ldap_url()
                );
            }
            ok
        }
        Err(e) => {
            eprintln!(
                "\n[t12 S/MIME SKIP] {scenario}: OpenLDAP unreachable at {} ({e}). Bring it up: \
                 docker compose -f docker-compose.ci.yml up -d --wait openldap ; then MW_LDAP_LIVE=1.\n",
                ldap_url()
            );
            false
        }
    }
}

fn live_directory() -> Directory {
    Directory::new(DirectoryConfig {
        endpoints: vec![LdapEndpoint {
            url: ldap_url(),
            base_dn: LDAP_BASE_DN.into(),
            bind_dn: Some(LDAP_ADMIN_DN.into()),
            tls: LdapTls::None,
            priority: 0,
            attr_map: AttrMap::default(),
        }],
    })
    .with_service_password(LDAP_ADMIN_DN, LDAP_ADMIN_PW)
}

/// The keyring seam (#12): `Engine::gal_lookup_cert` resolves a recipient's S/MIME
/// cert through the attached live directory. Alice has one; bob does not.
#[tokio::test]
async fn engine_gal_lookup_cert_live() {
    if !live() {
        eprintln!(
            "\n[t12 S/MIME SKIP] MW_LDAP_LIVE!=1 — real OpenLDAP not driven. See module doc.\n"
        );
        return;
    }
    if !ldap_reachable("engine_gal_lookup_cert_live").await {
        return;
    }

    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Engine::new(store);

    // Before a directory is attached the GAL path is the byte-unchanged empty default.
    assert!(
        engine
            .gal_lookup_cert("alice@example.com")
            .await
            .unwrap()
            .is_empty(),
        "no directory attached ⇒ GAL cert lookup is empty (default path)"
    );

    engine.attach_v7(
        V7Hooks::new().with_directory(Arc::new(live_directory()) as Arc<dyn DirectorySource>),
    );

    // Alice's userCertificate;binary resolves as DER (SEQUENCE tag 0x30) — this is
    // exactly what keyring::crypto_key_lookup materializes into an S/MIME CryptoKey.
    let certs = engine
        .gal_lookup_cert("alice@example.com")
        .await
        .expect("gal_lookup_cert");
    assert!(
        !certs.is_empty(),
        "alice's S/MIME cert resolved via the engine GAL seam"
    );
    assert_eq!(certs[0][0], 0x30, "DER starts with a SEQUENCE tag");

    // Bob has a GAL entry but NO certificate ⇒ empty (the cert-absent branch).
    let bob = engine.gal_lookup_cert("bob@example.com").await.unwrap();
    assert!(
        bob.is_empty(),
        "bob has no userCertificate ⇒ empty cert set"
    );

    // A non-existent recipient ⇒ empty, never an error.
    let none = engine.gal_lookup_cert("nobody@example.com").await.unwrap();
    assert!(none.is_empty());
}

/// The raw directory cert source the engine wraps, driven directly (belt-and-braces
/// against the same live server).
#[tokio::test]
async fn directory_lookup_cert_live() {
    if !live() {
        return;
    }
    if !ldap_reachable("directory_lookup_cert_live").await {
        return;
    }
    let dir = live_directory();
    let certs = dir
        .lookup_cert("alice@example.com")
        .await
        .expect("lookup_cert");
    assert_eq!(certs.len(), 1, "exactly one seeded cert for alice");
    assert_eq!(certs[0][0], 0x30, "DER SEQUENCE");
    assert!(
        dir.lookup_cert("bob@example.com").await.unwrap().is_empty(),
        "bob has no cert"
    );
}
