//! Live outbound S/MIME interoperability. Set MW_SMIME_INTEROP=1; missing
//! OpenSSL then fails rather than silently skipping. Committed inbound OpenSSL
//! messages are exercised by smime_e41 on every test run.
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
    let output = Command::new("openssl")
        .args(args)
        .output()
        .expect("OpenSSL on PATH");
    assert!(
        output.status.success(),
        "openssl {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
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
    for name in ["alice", "ecc", "p384", "ed"] {
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
            assert_eq!(run(&args), b"exact bytes\r\n", "{name} detached={detached}");
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
