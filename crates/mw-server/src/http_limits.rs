//! Explicit startup bounds for the built-in HTTP server.
use std::{io, time::Duration};
#[derive(Clone, Copy, Debug)]
pub struct HttpLimits {
    pub connections: usize,
    pub requests: usize,
    pub header_bytes: usize,
    pub headers: usize,
    pub header_timeout: Duration,
    pub body_timeout: Duration,
    pub http2_streams: u32,
    pub shutdown_timeout: Duration,
}
impl Default for HttpLimits {
    fn default() -> Self {
        Self {
            connections: 1024,
            requests: 256,
            header_bytes: 32768,
            headers: 100,
            header_timeout: Duration::from_secs(10),
            body_timeout: Duration::from_secs(60),
            http2_streams: 32,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}
impl HttpLimits {
    pub fn from_env() -> io::Result<Self> {
        Self::read(|key| std::env::var(key).ok())
    }
    fn read(mut env: impl FnMut(&str) -> Option<String>) -> io::Result<Self> {
        fn number(
            raw: Option<String>,
            key: &str,
            default: u64,
            min: u64,
            max: u64,
        ) -> io::Result<u64> {
            let Some(raw) = raw else { return Ok(default) };
            raw.trim()
                .parse::<u64>()
                .ok()
                .filter(|n| (min..=max).contains(n))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{key} must be an integer between {min} and {max}"),
                    )
                })
        }
        Ok(Self {
            connections: number(
                env("MW_HTTP_MAX_CONNECTIONS"),
                "MW_HTTP_MAX_CONNECTIONS",
                1024,
                16,
                8192,
            )? as usize,
            requests: number(
                env("MW_HTTP_MAX_REQUESTS"),
                "MW_HTTP_MAX_REQUESTS",
                256,
                1,
                4096,
            )? as usize,
            header_bytes: number(
                env("MW_HTTP_MAX_HEADER_BYTES"),
                "MW_HTTP_MAX_HEADER_BYTES",
                32768,
                8192,
                65536,
            )? as usize,
            headers: number(
                env("MW_HTTP_MAX_HEADERS"),
                "MW_HTTP_MAX_HEADERS",
                100,
                16,
                256,
            )? as usize,
            header_timeout: Duration::from_millis(number(
                env("MW_HTTP_HEADER_TIMEOUT_MS"),
                "MW_HTTP_HEADER_TIMEOUT_MS",
                10000,
                1000,
                60000,
            )?),
            body_timeout: Duration::from_millis(number(
                env("MW_HTTP_BODY_TIMEOUT_MS"),
                "MW_HTTP_BODY_TIMEOUT_MS",
                60000,
                1000,
                300000,
            )?),
            http2_streams: number(
                env("MW_HTTP2_MAX_STREAMS"),
                "MW_HTTP2_MAX_STREAMS",
                32,
                1,
                128,
            )? as u32,
            shutdown_timeout: Duration::from_millis(number(
                env("MW_HTTP_SHUTDOWN_TIMEOUT_MS"),
                "MW_HTTP_SHUTDOWN_TIMEOUT_MS",
                30000,
                1000,
                120000,
            )?),
        })
    }
    pub fn validate(self) -> io::Result<()> {
        if !(16..=8192).contains(&self.connections)
            || !(1..=4096).contains(&self.requests)
            || !(8192..=65536).contains(&self.header_bytes)
            || !(16..=256).contains(&self.headers)
            || !(1..=128).contains(&self.http2_streams)
            || !(Duration::from_secs(1)..=Duration::from_secs(60)).contains(&self.header_timeout)
            || !(Duration::from_secs(1)..=Duration::from_secs(300)).contains(&self.body_timeout)
            || !(Duration::from_secs(1)..=Duration::from_secs(120)).contains(&self.shutdown_timeout)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "HTTP startup limits outside supported bounds",
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_values_are_bounded_and_malformed_values_refuse_startup() {
        HttpLimits::default().validate().unwrap();
        for key in [
            "MW_HTTP_MAX_CONNECTIONS",
            "MW_HTTP_MAX_REQUESTS",
            "MW_HTTP_MAX_HEADER_BYTES",
            "MW_HTTP_MAX_HEADERS",
            "MW_HTTP_HEADER_TIMEOUT_MS",
            "MW_HTTP_BODY_TIMEOUT_MS",
            "MW_HTTP2_MAX_STREAMS",
            "MW_HTTP_SHUTDOWN_TIMEOUT_MS",
        ] {
            for value in ["0", "-1", "99999999", "", "1.1"] {
                assert!(
                    HttpLimits::read(|name| (name == key).then(|| value.into())).is_err(),
                    "{key}={value}"
                );
            }
        }
    }
}
