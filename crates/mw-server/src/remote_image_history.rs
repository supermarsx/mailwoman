//! Compatibility path; implementation lives in `mw-remote-content`.
pub(crate) use mw_remote_content::history::*;

/// Assemble host-owned gauges without making the cache crate depend on HTTP handlers.
fn gather() -> Sample {
    mw_remote_content::history::gather(HostCounters {
        trackers: super::remote_content::trackers_replaced(),
        coalesced: super::remote_content::image_flight::coalesced(),
        handle_hits: super::remote_content::handle_cache::counters().0,
        handle_misses: super::remote_content::handle_cache::counters().1,
        high_priority: super::high_priority_requests(),
        stale_served: super::swr::counters().stale,
        revalidated: super::swr::counters().revalidated + super::swr::counters().replaced,
        active: super::fetch_semaphore().active() as u64,
        queued: super::remote_content::queue_depth() as u64,
        accounts: super::rate_limiter()
            .lock()
            .map_or(0, |l| l.active_accounts() as u64),
    })
}
pub(crate) fn ensure_sampler() {
    mw_remote_content::history::ensure_sampler(gather);
}
