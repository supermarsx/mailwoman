//! SASL SCRAM-SHA-256 over ManageSieve (t32 f18), against a scripted server
//! that runs the server half of RFC 5802 for real: it checks the client's proof
//! and signs the exchange, so a wrong proof, a forged server signature or a
//! server that never proves itself are all exercised.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use hmac::{Hmac, KeyInit, Mac};
use mw_sieve::managesieve::{Connection, Credentials};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

const GREETING: &str = concat!(
    "\"IMPLEMENTATION\" \"Scripted SCRAM server\"\r\n",
    "\"SIEVE\" \"fileinto\"\r\n",
    "\"SASL\" \"PLAIN SCRAM-SHA-256\"\r\n",
    "\"VERSION\" \"1.0\"\r\n",
    "OK\r\n",
);

const SALT: &[u8] = b"pepper-and-salt!";
const ITERATIONS: u32 = 4096;

fn hmac(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut m = Hmac::<Sha256>::new_from_slice(key).unwrap();
    m.update(msg);
    m.finalize().into_bytes().into()
}

fn salted(password: &str) -> [u8; 32] {
    let mut msg = SALT.to_vec();
    msg.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac(password.as_bytes(), &msg);
    let mut out = u;
    for _ in 1..ITERATIONS {
        u = hmac(password.as_bytes(), &u);
        for (o, x) in out.iter_mut().zip(u.iter()) {
            *o ^= *x;
        }
    }
    out
}

/// How the scripted server ends a successful exchange.
#[derive(Clone, Copy)]
enum Ending {
    /// `OK (SASL "<server-final>")`.
    Code,
    /// The server-final as a challenge, the client's empty reply, then `OK`.
    Challenge,
    /// `OK (SASL ...)` with a signature made from the wrong key.
    ForgedSignature,
    /// A plain `OK` that never proves the server knows the password.
    NoProof,
}

fn unquote(line: &str) -> String {
    line.trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

/// Run the server half on `server`; the account's password is `pencil-on-server`.
async fn serve(server: DuplexStream, ending: Ending) -> Vec<String> {
    let (rd, mut wr) = tokio::io::split(server);
    let mut rd = BufReader::new(rd);
    let mut seen = Vec::new();
    let mut line = String::new();

    rd.read_line(&mut line).await.unwrap();
    seen.push(line.clone());
    let ir = line
        .trim_end()
        .strip_prefix("AUTHENTICATE \"SCRAM-SHA-256\" ")
        .expect("SCRAM with an initial response");
    let client_first = String::from_utf8(B64.decode(unquote(ir)).unwrap()).unwrap();
    let bare = client_first.strip_prefix("n,,").expect("gs2 header n,,");
    let cnonce = bare.split(',').find_map(|f| f.strip_prefix("r=")).unwrap();
    let server_first = format!("r={cnonce}srvNonce,s={},i={ITERATIONS}", B64.encode(SALT));
    wr.write_all(format!("\"{}\"\r\n", B64.encode(&server_first)).as_bytes())
        .await
        .unwrap();

    line.clear();
    rd.read_line(&mut line).await.unwrap();
    seen.push(line.clone());
    let client_final = String::from_utf8(B64.decode(unquote(&line)).unwrap()).unwrap();
    let (without_proof, proof) = client_final.rsplit_once(",p=").unwrap();
    let auth_message = format!("{bare},{server_first},{without_proof}");

    // Verify the proof the way a server does: H(proof XOR ClientSignature) == StoredKey.
    let salted = salted("pencil-on-server");
    let client_key = hmac(&salted, b"Client Key");
    let stored_key: [u8; 32] = Sha256::digest(client_key).into();
    let signature = hmac(&stored_key, auth_message.as_bytes());
    let mut recovered = B64.decode(proof).unwrap();
    for (r, s) in recovered.iter_mut().zip(signature.iter()) {
        *r ^= *s;
    }
    let recovered_stored: [u8; 32] = Sha256::digest(&recovered).into();
    if recovered_stored != stored_key {
        wr.write_all(b"NO \"Authentication failed\"\r\n")
            .await
            .unwrap();
        return seen;
    }

    let server_key = hmac(&salted, b"Server Key");
    let key = match ending {
        Ending::ForgedSignature => hmac(b"not the key", b"Server Key"),
        _ => server_key,
    };
    let server_final = format!("v={}", B64.encode(hmac(&key, auth_message.as_bytes())));
    match ending {
        Ending::Code | Ending::ForgedSignature => {
            wr.write_all(
                format!(
                    "OK (SASL \"{}\") \"Logged in\"\r\n",
                    B64.encode(&server_final)
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        }
        Ending::Challenge => {
            wr.write_all(format!("\"{}\"\r\n", B64.encode(&server_final)).as_bytes())
                .await
                .unwrap();
            line.clear();
            rd.read_line(&mut line).await.unwrap();
            seen.push(line.clone());
            wr.write_all(b"OK \"Logged in\"\r\n").await.unwrap();
        }
        Ending::NoProof => {
            wr.write_all(b"OK \"Logged in\"\r\n").await.unwrap();
        }
    }
    seen
}

async fn run(password: &'static str, ending: Ending) -> (mw_sieve::Result<()>, Vec<String>) {
    let (client, mut server) = tokio::io::duplex(1 << 16);
    server.write_all(GREETING.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.unwrap();
    let creds = Credentials::password_for(conn.capabilities(), "user", password);
    assert!(
        matches!(creds, Credentials::ScramSha256 { .. }),
        "SCRAM preferred when offered"
    );
    let server_task = tokio::spawn(serve(server, ending));
    let res = conn.authenticate(&creds).await;
    drop(conn);
    (res, server_task.await.unwrap())
}

#[tokio::test]
async fn scram_succeeds_and_checks_the_server_signature_from_the_ok_code() {
    let (res, seen) = run("pencil-on-server", Ending::Code).await;
    res.expect("SCRAM exchange");
    assert_eq!(seen.len(), 2);
    let client_final = String::from_utf8(B64.decode(unquote(&seen[1])).unwrap()).unwrap();
    assert!(client_final.starts_with("c=biws,r="), "{client_final}");
}

#[tokio::test]
async fn scram_accepts_the_server_final_as_a_challenge() {
    let (res, seen) = run("pencil-on-server", Ending::Challenge).await;
    res.expect("SCRAM exchange");
    assert_eq!(seen.len(), 3);
    assert_eq!(seen[2], "\"\"\r\n", "the empty reply to the server-final");
}

#[tokio::test]
async fn a_wrong_password_is_refused_by_the_server() {
    let (res, _) = run("wrong", Ending::Code).await;
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Authentication failed"), "{err}");
}

/// The server says OK, but its signature is not made from the password: this
/// is not the server the account belongs to, and the login must fail.
#[tokio::test]
async fn a_forged_server_signature_fails_even_after_ok() {
    let (res, _) = run("pencil-on-server", Ending::ForgedSignature).await;
    let err = res.unwrap_err();
    assert!(
        err.to_string().contains("signature does not match"),
        "{err}"
    );
}

#[tokio::test]
async fn a_server_that_never_proves_itself_fails() {
    let (res, _) = run("pencil-on-server", Ending::NoProof).await;
    let err = res.unwrap_err();
    assert!(err.to_string().contains("without proving"), "{err}");
}
