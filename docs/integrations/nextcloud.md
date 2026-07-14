# Nextcloud integration (V7)

V7 (release 26.8.0) adds a Nextcloud integration: a small first-party plugin
(`plugins/nextcloud`, package `nextcloud-plugin`) plus web UI. It lets a user:

- **attach from Nextcloud** — pick a file from Nextcloud (WebDAV) to attach,
- **save attachment to Nextcloud** — store a received attachment back to Nextcloud,
- **share large attachments as links** — create a Nextcloud share link (with optional
  password and expiry) instead of attaching bytes to the message.

CalDAV / CardDAV / tasks already work in Mailwoman core (`mw-dav`); the plugin just
auto-configures them from the linked Nextcloud account — no separate setup.

## How it connects

The plugin creates share links via Nextcloud's **OCS/WebDAV** APIs over the in-tree
`reqwest` (rustls) + `quick-xml` — no new dependency. As a WASM plugin it uses the host
`http-fetch` import under a `net_allowlist`.

Credential handling: the Nextcloud guest sends **no credentials** itself. The host
injects the linked account's authentication for the allowlisted Nextcloud host, so the
plugin never holds the user's Nextcloud secret.

## Setup

1. Link a Nextcloud account (base URL + credentials / app password).
2. The plugin auto-configures CalDAV/CardDAV/tasks against that account.
3. Attach-from / save-to / share-link actions appear in the compose and read UI.

### Share links

When composing, choosing "share via link" for a large attachment creates a Nextcloud
share and inserts the link. You can set an optional password and expiry on the share.

## CI

The Nextcloud plugin builds to `wasm32-wasip2` in the `wasm-plugin-build` job (via its
`build.sh`), and its share-link logic is host-tested against fixtures.
