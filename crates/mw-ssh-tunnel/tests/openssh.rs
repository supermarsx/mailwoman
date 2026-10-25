//! Optional independent OpenSSH acceptance; synthetic fixture key directory only.
use mw_net::{Dialer, DirectDialer};
use mw_ssh_tunnel::{Config, HostKeyPin, PinnedTarget, SshDialer, Tunnel, import_private_key};
use russh::keys::{HashAlg, PublicKey};
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[tokio::test]
async fn independent_openssh_all_required_key_types() {
    let Ok(endpoint) = std::env::var("MW_SSH_OPENSSH_PROOF") else {
        return;
    };
    let keys = std::env::var("MW_SSH_OPENSSH_KEYS").expect("synthetic OpenSSH keys directory");
    let address: std::net::SocketAddr = endpoint.parse().unwrap();
    for name in ["ed25519", "p256", "p384", "rsa"] {
        let pem = std::fs::read_to_string(std::path::Path::new(&keys).join(name)).unwrap();
        let public = PublicKey::from_openssh(
            &std::fs::read_to_string(std::path::Path::new(&keys).join(format!("{name}.pub")))
                .unwrap(),
        )
        .unwrap();
        let (credential, _, _) = import_private_key(&pem, None).unwrap();
        let tunnel = Tunnel::new(
            Config {
                host: address.ip().to_string(),
                port: address.port(),
                username: "root".into(),
                pins: vec![HostKeyPin {
                    algorithm: public.algorithm().to_string(),
                    sha256: public.fingerprint(HashAlg::Sha256).to_string(),
                }],
                credential,
                connect_timeout: Duration::from_secs(10),
                keepalive: Duration::from_secs(30),
                max_channels: 2,
            },
            Arc::new(DirectDialer),
        )
        .unwrap();
        let dialer = SshDialer::new(
            tunnel,
            vec![PinnedTarget {
                host: "localhost".into(),
                port: 8080,
                address: "127.0.0.1".parse().unwrap(),
            }],
        )
        .unwrap();
        let mut stream = dialer.dial("localhost", 8080).await.unwrap();
        stream
            .write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let mut response = String::new();
        tokio::time::timeout(
            Duration::from_secs(10),
            stream.read_to_string(&mut response),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(response.contains("independent OpenSSH tunnel proof"));
        println!("OpenSSH host+user {name}: pinned numeric direct-tcpip HTTP PASS");
    }
}
