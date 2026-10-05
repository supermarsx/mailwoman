//! S/MIME key generation (t29-e2): what `smime::generate` produces is judged by
//! `openssl`, not by the generator's own labels.
//!
//! Every claim the product makes about a generated key is checked against an
//! independent reader here: the certificate parses as X.509 and says RSA of the
//! size we report, the request verifies, a signature made with the key verifies
//! against the certificate, a message openssl encrypts to the certificate
//! decrypts, the PKCS#12 export opens with the passphrase and not with another,
//! and a certificate a CA issues from the request is accepted while one for a
//! different key is refused.
//!
//! These tests need the `openssl` command (3.x). They FAIL when it is missing
//! rather than skip: a skipped interop test proves nothing.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use base64::Engine;
use mw_crypto::smime::{self, GeneratedSmime};

const PASSPHRASE: &str = "correct horse battery staple";
const EMAIL: &str = "alice@example.org";
const NAME: &str = "Doe, Alice + Co=1";
/// 2026-10-05T00:00:00Z. The generator takes its clock as an argument.
const NOW: u64 = 1_791_158_400;

/// RSA key generation is the slow step (unoptimised bignum code in the dev
/// profile), so the tests share two keys instead of making one each.
fn alice() -> &'static GeneratedSmime {
    static KEY: OnceLock<GeneratedSmime> = OnceLock::new();
    KEY.get_or_init(|| {
        smime::generate_with_bits(NAME, EMAIL, PASSPHRASE, NOW, 2048).expect("generate alice")
    })
}

fn mallory() -> &'static GeneratedSmime {
    static KEY: OnceLock<GeneratedSmime> = OnceLock::new();
    KEY.get_or_init(|| {
        smime::generate_with_bits("Mallory", "mallory@example.org", PASSPHRASE, NOW, 2048)
            .expect("generate mallory")
    })
}

/// A scratch directory for one test, removed when the guard drops.
struct Scratch(PathBuf);
impl Scratch {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("mw-smime-made-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        Self(dir)
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("write scratch file");
        path
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Ran {
    ok: bool,
    stdout: String,
    stderr: String,
}

/// Run `openssl <args>`; a missing binary is a test failure, not a skip.
fn openssl(args: &[&str]) -> Ran {
    let out = Command::new("openssl")
        .args(args)
        .output()
        .expect("the `openssl` command is required by these tests and was not found on PATH");
    Ran {
        ok: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn openssl_ok(args: &[&str]) -> Ran {
    let ran = openssl(args);
    assert!(ran.ok, "openssl {args:?} failed:\n{}", ran.stderr);
    ran
}

fn p(path: &Path) -> &str {
    path.to_str().expect("utf-8 scratch path")
}

/// The line of `openssl … -modulus` output, for comparing two public keys.
fn modulus(ran: &Ran) -> String {
    ran.stdout
        .lines()
        .find(|l| l.starts_with("Modulus="))
        .expect("a Modulus= line")
        .to_string()
}

#[test]
fn the_certificate_is_x509_and_says_what_we_say_about_it() {
    let s = Scratch::new("cert");
    let made = alice();
    let cert = s.write("alice.pem", &made.cert.cert_pem);

    let text = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-text"]).stdout;
    assert!(text.contains("Version: 3 (0x2)"), "{text}");
    assert!(
        text.contains("Public Key Algorithm: rsaEncryption"),
        "{text}"
    );
    assert!(text.contains("Public-Key: (2048 bit)"), "{text}");
    assert!(
        text.contains("Signature Algorithm: sha256WithRSAEncryption"),
        "{text}"
    );
    assert!(text.contains("email:alice@example.org"), "{text}");
    assert!(text.contains("E-mail Protection"), "{text}");
    assert!(text.contains("CA:FALSE"), "{text}");
    assert!(
        text.contains("Digital Signature, Key Encipherment"),
        "{text}"
    );
    assert!(text.contains("Subject Key Identifier"), "{text}");
    // Never a CA and never a server certificate.
    assert!(!text.contains("CA:TRUE"), "{text}");
    assert!(!text.contains("Certificate Sign"), "{text}");
    assert!(!text.contains("TLS Web"), "{text}");

    // Our label is what openssl reads, not what the caller asked for.
    assert_eq!(made.cert.algorithm, "rsa-2048");
    assert_eq!(made.cert.addresses, vec![EMAIL.to_string()]);
    assert!(made.cert.self_issued);
    let fp = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-fingerprint", "-sha256"]).stdout;
    let fp: String = fp
        .trim()
        .rsplit('=')
        .next()
        .unwrap()
        .chars()
        .filter(|c| *c != ':')
        .collect();
    assert_eq!(made.cert.fingerprint, fp);

    // Validity: five minutes before `NOW`, 730 days after it.
    let dates = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-dates"]).stdout;
    assert!(
        dates.contains("notBefore=Oct  4 23:55:00 2026 GMT"),
        "{dates}"
    );
    assert!(
        dates.contains("notAfter=Oct  4 00:00:00 2028 GMT"),
        "{dates}"
    );
    assert_eq!(made.cert.not_before, "2026-10-04T23:55:00Z");
    assert_eq!(made.cert.not_after, "2028-10-04T00:00:00Z");

    // The name went in as a value: its `,` `+` `=` did not become DN structure.
    let subject = openssl_ok(&[
        "x509",
        "-in",
        p(&cert),
        "-noout",
        "-subject",
        "-issuer",
        "-nameopt",
        "multiline,utf8",
    ])
    .stdout;
    let cn_lines: Vec<&str> = subject
        .lines()
        .filter(|l| l.trim_start().starts_with("commonName"))
        .collect();
    assert_eq!(cn_lines.len(), 2, "subject and issuer CN:\n{subject}");
    for line in cn_lines {
        assert!(line.trim_end().ends_with("= Doe, Alice + Co=1"), "{line}");
    }
    assert_eq!(subject.matches("emailAddress").count(), 2, "{subject}");

    // Self-signed: it chains to itself and to nothing else.
    openssl_ok(&["verify", "-CAfile", p(&cert), p(&cert)]);
    let other = s.write("mallory.pem", &mallory().cert.cert_pem);
    assert!(
        !openssl(&["verify", "-CAfile", p(&other), p(&cert)]).ok,
        "alice's certificate must not verify under mallory's"
    );
}

#[test]
fn the_private_bundle_is_an_encrypted_pkcs8_key_for_that_certificate() {
    let s = Scratch::new("bundle");
    let made = alice();
    assert!(
        made.encrypted_private_bundle
            .starts_with("-----BEGIN ENCRYPTED PRIVATE KEY-----"),
        "the bundle is not an encrypted PKCS#8 PEM"
    );
    let bundle = s.write("alice.key.pem", &made.encrypted_private_bundle);
    let cert = s.write("alice.pem", &made.cert.cert_pem);
    let pass = format!("pass:{PASSPHRASE}");

    let key = openssl_ok(&[
        "rsa",
        "-in",
        p(&bundle),
        "-passin",
        &pass,
        "-noout",
        "-modulus",
    ]);
    let crt = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-modulus"]);
    assert_eq!(modulus(&key), modulus(&crt));
    openssl_ok(&[
        "rsa",
        "-in",
        p(&bundle),
        "-passin",
        &pass,
        "-noout",
        "-check",
    ]);

    assert!(
        !openssl(&["rsa", "-in", p(&bundle), "-passin", "pass:wrong", "-noout"]).ok,
        "the bundle opened with the wrong passphrase"
    );
}

#[test]
fn a_signature_made_with_the_key_verifies_against_the_certificate() {
    let s = Scratch::new("sign");
    let made = alice();
    let body = b"signed with a generated key";
    let b64 = smime::sign(
        body,
        &made.cert.cert_pem,
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .expect("sign");
    let der = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .unwrap();
    let signed = s.write("signed.der", &der);
    let cert = s.write("alice.pem", &made.cert.cert_pem);
    let out = s.path("out.txt");

    openssl_ok(&[
        "cms",
        "-verify",
        "-inform",
        "DER",
        "-in",
        p(&signed),
        "-CAfile",
        p(&cert),
        "-purpose",
        "smimesign",
        "-out",
        p(&out),
    ]);
    assert_eq!(std::fs::read(&out).unwrap(), body);

    // Not trusted under anybody else's certificate.
    let other = s.write("mallory.pem", &mallory().cert.cert_pem);
    assert!(
        !openssl(&[
            "cms",
            "-verify",
            "-inform",
            "DER",
            "-in",
            p(&signed),
            "-CAfile",
            p(&other),
            "-purpose",
            "smimesign",
            "-out",
            p(&out),
        ])
        .ok
    );
}

#[test]
fn a_message_openssl_encrypts_to_the_certificate_decrypts_with_the_key() {
    let s = Scratch::new("encrypt");
    let made = alice();
    let body = b"encrypted to a generated certificate";
    let plain = s.write("plain.txt", body);
    let cert = s.write("alice.pem", &made.cert.cert_pem);
    let enveloped = s.path("enveloped.der");

    openssl_ok(&[
        "cms",
        "-encrypt",
        "-binary",
        "-aes256",
        "-in",
        p(&plain),
        "-outform",
        "DER",
        "-out",
        p(&enveloped),
        p(&cert),
    ]);
    let der = std::fs::read(&enveloped).unwrap();
    let pt = smime::decrypt(&der, &made.encrypted_private_bundle, PASSPHRASE).expect("decrypt");
    assert_eq!(pt, body);

    assert!(
        smime::decrypt(&der, &mallory().encrypted_private_bundle, PASSPHRASE).is_err(),
        "another key decrypted alice's message"
    );
}

#[test]
fn the_certificate_request_verifies_and_asks_for_the_same_identity() {
    let s = Scratch::new("csr");
    let made = alice();
    let csr_pem = smime::certificate_request(
        &made.cert.cert_pem,
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .expect("csr");
    assert!(csr_pem.starts_with("-----BEGIN CERTIFICATE REQUEST-----"));
    let csr = s.write("alice.csr", &csr_pem);

    let ran = openssl_ok(&["req", "-in", p(&csr), "-noout", "-verify", "-text"]);
    let all = format!("{}{}", ran.stdout, ran.stderr);
    assert!(all.contains("verify OK"), "{all}");
    assert!(all.contains("email:alice@example.org"), "{all}");
    assert!(all.contains("E-mail Protection"), "{all}");
    assert!(all.contains("Public-Key: (2048 bit)"), "{all}");

    // The request carries the same public key as the certificate.
    let req = openssl_ok(&["req", "-in", p(&csr), "-noout", "-modulus"]);
    let cert = s.write("alice.pem", &made.cert.cert_pem);
    let crt = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-modulus"]);
    assert_eq!(modulus(&req), modulus(&crt));

    // A request needs the private key: the wrong passphrase, or a certificate
    // that belongs to another key, produces nothing.
    assert!(
        smime::certificate_request(&made.cert.cert_pem, &made.encrypted_private_bundle, "wrong")
            .is_err()
    );
    assert!(
        smime::certificate_request(
            &mallory().cert.cert_pem,
            &made.encrypted_private_bundle,
            PASSPHRASE
        )
        .is_err()
    );
}

#[test]
fn the_pkcs12_export_opens_with_the_passphrase_and_not_with_another() {
    let s = Scratch::new("p12");
    let made = alice();
    let p12 = smime::export_pkcs12(
        &made.cert.cert_pem,
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .expect("export");
    let file = s.write("alice.p12", &p12);
    let pass = format!("pass:{PASSPHRASE}");

    let key_out = s.path("key.pem");
    openssl_ok(&[
        "pkcs12",
        "-in",
        p(&file),
        "-passin",
        &pass,
        "-nocerts",
        "-noenc",
        "-out",
        p(&key_out),
    ]);
    let cert_out = s.path("cert.pem");
    openssl_ok(&[
        "pkcs12",
        "-in",
        p(&file),
        "-passin",
        &pass,
        "-nokeys",
        "-out",
        p(&cert_out),
    ]);
    let key = openssl_ok(&["rsa", "-in", p(&key_out), "-noout", "-modulus"]);
    let crt = openssl_ok(&["x509", "-in", p(&cert_out), "-noout", "-modulus"]);
    assert_eq!(modulus(&key), modulus(&crt));
    let fp = openssl_ok(&[
        "x509",
        "-in",
        p(&cert_out),
        "-noout",
        "-fingerprint",
        "-sha256",
    ])
    .stdout;
    assert_eq!(
        fp.trim().rsplit('=').next().unwrap().replace(':', ""),
        made.cert.fingerprint
    );

    // What the file is, as openssl describes it. The MAC line is pinned on
    // purpose: this export carries none (see `export_pkcs12`), and the day it
    // gains one this assertion is the reminder to update the product copy.
    let info = openssl_ok(&[
        "pkcs12",
        "-in",
        p(&file),
        "-passin",
        &pass,
        "-info",
        "-noout",
    ]);
    assert!(
        info.stderr.contains("PBES2, PBKDF2, AES-256-CBC"),
        "{}",
        info.stderr
    );
    assert!(info.stderr.contains("MAC is absent"), "{}", info.stderr);

    assert!(
        !openssl(&[
            "pkcs12",
            "-in",
            p(&file),
            "-passin",
            "pass:wrong",
            "-nocerts",
            "-noenc",
            "-out",
            p(&key_out),
        ])
        .ok,
        "the export opened with the wrong passphrase"
    );

    // And our own importer reads it back as the same certificate and key.
    let back = smime::import_pkcs12(&p12, PASSPHRASE).expect("re-import");
    assert_eq!(back.fingerprint, made.cert.fingerprint);
    assert_eq!(back.addresses, vec![EMAIL.to_string()]);
    assert_eq!(back.algorithm, "rsa-2048");
    assert_eq!(back.not_after, made.cert.not_after);
    assert!(smime::import_pkcs12(&p12, "wrong").is_err());

    // Export needs the key's passphrase and a certificate for that key.
    assert!(
        smime::export_pkcs12(&made.cert.cert_pem, &made.encrypted_private_bundle, "wrong").is_err()
    );
    assert!(
        smime::export_pkcs12(
            &mallory().cert.cert_pem,
            &made.encrypted_private_bundle,
            PASSPHRASE
        )
        .is_err()
    );
}

#[test]
fn a_ca_issued_certificate_is_attached_and_one_for_another_key_is_refused() {
    let s = Scratch::new("attach");
    let made = alice();
    let csr = s.write(
        "alice.csr",
        smime::certificate_request(
            &made.cert.cert_pem,
            &made.encrypted_private_bundle,
            PASSPHRASE,
        )
        .expect("csr"),
    );

    // A throwaway certificate authority issues from the request.
    let ca_key = s.path("ca.key");
    let ca_crt = s.path("ca.pem");
    openssl_ok(&[
        "req",
        "-x509",
        "-newkey",
        "rsa:2048",
        "-noenc",
        "-keyout",
        p(&ca_key),
        "-out",
        p(&ca_crt),
        "-days",
        "30",
        "-subj",
        "/CN=Test CA",
    ]);
    let issued = s.path("issued.pem");
    openssl_ok(&[
        "x509",
        "-req",
        "-in",
        p(&csr),
        "-CA",
        p(&ca_crt),
        "-CAkey",
        p(&ca_key),
        "-CAcreateserial",
        "-copy_extensions",
        "copy",
        "-days",
        "30",
        "-out",
        p(&issued),
    ]);
    let issued_pem = std::fs::read(&issued).unwrap();

    // Precondition: the certificate issued for this key is accepted.
    let attached =
        smime::attach_issued_cert(&issued_pem, &made.encrypted_private_bundle, PASSPHRASE)
            .expect("the issued certificate is for this key");
    assert!(!attached.self_issued);
    assert_eq!(attached.addresses, vec![EMAIL.to_string()]);
    assert_eq!(attached.algorithm, "rsa-2048");
    assert_ne!(attached.fingerprint, made.cert.fingerprint);

    // DER is accepted as well as PEM.
    let issued_der = s.path("issued.der");
    openssl_ok(&[
        "x509",
        "-in",
        p(&issued),
        "-outform",
        "DER",
        "-out",
        p(&issued_der),
    ]);
    let from_der = smime::attach_issued_cert(
        &std::fs::read(&issued_der).unwrap(),
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .expect("DER form");
    assert_eq!(from_der.fingerprint, attached.fingerprint);

    // The key signs under the issued certificate, and it chains to the CA.
    let b64 = smime::sign(
        b"issued",
        &attached.cert_pem,
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .unwrap();
    let signed = s.write(
        "signed.der",
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap(),
    );
    openssl_ok(&[
        "cms",
        "-verify",
        "-inform",
        "DER",
        "-in",
        p(&signed),
        "-CAfile",
        p(&ca_crt),
        "-purpose",
        "smimesign",
        "-out",
        p(&s.path("out.txt")),
    ]);

    // A certificate for a different key is refused, whoever signed it.
    let err = smime::attach_issued_cert(
        mallory().cert.cert_pem.as_bytes(),
        &made.encrypted_private_bundle,
        PASSPHRASE,
    )
    .expect_err("a certificate for another key was attached");
    assert!(
        err.to_string().contains("not for this private key"),
        "{err}"
    );
    // So is anything that is not a certificate, and a wrong passphrase.
    assert!(
        smime::attach_issued_cert(
            b"not a certificate",
            &made.encrypted_private_bundle,
            PASSPHRASE
        )
        .is_err()
    );
    assert!(
        smime::attach_issued_cert(&issued_pem, &made.encrypted_private_bundle, "wrong").is_err()
    );
}

/// Certificates this crate did not make: the address may be only in the subject
/// alternative name, and the key may not be RSA.
#[test]
fn a_foreign_certificate_is_described_from_its_own_contents() {
    let s = Scratch::new("foreign");
    let key = s.path("ec.key");
    let crt = s.path("ec.pem");
    openssl_ok(&[
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:P-256",
        "-noenc",
        "-keyout",
        p(&key),
        "-out",
        p(&crt),
        "-days",
        "30",
        "-subj",
        "/CN=San Only",
        "-addext",
        "subjectAltName=email:san-only@example.org,email:second@example.org",
    ]);
    let d = smime::describe_certificate(&std::fs::read(&crt).unwrap()).expect("describe");
    assert_eq!(d.algorithm, "ecdsa-p256");
    assert_eq!(
        d.addresses,
        vec![
            "san-only@example.org".to_string(),
            "second@example.org".to_string()
        ]
    );
    assert!(d.self_issued);

    // The recorded RSA fixture names its address in the subject DN only.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/smime/alice.crt.pem");
    let text = openssl_ok(&["x509", "-in", p(&fixture), "-noout", "-text"]).stdout;
    let d = smime::describe_certificate(&std::fs::read(&fixture).unwrap()).expect("describe");
    let bits = d.algorithm.strip_prefix("rsa-").expect("an RSA label");
    assert!(
        text.contains(&format!("Public-Key: ({bits} bit)")),
        "{text}"
    );
}

#[test]
fn generation_refuses_what_it_cannot_certify() {
    for bad in [
        "",
        "no-at-sign",
        "@example.org",
        "alice@",
        "a@b@c",
        "alice @example.org",
        "alice@example.org\r\nBcc: x@y",
        "<alice@example.org>",
        "älice@example.org",
    ] {
        assert!(
            smime::generate_with_bits("A", bad, PASSPHRASE, NOW, 2048).is_err(),
            "accepted {bad:?}"
        );
    }
    assert!(smime::generate_with_bits("A", EMAIL, "", NOW, 2048).is_err());
    assert!(smime::generate_with_bits("A", EMAIL, PASSPHRASE, NOW, 1024).is_err());
    assert!(smime::generate_with_bits(&"n".repeat(65), EMAIL, PASSPHRASE, NOW, 2048).is_err());
    assert!(smime::generate_with_bits("line\nbreak", EMAIL, PASSPHRASE, NOW, 2048).is_err());
}

/// The size `generate` uses when the caller does not choose, with no name given.
#[test]
fn the_default_key_size_is_the_documented_one() {
    let s = Scratch::new("default");
    let made = smime::generate("", EMAIL, PASSPHRASE, NOW).expect("generate");
    let cert = s.write("default.pem", &made.cert.cert_pem);
    let text = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-text"]).stdout;
    let want = format!("Public-Key: ({} bit)", smime::GENERATED_RSA_BITS);
    assert!(text.contains(&want), "{text}");
    assert_eq!(
        made.cert.algorithm,
        format!("rsa-{}", smime::GENERATED_RSA_BITS)
    );
    // No name: the subject is the address alone, with no empty CN.
    let subject = openssl_ok(&["x509", "-in", p(&cert), "-noout", "-subject"]).stdout;
    assert!(!subject.contains("CN"), "{subject}");
    assert!(subject.contains("alice@example.org"), "{subject}");
    openssl_ok(&["verify", "-CAfile", p(&cert), p(&cert)]);
}
