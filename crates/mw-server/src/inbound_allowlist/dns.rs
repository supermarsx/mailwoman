use super::config::Dns;
use hickory_resolver::{
    TokioResolver,
    config::{LookupIpStrategy, NameServerConfig, ResolveHosts, ResolverConfig},
    net::runtime::TokioRuntimeProvider,
};
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

pub(super) struct Answer {
    pub ips: Vec<IpAddr>,
    pub until: Instant,
}
pub(super) fn resolver(config: &Dns) -> Result<TokioResolver, &'static str> {
    let mut builder = if config.servers.is_empty() {
        TokioResolver::builder_tokio().map_err(|_| "resolverUnavailable")?
    } else {
        let mut servers = ResolverConfig::default();
        for socket in &config.servers {
            let mut server = NameServerConfig::udp_and_tcp(socket.ip());
            for connection in &mut server.connections {
                connection.port = socket.port();
            }
            servers.add_name_server(server);
        }
        TokioResolver::builder_with_config(servers, TokioRuntimeProvider::default())
    };
    let options = builder.options_mut();
    options.timeout = Duration::from_millis(config.timeout_ms);
    options.attempts = 1;
    options.num_concurrent_reqs = 1;
    options.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    options.use_hosts_file = ResolveHosts::Never;
    options.cache_size = 0;
    options.positive_min_ttl = None;
    options.positive_max_ttl = Some(Duration::from_secs(config.cache_seconds));
    builder.build().map_err(|_| "resolverUnavailable")
}
pub(super) async fn lookup(
    resolver: &TokioResolver,
    config: &Dns,
    host: &str,
) -> Result<Answer, &'static str> {
    let result = tokio::time::timeout(
        Duration::from_millis(config.timeout_ms),
        resolver.lookup_ip(format!("{host}.")),
    )
    .await
    .map_err(|_| "timeout")?
    .map_err(|_| "lookupFailed")?;
    let mut ips = Vec::new();
    for ip in result.iter() {
        let ip = ip.to_canonical();
        if !ips.contains(&ip) {
            ips.push(ip);
        }
        if ips.len() > config.max_addresses {
            return Err("tooManyAddresses");
        }
    }
    if ips.is_empty() {
        return Err("noAddresses");
    }
    Ok(Answer {
        ips,
        until: result
            .valid_until()
            .min(Instant::now() + Duration::from_secs(config.cache_seconds)),
    })
}
