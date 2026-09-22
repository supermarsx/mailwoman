//! A scripted POP3 server on a loopback socket, for engine-mode tests.
//!
//! Engine mode logs in by dialling a real mail server. This one speaks just
//! enough of RFC 1939 for a login and an empty resync: a greeting, `CAPA`,
//! `USER`/`PASS` against one fixed account, `STAT`, `UIDL`/`LIST` over an empty
//! maildrop, `NOOP` and `QUIT`. Anything else is answered `-ERR`. It is plaintext,
//! so the server under test must run with `MW_ENGINE_TLS=plaintext`.
//!
//! Each connection is served on its own task, because the POP3 backend opens a
//! fresh session per operation.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// A running scripted POP3 server.
pub struct MockPop3 {
    pub addr: SocketAddr,
    accepted: Arc<AtomicUsize>,
    refused: Arc<AtomicUsize>,
}

impl MockPop3 {
    /// Serve one account, `user`/`pass`, on a fresh loopback port.
    pub async fn start(user: &str, pass: &str) -> MockPop3 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accepted = Arc::new(AtomicUsize::new(0));
        let refused = Arc::new(AtomicUsize::new(0));
        let (user, pass) = (user.to_string(), pass.to_string());
        let (acc, refd) = (accepted.clone(), refused.clone());
        tokio::spawn(async move {
            loop {
                let Ok((sock, _)) = listener.accept().await else {
                    return;
                };
                let (user, pass) = (user.clone(), pass.clone());
                let (acc, refd) = (acc.clone(), refd.clone());
                tokio::spawn(async move {
                    let _ = serve(sock, &user, &pass, &acc, &refd).await;
                });
            }
        });
        MockPop3 {
            addr,
            accepted,
            refused,
        }
    }

    /// The `pop3://` URL a login form posts for this server.
    pub fn url(&self) -> String {
        format!("pop3://{}", self.addr)
    }

    /// How many `PASS` commands carried the right password.
    pub fn logins_accepted(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }

    /// How many `PASS` commands were refused.
    pub fn logins_refused(&self) -> usize {
        self.refused.load(Ordering::SeqCst)
    }
}

async fn serve(
    sock: tokio::net::TcpStream,
    user: &str,
    pass: &str,
    accepted: &AtomicUsize,
    refused: &AtomicUsize,
) -> std::io::Result<()> {
    let (read, mut write) = sock.into_split();
    let mut lines = BufReader::new(read).lines();
    write.write_all(b"+OK mock POP3 ready\r\n").await?;
    let mut given_user: Option<String> = None;
    let mut authed = false;
    while let Some(line) = lines.next_line().await? {
        let (cmd, arg) = match line.split_once(' ') {
            Some((c, a)) => (c.to_ascii_uppercase(), a.to_string()),
            None => (line.to_ascii_uppercase(), String::new()),
        };
        let reply: &[u8] = match cmd.as_str() {
            "CAPA" => b"+OK\r\nUSER\r\nUIDL\r\n.\r\n",
            "USER" => {
                given_user = Some(arg);
                b"+OK\r\n"
            }
            "PASS" => {
                if given_user.as_deref() == Some(user) && arg == pass {
                    authed = true;
                    accepted.fetch_add(1, Ordering::SeqCst);
                    b"+OK logged in\r\n"
                } else {
                    refused.fetch_add(1, Ordering::SeqCst);
                    b"-ERR [AUTH] invalid credentials\r\n"
                }
            }
            "STAT" if authed => b"+OK 0 0\r\n",
            "UIDL" | "LIST" if authed => b"+OK\r\n.\r\n",
            "NOOP" if authed => b"+OK\r\n",
            "QUIT" => {
                write.write_all(b"+OK bye\r\n").await?;
                return Ok(());
            }
            _ => b"-ERR unsupported\r\n",
        };
        write.write_all(reply).await?;
    }
    Ok(())
}
