//! Explicitly opted-in, deployment-wide telemetry. No credentials or mail data.
use serde::{Deserialize, Serialize};

pub const SETTINGS_KEY: &str = "admin.telemetry.v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub service_name: String,
    pub environment: String,
    pub traces: bool,
    pub metrics: bool,
    pub sample_ratio: f64,
    pub export_timeout_ms: u64,
    pub metrics_interval_secs: u64,
    pub span_queue_size: usize,
    pub span_batch_size: usize,
    pub span_export_delay_ms: u64,
    pub prometheus_enabled: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false, endpoint: String::new(), service_name: "mailwoman".into(),
            environment: String::new(), traces: true, metrics: true, sample_ratio: 0.1,
            export_timeout_ms: 5000, metrics_interval_secs: 60, span_queue_size: 2048,
            span_batch_size: 512, span_export_delay_ms: 5000, prometheus_enabled: false,
        }
    }
}

impl TelemetryConfig {
    pub fn validate(&self) -> Result<(), (&'static str, &'static str)> {
        if self.enabled && self.endpoint.is_empty() {
            return Err(("/endpoint", "Enter an OTLP gRPC collector endpoint before enabling export."));
        }
        if !self.endpoint.is_empty() {
            let url = url::Url::parse(&self.endpoint).map_err(|_| ("/endpoint", "Use an HTTP or HTTPS collector URL."))?;
            if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none()
                || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some()
            {
                return Err(("/endpoint", "Collector URLs must use HTTP or HTTPS without credentials, query parameters or fragments."));
            }
        }
        if self.service_name.trim().is_empty() || self.service_name.len() > 128 || self.service_name.chars().any(char::is_control) {
            return Err(("/serviceName", "Use a service name of 1 to 128 characters without control characters."));
        }
        if self.environment.len() > 64 || self.environment.chars().any(char::is_control) {
            return Err(("/environment", "Use up to 64 environment characters without control characters."));
        }
        if !self.sample_ratio.is_finite() || !(0.0..=1.0).contains(&self.sample_ratio) {
            return Err(("/sampleRatio", "Sampling must be between 0 and 1."));
        }
        if !(100..=30000).contains(&self.export_timeout_ms) { return Err(("/exportTimeoutMs", "Export timeout must be 100 to 30000 milliseconds.")); }
        if !(10..=3600).contains(&self.metrics_interval_secs) { return Err(("/metricsIntervalSecs", "Metrics interval must be 10 to 3600 seconds.")); }
        if !(128..=32768).contains(&self.span_queue_size) { return Err(("/spanQueueSize", "Span queue must hold 128 to 32768 entries.")); }
        if !(1..=4096).contains(&self.span_batch_size) || self.span_batch_size > self.span_queue_size {
            return Err(("/spanBatchSize", "Batch size must be 1 to 4096 and cannot exceed the queue size."));
        }
        if !(100..=60000).contains(&self.span_export_delay_ms) { return Err(("/spanExportDelayMs", "Batch delay must be 100 to 60000 milliseconds.")); }
        if self.enabled && !self.traces && !self.metrics { return Err(("/enabled", "Select at least one export signal.")); }
        Ok(())
    }

    /// Existing explicit environment configuration remains an operator override.
    /// Returned names explain exactly which controls are pinned in the editor.
    pub fn with_environment(mut self, env: impl Fn(&str) -> Option<String>) -> Result<(Self, Vec<&'static str>), (&'static str, &'static str)> {
        let mut pinned = Vec::new();
        if let Some(endpoint) = env("MW_OTLP_ENDPOINT").filter(|v| !v.is_empty()) {
            self.endpoint = endpoint; self.enabled = true;
            pinned.extend(["endpoint", "enabled"]);
        }
        if let Some(name) = env("MW_OTEL_SERVICE").filter(|v| !v.is_empty()) {
            self.service_name = name; pinned.push("serviceName");
        }
        if env("MW_METRICS_TOKEN").is_some_and(|v| !v.is_empty()) {
            self.prometheus_enabled = true; pinned.push("prometheusEnabled");
        }
        self.validate()?;
        Ok((self, pinned))
    }
}

pub async fn load(store: &mw_store::Store) -> anyhow::Result<TelemetryConfig> {
    let config: TelemetryConfig = match store.get_setting(SETTINGS_KEY).await? {
        Some(value) => serde_json::from_str(&value)?,
        None => TelemetryConfig::default(),
    };
    config.validate().map_err(|(_, message)| anyhow::anyhow!(message))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_never_enable_an_export_or_scrape_endpoint() {
        let (config, pinned) = TelemetryConfig::default().with_environment(|_| None).unwrap();
        assert!(!config.enabled); assert!(!config.prometheus_enabled); assert!(pinned.is_empty());
        let disabled = TelemetryConfig { endpoint: "https://collector.example".into(), ..config };
        assert!(!disabled.enabled);
        assert!(disabled.validate().is_ok());
    }

    #[test]
    fn refuses_credentials_and_unbounded_queues_even_while_disabled() {
        for endpoint in ["https://user:secret@collector.test", "https://collector.test?token=secret", "file:///tmp/data", "https://collector.test/#secret"] {
            assert_eq!(TelemetryConfig { endpoint: endpoint.into(), ..Default::default() }.validate().unwrap_err().0, "/endpoint");
        }
        assert_eq!(TelemetryConfig { span_queue_size: 128, span_batch_size: 512, ..Default::default() }.validate().unwrap_err().0, "/spanBatchSize");
        assert!(TelemetryConfig { sample_ratio: f64::NAN, ..Default::default() }.validate().is_err());
    }

    #[test]
    fn explicit_environment_opt_in_is_reported_without_returning_a_token() {
        let (config, pinned) = TelemetryConfig::default().with_environment(|key| match key {
            "MW_OTLP_ENDPOINT" => Some("http://collector:4317".into()),
            "MW_METRICS_TOKEN" => Some("private-token".into()), _ => None,
        }).unwrap();
        assert!(config.enabled && config.prometheus_enabled);
        assert_eq!(pinned, ["endpoint", "enabled", "prometheusEnabled"]);
        assert!(!serde_json::to_string(&config).unwrap().contains("private-token"));
    }
}
