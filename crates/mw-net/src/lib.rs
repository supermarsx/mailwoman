//! The outbound dial seam (t32-ext §J).
//!
//! Every protocol client in the workspace used to call `TcpStream::connect`
//! itself, so nothing could route an upstream connection anywhere but straight
//! out of the host. This crate holds the one shape they share instead:
//!
//! * [`AsyncStream`] / [`BoxStream`]: any bidirectional byte stream, boxed so a
//!   TCP socket, a channel inside an SSH session or a test pipe look the same to
//!   the protocol code and to `tokio_rustls` on top of it;
//! * [`Dialer`]: "give me a stream to `host:port`". [`DirectDialer`] is today's
//!   behaviour (a TCP connect with `TCP_NODELAY`); `mw-ssh-tunnel` provides one
//!   that opens a `direct-tcpip` channel instead.
//!
//! TLS is never the dialer's business: the protocol crate wraps whatever stream
//! the dialer returns, with its own `ring` configuration and minimum version, and
//! validates the certificate against the mail server's name. A dialer that routes
//! through another host therefore cannot read or alter the mail session.
//!
//! The crate depends on tokio only, because the protocol crates depend on it.

use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// Any bidirectional async byte stream a protocol connection can run over.
///
/// Blanket-implemented, so `TcpStream`, `tokio_rustls` streams, `tokio::io::duplex`
/// halves and SSH channel streams all qualify.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> AsyncStream for T {}

/// A boxed [`AsyncStream`]: what a [`Dialer`] returns and what the protocol
/// crates' plaintext and TLS stream types are built over.
pub type BoxStream = Box<dyn AsyncStream>;

/// A boxed, `Send` future (the object-safe return type of [`Dialer::dial`]).
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Explicit endpoint verification settings. Protocol clients still terminate TLS
/// themselves and enforce their configured minimum version. No insecure verifier.
#[derive(Debug, Clone, Default)]
pub struct TlsOptions {
    pub server_name: Option<String>,
    /// Additional public trust anchors in DER form; malformed certificates fail closed.
    pub additional_roots: Vec<Vec<u8>>,
}

/// Opens the byte stream to an upstream server.
///
/// `host` is the name or address literal the account is configured with and
/// `port` its port. An implementation may refuse a target (an SSH tunnel refuses
/// anything outside its allowed set); it returns an `io::Error` then, and the
/// protocol crate reports it like any other connect failure.
pub trait Dialer: Send + Sync + fmt::Debug {
    /// Open a stream to `host:port`.
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>>;
}

/// A shared dialer handle, as the protocol crates' `*_via` constructors take it.
pub type SharedDialer = Arc<dyn Dialer>;

/// A direct TCP connection from this host: exactly what every protocol crate did
/// before the seam existed (`TcpStream::connect((host, port))`, then
/// `TCP_NODELAY`, best effort).
#[derive(Debug, Clone, Copy, Default)]
pub struct DirectDialer;

impl DirectDialer {
    /// The same connect as [`Dialer::dial`], returning the concrete socket.
    pub async fn connect_tcp(host: &str, port: u16) -> io::Result<TcpStream> {
        let tcp = TcpStream::connect((host, port)).await?;
        tcp.set_nodelay(true).ok();
        Ok(tcp)
    }
}

impl Dialer for DirectDialer {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
        Box::pin(async move {
            let tcp = Self::connect_tcp(host, port).await?;
            Ok(Box::new(tcp) as BoxStream)
        })
    }
}

/// Dial through `dialer` when one is given, else directly.
///
/// The protocol crates call this so "no dialer configured" and "the direct
/// dialer" are the same code path.
pub async fn dial(dialer: Option<&dyn Dialer>, host: &str, port: u16) -> io::Result<BoxStream> {
    match dialer {
        Some(d) => d.dial(host, port).await,
        None => DirectDialer.dial(host, port).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn the_direct_dialer_connects_over_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            s.write_all(b"* OK hello\r\n").await.unwrap();
        });
        let mut stream = dial(None, "127.0.0.1", addr.port()).await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"* OK hello\r\n");
        server.await.unwrap();
    }

    /// A dialer that records its targets and answers from an in-memory pipe.
    #[derive(Debug, Default)]
    struct Recording(Mutex<Vec<(String, u16)>>);

    impl Dialer for Recording {
        fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
            Box::pin(async move {
                self.0.lock().unwrap().push((host.to_string(), port));
                if host == "refused.example" {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "target not allowed",
                    ));
                }
                let (a, mut b) = tokio::io::duplex(64);
                tokio::spawn(async move {
                    b.write_all(b"via pipe").await.unwrap();
                });
                Ok(Box::new(a) as BoxStream)
            })
        }
    }

    #[tokio::test]
    async fn a_given_dialer_is_used_instead_of_tcp_and_can_refuse() {
        let d = Recording::default();
        let mut s = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            dial(Some(&d), "mail.example", 993),
        )
        .await
        .expect("the dialer answers at once")
        .unwrap();
        let mut buf = [0u8; 8];
        s.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"via pipe");
        let err = dial(Some(&d), "refused.example", 993)
            .await
            .err()
            .expect("refused");
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            *d.0.lock().unwrap(),
            vec![
                ("mail.example".to_string(), 993),
                ("refused.example".to_string(), 993)
            ]
        );
    }
}
