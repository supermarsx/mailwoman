# Desktop shell (Tauri v2) — install & self-contained mode

The V5 desktop shell (`apps/desktop`, Windows/macOS/Linux) is a **thin client**: it
ships the **same built SPA the server serves** (`apps/web/dist`, hash-verified at
launch per SPEC §7.4) and adds OS integration only — **no protocol logic, no forked
UI**. It points at a Mailwoman server exactly like the browser does, over the same
JMAP surface. Everything here is additive; the browser deployment is unchanged.

## Two ways to run it

1. **Connected to a server** (the default) — the shell loads the SPA and points it at
   your Mailwoman server (work and/or personal; multiple servers are supported). The
   transport is identical to the browser's, with one addition: native clients use a
   **bearer token** instead of the cookie (see [Native auth](#native-auth-bearer-token)),
   so there is no cross-origin cookie/CSRF fight.
2. **Self-contained** (§4.1) — for a laptop user with no server to deploy, the shell
   **spawns a bundled `mw-server` as a sibling process** on loopback
   (`127.0.0.1:<ephemeral>`), health-probes `/healthz`, and points the SPA at it. The
   engine is a **spawned process, never linked into the shell**. The app then works
   fully offline of any external server.

## Building the shell

`tauri build` produces the app. The repo scripts sequence the build correctly (the
§7.4 bundle-hash must be emitted from the same `dist` the shell embeds):

```sh
# Linux/macOS
bash scripts/build-shells.sh
# Windows
pwsh scripts/build-shells.ps1
```

`build-shells` (1) builds the shared SPA, (2) emits the UI-bundle hash, (3) bundles
the release `mw-server` into the shell resources for self-contained mode, (4) runs
`tauri build --no-bundle`, and (5) asserts the §16 size budgets (see below). Set
`MW_SELF_CONTAINED=0` to build a thin shell without the bundled engine.

CI builds this on **Windows + Linux** (macOS best-effort) in the `desktop-shell` job.

### Bundle-size budgets (§16)

`scripts/check-bundle-size.mjs` (run as CI step 5) enforces:

| Variant | Budget | Contents |
|---|---|---|
| **Thin shell** | **< 10 MB** | SPA + Tauri/WebView runtime, no engine |
| **Self-contained** | **< 40 MB** | thin shell + the bundled sibling `mw-server` |

The engine appears **only** in the self-contained variant, as the bundled
`mw-server` resource — never linked into the thin shell.

## UI-bundle integrity gate (§7.4)

At build time the shell records the SHA-256 of every embedded `dist` asset
(`bundle-hash.json`, compiled in via `include_str!`). On launch, **before** it loads
the SPA or points at any server, the shell re-hashes every embedded asset and aborts
if anything mismatches or is missing (tamper gate). A legit build logs
`UI-bundle integrity OK: N/N files match`.

## Native auth (bearer token)

The browser keeps its HttpOnly `mw_session` cookie + CSRF path **verbatim**. Native
clients opt in: `POST /api/login` with `{ "clientType": "native" }` returns a **bearer
token** in the JSON body and sets **no cookie**. The shell stores that token in the OS
keychain (Windows Credential Manager / macOS Keychain / Linux Secret Service, via the
`keyring` crate) and sends it as `Authorization: Bearer <token>` on `/jmap/*` and
`/api/*`. Bearer requests skip the cookie-only CSRF guard (no ambient authority to
protect). Token rotation uses the existing `/api/session/rotate`.

For a remote server to accept the shell origin, set `MW_NATIVE_ORIGINS` (see
[push.md](./push.md#server-configuration)); it is **empty/off by default**, so
browser-only deployments are unaffected.

## OS integration

All native capabilities are reached through the SPA's feature-detected capability
layer (`apps/web/src/platform`), which degrades gracefully in a plain browser:

- **Native notifications** with action buttons (archive / delete / reply),
- **OS keychain** wrapping the session token + the client key-vault passphrase,
- **Default-`mailto:` handler** + `mailwoman:` deep links,
- **Share targets**, **badge counts**, **biometric app-lock** (Windows Hello),
  **drag-out attachments**,
- **Screen-capture protection** — real OS exclusion on Windows/macOS; see the honest
  matrix in [`../security/screen-capture.md`](../security/screen-capture.md).

## Auto-update

The Tauri updater is wired (`tauri-plugin-updater`) but ships **inactive**: V5 does
not run a hosted update feed or ship a signing key. A hosted update feed + signing
account is an **ops/sponsorship follow-up** (§28.7), not a V5 gate — the config and
plumbing are present so enabling it later needs no code change.
