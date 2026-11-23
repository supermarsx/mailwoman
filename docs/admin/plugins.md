---
title: Plugins in the admin console
screen: true
---

# Plugins

The **Plugins** page lists the engine plugins, the WebAssembly components that run inside the server, with their trust, whether they are running, and whether they are approved and enabled. This page describes the screen. What plugins are, which kinds the server actually runs, and the limits they work under are described in [Plugins](/plugins/); read that first if you are deciding whether you need one.

![The Plugins page of the administration console with the registered plugins, their trust, whether they are running and their approval, and the Register a plugin button.](/screenshots/admin-ops/opsc-plugins-list.webp){.mw-shot}

The page shows what the server reports. An enabled plugin the server has not loaded is listed as **Not loaded**, and its page says why. A plugin opens at `#/plugins/<id>`. Below the list is the **Third-party plugin allowlist**.

## Before you start

- You need the console permission `plugins.read` to see the page; changes need `plugins.write` and, for some actions, a fresh password confirmation.
- A plugin runs only after it is registered, approved, granted at least one capability it needs, and enabled. **Approve** and **Enable** act at once. **Disable** asks first.

<a id="register"></a>
## Register a plugin

Choose **Register a plugin**. The form opens at `#/plugins/+register`.

![The Register a plugin form with the Component choice, the host list and the Register button.](/screenshots/admin-ops/opsc-plugins-register.webp){.mw-crop}

- **Component** lists the first-party components built into the server that are not registered yet, and **Third-party component**. A first-party component needs only the choice and, if you want to limit them, the **Hosts the plugin may contact**; with no host the component's default hosts are used.
- For a **Third-party component** the form also asks for the **Plugin id**, **Name**, **Version** and **Capabilities the plugin declares**. The server reads the component from the file named after the id in the third-party plugin directory, and that file's digest must be approved in the allowlist first. With no host the plugin has no network access. The capability `account-backend` is not offered for a third-party component.

Choose **Register**. The plugin is then listed as not approved.

<a id="grants"></a>
## Granted capabilities

On the plugin's page, **Granted capabilities** lists the capabilities the plugin declares. Tick what it may use and choose **Save grants**. **Saving replaces the earlier grant**: a capability you leave unticked is revoked.

<a id="network"></a>
## Network and limits

**Network and limits** shows **Hosts it may contact**, **Memory**, **Time per call** and, when set, **Fuel per call**. The host list is set when the plugin is registered. To change it, uninstall the plugin and register it again.

Plugin requests use the **Plugins** traffic class of the outbound connections; see [Outbound connections](/security/egress). A hostname grant does not allow private addresses. [Plugins](/plugins/) lists the request limits.

<a id="endpoint"></a>
## Spam classifier address and test

A plugin that declares `spam-action` has a **Spam classifier** panel with the **Classifier address**, the place where the plugin reaches its daemon. The host must also be in the plugin's host list; empty uses the component's default. Choose **Save address** to store it.

![The page of the Rspamd plugin with markers on Status, Granted capabilities, Network and limits and Spam classifier.](/screenshots/admin-ops/opsc-plugins-detail.webp){.mw-crop}

1. **Status** shows the trust and whether the plugin is loaded, with the reason when it is not.
2. **Granted capabilities** and **Save grants**.
3. **Network and limits**.
4. **Spam classifier**, with **Save address** and **Test classifier**.
{.mw-callouts}

**Test classifier** asks the loaded plugin for a verdict and shows **Test verdict:** with its answer. While the plugin is not loaded the button is unavailable and the page says **The plugin is not loaded, so it cannot be tested.** [Spam classifier plugins](/plugins/spam-trainers) describes the classifier itself.

<a id="unsigned"></a>
## Running without a signature

A third-party plugin without a signature shows the panel **Running without a signature** with the switch **Allow this unsigned plugin to run**. The server checks the file against its approved digest either way; the switch only says that a missing signature is accepted. Turning it on asks first. First-party plugins do not show this panel.

## Uninstall

On the plugin's page, **Uninstall** under **Remove** deletes the plugin's stored data for every account, its allowlist pins, its registration and its grants. The component file stays on disk. The confirmation asks you to type the plugin id.

## Third-party plugin allowlist

The allowlist lists **Components on disk** in the third-party plugin directory with the SHA-256 digest the server computed over each file, and **Approved and revoked pins**. Approving a component pins those exact bytes and asks you to type the id. Revoking a pin removes the approval and disables the plugin; if it is running it is stopped at once. Both ask first. With no third-party files the page says **No third-party components are present. Place a component in the third-party plugin directory for it to appear here.**

## Options

This screen has no entries in the settings reference. Where the server looks for components is set with environment variables, listed in [Plugins](/plugins/#where-components-come-from). The size of the key-value store each plugin gets is set in [Settings](/admin/settings) by `plugins.kv.maxKeyBytes`, `plugins.kv.maxValueBytes`, `plugins.kv.maxTotalBytes` and `plugins.kv.maxKeys`.

## Troubleshooting

- **Not loaded: an administrator has not approved this plugin.** Choose **Approve**.
- **Not loaded: no capability it needs has been granted.** Tick a capability under **Granted capabilities** and save.
- **Not loaded: this server version does not call the hooks this plugin provides.** The plugin is registered but the server has no code that calls what it offers. Enabling it does nothing.
- **Not loaded: this server runs in proxy mode and does not process mail itself.** Plugins that serve mail need the native mail engine.
- **Restart required to load this plugin.** A plugin that was not running starts only after the server restarts.
- **This plugin is already registered. Uninstall it before registering it again.**
- **No component file with an approved digest was found for this id. Approve the file's digest in the allowlist first.** Approve the third-party file's digest, then register.

## Related

- [Plugins](/plugins/)
- [UI plugins in the admin console](/admin/ui-plugins)
- [Mail provider bridges](/bridges/)
- [Plugin security](/security/plugins)
