---
title: Plugins
---

# Plugins

Mailwoman has two plugin systems. **Engine plugins** are WebAssembly components that run inside the server in a sandbox. **UI plugins** are bundles for the web client that run in a sandboxed frame. An administrator manages both in the admin console, under **Plugins** and **UI plugins**.

Both systems are deny-by-default: a plugin does nothing until an administrator has approved it, granted capabilities and enabled it.

## What runs today

Only two kinds of engine plugin are started by the server:

- **Bridges** (a plugin that declares the `account-backend` capability) serve a mailbox from a provider. See [Mail provider bridges](../bridges/index.md).
- **One spam classifier** (a plugin that declares `spam-action`) checks new inbox messages. See [Spam classifier plugins](./spam-trainers.md).

Plugins that only declare other capabilities are registered and can be approved, but the server has no code that calls them. The console reports them as **Not loaded: this server version does not call the hooks this plugin provides.**

| Plugin id | Capabilities | Role | Status |
|---|---|---|---|
| `bridge-graph` | `account-backend`, `net`, `addrbook-source`, `store-kv-scoped` | Microsoft Graph mailbox | Loaded per connected mailbox |
| `bridge-gmail` | `account-backend`, `net` | Gmail API mailbox | Loaded per connected mailbox |
| `bridge-ews` | `account-backend`, `net`, `addrbook-source` | Exchange mailbox | Loaded per mailbox on an EWS domain |
| `spam-rspamd` | `spam-action`, `net`, `store-kv-scoped` | Rspamd classifier | Loaded as the classifier |
| `spam-spamassassin` | `spam-action`, `net`, `store-kv-scoped` | SpamAssassin classifier | Loaded as the classifier |
| `languagetool` | `dlp-detector`, `net` | Grammar check hook | Never loaded |
| `nextcloud` | `message-pipeline`, `net` | Share-link hook | Never loaded; Nextcloud works through the [native integration](../integrations/nextcloud.md) |

The components are the `.wasm` files in the plugin directory. A masked-email plugin source exists in the repository but is not built or included in releases; [masked email addresses](./masked-email.md) work without it.

## Approve and enable

![The page of the Microsoft Graph bridge plugin: Status shows First-party, digest built in and Not loaded because an administrator has not approved it; below are Granted capabilities with Save grants, Network and limits, and Uninstall.](/screenshots/integrations/admin-plugin-detail.webp){.mw-shot}

1. Open **Plugins** in the admin console. If the plugin is not listed, choose **Register a plugin**, pick the component and enter its id.
2. Choose **Approve** on the plugin's row in the list. Choose **Open** to see its page.
3. On the plugin's page, under **Granted capabilities**, tick what it needs and choose **Save grants**. A bridge needs `account-backend` and `net`. Saving replaces the earlier grant; leaving a capability out revokes it.
4. Back in the list, choose **Enable** on the plugin's row.

**Approve** and **Enable** act at once; **Disable** asks first. The list shows **Trust** (**First-party, digest built in** or **Third-party, digest approved**), **Running** (**Loaded**, **Not loaded**, **Restart required**) and **Approval**. When a plugin is not loaded, its page shows the reason, for example:

- **Not loaded: an administrator has not approved this plugin.**
- **Not loaded: no capability it needs has been granted.**
- **Not loaded: no component file passed the digest check.**
- **Not loaded: this server runs in proxy mode and does not process mail itself.**
- **Not loaded: no account is bound to this bridge.**
- **Not loaded: another spam classifier is loaded. Only one runs at a time.**

A plugin can be approved and enabled and still show **Not loaded**, for example a bridge that no mailbox is bound to yet:

![The Plugins list with the Microsoft Graph bridge showing Approved and Enabled in the Approval column and Not loaded in the Running column.](/screenshots/integrations/admin-plugin-approved.webp){.mw-shot}

Starting a plugin that was not running needs a server restart (**Restart required to load this plugin.**). A change that removes permission stops a running bridge without a restart.

## Capabilities

| Capability | Meaning |
|---|---|
| `account-backend` | Serve a mailbox: sync, fetch, flags, move, send. Only first-party plugins can hold it. |
| `net` | Outbound HTTP to the hosts in the plugin's host list. |
| `spam-action` | Classify a message as spam, ham or unknown. |
| `dlp-detector` | Return data-loss findings for a message body. Not called by the server. |
| `addrbook-source` | Search an address book. Not called by the server. |
| `autoconfig-source` | Look up mail settings for an address. Not called by the server. |
| `message-pipeline` | Rewrite messages on the way in or out. Not called by the server. |
| `store-kv-scoped` | A small key-value store private to the plugin and account. |

The effective grant is the capabilities the plugin declares that the administrator also granted. Nothing undeclared is available.

## Sandbox and limits

- **No ambient access.** A plugin has no file system, network, environment or standard input and output. Every capability goes through a host function the server checks.
- **Memory and time.** Each call runs with a memory ceiling and a time limit per call, and an optional fuel budget. Plugin defaults are 64 MiB and 5,000 ms; the bundled bridges declare 64 to 128 MiB and 10 to 15 seconds, the spam classifiers 32 MiB and 10 seconds. A plugin that exceeds a limit gets a typed error; the server continues. The plugin page shows **Memory**, **Time per call** and **Fuel per call**.
- **Network.** A plugin can contact only the hosts in its list, set when it is registered. To change them, uninstall and register again. Requests use the **Approved plugin and provider requests** traffic class in **Egress**. A failed configured route never falls back to a direct connection, and redirects are not followed. A request has a 15-second deadline and an 8 MiB request and response limit. Only the methods GET, HEAD, POST, PUT, PATCH, DELETE and OPTIONS and a short list of headers (`authorization`, `content-type`, `accept`, `soapaction`, `prefer`, `consistencylevel`, `if-match`, `if-none-match`, `range`) are accepted. Cookies and proxy credentials are never forwarded.
- **Private addresses.** A hostname grant does not allow private addresses. An on-premises service needs an internal-destination entry for that traffic class. Metadata and reserved addresses stay blocked. SOCKS with remote name resolution is refused for plugin requests.
- **Stored data.** The key-value store is separate for each plugin and account. Limits per namespace: 256 bytes per key, 64 KiB per value, 5 MiB in total and 1,000 keys. Administrators can change them with `plugins.kv.maxKeyBytes`, `plugins.kv.maxValueBytes`, `plugins.kv.maxTotalBytes` and `plugins.kv.maxKeys` (environment `MW_PLUGIN_KV_MAX_KEY_BYTES`, `MW_PLUGIN_KV_MAX_VALUE_BYTES`, `MW_PLUGIN_KV_MAX_TOTAL_BYTES`, `MW_PLUGIN_KV_MAX_KEYS`).
- **Logging.** A plugin can write log lines, each with a level and a message.

## Where components come from

| Setting | Meaning |
|---|---|
| `MW_PLUGIN_DIR` | Directory of first-party components. The server also looks in a `plugins` folder next to the server binary and in `/usr/lib/mailwoman/plugins`. |
| `MW_THIRDPARTY_PLUGIN_DIR` | Directory of third-party components. |

A first-party component loads only if its SHA-256 digest equals the digest built into the server. A file that does not match is skipped and logged.

A third-party component is a `<id>.wasm` file in the third-party directory. The **Third-party plugin allowlist** at the bottom of the **Plugins** page lists the files found with the SHA-256 the server computed. Approving a digest pins those exact bytes and asks you to type the id. Revoking removes the approval and disables the plugin. The registration form has no signature field, so a third-party plugin runs only when **Allow this unsigned plugin to run** is on for it; a banner then reports that an unsigned plugin is loaded.

Third-party components cannot hold `account-backend`. In practice the only third-party plugin the server calls is a spam classifier.

**Uninstall** deletes the plugin's stored data for every account, its allowlist pins, its registration and its grants. The component file stays on disk. The confirmation asks you to type the plugin id.

## Writing a plugin

The interface is `crates/mw-plugin/wit/plugin.wit`, package `mailwoman:plugin@0.1.0`. A component targets the `plugin` world, or `plugin-pim` (which adds `calendar`, `tasks` and `bridge-parity` exports). Host functions available to a plugin are `http-fetch`, `oauth-token` and `basic-credentials` (account-bound, for bridges), `kv-get`, `kv-put`, `kv-delete`, `kv-list`, `log`, `now` and `random`. Hooks a component can export are `account-backend`, `message-pipeline`, `addrbook-source`, `autoconfig-source`, `dlp-detect` and `spam-action`. Only `account-backend` and `spam-action` are called by the server. The manifest declares `id`, `name`, `version`, `capabilities`, `net_allowlist` and `[limits]`. Building uses the `wasm32-wasip2` target; see the build scripts in `plugins/`.

## UI plugins

UI plugins are a separate system with their own sandbox. See [UI plugins](./ui-plugins.md).

## Related

- [UI plugins](./ui-plugins.md)
- [Spam classifier plugins](./spam-trainers.md)
- [Mail provider bridges](../bridges/index.md)
- [Plugins in the admin console](../admin/plugins.md)
- [Plugin security](../security/plugins.md)
