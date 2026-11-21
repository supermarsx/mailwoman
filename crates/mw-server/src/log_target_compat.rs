//! Startup/reload compatibility for the implementation-crate rename. No event
//! target rewriting. This mirrors EnvFilter's comma-separated directive input.
//! Generated aliases precede the unchanged input so explicit core selectors win.
pub(crate) fn legacy_server_directives(input: &str) -> String {
    let aliases = input
        .split(',')
        .filter_map(|directive| {
            let directive = directive.trim();
            let suffix = directive.strip_prefix("mw_server")?;
            if suffix.is_empty()
                || suffix.starts_with("::")
                || suffix.starts_with('[')
                || suffix.starts_with('=')
            {
                Some(format!("mw_server_core{suffix}"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if aliases.is_empty() {
        return input.to_owned();
    }
    format!("{},{}", aliases.join(","), input)
}
