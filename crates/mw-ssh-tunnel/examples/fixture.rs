//! Loopback-only integration fixture. Never use these synthetic credentials in production.
use russh::{
    Channel,
    keys::{Algorithm, HashAlg, PrivateKey},
    server,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};

struct Forward {
    target: SocketAddr,
}
impl server::Handler for Forward {
    type Error = russh::Error;
    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<server::Auth, Self::Error> {
        Ok(if user == "proof" && password == "ssh-browser-fixture" {
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
        if host != self.target.ip().to_string() || port != u32::from(self.target.port()) {
            return Ok(());
        }
        let Ok(mut target) = tokio::net::TcpStream::connect(self.target).await else {
            return Ok(());
        };
        reply.accept().await;
        println!("accepted direct-tcpip channel to configured loopback target");
        tokio::spawn(async move {
            let mut stream = channel.into_stream();
            let _ = tokio::io::copy_bidirectional(&mut stream, &mut target).await;
        });
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind: SocketAddr = std::env::var("MW_SSH_FIXTURE_BIND")
        .unwrap_or_else(|_| "127.0.0.1:42320".into())
        .parse()?;
    let target: SocketAddr = std::env::var("MW_SSH_FIXTURE_TARGET")
        .unwrap_or_else(|_| "127.0.0.1:42421".into())
        .parse()?;
    if !bind.ip().is_loopback() || !target.ip().is_loopback() {
        return Err("fixture must stay on loopback".into());
    }
    let host = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519)?;
    println!(
        "{{\"algorithm\":\"ssh-ed25519\",\"sha256\":\"{}\",\"port\":{}}}",
        host.public_key().fingerprint(HashAlg::Sha256),
        bind.port()
    );
    let config = Arc::new(server::Config {
        keys: vec![host],
        auth_rejection_time: Duration::from_millis(50),
        auth_rejection_time_initial: Some(Duration::from_millis(50)),
        ..Default::default()
    });
    let listener = tokio::net::TcpListener::bind(bind).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let config = config.clone();
        tokio::spawn(async move {
            if let Ok(session) = server::run_stream(config, stream, Forward { target }).await {
                let _ = session.await;
            }
        });
    }
}
