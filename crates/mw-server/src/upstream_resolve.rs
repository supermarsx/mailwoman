//! Which mail server a sign-in goes to (t32-h9).
//!
//! The browser sends an address and a password, nothing else. The server alone
//! decides the upstream: a JMAP session URL in proxy mode, an `imap(s)://` or
//! `pop3(s)://` server in engine mode. A login body that names a server is refused
//! before this module is reached (`lib.rs` `login`, 400 `clientServerNotAllowed`).
//!
//! [`resolve_upstream`] is the one place the choice is made, in this order:
//!
//! 1. the enabled canonical domain's configured upstream, following aliases;
//! 2. the effective `upstream.default` setting, then `MW_UPSTREAM_DEFAULT`
//!    ([`SecurityConfig::upstream_default`]);
//! 3. proxy mode only: a `MW_JMAP_UPSTREAMS` list that names exactly one usable
//!    origin, so a proxy install configured before t32-h9 keeps working (§2.6, the
//!    `MW_JMAP_UPSTREAMS` migration path);
//! 4. otherwise [`ResolveError::NotConfigured`], which the login answers with 404
//!    `addressNotConfigured`.
//!
//! There is no lookup of the address's domain in DNS or autoconfig at sign-in.
//! Detection stays an admin action (§2.6 `POST /admin/domains/{name}/detect`):
//! an anonymous request must not make the server dial hosts a stranger's domain
//! names.

use crate::{AppState, SecurityConfig, engine_mode, upstream_list_origin};

/// The upstream a sign-in uses, in the form the login path hands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Upstream {
    /// Proxy mode: a JMAP server (an origin or a session URL).
    Jmap(String),
    /// Engine mode: an `imap(s)://` or `pop3(s)://` server URL.
    Mail(String),
}

impl Upstream {
    pub(crate) fn url(&self) -> &str {
        match self {
            Upstream::Jmap(u) | Upstream::Mail(u) => u,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolveError {
    /// Nothing the operator configured places this address.
    NotConfigured,
}

/// The domain of a sign-in name: the part after the last `@`, lower-cased.
/// `None` for a bare user name.
pub(crate) fn domain_of(username: &str) -> Option<String> {
    let (_, domain) = username.trim().rsplit_once('@')?;
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    (!domain.is_empty()).then_some(domain)
}

/// Resolve the upstream for `username` (the address the user typed).
pub(crate) async fn resolve_upstream(
    state: &AppState,
    username: &str,
) -> Result<Upstream, ResolveError> {
    resolve_for(&state.store, &state.security, state.engine.is_some(), username).await
}

pub(crate) async fn resolve_for(
    store: &mw_store::Store,
    security: &SecurityConfig,
    engine: bool,
    username: &str,
) -> Result<Upstream, ResolveError> {
    let identity = crate::domain_config::canonical(store, username, None)
        .await
        .map_err(|_| ResolveError::NotConfigured)?;
    let subject = mw_store::settings_policy::SettingSubject {
        domain: &identity.domain,
        account_id: "",
        address: &identity.address,
    };
    let listed =
        crate::settings_registry::effective_for(store, "signin.domainMode", &subject)
            .await
            .map_err(|_| ResolveError::NotConfigured)?
            .is_some_and(|v| v.value == "listed");
    let row = crate::domain_config::configured_row(store, &identity)
        .await
        .map_err(|_| ResolveError::NotConfigured)?;
    match row {
        Some(row) => {
            if !row.enabled {
                return Err(ResolveError::NotConfigured);
            }
            let config = crate::domain_config::UpstreamConfig::parse(&row.upstream_json)
                .map_err(|_| ResolveError::NotConfigured)?;
            if engine {
                if let Some((endpoint, protocol)) = config.mail() {
                    return Ok(Upstream::Mail(endpoint.mail_url(protocol)));
                }
                if config.jmap.is_some() {
                    return Err(ResolveError::NotConfigured);
                }
            } else {
                if let Some(jmap) = config.jmap {
                    return Ok(Upstream::Jmap(jmap.session_url));
                }
                if config.mail().is_some() {
                    return Err(ResolveError::NotConfigured);
                }
            }
        }
        None if listed => return Err(ResolveError::NotConfigured),
        None => {}
    }
    if let Some(default) =
        crate::settings_registry::effective_for(store, "upstream.default", &subject)
            .await
            .map_err(|_| ResolveError::NotConfigured)?
            .and_then(|v| v.value.as_str().map(str::to_string))
    {
        return if fits_mode(engine, &default) {
            Ok(wrap(engine, default))
        } else {
            Err(ResolveError::NotConfigured)
        };
    }
    resolve_from_config(security, engine)
}

/// Steps 2-4: the deployment-wide configuration.
pub(crate) fn resolve_from_config(
    security: &SecurityConfig,
    engine: bool,
) -> Result<Upstream, ResolveError> {
    if let Some(url) = security.upstream_default.as_deref() {
        if fits_mode(engine, url) {
            return Ok(wrap(engine, url.to_string()));
        }
        tracing::warn!(
            "MW_UPSTREAM_DEFAULT={url:?} is not {}; no sign-in can use it",
            if engine {
                "an imap(s):// or pop3(s):// URL, which engine mode needs"
            } else {
                "an http(s) JMAP URL, which proxy mode needs"
            }
        );
        return Err(ResolveError::NotConfigured);
    }
    if !engine && let Some(list) = &security.jmap_upstreams {
        let mut usable = list.iter().filter(|e| upstream_list_origin(e).is_some());
        if let (Some(only), None) = (usable.next(), usable.next()) {
            return Ok(Upstream::Jmap(only.clone()));
        }
    }
    Err(ResolveError::NotConfigured)
}

fn wrap(engine: bool, url: String) -> Upstream {
    if engine {
        Upstream::Mail(url)
    } else {
        Upstream::Jmap(url)
    }
}

/// Whether `url` is an upstream the running mode can use.
fn fits_mode(engine: bool, url: &str) -> bool {
    if engine {
        url.contains("://") && engine_mode::is_mail_url(url)
    } else {
        mw_jmap::client::session_url(url).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn security(default: Option<&str>, list: Option<&[&str]>) -> SecurityConfig {
        SecurityConfig {
            upstream_default: default.map(str::to_string),
            jmap_upstreams: list.map(|l| l.iter().map(|s| s.to_string()).collect()),
            ..SecurityConfig::default()
        }
    }

    #[test]
    fn the_domain_is_the_part_after_the_last_at() {
        assert_eq!(domain_of("Ada@Example.ORG").as_deref(), Some("example.org"));
        assert_eq!(domain_of("a@b@c.test.").as_deref(), Some("c.test"));
        assert_eq!(domain_of("testuser"), None);
        assert_eq!(domain_of("ada@"), None);
    }

    #[test]
    fn the_default_is_used_when_it_fits_the_mode() {
        let s = security(Some("imap://greenmail:3143"), None);
        assert_eq!(
            resolve_from_config(&s, true),
            Ok(Upstream::Mail("imap://greenmail:3143".into()))
        );
        // The same value is no JMAP server.
        assert_eq!(
            resolve_from_config(&s, false),
            Err(ResolveError::NotConfigured)
        );

        let s = security(Some("http://mock:8181/.well-known/jmap"), None);
        assert_eq!(
            resolve_from_config(&s, false),
            Ok(Upstream::Jmap("http://mock:8181/.well-known/jmap".into()))
        );
        assert_eq!(
            resolve_from_config(&s, true),
            Err(ResolveError::NotConfigured)
        );
    }

    #[test]
    fn a_single_listed_origin_stands_in_for_the_default_in_proxy_mode() {
        let one = security(None, Some(&["https://jmap.example.org"]));
        assert_eq!(
            resolve_from_config(&one, false),
            Ok(Upstream::Jmap("https://jmap.example.org".into()))
        );
        assert_eq!(
            resolve_from_config(&one, true),
            Err(ResolveError::NotConfigured)
        );
        // Two usable origins: the server cannot choose, and neither may the client.
        let two = security(None, Some(&["https://a.example", "https://b.example"]));
        assert_eq!(
            resolve_from_config(&two, false),
            Err(ResolveError::NotConfigured)
        );
        // An entry that names no origin does not count.
        let junk = security(None, Some(&["https://a.example", "not a url"]));
        assert_eq!(
            resolve_from_config(&junk, false),
            Ok(Upstream::Jmap("https://a.example".into()))
        );
        // The default wins over the list.
        let both = security(Some("https://d.example"), Some(&["https://a.example"]));
        assert_eq!(
            resolve_from_config(&both, false),
            Ok(Upstream::Jmap("https://d.example".into()))
        );
    }

    #[test]
    fn nothing_configured_places_no_address() {
        assert_eq!(
            resolve_from_config(&security(None, None), false),
            Err(ResolveError::NotConfigured)
        );
        assert_eq!(
            resolve_from_config(&security(None, None), true),
            Err(ResolveError::NotConfigured)
        );
    }
}
