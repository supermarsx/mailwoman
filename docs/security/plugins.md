# Plugins: authoring, signing, capabilities, and resource limits (V7)

V7 (release 26.8.0) adds an engine-plugin runtime (`mw-plugin`): a **wasmtime +
WASI-p2 (component model)** host that loads admin-approved, capability-gated
plugins. The bridges (Graph/EWS/Gmail), the LanguageTool grammar plugin, and the
Nextcloud plugin all run through this host. This document covers writing a plugin,
signing it, the capability model, and the resource limits the host enforces.

The host is the trust boundary. `mw-plugin` is `#![forbid(unsafe_code)]` at its own
boundary — the `wasmtime` dependency carries its own `unsafe` internally, which is
the point: the host mediates every capability a guest can exercise.

## The plugin ABI (WIT)

Plugins are `wasm32-wasip2` **components** built against the frozen WIT world
`mailwoman:plugin` (`crates/mw-plugin/wit/plugin.wit`). A guest exports the hooks it
implements; the host provides a fixed set of imports.

**Guest exports** (each optional, capability-gated):

- `account-backend` — the engine account-backend seam (the bridge role, §6.5):
  `list-mailboxes`, `sync(cursor)`, `fetch(refs)`, `store-flags`, `move`, `submit`,
  `poll-changes`. Loaded plugins that export this are indistinguishable from a native
  backend such as `mw-imap`.
- `message-in` / `message-out` — the message pipeline hooks.
- `addrbook-source` — an address-book/GAL source.
- `autoconfig-source` — an autoconfig source.
- `dlp-detect` — a DLP detector.
- `spam-action` — a spam-action hook.

**Host imports** (the only authority a guest has):

- `http-fetch(req) -> resp` — host-mediated HTTP. The host enforces the manifest
  `net_allowlist`; a guest **cannot open a socket** and cannot reach any host outside
  the allowlist.
- `oauth-token(account) -> token` — the host acquires and injects the OAuth token; the
  guest never sees client secrets or refresh tokens.
- `kv-get` / `kv-put` — a scoped KV scratch namespace.
- `log(level, msg)` — logging with a no-content floor.
- `now`, `random` — the host clock and RNG (a guest has no ambient WASI clock/RNG).

Two ABI notes frozen during V7 (relevant if you regenerate bindings):

- `flags` is spelled `msg-flags` (WIT reserves `flags`).
- change delivery is the synchronous `poll-changes: func() -> result<list<change-event>>`
  (bridges delta-poll), not a WASI stream.

Reference guest + reproducible build: `crates/mw-plugin/tests/guest-fixture/`
(`build.sh` builds the component). `plugins/languagetool/` is the smallest real
plugin and the canonical jail example.

## The manifest (`plugin.toml`)

Every plugin ships a manifest. **The host denies everything not declared here.**

```toml
id = "com.example.myplugin"
name = "My Plugin"
version = "1.0.0"
# Hex-encoded detached Ed25519 signature over the component bytes (see Signing).
signature = "…"
# Capabilities the plugin requires — granted only after admin approval.
capabilities = ["net", "dlp-detector"]
# Hosts http-fetch may reach. EMPTY ⇒ no outbound network at all.
net_allowlist = ["api.example.com"]

[limits]
memory_mb = 64      # linear-memory ceiling (MiB)
deadline_ms = 5000  # CPU wall-clock deadline (epoch-interruption)
# fuel = 20000000   # optional deterministic fuel budget, in addition to the deadline
```

## Capability model (deny by default)

A capability is a named permission the manifest requests and an admin grants. Nothing
is implicit. The capabilities:

| Capability | Grants |
|---|---|
| `account-backend` | implement the engine account-backend seam (a bridge) |
| `net` | outbound `http-fetch`, restricted to `net_allowlist` |
| `dlp-detector` | a DLP detector hook |
| `spam-action` | a spam-action hook |
| `addrbook-source` | an address-book source |
| `autoconfig-source` | an autoconfig source |
| `message-pipeline` | a message in/out pipeline hook |
| `store-kv-scoped` | a scoped KV scratch namespace |

Enforcement is structural, not advisory:

- A hook whose capability was not granted returns a typed **`CapabilityDenied`** — the
  guest export is never invoked.
- `http-fetch` to a host outside `net_allowlist` is **denied** even when `net` is
  granted. An empty allowlist means no outbound network.
- There is no ambient WASI authority: no default filesystem, clock, RNG, or network
  beyond the declared imports.

## Resource limits

The host bounds every plugin instance (`PluginLimits`):

- **Memory** — a linear-memory ceiling (`memory_mb`) via a `wasmtime::ResourceLimiter`.
  Exceeding it traps cleanly.
- **CPU** — a wall-clock **deadline** (`deadline_ms`) via epoch-interruption. A busy
  loop is interrupted; the host survives.
- **Fuel** — an optional deterministic instruction budget (`fuel`) in addition to the
  deadline.

Any trip returns a typed **`PluginError::LimitExceeded`** — never a panic, never a
host crash. Instances are recycled per session. Defaults lean conservative
(64 MiB / 5000 ms); tune per hook at grant time.

The `jail` CI job proves this against the real LanguageTool component: it loads only
when its capability is granted, `http-fetch` is denied outside the allowlist, the DLP
hook is denied without its capability, and the memory/deadline ceilings trip cleanly.

## Signing (Ed25519)

Plugins are verified against a **detached Ed25519 signature over the component
bytes**, recorded hex-encoded in the manifest `signature` field. On load the host
verifies the signature against the configured trust root:

- **Valid signature** → loads normally.
- **Missing/invalid signature** → loads **only** if the admin has set
  `allow_unsigned`. An unsigned load is flagged on the handle so the UI shows a
  **permanent unsigned banner** and writes an audit record. Absent `allow_unsigned`,
  the load fails with `SignatureInvalid`.

To sign: produce a detached Ed25519 signature over the exact `.wasm` component bytes
with the registry signing key, hex-encode it, and place it in `signature`. The trust
root (the public key) is a server config path.

## Build & verify

Build a component to `wasm32-wasip2`; the `wasm-component-ld` linker componentizes the
cdylib automatically from wit-bindgen's embedded `component-type` sections — no
`wasm-tools component new` step is needed. Each first-party plugin ships a `build.sh`
that adds the target, builds `--release`, and refreshes the committed fixture. The
`wasm-plugin-build` CI job runs every `build.sh` and asserts each output carries the
component-model preamble (`00 61 73 6d 0d 00 01 00`), not a bare core module.

## Scope boundary (honest)

- **Engine (WASM) plugin tier only.** The declarative **TypeScript UI-plugin tier
  (§22.2) is not implemented in V7** — it is document-only and tracked for 1.0.
- **The WIT exports the account-backend (MAIL) interface.** Bridges also implement and
  fixture-test calendar/tasks/reactions/voting (advertised via `capabilities()`), but
  those are **not yet drivable through the plugin seam** — the WIT export for them is a
  post-1.0 extension. See `docs/bridges/` and `docs/RELEASE-NOTES-26.8.md`.
