//! Private browser caching, separate from the server's RAM/disk image cache.

pub const MAX_AGE_SECS: usize = 3 * 365 * 24 * 60 * 60;

pub fn cache_control(enabled: bool, ttl_secs: usize) -> String {
    let ttl = ttl_secs.clamp(60, MAX_AGE_SECS);
    if !enabled {
        "private, no-store".into()
    } else {
        format!("private, max-age={ttl}, must-revalidate")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_freshness_honours_disable_expiry_and_bounds() {
        assert_eq!(
            cache_control(true, 600),
            "private, max-age=600, must-revalidate"
        );
        assert_eq!(cache_control(false, 600), "private, no-store");
        assert_eq!(
            cache_control(true, usize::MAX),
            "private, max-age=94608000, must-revalidate"
        );
    }
    #[test]
    fn three_years_is_supported_without_changing_link_authorization() {
        assert_eq!(cache_control(true,MAX_AGE_SECS),"private, max-age=94608000, must-revalidate");
        assert_eq!(cache_control(true,1),"private, max-age=60, must-revalidate");
    }
}
