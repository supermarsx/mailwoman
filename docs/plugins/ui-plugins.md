---
title: UI plugins
---

# UI plugins

A UI plugin is a bundle of script for the web client that an administrator registers, approves and grants capabilities. It is meant to run in a sandboxed frame that cannot reach the page, cookies, storage or network except through a checked channel to the server. For engine plugins that run in the server, see [Plugins](./index.md).

::: warning What is connected today
The registry, signature check, approval, grants and the checked channel (the broker) exist and are tested. Two parts are missing:

- The server checks the signature of the bundle you upload but does not store the bundle, and the web client has no way to fetch it. The frame therefore starts with no plugin code in it.
- The extension points in the manifest (`compose-action`, `message-toolbar`, `settings-panel`) are stored but are not mapped to places in the composer, the message toolbar or the settings screen.

An approved and enabled UI plugin appears as a labelled, empty frame in the web client. It does not add controls. Do not plan a deployment that depends on a UI plugin adding features.
:::

## Register a plugin (administrator)

1. Open **UI plugins** in the admin console and choose **Register a UI plugin**.
2. Provide the manifest, `ui-plugin.json`, in one of three ways: **Manifest file**, **Or the manifest's address on this server**, or **Or paste the manifest**. The console reads only from its own server, so download a manifest from elsewhere and choose it as a file.
3. Check **What the manifest declares**: id, signature, capabilities and extension points.
4. For a signed manifest, choose the **Bundle file**. The signature is checked against it.
5. For a manifest without a signature, turn on **Allow this plugin to register without a signature**. Without it the server refuses the registration.
6. Choose **Register**.
7. Open the plugin and choose **Approve and enable** (then **Enable** or **Disable** later). **Disable** and **Delete** ask first.
8. Under **Grant a capability**, choose **Grant** for each capability the manifest declares. `net:host-allowlist` takes a list of hosts. The page does not show which capabilities are already granted; granting again replaces the earlier grant.

<!-- screenshot: admin-ui-plugins-register | Admin > UI plugins > Register a UI plugin with a manifest preview (id, signature, capabilities, extension points) | element: register form | callouts: 1 manifest file, 2 bundle file, 3 allow unsigned, 4 Register | stack M | seed admin-basic -->

A plugin is offered to signed-in users once it is approved and enabled. The permissions **Plugins** (`cap.plugins`) and **Individual UI plugin** (`cap.ui_plugin.*`, per plugin) must also be on for the user.

## Manifest

| Field | Meaning |
|---|---|
| `id`, `name`, `version` | Text. |
| `signature` | A detached Ed25519 signature over the bundle bytes, base64-encoded, or `null` for an unsigned plugin. |
| `extensionPoints` | A list of `compose-action`, `message-toolbar` or `settings-panel`. |
| `capabilities` | A list of `ui:compose-action`, `ui:message-toolbar`, `ui:settings-panel`, `net:host-allowlist` or `store:kv-scoped`. The server rejects any other name. |
| `csp` | Advisory. The host sets its own policy and the manifest cannot widen it. |

## Signatures

A signed plugin's signature is checked against the public keys in `MW_UI_PLUGIN_TRUST_KEYS`, a comma-separated list of hex-encoded 32-byte Ed25519 public keys set when the server starts. When the variable is empty, every signed plugin is refused. A plugin registered without a signature shows the persistent banner **Unsigned plugins are running** to users, listing the plugin ids. The banner cannot be dismissed by the plugin.

## The sandbox

- The plugin runs in an `<iframe srcdoc>` with `sandbox="allow-scripts"` and no `allow-same-origin`. The frame has an opaque origin, with no access to the page's DOM, cookies or storage.
- The frame's content security policy is set by the host: `default-src 'none'`, inline script and style only, `img-src data:`, `font-src data:`, `connect-src 'none'`, no forms, no base URI and no framing. The plugin can open no network connection of its own.
- The plugin talks to the host with `window.mailwoman.rpc(cap, method, ...args)`, which sends a message in the form `{v, id, cap, method, args}` and gets `{v, id, ok}` or `{v, id, err}` back. The host accepts a message only from the plugin's own frame.
- The broker refuses a call whose capability is not granted or whose method is not listed for that capability.

| Capability | Methods |
|---|---|
| `ui:compose-action`, `ui:message-toolbar`, `ui:settings-panel` | None. Rendering is driven by the host. |
| `net:host-allowlist` | `fetch(url)`: a GET to a host on the plugin's granted host list (exact host or a subdomain). Another host is refused. |
| `store:kv-scoped` | `get(key)` and `put(key, value)`, in storage private to the plugin and the signed-in account. |

Errors are `capability-denied`, `method-denied`, `bad-request`, `not-implemented` and `internal`. Calls go to `POST /api/ui-plugins/{id}/rpc`, which needs a mailbox session.

## Related

- [Plugins](./index.md)
- [UI plugins in the admin console](../admin/ui-plugins.md)
- [Plugin security](../security/plugins.md)
