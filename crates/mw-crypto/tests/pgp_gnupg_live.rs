//! Live GnuPG interop for t32-e37 (t32-ext §U tests): GnuPG encrypts and signs →
//! we decrypt and verify; we encrypt and sign → GnuPG decrypts and verifies.
//!
//! Runs only with `MW_GPG_INTEROP=1` (the GnuPG container job, or a host with
//! `gpg` 2.2+ on PATH); then a missing `gpg` FAILS rather than skipping, so the
//! job cannot pass without exercising GnuPG. Without the variable every test
//! returns early and says so. The recorded-fixture checks in `pgp.rs` and
//! `pgp_e37_red.rs` run everywhere.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use mw_crypto::pgp::manage::{self, RevocationReason};
use mw_crypto::pgp::{self, GenerateOptions, KeyProfile};
use mw_crypto::verdict;

const FIXTURE_PASS: &str = "interop-pass";
const PW: &str = "pw-live";

fn enabled() -> bool {
    match std::env::var("MW_GPG_INTEROP").as_deref() {
        Ok("1") => {
            assert!(
                Command::new("gpg").arg("--version").output().is_ok(),
                "MW_GPG_INTEROP=1 but gpg is not on PATH"
            );
            true
        }
        _ => {
            eprintln!("MW_GPG_INTEROP is not 1: live GnuPG interop not run");
            false
        }
    }
}

fn fixture(rel: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/pgp")
        .join(rel)
}

/// A throwaway GnuPG home.
struct Gpg {
    home: PathBuf,
}

impl Gpg {
    fn new(tag: &str) -> Gpg {
        let mut raw = [0u8; 8];
        getrandom_fill(&mut raw);
        let home = std::env::temp_dir().join(format!("mw-gpg-{tag}-{}", hex::encode(raw)));
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("gpg-agent.conf"), "allow-loopback-pinentry\n").unwrap();
        let g = Gpg { home };
        // `--trust-model always` (used below) never creates the trust database,
        // but importing a revocation updates it; make it now.
        let _ = Command::new("gpg")
            .env("GNUPGHOME", g.home_arg())
            .args(["--batch", "--check-trustdb"])
            .output();
        g
    }

    /// Generate a GnuPG key without a passphrase (GnuPG's default algorithms)
    /// and return its armored public key.
    fn generate(&self, uid: &str) -> String {
        let (_, status, ok) = self.run(
            &[
                "--passphrase",
                "",
                "--quick-gen-key",
                uid,
                "future-default",
                "default",
                "never",
            ],
            b"",
        );
        assert!(ok, "gpg key generation failed: {status}");
        let (public, status, ok) = self.run(&["--armor", "--export", uid], b"");
        assert!(ok, "{status}");
        String::from_utf8(public).unwrap()
    }

    fn home_arg(&self) -> String {
        to_gpg_path(&self.home)
    }

    /// Run gpg with `args`, feeding `stdin`; returns (stdout, status-fd text).
    fn run(&self, args: &[&str], stdin: &[u8]) -> (Vec<u8>, String, bool) {
        use std::io::Write;
        let mut child = Command::new("gpg")
            .env("GNUPGHOME", self.home_arg())
            .args([
                "--batch",
                "--yes",
                "--no-tty",
                "--pinentry-mode",
                "loopback",
                "--trust-model",
                "always",
                "--status-fd",
                "2",
            ])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn gpg");
        child.stdin.take().unwrap().write_all(stdin).unwrap();
        let out = child.wait_with_output().unwrap();
        (
            out.stdout,
            String::from_utf8_lossy(&out.stderr).into_owned(),
            out.status.success(),
        )
    }

    fn import(&self, armored: &str, passphrase: Option<&str>) {
        let mut args = vec!["--import"];
        if let Some(p) = passphrase {
            args.extend(["--passphrase", p]);
        }
        let (_, status, ok) = self.run(&args, armored.as_bytes());
        assert!(ok, "gpg --import failed: {status}");
    }
}

impl Drop for Gpg {
    fn drop(&mut self) {
        let _ = Command::new("gpgconf")
            .env("GNUPGHOME", self.home_arg())
            .args(["--kill", "gpg-agent"])
            .output();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// A path as the `gpg` on PATH reads it. The MSYS2 build Git for Windows ships
/// takes `/c/...` and treats `C:/...` as relative; a native or Linux gpg takes the
/// path as is. Which one is installed shows in its `Home:` line.
fn to_gpg_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    let msys = cfg!(windows)
        && Command::new("gpg")
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Home: /"))
            .unwrap_or(false);
    match s.as_bytes() {
        [d, b':', b'/', ..] if msys => format!("/{}{}", (*d as char).to_ascii_lowercase(), &s[2..]),
        _ => s,
    }
}

fn getrandom_fill(buf: &mut [u8]) {
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(buf);
}

fn compatible(uid: &str) -> pgp::GeneratedKey {
    pgp::generate_key_with(
        uid,
        PW,
        &GenerateOptions {
            profile: KeyProfile::Compatible,
            expires_in_secs: None,
        },
    )
    .unwrap()
}

#[test]
fn we_encrypt_and_sign_gnupg_decrypts_and_verifies() {
    if !enabled() {
        return;
    }
    let g = Gpg::new("dec");
    let gee_pub = g.generate("Gee <gee@example.com>");
    let ours = compatible("Ours <ours@example.com>");
    g.import(&ours.public_key_armored, None);

    let enc = pgp::encrypt_with(
        b"from mailwoman to gnupg",
        &[gee_pub],
        Some((&ours.encrypted_private_bundle, PW)),
    )
    .unwrap();
    assert_eq!(enc.seipd_version, 1);
    let (plain, status, ok) = g.run(&["--decrypt"], enc.armored.as_bytes());
    assert!(ok, "gpg could not decrypt: {status}");
    assert_eq!(plain, b"from mailwoman to gnupg");
    assert!(status.contains("GOODSIG"), "{status}");
    assert!(status.contains(&ours.fingerprint), "{status}");
}

#[test]
fn gnupg_encrypts_and_signs_we_decrypt_and_verify() {
    if !enabled() {
        return;
    }
    let g = Gpg::new("enc");
    g.import(
        &std::fs::read_to_string(fixture("gnupg-secret.asc")).unwrap(),
        Some(FIXTURE_PASS),
    );
    let ours = compatible("Ours <ours@example.com>");
    g.import(&ours.public_key_armored, None);
    let (ct, status, ok) = g.run(
        &[
            "--passphrase",
            FIXTURE_PASS,
            "--armor",
            "--sign",
            "--encrypt",
            "-u",
            "bob@example.com",
            "-r",
            "ours@example.com",
        ],
        b"from gnupg to mailwoman",
    );
    assert!(ok, "gpg encrypt failed: {status}");
    let bob_pub = std::fs::read_to_string(fixture("gnupg-public.asc")).unwrap();
    let out = pgp::decrypt_with(&ct, &ours.encrypted_private_bundle, PW, &[&bob_pub]).unwrap();
    assert_eq!(out.plaintext, b"from gnupg to mailwoman");
    assert_eq!(
        out.verification.state,
        verdict::VALID,
        "{:?}",
        out.verification
    );
}

#[test]
fn rsa_key_round_trip_with_gnupg() {
    if !enabled() {
        return;
    }
    let g = Gpg::new("rsa");
    let secret = std::fs::read_to_string(fixture("e37/rsa-signsub-secret.asc")).unwrap();
    let public = std::fs::read_to_string(fixture("e37/rsa-signsub-public.asc")).unwrap();
    g.import(&secret, Some(FIXTURE_PASS));
    let ct = pgp::encrypt(b"to an rsa key", std::slice::from_ref(&public), None).unwrap();
    let (plain, status, ok) = g.run(&["--passphrase", FIXTURE_PASS, "--decrypt"], ct.as_bytes());
    assert!(ok, "{status}");
    assert_eq!(plain, b"to an rsa key");

    // And the reverse: GnuPG signs with the RSA signing subkey, we verify.
    let (sig, status, ok) = g.run(
        &[
            "--passphrase",
            FIXTURE_PASS,
            "--armor",
            "--detach-sign",
            "-u",
            "rita@example.com",
        ],
        b"rsa data",
    );
    assert!(ok, "{status}");
    let v = pgp::verify_detached_with(b"rsa data", std::str::from_utf8(&sig).unwrap(), &[&public])
        .unwrap();
    assert_eq!(v.state, verdict::VALID, "{v:?}");
}

#[test]
fn gnupg_verifies_our_signatures_and_reads_our_key_material() {
    if !enabled() {
        return;
    }
    let g = Gpg::new("sig");
    let ours = compatible("Ours <ours@example.com>");
    // Our compatible secret key imports into GnuPG under its passphrase.
    g.import(&ours.encrypted_private_bundle, Some(PW));

    let sig = pgp::sign_detached(b"detached data", &ours.encrypted_private_bundle, PW).unwrap();
    let dir = &g.home;
    std::fs::write(dir.join("data.txt"), b"detached data").unwrap();
    std::fs::write(dir.join("data.sig"), sig.as_bytes()).unwrap();
    let path = |n: &str| to_gpg_path(&dir.join(n));
    let (_, status, ok) = g.run(&["--verify", &path("data.sig"), &path("data.txt")], b"");
    assert!(ok && status.contains("GOODSIG"), "{status}");

    let clear = pgp::clear_sign("cleartext body\n", &ours.encrypted_private_bundle, PW).unwrap();
    let (_, status, ok) = g.run(&["--verify"], clear.as_bytes());
    assert!(ok && status.contains("GOODSIG"), "{status}");

    // GnuPG decrypts a message to our key with our secret key.
    let ct = pgp::encrypt(
        b"own key",
        std::slice::from_ref(&ours.public_key_armored),
        None,
    )
    .unwrap();
    let (plain, status, ok) = g.run(&["--passphrase", PW, "--decrypt"], ct.as_bytes());
    assert!(ok, "{status}");
    assert_eq!(plain, b"own key");
}

#[test]
fn gnupg_imports_our_revocation_certificate() {
    if !enabled() {
        return;
    }
    let g = Gpg::new("rev");
    let ours = compatible("Ours <ours@example.com>");
    g.import(&ours.public_key_armored, None);
    let cert = manage::revocation_certificate(
        &ours.encrypted_private_bundle,
        PW,
        RevocationReason::Compromised,
        "test",
    )
    .unwrap();
    g.import(&cert, None);
    let (listing, status, ok) = g.run(&["--with-colons", "--list-keys", "ours@example.com"], b"");
    assert!(ok, "{status}");
    let listing = String::from_utf8_lossy(&listing);
    // Field 2 of the `pub` record is `r` for a revoked key.
    assert!(
        listing.lines().any(|l| l.starts_with("pub:r:")),
        "{listing}"
    );
}
