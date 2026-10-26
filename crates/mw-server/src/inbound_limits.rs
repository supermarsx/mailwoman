//! Startup-only bounds for inbound connection negotiation.
use std::{io, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InboundLimits {
    pub tls_timeout: Duration,
    pub tls_handshakes: usize,
    pub proxy_timeout: Duration,
    pub proxy_handshakes: usize,
}

impl Default for InboundLimits {
    fn default() -> Self {
        Self {
            tls_timeout: Duration::from_secs(10),
            tls_handshakes: 128,
            proxy_timeout: Duration::from_secs(5),
            proxy_handshakes: 128,
        }
    }
}

impl InboundLimits {
    pub fn validate(self) -> io::Result<()> {
        if !(Duration::from_secs(1)..=Duration::from_secs(60)).contains(&self.tls_timeout)
            || !(1..=1024).contains(&self.tls_handshakes)
            || !(Duration::from_secs(1)..=Duration::from_secs(30)).contains(&self.proxy_timeout)
            || !(1..=1024).contains(&self.proxy_handshakes)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "inbound negotiation limits are outside allowed startup bounds",
            ));
        }
        Ok(())
    }

    pub fn from_env() -> io::Result<Self> {
        Self::read(|name| std::env::var(name).ok())
    }

    fn read(mut value: impl FnMut(&str) -> Option<String>) -> io::Result<Self> {
        fn bounded(
            value: Option<String>,
            name: &str,
            default: u64,
            min: u64,
            max: u64,
        ) -> io::Result<u64> {
            let Some(raw) = value else { return Ok(default) };
            raw.trim()
                .parse::<u64>()
                .ok()
                .filter(|n| (min..=max).contains(n))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{name} must be an integer between {min} and {max}"),
                    )
                })
        }
        Ok(Self {
            tls_timeout: Duration::from_millis(bounded(
                value("MW_TLS_HANDSHAKE_TIMEOUT_MS"),
                "MW_TLS_HANDSHAKE_TIMEOUT_MS",
                10000,
                1000,
                60000,
            )?),
            tls_handshakes: bounded(
                value("MW_TLS_MAX_HANDSHAKES"),
                "MW_TLS_MAX_HANDSHAKES",
                128,
                1,
                1024,
            )? as usize,
            proxy_timeout: Duration::from_millis(bounded(
                value("MW_PROXY_HEADER_TIMEOUT_MS"),
                "MW_PROXY_HEADER_TIMEOUT_MS",
                5000,
                1000,
                30000,
            )?),
            proxy_handshakes: bounded(
                value("MW_PROXY_MAX_HANDSHAKES"),
                "MW_PROXY_MAX_HANDSHAKES",
                128,
                1,
                1024,
            )? as usize,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_bounds_are_explicit_and_invalid_values_fail_closed() {
        assert_eq!(
            InboundLimits::read(|_| None).unwrap(),
            InboundLimits::default()
        );
        for key in [
            "MW_TLS_HANDSHAKE_TIMEOUT_MS",
            "MW_TLS_MAX_HANDSHAKES",
            "MW_PROXY_HEADER_TIMEOUT_MS",
            "MW_PROXY_MAX_HANDSHAKES",
        ] {
            for invalid in ["", "-1", "0", "4294967295", "ten", "1.5"] {
                assert!(
                    InboundLimits::read(|name| (name == key).then(|| invalid.into())).is_err(),
                    "{key}={invalid}"
                );
            }
        }
        let custom = InboundLimits::read(|name| match name {
            "MW_TLS_HANDSHAKE_TIMEOUT_MS" => Some("60000".into()),
            "MW_PROXY_HEADER_TIMEOUT_MS" => Some("1000".into()),
            "MW_TLS_MAX_HANDSHAKES" | "MW_PROXY_MAX_HANDSHAKES" => Some("1".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(custom.tls_timeout, Duration::from_secs(60));
        assert_eq!(custom.proxy_timeout, Duration::from_secs(1));
        assert_eq!(custom.tls_handshakes, 1);
    }
}
