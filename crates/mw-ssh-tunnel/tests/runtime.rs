use mw_net::{Dialer, DirectDialer};
use mw_ssh_tunnel::{Config, Credential, HostKeyPin, PinnedTarget, SshDialer, Tunnel};
use russh::{
    Channel,
    keys::{Algorithm, HashAlg, PrivateKey, PublicKey},
    server,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
struct Echo {
    user: PublicKey,
    auth: Arc<AtomicUsize>,
    channels: Arc<AtomicUsize>,
}

#[tokio::test]
async fn reconnects_after_authenticated_server_disconnect() {
    let host = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let user = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let pin = HostKeyPin {
        algorithm: "ssh-ed25519".into(),
        sha256: host.public_key().fingerprint(HashAlg::Sha256).to_string(),
    };
    let auth = Arc::new(AtomicUsize::new(0));
    let channels = Arc::new(AtomicUsize::new(0));
    let handler = Echo {
        user: user.public_key().clone(),
        auth: auth.clone(),
        channels,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = Arc::new(server::Config {
        keys: vec![host],
        auth_rejection_time: Duration::ZERO,
        auth_rejection_time_initial: Some(Duration::ZERO),
        ..Default::default()
    });
    let (tx, mut rx) = tokio::sync::mpsc::channel(2);
    let task = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let session = server::run_stream(config.clone(), stream, handler.clone())
                .await
                .unwrap();
            tx.send(session.handle()).await.unwrap();
            tokio::spawn(async move {
                let _ = session.await;
            });
        }
    });
    let tunnel = Tunnel::new(
        Config {
            host: "127.0.0.1".into(),
            port,
            username: "proof".into(),
            pins: vec![pin],
            credential: Credential::PrivateKey(Arc::new(user)),
            connect_timeout: Duration::from_secs(2),
            keepalive: Duration::from_secs(30),
            max_channels: 2,
        },
        Arc::new(DirectDialer),
    )
    .unwrap();
    let dialer = SshDialer::new(
        tunnel,
        vec![PinnedTarget {
            host: "service".into(),
            port: 8080,
            address: "127.0.0.1".parse().unwrap(),
        }],
    )
    .unwrap();
    let first = dialer.dial("service", 8080).await.unwrap();
    let handle = rx.recv().await.unwrap();
    drop(first);
    handle
        .disconnect(
            russh::Disconnect::ByApplication,
            "fixture restart".into(),
            String::new(),
        )
        .await
        .unwrap();
    let second = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match dialer.dial("service", 8080).await {
                Ok(stream) => break stream,
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap();
    drop(second);
    assert_eq!(auth.load(Ordering::SeqCst), 2);
    task.abort();
}

#[tokio::test]
async fn failed_connections_back_off_without_repeated_dialing() {
    #[derive(Debug)]
    struct Failing(Arc<AtomicUsize>);
    impl Dialer for Failing {
        fn dial<'a>(
            &'a self,
            _: &'a str,
            _: u16,
        ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
            Box::pin(async move {
                self.0.fetch_add(1, Ordering::SeqCst);
                Err(std::io::Error::from(std::io::ErrorKind::ConnectionRefused))
            })
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let user = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let tunnel = Tunnel::new(
        Config {
            host: "jump".into(),
            port: 22,
            username: "proof".into(),
            pins: vec![HostKeyPin {
                algorithm: "ssh-ed25519".into(),
                sha256: user.public_key().fingerprint(HashAlg::Sha256).to_string(),
            }],
            credential: Credential::PrivateKey(Arc::new(user)),
            connect_timeout: Duration::from_secs(2),
            keepalive: Duration::from_secs(30),
            max_channels: 2,
        },
        Arc::new(Failing(calls.clone())),
    )
    .unwrap();
    let dialer = SshDialer::new(
        tunnel,
        vec![PinnedTarget {
            host: "service".into(),
            port: 8080,
            address: "127.0.0.1".parse().unwrap(),
        }],
    )
    .unwrap();
    assert_eq!(
        dialer
            .dial("service", 8080)
            .await
            .err()
            .unwrap()
            .to_string(),
        "jump_connect_failed"
    );
    assert_eq!(
        dialer
            .dial("service", 8080)
            .await
            .err()
            .unwrap()
            .to_string(),
        "reconnect_backoff"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(
        dialer
            .dial("service", 8080)
            .await
            .err()
            .unwrap()
            .to_string(),
        "jump_connect_failed"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
impl server::Handler for Echo {
    type Error = russh::Error;
    async fn auth_publickey(
        &mut self,
        user: &str,
        key: &PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        self.auth.fetch_add(1, Ordering::SeqCst);
        Ok(if user == "proof" && key == &self.user {
            server::Auth::Accept
        } else {
            server::Auth::reject()
        })
    }
    async fn channel_open_direct_tcpip(
        &mut self,
        channel: Channel<server::Msg>,
        host: &str,
        port: u32,
        _: &str,
        _: u32,
        reply: server::ChannelOpenHandle,
        _: &mut server::Session,
    ) -> Result<(), Self::Error> {
        self.channels.fetch_add(1, Ordering::SeqCst);
        if host != "127.0.0.1" || port != 8080 {
            return Ok(());
        }
        reply.accept().await;
        tokio::spawn(async move {
            let mut stream = channel.into_stream();
            let mut bytes = [0; 256];
            while let Ok(n) = stream.read(&mut bytes).await {
                if n == 0 {
                    break;
                }
                if stream.write_all(&bytes[..n]).await.is_err() {
                    break;
                }
            }
        });
        Ok(())
    }
}
async fn fixture(
    wrong_pin: bool,
) -> (
    SshDialer,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let host = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let user = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let auth = Arc::new(AtomicUsize::new(0));
    let channels = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let pin = HostKeyPin {
        algorithm: "ssh-ed25519".into(),
        sha256: if wrong_pin {
            user.public_key()
        } else {
            host.public_key()
        }
        .fingerprint(HashAlg::Sha256)
        .to_string(),
    };
    let handler = Echo {
        user: user.public_key().clone(),
        auth: auth.clone(),
        channels: channels.clone(),
    };
    let task = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        if let Ok(connection) = server::run_stream(
            Arc::new(server::Config {
                keys: vec![host],
                auth_rejection_time: Duration::ZERO,
                auth_rejection_time_initial: Some(Duration::ZERO),
                ..Default::default()
            }),
            socket,
            handler,
        )
        .await
        {
            let _ = connection.await;
        }
    });
    let tunnel = Tunnel::new(
        Config {
            host: "127.0.0.1".into(),
            port,
            username: "proof".into(),
            pins: vec![pin],
            credential: Credential::PrivateKey(Arc::new(user)),
            connect_timeout: Duration::from_millis(500),
            keepalive: Duration::from_secs(30),
            max_channels: 1,
        },
        Arc::new(DirectDialer),
    )
    .unwrap();
    (
        SshDialer::new(
            tunnel,
            vec![
                PinnedTarget {
                    host: "service.example.test".into(),
                    port: 443,
                    address: "127.0.0.1".parse().unwrap(),
                },
                PinnedTarget {
                    host: "service.example.test".into(),
                    port: 8080,
                    address: "127.0.0.1".parse().unwrap(),
                },
            ],
        )
        .unwrap(),
        auth,
        channels,
        task,
    )
}
#[tokio::test]
async fn host_pin_refused_before_any_authentication() {
    let (dialer, auth, channels, task) = fixture(true).await;
    let error = dialer
        .dial("service.example.test", 8080)
        .await
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "host_key_mismatch");
    assert_eq!(auth.load(Ordering::SeqCst), 0);
    assert_eq!(channels.load(Ordering::SeqCst), 0);
    task.abort();
}
#[tokio::test]
async fn exact_targets_pool_and_channel_limit() {
    let (dialer, auth, channels, task) = fixture(false).await;
    assert_eq!(
        dialer
            .dial("other.example.test", 8080)
            .await
            .err()
            .unwrap()
            .to_string(),
        "target_not_allowed"
    );
    assert_eq!(auth.load(Ordering::SeqCst), 0);
    let mut first = dialer.dial("service.example.test", 8080).await.unwrap();
    first.write_all(b"actual channel bytes").await.unwrap();
    let mut reply = [0; 20];
    first.read_exact(&mut reply).await.unwrap();
    assert_eq!(&reply, b"actual channel bytes");
    assert_eq!(
        dialer
            .dial("service.example.test", 8080)
            .await
            .err()
            .unwrap()
            .to_string(),
        "channel_limit"
    );
    drop(first);
    let _second = dialer.dial("service.example.test", 8080).await.unwrap();
    assert_eq!(auth.load(Ordering::SeqCst), 1);
    assert_eq!(channels.load(Ordering::SeqCst), 2);
    task.abort();
}
