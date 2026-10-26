# Nextcloud integration (V7)

V7 (release 26.8.0) adds a Nextcloud integration: a small first-party plugin
(`plugins/nextcloud`, package `nextcloud-plugin`) plus web UI. It lets a user:

- **attach from Nextcloud** — pick a file from Nextcloud (WebDAV) to attach,
- **save attachment to Nextcloud** — store a received attachment back to Nextcloud,
- **share large attachments as links** — create a Nextcloud share link (with optional
  password and expiry) instead of attaching bytes to the message.

Current account linking and DAV operations use the server-managed integration described in [Nextcloud account links](../admin/nextcloud.md). Native-engine users explicitly discover and connect calendar, task and address-book collections, then synchronize them on demand. The historical guest plugin does not automatically enroll these collections.

## Historical guest plugin

The optional guest plugin creates share links via Nextcloud's **OCS/WebDAV** APIs over the in-tree
`reqwest` (rustls) + `quick-xml` — no new dependency. As a WASM plugin it uses the host
`http-fetch` import under a `net_allowlist`.

Credential handling: the Nextcloud guest sends **no credentials** itself. The host
injects the linked account's authentication for the allowlisted Nextcloud host, so the
plugin never holds the user's Nextcloud secret.

## Setup

1. An administrator approves an HTTPS instance; each mailbox user links their own Nextcloud username and app password.
2. For native PIM, discover and explicitly connect collections under that account link, then synchronize on demand.
3. Attach-from and save-to actions appear in the compose and read UI. **Share-link
   actions do not** — see "Share links" below; the component exists but nothing
   mounts it.

### Share links

**Not reachable from the composer yet (recorded 26.19).** The intended behaviour:
choosing "share via link" for a large attachment creates a Nextcloud share and inserts
the link, with an optional password and expiry.

The component that does all of this is written, complete and exported
(`modules/nextcloud/ShareLinkComposer.tsx` — password, expiry, `onCreated`). The
composer imports only `NextcloudAttach` from the same barrel, so the share-link
component's **only** mount is its own unit test. This is a missing import, not a
missing feature: it is the smallest gap in the tree with a documentation claim
attached to it.

## CI

The Nextcloud plugin builds to `wasm32-wasip2` in the `wasm-plugin-build` job (via its
`build.sh`), and its share-link logic is host-tested against fixtures.
