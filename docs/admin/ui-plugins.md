---
title: UI plugins in the admin console
screen: true
---

# UI plugins

The **UI plugins** page lists the registered web-client plugins with their signature state and whether they are approved and enabled. This page describes the screen. For what a UI plugin is, the sandbox it would run in and what is not connected, read [UI plugins](/plugins/ui-plugins) first.

::: warning What an approved plugin does today
The screen's own text says UI plugins add controls to the web client. They do not: the server does not store the bundle and the web client does not map extension points to places in the interface, so an approved and enabled plugin appears as an empty, labelled frame. Details are in [UI plugins](/plugins/ui-plugins).
:::

![The UI plugins page listing one registered plugin with its signature state and its approval.](/screenshots/admin-ops/opsc-ui-plugins-list.webp){.mw-shot}

## Before you start

- Seeing the page needs the console permission `plugins.read`. Registering, approving, granting, enabling, disabling and deleting need `ui_plugins.write`.
- A plugin opens at `#/ui-plugins/<id>`. **Approve and enable** and **Enable** act at once. **Disable** and **Delete** ask first.

<a id="manifest"></a>
## Manifest

Choose **Register a UI plugin**. The form opens at `#/ui-plugins/+register`. Give it the plugin's `ui-plugin.json` in one of three ways: **Manifest file**, **Or the manifest's address on this server** (with **Read**), or **Or paste the manifest**. The console reads only from its own server, so a manifest on another server has to be downloaded and chosen as a file.

![The Register a UI plugin form with a manifest pasted in and the preview of what the manifest declares.](/screenshots/admin-ops/opsc-ui-plugins-register.webp){.mw-crop}

1. **Or paste the manifest**.
2. **What the manifest declares** shows the id, name, version, signature, capabilities and extension points before anything is sent.
{.mw-callouts}

The console checks the manifest before it sends it: `id`, `name` and `version` must be text, `capabilities` and `extensionPoints` must be lists of text, and every capability must be one the server knows (`ui:compose-action`, `ui:message-toolbar`, `ui:settings-panel`, `net:host-allowlist`, `store:kv-scoped`). A manifest that fails shows the reason, for example **The manifest declares ui:example, which this server does not know.**

<a id="bundle"></a>
## Bundle file

**Bundle file** is required for a signed manifest: the server checks the signature against this file. A signed manifest without a bundle shows **A signed manifest needs its bundle file: the signature is checked against it.**

<a id="unsigned"></a>
## Registering without a signature

For a manifest whose signature is `null` the form shows **Allow this plugin to register without a signature**. Without it the server refuses the registration. An unsigned plugin that is later enabled makes the web client show a banner that unsigned plugins are running.

<a id="grants"></a>
## Granting capabilities

On the plugin's page, **Grant a capability** has a **Grant** button for each capability the manifest declares. `net:host-allowlist` also takes the hosts the plugin may fetch from, comma-separated. The server does not report which capabilities are already granted, so the page shows no grant state; granting again replaces the earlier grant.

## Approve, enable and delete

**Approve and enable** approves a plugin and turns it on. **Enable** and **Disable** switch an approved plugin. **Delete** removes the registration and its grants. After you register a plugin the page says **Approve it to offer it to people.**

## Options

This screen has no entries in the settings reference. A signed plugin is checked against the public keys in the environment variable `MW_UI_PLUGIN_TRUST_KEYS` (comma-separated hexadecimal Ed25519 keys). With the variable empty every signed plugin is refused.

## Troubleshooting

- **The manifest is not valid JSON.** The pasted text does not parse.
- **That address is on another server, which the console may not read from. Download the manifest and choose the file instead.**
- **The server refused the request:** followed by the server's reason.

## Related

- [UI plugins](/plugins/ui-plugins)
- [Plugins in the admin console](/admin/plugins)
- [Plugin security](/security/plugins)
