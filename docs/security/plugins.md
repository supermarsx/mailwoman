# Engine plugins: trust, capabilities and limits

Engine plugins are WebAssembly components that the server runs inside a
[wasmtime](https://wasmtime.dev/) sandbox. The provider bridges (Microsoft Graph, EWS,
Gmail) and the spam classifiers (Rspamd, SpamAssassin) are engine plugins. This page
describes which components the server agrees to run, what a running plugin can reach,
and the resource limits it runs under.

UI plugins, which run in the browser, are a separate mechanism; see
[UI plugins](/admin/ui-plugins).

## Which plugins the server loads

The server loads an engine plugin only in one of two roles:

| Role | Declared capability | When it is loaded |
|---|---|---|
| Bridge (an account backend) | `account-backend` | At start-up, once per account bound to that bridge |
| Spam classifier | `spam-action` | At start-up and after every registry change |

The plugin interface also defines DLP-detector, message-pipeline, address-book and
autoconfig hooks. The server has no caller for those hooks, so a plugin that declares
only them is registered but never loaded.

## Which components are trusted

Trust is decided by the SHA-256 digest of the component file:

- **First-party plugins** (`bridge-graph`, `bridge-ews`, `bridge-gmail`,
  `languagetool`, `nextcloud`, `spam-rspamd`, `spam-spamassassin`) are trusted by a
  digest compiled into the server. The server looks for `<id>.wasm` in
  `MW_PLUGIN_DIR`, then in `plugins/` beside the server binary, then in
  `/usr/lib/mailwoman/plugins`; if its bytes do not hash to the built-in digest, it is not loaded.
  No administrator setting overrides this.
- **Third-party plugins** load only from `MW_THIRDPARTY_PLUGIN_DIR` (unset by default,
  which turns third-party loading off). The file must be named after the plugin id,
  and its SHA-256 must match a digest an administrator approved in the plugin
  allowlist. The server reads the file once, hashes those bytes and loads the same
  bytes, so a file swapped after the check is not loaded. Revoking a digest disables
  the plugin and stops it if it is running.

<!-- screenshot: id=security/plugins-registry; screen=Admin > Plugins; state=one first-party plugin and one third-party plugin with an approved digest; element=plugin list with trust column (First-party, digest built in / Third-party, digest approved) -->

A component's manifest may carry an Ed25519 signature field. The server holds no
trusted signing key for engine plugins, so a manifest that carries a signature fails
to load. In practice a third-party component has no signature and runs only after an
administrator also turns on **Running without a signature** for it; the console then
shows a permanent warning that the server checks the file against its approved digest
only. The digest pin is what identifies the code; it says nothing about whether the
code is safe.

Registering, approving, enabling, disabling, granting and allowing unsigned all
require an admin session with the plugin capability (`plugins.write`), and each change
writes an admin audit record. A refused load is also audited.

## Capabilities

A plugin's manifest (`plugin.toml`) lists the capabilities it asks for. The
capabilities a running plugin actually has are the intersection of the manifest and
the administrator's grants. A third-party plugin never receives `account-backend`,
whatever is granted, because that capability lets a plugin act as the user's mailbox
and send as the user.

| Capability | What it allows |
|---|---|
| `account-backend` | Act as the mail backend for a bound account; ask the host for that account's OAuth access token or stored password (first-party only) |
| `net` | Outbound HTTP through the host, to hosts in the manifest's `net_allowlist` |
| `spam-action` | Classify messages as a spam classifier |
| `store-kv-scoped` | A small key-value store private to the plugin (and account) |
| `dlp-detector`, `message-pipeline`, `addrbook-source`, `autoconfig-source` | Hooks the server does not call |

A call to a host function whose capability was not granted returns an error to the
plugin and does nothing.

### What a plugin can reach

- **No ambient system access.** The WebAssembly System Interface context has no
  preopened directories, environment variables, arguments, standard input/output or
  sockets. Standard WASI clocks and random numbers are available, as are the host's
  own `now` and `random` functions.
- **HTTP only through the host.** `http-fetch` requires `net` and a URL whose host
  matches the manifest's `net_allowlist` (an empty list means no network). The server
  then checks again, on every request, that the plugin is still enabled and approved,
  that `net` is still granted and that the host is listed, and sends the request
  through the [egress policy](./egress.md) under the `plugins` traffic class, so
  private, loopback and metadata addresses are refused and an egress route applies.
  An `Authorization` header is passed on only by the first-party bridges, only for a
  bound account, and only to their provider's fixed host (`graph.microsoft.com`,
  `gmail.googleapis.com`) or the EWS endpoint configured for that account.
- **Credentials.** `oauth-token` gives a bridge a short-lived access token for its
  bound account; the refresh token and client secret stay on the host.
  `basic-credentials` gives the EWS bridge the stored user name and password for its
  bound account, because NTLM needs the password. Both require `account-backend`, so
  only first-party bridges can call them.
- **Logging.** Messages a plugin passes to `log` are discarded.

::: warning
A first-party bridge sees the full content of the mailbox it serves, and the EWS
bridge sees that account's password. Their trust rests on the digest built into the
server, which identifies the exact code shipped with your release.
:::

## Resource limits

Each plugin instance runs with limits from its manifest (`[limits]`), defaulting to:

| Limit | Default | Mechanism |
|---|---|---|
| `memory_mb` | 64 MiB | wasmtime resource limiter on linear memory |
| `deadline_ms` | 5000 ms | Epoch interruption (a wall-clock deadline) |
| `fuel` | none | Optional instruction budget, in addition to the deadline |

Exceeding a limit stops the call with a `LimitExceeded` error; the server process
keeps running.

## Writing a plugin

Plugins are `wasm32-wasip2` components built against the WIT world `mailwoman:plugin`
in `crates/mw-plugin/wit/plugin.wit`. A guest exports the interfaces it implements and
imports the `host` interface (`http-fetch`, `oauth-token`, `basic-credentials`,
`kv-get`, `kv-put`, `kv-delete`, `kv-list`, `log`, `now`, `random`).

A minimal manifest:

```toml
id = "com.example.classifier"
name = "Example classifier"
version = "1.0.0"
capabilities = ["spam-action", "net"]
# Exact host names or *.suffix wildcards; empty means no network.
net_allowlist = ["classifier.example.com"]

[limits]
memory_mb = 64
deadline_ms = 5000
```

The server's own HTTP check compares the request host with the allowlist entries
exactly, so list each host by name.

Build with the `wasm32-wasip2` target; the reference guest in
`crates/mw-plugin/tests/guest-fixture/` and each first-party plugin under `plugins/`
has a `build.sh`. To deploy a third-party plugin, copy `<id>.wasm` into
`MW_THIRDPARTY_PLUGIN_DIR`, approve its digest in the plugin allowlist, register the
id, grant capabilities, allow it to run unsigned, and enable it.

## Related

- [Plugins (admin)](/admin/plugins)
- [Spam trainers](/plugins/spam-trainers)
- [Outbound connections](./egress.md)
