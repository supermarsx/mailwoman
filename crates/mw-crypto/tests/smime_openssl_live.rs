//! Live outbound S/MIME interoperability. Set MW_SMIME_INTEROP=1; missing
//! OpenSSL then fails rather than silently skipping. Committed inbound OpenSSL
//! messages are exercised by smime_e41 on every test run.
//!
//! Ed25519 in CMS (RFC 8419) needs OpenSSL 3.2 or later: 3.0 and 3.1 refuse even
//! their own Ed25519 signatures ("eddsa_digest_signverify_init: invalid digest").
//! When the `openssl` on PATH is older, the Ed25519 signatures are judged by the
//! command named in MW_SMIME_OPENSSL_ED25519 instead, and the test fails if that
//! is unset. No key is left unjudged.
use std::path::{Path, PathBuf};
use std::process::Command;

use mw_crypto::smime_cms::{self, Cipher, Recipient, SignOptions};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/smime/e41")
        .join(name)
}

fn enabled() -> bool {
    if std::env::var("MW_SMIME_INTEROP").as_deref() != Ok("1") {
        eprintln!("MW_SMIME_INTEROP is not 1: live OpenSSL interop not run");
        return false;
    }
    true
}

fn run(args: &[&str]) -> Vec<u8> {
    run_with("openssl", args)
}

fn run_with(tool: &str, args: &[&str]) -> Vec<u8> {
    let output = Command::new(tool)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("{tool} (OpenSSL): {e}"));
    assert!(
        output.status.success(),
        "{tool} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// `(major, minor)` from `<tool> version` ("OpenSSL 3.0.13 30 Jan 2024").
fn openssl_version(tool: &str) -> (u32, u32) {
    let out = String::from_utf8(run_with(tool, &["version"])).unwrap();
    let ver = out
        .split_whitespace()
        .nth(1)
        .unwrap_or_else(|| panic!("{tool} version: {out:?}"));
    let mut parts = ver.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

/// The OpenSSL that judges Ed25519 CMS: the one on PATH when it is 3.2 or later,
/// otherwise MW_SMIME_OPENSSL_ED25519, which must then name an OpenSSL 3.2+.
fn ed25519_openssl() -> String {
    if openssl_version("openssl") >= (3, 2) {
        return "openssl".into();
    }
    let tool = std::env::var("MW_SMIME_OPENSSL_ED25519")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            panic!(
                "the openssl on PATH is older than 3.2 and cannot verify Ed25519 CMS; \
                 set MW_SMIME_OPENSSL_ED25519 to an OpenSSL 3.2+ command"
            )
        });
    assert!(
        openssl_version(&tool) >= (3, 2),
        "MW_SMIME_OPENSSL_ED25519={tool} is older than OpenSSL 3.2"
    );
    tool
}

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!("mw-smime-live-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn write(&self, name: &str, bytes: &[u8]) -> String {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_str().unwrap().to_owned()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn openssl_verifies_mailwoman_signatures() {
    if !enabled() {
        return;
    }
    let scratch = Scratch::new("sign");
    let content = scratch.write("body", b"exact bytes\r\n");
    let ed_tool = ed25519_openssl();
    for name in ["alice", "ecc", "p384", "ed"] {
        let tool = if name == "ed" {
            ed_tool.as_str()
        } else {
            "openssl"
        };
        for detached in [true, false] {
            let der = smime_cms::sign(
                b"exact bytes\r\n",
                &std::fs::read(fixture(&format!("{name}.pem"))).unwrap(),
                &std::fs::read_to_string(fixture(&format!("{name}.key"))).unwrap(),
                "",
                &SignOptions {
                    detached,
                    ..Default::default()
                },
            )
            .unwrap();
            let signature = scratch.write("signature.der", &der);
            let mut args = vec![
                "cms",
                "-verify",
                "-binary",
                "-inform",
                "DER",
                "-in",
                &signature,
                "-noverify",
            ];
            if detached {
                args.extend(["-content", &content]);
            }
            assert_eq!(
                run_with(tool, &args),
                b"exact bytes\r\n",
                "{name} detached={detached} ({tool})"
            );
        }
    }
}

#[test]
fn openssl_decrypts_mailwoman_cbc_and_gcm_for_rsa_and_ecdh() {
    if !enabled() {
        return;
    }
    let scratch = Scratch::new("encrypt");
    for name in ["alice", "ecc", "p384"] {
        for cipher in [Cipher::Cbc, Cipher::Gcm] {
            let cert = fixture(&format!("{name}.pem"));
            let key = fixture(&format!("{name}.key"));
            let encrypted = smime_cms::encrypt(
                b"interop\r\n",
                &[Recipient {
                    cert: std::fs::read(&cert).unwrap(),
                    gcm: Some(true),
                }],
                cipher,
            )
            .unwrap();
            let input = scratch.write("encrypted.der", &encrypted.der);
            assert_eq!(
                run(&[
                    "cms",
                    "-decrypt",
                    "-binary",
                    "-inform",
                    "DER",
                    "-in",
                    &input,
                    "-recip",
                    cert.to_str().unwrap(),
                    "-inkey",
                    key.to_str().unwrap()
                ]),
                b"interop\r\n"
            );
        }
    }
}

#[test]
fn openssl_imports_mailwoman_pkcs12_and_reads_ocsp_request() {
    if !enabled() {
        return;
    }
    let scratch = Scratch::new("containers");
    for name in ["alice", "ecc", "p384", "ed"] {
        let cert = std::fs::read_to_string(fixture(&format!("{name}.pem"))).unwrap();
        let key = std::fs::read_to_string(fixture(&format!("{name}.key"))).unwrap();
        let p12 = mw_crypto::smime::export_pkcs12(&cert, &key, "").unwrap();
        let input = scratch.write("key.p12", &p12);
        run(&["pkcs12", "-in", &input, "-passin", "pass:", "-noout"]);
    }
    let alice =
        smime_cms::parse_certificate(&std::fs::read(fixture("alice.pem")).unwrap()).unwrap();
    let issuer =
        smime_cms::parse_certificate(&std::fs::read(fixture("inter.pem")).unwrap()).unwrap();
    let query = mw_crypto::pkix::revocation::query_for(&alice, &issuer);
    let req = mw_crypto::pkix::revocation::ocsp_request(&query).unwrap();
    let input = scratch.write("request.der", &req);
    let output = run(&["ocsp", "-reqin", &input, "-req_text"]);
    assert!(String::from_utf8_lossy(&output).contains("Certificate ID:"));
}
