//! `nextcloud-plugin` — the Nextcloud large-attachment share-link engine hook
//! (plan §8, SPEC §18.4).
//!
//! A thin `message-pipeline` (message-out) component: given a small JSON share
//! request (a Nextcloud `base_url` + the uploaded file's WebDAV `path`, plus an
//! optional public-link `password`/`expiry`) it POSTs the Nextcloud OCS
//! `files_sharing` create-share API via the host `http-fetch` — **host-mediated,
//! under a net allowlist** — and returns the public share URL. The host's injected
//! fetcher (e14) attaches the linked account's credentials for the allowlisted host,
//! so the guest never handles secrets. The web attach/save/share UI is e7's (done);
//! this is the engine-side hook it drives through the mount (e14).
//!
//! CalDAV/CardDAV/tasks already work in core (`mw-dav`); this plugin only creates
//! share links. Build shapes mirror `languagetool` (guest behind
//! `#[cfg(target_arch = "wasm32")]`).

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "nextcloud";

/// The Nextcloud OCS `files_sharing` create-share path (appended to the request's
/// `base_url`). `?format=json` makes OCS answer JSON instead of its XML default.
pub const OCS_SHARES_PATH: &str = "/ocs/v2.php/apps/files_sharing/api/v1/shares";

/// OCS `shareType` for a public link (Nextcloud share types).
pub const SHARE_TYPE_PUBLIC_LINK: u8 = 3;

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

#[cfg(target_arch = "wasm32")]
mod component;
