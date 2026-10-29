//! Network, port and security-posture checks for the server diagnostics suite
//! (t39 L3). Summaries carry counts and fixed text only: no host names,
//! addresses, credentials or upstream answers.
use super::CheckResult;
use crate::{AppState, domain_config::TlsMode, domain_selection::BackendConfig};
use axum::http::{Extensions, HeaderMap};
use mw_net::Dialer;
use std::{
    collections::BTreeSet,
    net::IpAddr,
    time::{Duration, Instant},
};

const STEP: Duration = Duration::from_secs(5);
const MAX_PORTS: usize = 32;

fn row(id: &'static str, status: &'static str, start: Instant, summary: String) -> CheckResult {
    CheckResult::timed(id, status, start.elapsed(), summary.into())
}

/// `(host, port)` of `MW_PUBLIC_URL`, the same variable `ExternalBase` reads.
fn public_target() -> Option<(String, u16)> {
    let raw = std::env::var("MW_PUBLIC_URL").ok()?;
    let url = reqwest::Url::parse(raw.trim()).ok()?;
    let host = url.host_str()?.trim_matches(['[', ']']).to_string();
    Some((host, url.port_or_known_default()?))
}

/// Saved, enabled socket endpoints of enabled domains:
/// `(host, port, allow_private, plaintext)`.
pub(super) async fn saved_endpoints(state: &AppState) -> Vec<(String, u16, bool, bool)> {
    let mut out = BTreeSet::new();
    let Ok(rows) = state.store.list_domain_configs_limited(128).await else {
        return Vec::new();
    };
    for row in rows.into_iter().filter(|r| r.enabled) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&row.upstream_json) else {
            continue;
        };
        let endpoints: Vec<crate::domain_config::Endpoint> = if value.get("version").is_some() {
            serde_json::from_value::<BackendConfig>(value)
                .map(|c| {
                    c.backends
                        .into_iter()
                        .filter(|b| b.enabled)
                        .filter_map(|b| b.endpoint)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            crate::domain_config::UpstreamConfig::parse(&row.upstream_json)
                .map(|c| {
                    [c.imap, c.pop3, c.smtp, c.sieve]
                        .into_iter()
                        .flatten()
                        .collect()
                })
                .unwrap_or_default()
        };
        for e in endpoints {
            let plaintext = match &e.security {
                Some(security) => security.allowed.contains(&TlsMode::None),
                None => e.tls == TlsMode::None,
            };
            out.insert((
                e.host.trim_matches(['[', ']']).to_ascii_lowercase(),
                e.port,
                e.allow_private,
                plaintext,
            ));
        }
    }
    out.into_iter().collect()
}

async fn public_dns_and_port() -> [CheckResult; 2] {
    let start = Instant::now();
    let Some((host, port)) = public_target() else {
        let skip = |id| {
            row(
                id,
                "skipped",
                start,
                "MW_PUBLIC_URL is not set, so there is no public name to check".into(),
            )
        };
        return [skip("network-public-dns"), skip("network-public-port")];
    };
    let resolved = tokio::time::timeout(STEP, tokio::net::lookup_host((host.as_str(), port)))
        .await
        .ok()
        .and_then(Result::ok)
        .map(|addresses| addresses.collect::<Vec<_>>())
        .unwrap_or_default();
    let dns = if resolved.is_empty() {
        row(
            "network-public-dns",
            "fail",
            start,
            "The host name in MW_PUBLIC_URL does not resolve from this server".into(),
        )
    } else {
        row(
            "network-public-dns",
            "pass",
            start,
            format!(
                "The host name in MW_PUBLIC_URL resolves to {} address(es)",
                resolved.len()
            ),
        )
    };
    let start = Instant::now();
    let mut connected = false;
    for address in resolved.iter().take(4) {
        if tokio::time::timeout(STEP, tokio::net::TcpStream::connect(address))
            .await
            .is_ok_and(|r| r.is_ok())
        {
            connected = true;
            break;
        }
    }
    let port_row = if resolved.is_empty() {
        row(
            "network-public-port",
            "skipped",
            start,
            "Not checked because the public name does not resolve".into(),
        )
    } else if connected {
        row(
            "network-public-port",
            "pass",
            start,
            format!("Port {port} of the public URL accepts TCP connections from this server"),
        )
    } else {
        row(
            "network-public-port",
            "warn",
            start,
            format!(
                "Port {port} of the public URL refused or timed out from this server; check the firewall, \
                 the reverse proxy, or NAT hairpinning"
            ),
        )
    };
    [dns, port_row]
}

async fn backend_ports(state: &AppState, allowed: bool) -> CheckResult {
    let start = Instant::now();
    if !allowed {
        return row(
            "network-backend-ports",
            "skipped",
            start,
            "Needs the domains.upstream_write capability".into(),
        );
    }
    let mut targets: Vec<(String, u16, bool)> = saved_endpoints(state)
        .await
        .into_iter()
        .map(|(h, p, private, _)| (h, p, private))
        .collect();
    targets.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    if targets.is_empty() {
        return row(
            "network-backend-ports",
            "skipped",
            start,
            "No saved socket backends to check".into(),
        );
    }
    let limited = targets.len() > MAX_PORTS;
    targets.truncate(MAX_PORTS);
    let total = targets.len();
    let mut work = tokio::task::JoinSet::new();
    for (host, port, private) in targets {
        work.spawn(async move {
            // The same address policy as a sign-in; TCP only, nothing is written.
            let dialer = crate::domain_transport::GuardedDialer::new(&host, port, private);
            tokio::time::timeout(STEP, dialer.dial(&host, port))
                .await
                .is_ok_and(|r| r.is_ok())
        });
    }
    let mut open = 0;
    while let Some(result) = work.join_next().await {
        open += usize::from(result.unwrap_or(false));
    }
    let suffix = if limited {
        format!(" (first {MAX_PORTS} only)")
    } else {
        String::new()
    };
    row(
        "network-backend-ports",
        if open == total { "pass" } else { "warn" },
        start,
        format!(
            "{open} of {total} saved backend host:port pairs accept a direct TCP connection{suffix}; \
             the backends suite checks the configured route and TLS"
        ),
    )
}

pub(super) async fn network(state: &AppState, upstream_capability: bool) -> Vec<CheckResult> {
    let (public, ports) = tokio::join!(
        public_dns_and_port(),
        backend_ports(state, upstream_capability)
    );
    let mut out = Vec::from(public);
    out.push(ports);
    out
}

fn deployment_summary(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "https" => ("security-https", "Requests reach this server over HTTPS"),
        "secureCookies" => (
            "security-secure-cookies",
            "Session cookies carry the Secure attribute",
        ),
        "hsts" => ("security-hsts", "Strict-Transport-Security is sent"),
        "csrf" => ("security-csrf", "Strict CSRF checking is on"),
        "adminFactors" => (
            "security-admin-factors",
            "Every administrator has a second factor",
        ),
        "signinPacing" => (
            "security-signin-pacing",
            "A retry cooldown follows a failed sign-in",
        ),
        "credentialLocks" => (
            "security-credential-locks",
            "Repeated sign-in failures lock the credential",
        ),
        "geography" => (
            "security-geography",
            "Sign-in geography policy has the data it needs",
        ),
        "renderIsolation" => (
            "security-render-isolation",
            "Message rendering runs in the required isolated worker",
        ),
        _ => return None,
    })
}

/// Classify a fail2ban `rhost` for the current request.
pub(super) fn fail2ban_address_status(source: Option<IpAddr>) -> (&'static str, &'static str) {
    match source {
        None => (
            "fail",
            "Requests arrive without a client address, so no fail2ban line can name a source",
        ),
        Some(ip) if ip.is_loopback() || !mw_egress::ip_allowed(&ip) => (
            "warn",
            "The address a fail2ban line would name for this request is loopback or private; \
             behind a reverse proxy, set MW_TRUSTED_PROXIES or fail2ban bans the proxy",
        ),
        Some(_) => (
            "pass",
            "fail2ban lines name the public client address of this request",
        ),
    }
}

pub(super) async fn security(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Vec<CheckResult> {
    let mut out = Vec::new();
    let start = Instant::now();
    match crate::deployment_security::checks(state, headers, ext).await {
        Ok(checks) => {
            for check in checks {
                let Some((id, summary)) = deployment_summary(check.id) else {
                    continue;
                };
                let status = match check.status {
                    "pass" => "pass",
                    "warning" => "warn",
                    _ => continue,
                };
                out.push(row(id, status, start, summary.into()));
            }
        }
        Err(_) => out.push(row(
            "security-deployment",
            "fail",
            start,
            "Deployment security settings could not be read".into(),
        )),
    }
    let start = Instant::now();
    let saved = saved_endpoints(state).await;
    let public_plain = saved.iter().filter(|e| e.3 && !e.2).count();
    let private_plain = saved.iter().filter(|e| e.3 && e.2).count();
    out.push(if public_plain > 0 {
        row(
            "security-plaintext-backends",
            "fail",
            start,
            format!(
                "{public_plain} saved backend(s) allow passwords without TLS on a non-private network"
            ),
        )
    } else if private_plain > 0 {
        row(
            "security-plaintext-backends",
            "warn",
            start,
            format!("{private_plain} saved private-network backend(s) allow passwords without TLS"),
        )
    } else {
        row(
            "security-plaintext-backends",
            "pass",
            start,
            "No saved backend allows passwords without TLS".into(),
        )
    });
    let start = Instant::now();
    let (status, summary) =
        fail2ban_address_status(crate::scope_mw::proxy::client_ip(headers, ext));
    out.push(row(
        "fail2ban-client-address",
        status,
        start,
        summary.into(),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fail2ban_address_classification() {
        assert_eq!(fail2ban_address_status(None).0, "fail");
        assert_eq!(
            fail2ban_address_status(Some("127.0.0.1".parse().unwrap())).0,
            "warn"
        );
        assert_eq!(
            fail2ban_address_status(Some("10.1.2.3".parse().unwrap())).0,
            "warn"
        );
        assert_eq!(
            fail2ban_address_status(Some("8.8.8.8".parse().unwrap())).0,
            "pass"
        );
    }
    #[test]
    fn every_reported_deployment_check_has_a_fixed_summary() {
        for id in [
            "https",
            "secureCookies",
            "hsts",
            "csrf",
            "adminFactors",
            "signinPacing",
            "credentialLocks",
            "geography",
            "renderIsolation",
        ] {
            assert!(deployment_summary(id).is_some(), "{id}");
        }
        assert!(deployment_summary("httpAdmission").is_none());
    }
}
