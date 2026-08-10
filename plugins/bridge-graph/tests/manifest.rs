//! The plugin id the host registry wires against.
//!
//! `plugin_id()` is what host-side registry code uses to find this bridge; the admin
//! registry (migration 0008) ingests `plugin.toml`. If the two ever disagree the
//! bridge simply never resolves — a silent "backend not found" rather than an error
//! that points anywhere useful, so it is worth one assertion.

#[test]
fn the_exported_plugin_id_matches_the_shipped_manifest() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/plugin.toml"))
        .expect("plugin.toml ships with the crate");
    let declared = manifest
        .lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("id ="))
        .map(|v| v.trim().trim_matches('"'))
        .expect("plugin.toml declares an id");

    assert_eq!(bridge_graph::plugin_id(), declared);
    assert_eq!(bridge_graph::plugin_id(), bridge_graph::PLUGIN_ID);
    assert_eq!(bridge_graph::PLUGIN_ID, "bridge-graph");
}
