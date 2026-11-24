//! Counters with exactly the bucket rules and snapshot shapes of http-load.mjs
//! (`Measurements.snapshot()`, `bytesSnapshot`, the 503 reasons), so the Node
//! adapter can rebuild every report field with Node's own `merge()` / `report()`
//! functions. A finer log-linear histogram in microseconds is kept beside the
//! Node one for the native report.
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const BUCKETS: usize = 3002;

/// Status key of the Node counters: an HTTP code or one of three words.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Status {
    Http(u16),
    Network,
    Scheduled,
    Aborted,
}
impl Status {
    pub fn key(self) -> String {
        match self {
            Status::Http(code) => code.to_string(),
            Status::Network => "network".into(),
            Status::Scheduled => "scheduled".into(),
            Status::Aborted => "campaign-aborted".into(),
        }
    }
    pub fn success(self) -> bool {
        matches!(self, Status::Http(200..=299))
    }
}

fn bucket_of(ms: f64) -> (u32, usize) {
    if ms <= 1000.0 {
        let b = ms.ceil().max(0.0) as u32;
        (b, b as usize)
    } else if ms <= 10000.0 {
        let b = ((ms / 10.0).ceil() * 10.0) as u32;
        (b, 1000 + ((b - 1000) / 10) as usize)
    } else {
        let b = (((ms / 100.0).ceil() * 100.0) as u32).min(120100);
        (b, 1900 + ((b - 10000) / 100) as usize)
    }
}
fn bucket_value(index: usize) -> u32 {
    if index <= 1000 {
        index as u32
    } else if index <= 1900 {
        1000 + (index as u32 - 1000) * 10
    } else {
        10000 + (index as u32 - 1900) * 100
    }
}

#[derive(Clone)]
pub struct Meas {
    pub count: u64,
    pub bytes: u64,
    pub accepted_decoded: u64,
    pub accepted_in_window: u64,
    pub accepted_in_window_bytes: u64,
    pub errors: u64,
    pub accepted: u64,
    pub timeouts: u64,
    pub total_ms: f64,
    pub max_ms: f64,
    pub hist: Vec<u64>,
    pub statuses: BTreeMap<Status, u64>,
}
impl Default for Meas {
    fn default() -> Self {
        Meas {
            count: 0,
            bytes: 0,
            accepted_decoded: 0,
            accepted_in_window: 0,
            accepted_in_window_bytes: 0,
            errors: 0,
            accepted: 0,
            timeouts: 0,
            total_ms: 0.0,
            max_ms: 0.0,
            hist: vec![0; BUCKETS],
            statuses: BTreeMap::new(),
        }
    }
}
impl Meas {
    /// `Measurements.add(ms,status,bytes,ok,timeout,inWindow)`
    pub fn add(
        &mut self,
        ms: f64,
        status: Status,
        bytes: u64,
        ok: bool,
        timeout: bool,
        in_window: bool,
    ) {
        let accepted = ok && status.success();
        if accepted {
            self.accepted_decoded += bytes;
            if in_window {
                self.accepted_in_window += 1;
                self.accepted_in_window_bytes += bytes;
            }
        }
        self.count += 1;
        self.bytes += bytes;
        self.total_ms += ms;
        self.max_ms = self.max_ms.max(ms);
        if !ok {
            self.errors += 1;
        }
        if accepted {
            self.accepted += 1;
        }
        if timeout {
            self.timeouts += 1;
        }
        *self.statuses.entry(status).or_default() += 1;
        self.hist[bucket_of(ms).1] += 1;
    }
    pub fn sample(&mut self, ms: f64) {
        self.add(ms, Status::Scheduled, 0, true, false, false);
    }
    pub fn merge(&mut self, o: &Meas) {
        self.count += o.count;
        self.bytes += o.bytes;
        self.accepted_decoded += o.accepted_decoded;
        self.accepted_in_window += o.accepted_in_window;
        self.accepted_in_window_bytes += o.accepted_in_window_bytes;
        self.errors += o.errors;
        self.accepted += o.accepted;
        self.timeouts += o.timeouts;
        self.total_ms += o.total_ms;
        self.max_ms = self.max_ms.max(o.max_ms);
        for (a, b) in self.hist.iter_mut().zip(&o.hist) {
            *a += b;
        }
        for (k, v) in &o.statuses {
            *self.statuses.entry(*k).or_default() += v;
        }
    }
    pub fn percentile(&self, fraction: f64) -> u32 {
        let target = (self.count as f64 * fraction).ceil() as u64;
        let mut n = 0;
        for (index, count) in self.hist.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            n += count;
            if n >= target {
                return bucket_value(index);
            }
        }
        0
    }
    /// `Measurements.snapshot()`
    pub fn snapshot(&self) -> Value {
        let statuses: Map<String, Value> = self
            .statuses
            .iter()
            .map(|(k, v)| (k.key(), json!(v)))
            .collect();
        let histogram: Vec<Value> = self
            .hist
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(i, c)| json!([bucket_value(i), c]))
            .collect();
        json!({"count": self.count, "bytes": self.bytes, "acceptedDecodedBytes": self.accepted_decoded,
            "acceptedDuringScheduledWindow": self.accepted_in_window, "acceptedDuringScheduledWindowDecodedBytes": self.accepted_in_window_bytes,
            "errors": self.errors, "accepted": self.accepted, "timeouts": self.timeouts, "totalMs": self.total_ms, "maximumMs": self.max_ms,
            "statuses": statuses, "histogram": histogram})
    }
}

const SIZE_STEP: f64 = 0.04879016416943205; // Math.log(1.05)

#[derive(Default, Clone)]
pub struct BytesProfile {
    pub sizes: BTreeMap<u32, u64>,
    pub count: u64,
    pub sum: u64,
    pub request_body_bytes: u64,
    pub responses: u64,
    pub compressed: u64,
    pub encodings: BTreeMap<String, u64>,
    pub measured: u64,
    pub wire_bytes: u64,
    pub decoded_bytes: u64,
}
impl BytesProfile {
    /// `addResponseSize`
    pub fn add_size(&mut self, bytes: u64) {
        let bucket = if bytes == 0 {
            0
        } else {
            ((bytes as f64).ln() / SIZE_STEP).ceil() as u32 + 1
        };
        *self.sizes.entry(bucket).or_default() += 1;
        self.count += 1;
        self.sum += bytes;
    }
    /// `addCompression`
    pub fn add_compression(&mut self, encoding: Option<&str>, wire: Option<u64>, decoded: u64) {
        self.responses += 1;
        let coded = encoding.is_some_and(|e| !e.is_empty() && e != "identity");
        if coded {
            self.compressed += 1;
            let name: String = encoding
                .unwrap()
                .to_lowercase()
                .chars()
                .take(20)
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == ',' || *c == '-')
                .collect();
            *self.encodings.entry(name).or_default() += 1;
            if let Some(w) = wire {
                self.measured += 1;
                self.wire_bytes += w;
                self.decoded_bytes += decoded;
            }
        }
    }
    pub fn merge(&mut self, o: &BytesProfile) {
        for (k, v) in &o.sizes {
            *self.sizes.entry(*k).or_default() += v;
        }
        self.count += o.count;
        self.sum += o.sum;
        self.request_body_bytes += o.request_body_bytes;
        self.responses += o.responses;
        self.compressed += o.compressed;
        for (k, v) in &o.encodings {
            *self.encodings.entry(k.clone()).or_default() += v;
        }
        self.measured += o.measured;
        self.wire_bytes += o.wire_bytes;
        self.decoded_bytes += o.decoded_bytes;
    }
    /// `bytesSnapshot`
    pub fn snapshot(&self) -> Value {
        json!({"schema": 1, "sizes": self.sizes.iter().map(|(k, v)| json!([k, v])).collect::<Vec<_>>(), "count": self.count, "sum": self.sum,
            "requestBodyBytes": self.request_body_bytes, "responses": self.responses, "compressed": self.compressed, "encodings": self.encodings,
            "measured": self.measured, "wireBytes": self.wire_bytes, "decodedBytes": self.decoded_bytes})
    }
}

pub const REASONS: [&str; 5] = [
    "publishedGuardFenceRefusal",
    "authorizationNotConfirmedBeforeDispatch",
    "requestProtectionUnavailable",
    "empty503",
    "other503",
];

#[derive(Default, Clone)]
pub struct Reasons503 {
    pub total: u64,
    pub counts: [u64; 5],
}
impl Reasons503 {
    pub fn add(&mut self, reason: usize) {
        self.total += 1;
        self.counts[reason] += 1;
    }
    pub fn merge(&mut self, o: &Reasons503) {
        self.total += o.total;
        for i in 0..5 {
            self.counts[i] += o.counts[i];
        }
    }
    pub fn snapshot(&self) -> Value {
        let counts: Map<String, Value> = REASONS
            .iter()
            .zip(self.counts)
            .map(|(k, v)| ((*k).to_owned(), json!(v)))
            .collect();
        json!({"schema": 1, "total": self.total, "counts": counts})
    }
}
/// `classifyHttp503(body, complete)`; `body` None = over 4096 bytes.
pub fn classify_503(body: Option<&[u8]>) -> usize {
    let Some(body) = body.filter(|b| b.len() <= 4096) else {
        return 4;
    };
    if body.is_empty() {
        return 3;
    }
    let text = String::from_utf8_lossy(body);
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => match map.get("code").and_then(Value::as_str) {
            Some(code) => REASONS[..3].iter().position(|r| *r == code).unwrap_or(4),
            None => 4,
        },
        _ => 4,
    }
}

/// principalQueue, httpAndValidation, validation (`timings()`)
#[derive(Default, Clone)]
pub struct Timings {
    pub queue: Meas,
    pub http: Meas,
    pub validation: Meas,
}
impl Timings {
    pub fn merge(&mut self, o: &Timings) {
        self.queue.merge(&o.queue);
        self.http.merge(&o.http);
        self.validation.merge(&o.validation);
    }
    pub fn snapshot(&self) -> Value {
        json!({"principalQueue": self.queue.snapshot(), "httpAndValidation": self.http.snapshot(), "validation": self.validation.snapshot()})
    }
}

/// Log-linear histogram of microseconds: 64 sub-buckets per power of two (under 1.6% bucket width).
#[derive(Clone)]
pub struct Hdr {
    pub counts: Vec<u64>,
    pub max: u64,
    pub total: u64,
    pub sum: f64,
}
impl Default for Hdr {
    fn default() -> Self {
        Hdr {
            counts: vec![0; 64 * 40],
            max: 0,
            total: 0,
            sum: 0.0,
        }
    }
}
fn hdr_index(v: u64) -> usize {
    if v < 128 {
        v as usize
    } else {
        let e = 63 - v.leading_zeros() as u64;
        ((e - 6) * 64 + (v >> (e - 6))) as usize
    }
}
fn hdr_upper(index: usize) -> u64 {
    if index < 128 {
        index as u64
    } else {
        let g = index as u64 / 64;
        let e = g + 5;
        let m = index as u64 - (g - 1) * 64;
        ((m + 1) << (e - 6)) - 1
    }
}
impl Hdr {
    pub fn record(&mut self, micros: f64) {
        let v = micros.max(0.0).round() as u64;
        let i = hdr_index(v).min(self.counts.len() - 1);
        self.counts[i] += 1;
        self.total += 1;
        self.max = self.max.max(v);
        self.sum += micros;
    }
    pub fn merge(&mut self, o: &Hdr) {
        for (a, b) in self.counts.iter_mut().zip(&o.counts) {
            *a += b;
        }
        self.total += o.total;
        self.max = self.max.max(o.max);
        self.sum += o.sum;
    }
    pub fn quantile(&self, q: f64) -> u64 {
        let target = ((self.total as f64) * q).ceil().max(1.0) as u64;
        let mut n = 0;
        for (i, c) in self.counts.iter().enumerate() {
            n += c;
            if n >= target {
                return hdr_upper(i).min(self.max);
            }
        }
        self.max
    }
    pub fn report(&self) -> Value {
        json!({"samples": self.total, "meanUs": if self.total > 0 { self.sum / self.total as f64 } else { 0.0 },
            "p50Us": self.quantile(0.5), "p90Us": self.quantile(0.9), "p99Us": self.quantile(0.99), "p999Us": self.quantile(0.999), "p9999Us": self.quantile(0.9999), "maxUs": self.max,
            "resolution": "log-linear, 64 sub-buckets per power of two (bucket upper bound, under 1.6% wide); latency from the scheduled slot under late=dispatch, else from send"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn node_buckets() {
        assert_eq!(bucket_of(0.0), (0, 0));
        assert_eq!(bucket_of(0.2), (1, 1));
        assert_eq!(bucket_of(1000.0), (1000, 1000));
        assert_eq!(bucket_of(1000.5), (1010, 1001));
        assert_eq!(bucket_of(10000.0), (10000, 1900));
        assert_eq!(bucket_of(10000.1), (10100, 1901));
        assert_eq!(bucket_of(1e9), (120100, 3001));
        for i in 0..BUCKETS {
            assert_eq!(bucket_of(bucket_value(i) as f64).1, i);
        }
        for v in [0u64, 1, 127, 128, 129, 255, 256, 1000, 123456, 99_999_999] {
            let i = hdr_index(v);
            assert!(hdr_upper(i) >= v && (i == 0 || hdr_upper(i - 1) < v), "{v}");
        }
        assert_eq!(
            classify_503(Some(br#"{"code":"requestProtectionUnavailable"}"#)),
            2
        );
        assert_eq!(classify_503(Some(b"")), 3);
        assert_eq!(classify_503(None), 4);
    }
}
