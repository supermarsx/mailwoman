//! Explicit role-based backend selection; selection never retries credentials.
//!
//! With `failover` on, the enabled mail backends of the selected protocol form an
//! ordered chain that [`failover`] walks on connect or TLS failures only. Each
//! sign-in or reconnect sends credentials to one backend at most.
pub(crate) mod failover;
use crate::{
    domain_config::{DavEndpoints, Endpoint, JmapEndpoint, UpstreamConfig},
    username_format::UsernameFormat,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ServiceRole {
    Mail,
    Submission,
    Rules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Protocol {
    Imap,
    Pop3,
    Jmap,
    Ews,
    Smtp,
    Sieve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Backend {
    pub id: String,
    pub role: ServiceRole,
    pub protocol: Protocol,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub preferred: bool,
    #[serde(default = "priority")]
    pub priority: u16,
    #[serde(default)]
    pub endpoint: Option<Endpoint>,
    #[serde(default)]
    pub session_url: Option<String>,
    #[serde(default)]
    pub ews: Option<crate::domain_provider::config::EwsConfig>,
    #[serde(default)]
    pub username: UsernameFormat,
}
fn enabled() -> bool {
    true
}
fn priority() -> u16 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BackendConfig {
    pub version: u8,
    pub backends: Vec<Backend>,
    #[serde(default)]
    pub dav: Option<DavEndpoints>,
    /// Try the next enabled mail backend when the selected one cannot be reached
    /// (DNS, connect, timeout, TLS or certificate failure). Never after an
    /// authentication answer.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub failover: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Selection {
    pub mail: Option<Backend>,
    pub submission: Option<Backend>,
    pub rules: Option<Backend>,
    /// Failover chain after `mail`, in order. Omitted when empty so that the
    /// binding of a configuration without failover is unchanged.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mail_fallbacks: Vec<Backend>,
}

impl BackendConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 2 || self.backends.len() > 32 {
            return Err("backend lists require version 2 and at most 32 endpoints".into());
        }
        if let Some(dav) = &self.dav {
            for url in [&dav.caldav_url, &dav.carddav_url].into_iter().flatten() {
                crate::domain_config::validate_http_url(url)?;
            }
        }
        let mut ids = HashSet::new();
        for backend in &self.backends {
            if backend.id.is_empty()
                || backend.id.len() > 64
                || !backend
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                || !ids.insert(&backend.id)
            {
                return Err(
                    "backend IDs must be unique, 1–64 letters, digits, hyphens or underscores"
                        .into(),
                );
            }
            backend.username.validate()?;
            match (backend.role, backend.protocol) {
                (
                    ServiceRole::Mail,
                    Protocol::Imap | Protocol::Pop3 | Protocol::Jmap | Protocol::Ews,
                )
                | (ServiceRole::Submission, Protocol::Smtp)
                | (ServiceRole::Rules, Protocol::Sieve) => {}
                _ => return Err("the backend protocol does not implement its service role".into()),
            }
            if backend.protocol != Protocol::Ews && backend.ews.is_some() {
                return Err("EWS configuration requires the EWS mail protocol".into());
            }
            if backend.protocol == Protocol::Ews {
                if backend.endpoint.is_some() || backend.session_url.is_some() {
                    return Err("EWS uses its configured HTTPS endpoint only".into());
                }
                backend
                    .ews
                    .as_ref()
                    .ok_or("EWS needs its endpoint configuration")?
                    .validate()?;
            } else if backend.protocol == Protocol::Jmap {
                if backend.username != UsernameFormat::FullEmail {
                    return Err(
                        "JMAP username formatting is not supported by the current proxy transport"
                            .into(),
                    );
                }
                if backend.endpoint.is_some() {
                    return Err("JMAP uses a session URL, not a mail socket endpoint".into());
                }
                crate::domain_config::validate_http_url(
                    backend
                        .session_url
                        .as_deref()
                        .ok_or("JMAP needs a session URL")?,
                )?;
            } else {
                if backend.session_url.is_some() {
                    return Err("mail socket protocols cannot use a JMAP session URL".into());
                }
                backend
                    .endpoint
                    .as_ref()
                    .ok_or("a socket backend needs its endpoint")?
                    .validate_for(match backend.protocol {
                        Protocol::Imap => "imap",
                        Protocol::Pop3 => "pop3",
                        Protocol::Smtp => "smtp",
                        Protocol::Sieve => "sieve",
                        Protocol::Jmap | Protocol::Ews => unreachable!(),
                    })?;
            }
            if backend.preferred && !backend.enabled {
                return Err("a disabled backend cannot be preferred".into());
            }
        }
        for role in [
            ServiceRole::Mail,
            ServiceRole::Submission,
            ServiceRole::Rules,
        ] {
            if self
                .backends
                .iter()
                .filter(|b| b.role == role && b.preferred)
                .count()
                > 1
            {
                return Err("choose at most one preferred backend per service role".into());
            }
        }
        if self
            .selected(ServiceRole::Mail)
            .is_some_and(|backend| backend.protocol == Protocol::Ews)
            && (self.selected(ServiceRole::Submission).is_some()
                || self.selected(ServiceRole::Rules).is_some())
        {
            return Err(
                "the EWS backend owns submission and cannot use a separate SMTP or Sieve backend"
                    .into(),
            );
        }
        if self.failover
            && let Some(primary) = self.selected(ServiceRole::Mail)
        {
            if !matches!(primary.protocol, Protocol::Imap | Protocol::Pop3) {
                return Err("failover applies to IMAP and POP3 mail backends only".into());
            }
            let policy = |b: &Backend| {
                b.endpoint.as_ref().map(|e| {
                    serde_json::json!([e.tls, e.auth, e.security, e.authentication, b.username])
                })
            };
            for backend in self.same_protocol_chain(primary) {
                if policy(backend) != policy(primary) {
                    return Err(
                        "failover backends must share the selected backend's connection security, authentication and username format"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }

    fn same_protocol_chain<'a>(&'a self, primary: &'a Backend) -> Vec<&'a Backend> {
        let mut chain: Vec<&Backend> = self
            .backends
            .iter()
            .filter(|b| {
                b.role == ServiceRole::Mail
                    && b.enabled
                    && b.protocol == primary.protocol
                    && b.id != primary.id
            })
            .collect();
        chain.sort_by(|a, b| (a.priority, &a.id).cmp(&(b.priority, &b.id)));
        chain
    }

    /// The mail backends tried after the selected one, bounded so that one
    /// connection attempt touches at most [`failover::MAX_BACKENDS`] backends.
    pub fn mail_fallbacks(&self) -> Vec<&Backend> {
        if !self.failover {
            return Vec::new();
        }
        let Some(primary) = self.selected(ServiceRole::Mail) else {
            return Vec::new();
        };
        if !matches!(primary.protocol, Protocol::Imap | Protocol::Pop3) {
            return Vec::new();
        }
        let mut chain = self.same_protocol_chain(primary);
        chain.truncate(failover::MAX_BACKENDS - 1);
        chain
    }

    pub fn selected(&self, role: ServiceRole) -> Option<&Backend> {
        self.backends
            .iter()
            .filter(|b| b.role == role && b.enabled)
            .min_by(|a, b| {
                (!a.preferred, a.priority, &a.id).cmp(&(!b.preferred, b.priority, &b.id))
            })
    }

    pub fn resolve(&self) -> Result<UpstreamConfig, String> {
        self.validate()?;
        let selection = Selection {
            mail: self.selected(ServiceRole::Mail).cloned(),
            submission: self.selected(ServiceRole::Submission).cloned(),
            rules: self.selected(ServiceRole::Rules).cloned(),
            mail_fallbacks: self.mail_fallbacks().into_iter().cloned().collect(),
        };
        let mut result = UpstreamConfig {
            dav: self.dav.clone(),
            ..UpstreamConfig::default()
        };
        if let Some(mail) = &selection.mail {
            match mail.protocol {
                Protocol::Imap => result.imap = mail.endpoint.clone(),
                Protocol::Pop3 => result.pop3 = mail.endpoint.clone(),
                Protocol::Ews => result.ews = mail.ews.clone(),
                Protocol::Jmap => {
                    result.jmap = Some(JmapEndpoint {
                        session_url: mail.session_url.clone().ok_or("JMAP needs a session URL")?,
                    })
                }
                _ => return Err("invalid mail backend".into()),
            }
        }
        result.smtp = selection
            .submission
            .as_ref()
            .and_then(|b| b.endpoint.clone());
        result.sieve = selection.rules.as_ref().and_then(|b| b.endpoint.clone());
        result.mail_chain = selection
            .mail
            .iter()
            .chain(&selection.mail_fallbacks)
            .filter_map(|b| b.endpoint.clone().map(|e| (b.id.clone(), e)))
            .collect();
        if result.mail_chain.len() < 2 {
            result.mail_chain.clear();
        }
        result.selection = Some(selection);
        Ok(result)
    }
}

impl Selection {
    pub fn backend(&self, role: ServiceRole) -> Option<&Backend> {
        match role {
            ServiceRole::Mail => self.mail.as_ref(),
            ServiceRole::Submission => self.submission.as_ref(),
            ServiceRole::Rules => self.rules.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn config(rows: serde_json::Value) -> BackendConfig {
        serde_json::from_value(json!({ "version": 2, "backends": rows })).unwrap()
    }
    #[test]
    fn ews_requires_one_explicit_https_mail_authority() {
        let row = json!({"id":"exchange","role":"mail","protocol":"ews","ews":{
            "endpointUrl":"https://exchange.example/EWS/Exchange.asmx","authentication":"basic"
        }});
        let selected = config(json!([row.clone()])).resolve().unwrap();
        assert_eq!(
            selected.ews.unwrap().endpoint_url,
            "https://exchange.example/EWS/Exchange.asmx"
        );
        for replacement in [
            json!({"id":"exchange","role":"submission","protocol":"ews","ews":row["ews"]}),
            json!({"id":"exchange","role":"mail","protocol":"ews"}),
            json!({"id":"exchange","role":"mail","protocol":"ews","sessionUrl":"https://elsewhere.example","ews":row["ews"]}),
            json!({"id":"exchange","role":"mail","protocol":"imap","ews":row["ews"],"endpoint":{"host":"mail.example","port":993,"tls":"implicit"}}),
        ] {
            assert!(config(json!([replacement])).validate().is_err());
        }
    }
    fn endpoint(id: &str, priority: u16, preferred: bool) -> serde_json::Value {
        json!({"id":id,"role":"mail","protocol":"imap","priority":priority,"preferred":preferred,"endpoint":{"host":"mail.example.test","port":993,"tls":"implicit"}})
    }
    #[test]
    fn deterministic_order_is_preferred_then_priority_then_id() {
        let c = config(json!([
            endpoint("b", 1, false),
            endpoint("a", 1, false),
            endpoint("preferred", 500, true)
        ]));
        c.validate().unwrap();
        assert_eq!(c.selected(ServiceRole::Mail).unwrap().id, "preferred");
        let c = config(json!([endpoint("b", 1, false), endpoint("a", 1, false)]));
        assert_eq!(c.selected(ServiceRole::Mail).unwrap().id, "a");
        let selected = c.resolve().unwrap();
        assert!(selected.imap.is_some());
        assert!(selected.pop3.is_none());
    }
    #[test]
    fn disabled_and_empty_lists_never_create_an_implicit_endpoint() {
        let mut row = endpoint("disabled", 0, false);
        row["enabled"] = json!(false);
        let c = config(json!([row]));
        let selected = c.resolve().unwrap();
        assert!(selected.mail().is_none());
        assert!(selected.selection.is_some());
        assert!(
            config(json!([
                endpoint("duplicate", 1, false),
                endpoint("duplicate", 2, false)
            ]))
            .validate()
            .is_err()
        );
        assert!(
            config(json!([endpoint("a", 1, true), endpoint("b", 2, true)]))
                .validate()
                .is_err()
        );
    }
    #[test]
    fn failover_chain_is_opt_in_same_protocol_ordered_and_bounded() {
        let rows = json!([
            endpoint("primary", 1, true),
            endpoint("d", 40, false),
            endpoint("b", 20, false),
            endpoint("c", 30, false),
            {"id":"pop","role":"mail","protocol":"pop3","priority":2,"endpoint":{"host":"pop.example.test","port":995,"tls":"implicit"}}
        ]);
        let off = config(rows.clone());
        assert!(off.mail_fallbacks().is_empty(), "failover is opt-in");
        let plain_binding = off.resolve().unwrap().binding().unwrap();
        let mut on: BackendConfig =
            serde_json::from_value(json!({"version":2,"failover":true,"backends":rows})).unwrap();
        on.validate().unwrap();
        let ids: Vec<_> = on.mail_fallbacks().iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["b", "c"], "same protocol, priority order, bounded");
        let resolved = on.resolve().unwrap();
        let chain: Vec<_> = resolved
            .mail_chain
            .iter()
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(chain, ["primary", "b", "c"]);
        assert_ne!(
            resolved.binding().unwrap(),
            plain_binding,
            "the chain is part of the binding"
        );
        on.backends.retain(|b| b.id == "primary");
        assert!(on.resolve().unwrap().mail_chain.is_empty());
        assert_eq!(on.resolve().unwrap().binding().unwrap(), {
            let mut single = config(json!([endpoint("primary", 1, true)]));
            single.failover = false;
            single.resolve().unwrap().binding().unwrap()
        });
    }
    #[test]
    fn failover_backends_must_share_security_and_username_policy() {
        let mut other = endpoint("b", 2, false);
        other["endpoint"]["tls"] = json!("starttls");
        let mixed: BackendConfig = serde_json::from_value(
            json!({"version":2,"failover":true,"backends":[endpoint("a", 1, false), other]}),
        )
        .unwrap();
        assert!(mixed.validate().is_err());
        let mut named = endpoint("b", 2, false);
        named["username"] = json!({"format":"localUser"});
        let mixed: BackendConfig = serde_json::from_value(
            json!({"version":2,"failover":true,"backends":[endpoint("a", 1, false), named]}),
        )
        .unwrap();
        assert!(mixed.validate().is_err());
        let jmap: BackendConfig = serde_json::from_value(json!({"version":2,"failover":true,"backends":[
            {"id":"j","role":"mail","protocol":"jmap","sessionUrl":"https://jmap.example.test/session"}
        ]}))
        .unwrap();
        assert!(jmap.validate().is_err(), "failover is IMAP/POP3 only");
    }
    #[test]
    fn binding_changes_with_auth_identity_and_selected_authority() {
        let mut rows = json!([
            endpoint("primary", 1, false),
            endpoint("secondary", 2, false)
        ]);
        let initial = config(rows.clone()).resolve().unwrap();
        let original = "Alice@alias.example";
        assert_eq!(
            initial.auth_username(ServiceRole::Mail, original).unwrap(),
            original
        );
        rows[0]["username"] = json!({"format":"domainUser","domain":"CORP"});
        let formatted = config(rows.clone()).resolve().unwrap();
        assert_eq!(
            formatted
                .auth_username(ServiceRole::Mail, original)
                .unwrap(),
            "CORP\\Alice"
        );
        assert_ne!(initial.binding().unwrap(), formatted.binding().unwrap());
        rows[0]["enabled"] = json!(false);
        assert_ne!(
            formatted.binding().unwrap(),
            config(rows).resolve().unwrap().binding().unwrap()
        );
        assert_eq!(
            UpstreamConfig::parse(
                r#"{"imap":{"host":"mail.example.test","port":993,"tls":"implicit"}}"#
            )
            .unwrap()
            .binding()
            .unwrap(),
            None
        );
    }
}
