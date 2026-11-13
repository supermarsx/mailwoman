//! An event-time snapshot for the account holder. Browser attributes are reported
//! by the client, and recognition is informational; neither authenticates a user.
use axum::http::{Extensions, HeaderMap};
use hmac::{Hmac, KeyInit, Mac};
use mw_store::SessionDevice;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Evidence {
    #[serde(skip)]
    pub subject: Option<String>,
    pub ip: Option<String>,
    pub network: String,
    pub country: Option<String>,
    pub device_type: String,
    pub browser: Option<String>,
    pub browser_major: Option<u32>,
    pub browser_engine: Option<String>,
    pub os: Option<String>,
    pub client_type: String,
    pub client_version: Option<String>,
    pub user_agent: Option<String>,
    pub recognized: Option<bool>,
    pub fingerprint: Option<String>,
    pub authentication: Option<String>,
}
fn bounded(value: &str, max: usize) -> Option<String> {
    let text: String = value
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect();
    (!text.is_empty()).then_some(text)
}
impl Evidence {
    pub(crate) async fn capture(
        state: &crate::AppState,
        session: &str,
        device: &SessionDevice,
        recognized: Option<bool>,
        method: Option<&str>,
    ) -> Self {
        let scope = state
            .store
            .session_policy_subject(session)
            .await
            .ok()
            .flatten()
            .map(|subject| subject.personal_subject);
        Self::from_device(
            device,
            recognized,
            method,
            scope.as_deref(),
            &state
                .store
                .signin_challenge_secret("security-notice-display"),
        )
    }
    fn from_device(
        device: &SessionDevice,
        recognized: Option<bool>,
        method: Option<&str>,
        scope: Option<&str>,
        secret: &str,
    ) -> Self {
        let ip = device.ip.parse::<std::net::IpAddr>().ok();
        let (family, os, major) = crate::session_binding::parse_user_agent(&device.user_agent);
        let ua = &device.user_agent;
        let device_type =
            if ua.contains("iPad") || (ua.contains("Android") && !ua.contains("Mobile")) {
                "tablet"
            } else if ua.contains("Mobile") || ua.contains("iPhone") || ua.contains("iPod") {
                "phone"
            } else if os.is_some() {
                "desktop"
            } else {
                "unknown"
            };
        let engine = if os == Some("ios") || family == Some("safari") {
            Some("WebKit")
        } else if family == Some("firefox") {
            Some("Gecko")
        } else if matches!(family, Some("chrome" | "edge" | "opera" | "samsung")) {
            Some("Blink")
        } else {
            None
        };
        let fingerprint = scope
            .zip(device.recognition_hash.as_deref())
            .map(|(scope, marker)| {
                let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                    .expect("HMAC accepts any key");
                mac.update(b"mailwoman-notice-device-v1\0");
                for part in [scope, marker] {
                    mac.update(&(part.len() as u64).to_be_bytes());
                    mac.update(part.as_bytes());
                }
                mac.finalize().into_bytes()[..8]
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect()
            });
        Self {
            subject: scope.map(str::to_owned),
            ip: ip.map(|v| v.to_string()),
            network: if ip.is_some_and(crate::geo_lookup::is_local) {
                "internal"
            } else if ip.is_some() {
                "public"
            } else {
                "unknown"
            }
            .into(),
            country: ip
                .filter(|v| !crate::geo_lookup::is_local(*v))
                .and_then(mw_engine::security::country_for_ip),
            device_type: device_type.into(),
            browser: family.map(str::to_owned),
            browser_major: major,
            browser_engine: engine.map(str::to_owned),
            os: os.map(str::to_owned),
            client_type: bounded(&device.client_type, 32).unwrap_or_else(|| "unknown".into()),
            client_version: bounded(&device.client_version, 128),
            user_agent: bounded(ua, 512),
            recognized,
            fingerprint,
            authentication: method.and_then(|m| bounded(m, 64)),
        }
    }
    pub(crate) fn text(&self, pt: bool) -> String {
        let unknown = if pt {
            "Não disponível"
        } else {
            "Not available"
        };
        let location = if self.network == "internal" {
            if pt {
                "Rede interna"
            } else {
                "Internal network"
            }
        } else {
            self.country.as_deref().unwrap_or(unknown)
        };
        let recognized = match self.recognized {
            Some(true) => {
                if pt {
                    "Reconhecido"
                } else {
                    "Recognized"
                }
            }
            Some(false) => {
                if pt {
                    "Não reconhecido"
                } else {
                    "Unrecognized"
                }
            }
            None => unknown,
        };
        let labels = if pt {
            [
                "IP",
                "Localização",
                "Tipo de dispositivo",
                "Reconhecimento",
                "Navegador",
                "Motor do navegador",
                "Sistema operativo",
                "Tipo de cliente",
                "Versão do cliente",
                "Impressão digital",
                "Autenticação",
                "Agente do utilizador",
            ]
        } else {
            [
                "IP address",
                "Location",
                "Device type",
                "Device recognition",
                "Browser",
                "Browser engine",
                "Operating system",
                "Client type",
                "Client version",
                "Device fingerprint",
                "Authentication",
                "User agent",
            ]
        };
        let browser = format!(
            "{}{}",
            self.browser.as_deref().unwrap_or(unknown),
            self.browser_major
                .map(|v| format!(" {v}"))
                .unwrap_or_default()
        );
        let values = [
            self.ip.as_deref().unwrap_or(unknown),
            location,
            &self.device_type,
            recognized,
            &browser,
            self.browser_engine.as_deref().unwrap_or(unknown),
            self.os.as_deref().unwrap_or(unknown),
            &self.client_type,
            self.client_version.as_deref().unwrap_or(unknown),
            self.fingerprint.as_deref().unwrap_or(unknown),
            self.authentication.as_deref().unwrap_or(unknown),
            self.user_agent.as_deref().unwrap_or(unknown),
        ];
        labels
            .into_iter()
            .zip(values)
            .map(|(label, value)| format!("{label}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
pub(crate) async fn request(
    state: &crate::AppState,
    session: &str,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Evidence {
    let device = crate::security_center::device(headers, ext, None);
    let recognized = match device.recognition_hash.as_deref() {
        Some(marker) => state.store.browser_device_known(session, marker).await.ok(),
        None => None,
    };
    Evidence::capture(state, session, &device, recognized, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_is_bounded_scoped_and_labels_internal_networks() {
        let device = SessionDevice {
            ip: "10.2.3.4".into(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0) Chrome/142.0.0.0 Safari/537.36".into(),
            recognition_hash: Some("a".repeat(64)),
            client_type: "browser".into(),
            ..Default::default()
        };
        let snapshot = Evidence::from_device(
            &device,
            Some(false),
            Some("password"),
            Some("owner"),
            "secret",
        );
        assert_eq!(snapshot.network, "internal");
        assert_eq!(snapshot.country, None);
        assert_eq!(snapshot.browser.as_deref(), Some("chrome"));
        assert_eq!(snapshot.browser_major, Some(142));
        assert_eq!(snapshot.device_type, "desktop");
        assert_eq!(snapshot.recognized, Some(false));
        assert_ne!(
            snapshot.fingerprint,
            Evidence::from_device(&device, Some(false), None, Some("other"), "secret").fingerprint
        );
        assert_eq!(snapshot.fingerprint.as_ref().unwrap().len(), 16);
        assert!(snapshot.text(false).contains("Internal network"));
        assert!(!snapshot.text(false).contains(&"a".repeat(64)));
    }
}
