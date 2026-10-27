//! Short-lived observations belong to one immutable SMTP transport authority.
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub(crate) struct Support {
    observed: Mutex<Option<(Instant, bool)>>,
    pub(crate) probe: tokio::sync::Mutex<()>,
}
impl Support {
    pub(crate) fn get(&self) -> Option<bool> {
        let guard = self.observed.lock().ok()?;
        let (at, supported) = (*guard)?;
        (at.elapsed() < Duration::from_secs(if supported { 300 } else { 30 })).then_some(supported)
    }
    pub(crate) fn record(&self, supported: bool) {
        if let Ok(mut guard) = self.observed.lock() {
            *guard = Some((Instant::now(), supported));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expired_positive_and_negative_observations_are_unknown() {
        let support = Support::default();
        assert_eq!(support.get(), None);
        support.record(true);
        assert_eq!(support.get(), Some(true));
        *support.observed.lock().unwrap() = Some((Instant::now() - Duration::from_secs(301), true));
        assert_eq!(support.get(), None);
        support.record(false);
        assert_eq!(support.get(), Some(false));
        *support.observed.lock().unwrap() = Some((Instant::now() - Duration::from_secs(31), false));
        assert_eq!(support.get(), None);
    }
}
