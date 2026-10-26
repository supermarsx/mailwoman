# Admin console: Plugins

The **Plugins** page (`#/plugins`) lists the engine plugins (WebAssembly components) with their trust,
whether they run and whether they are approved and enabled. What the page shows as running is what the
server reports (`loaded`, `restartRequired`, `notLoadedReason`): an enabled plugin the server has not
loaded is shown as not running, with the reason. A plugin opens at `#/plugins/<id>`. Below the list is
the third-party allowlist.

A plugin runs only after it is registered, approved, granted at least one capability it needs and
enabled. Approve and Enable act at once; Disable asks first.

<a id="register"></a>
## Register a plugin

**Register a plugin** opens the form at `#/plugins/+register`. A first-party component (built into the
server, trusted by the digest compiled in) is registered by its id, plus a host list when one is given.
A third-party component also needs its id (the file name in the third-party plugin directory), name,
version and the capabilities it declares; its file's digest must be approved in the allowlist first.
The first-party-only capability (`account-backend`) is never offered for a third-party component.

<a id="grants"></a>
## Granted capabilities

The plugin can use only the capabilities ticked on its page. Saving replaces the earlier grant.

<a id="network"></a>
## Network and limits

The hosts the plugin may contact are set when it is registered (uninstall and register again to change
them); it cannot reach other hosts. Memory, time per call and fuel are the sandbox limits of the
component.

Plugin HTTP calls use the **Plugins** traffic class in Outbound routing. The host
rechecks the current registry permission and the bound account's feature policy
for each call. Domain routes use the account's canonical policy domain. A failed
configured route never falls back to a direct connection, and plugin HTTP calls
never follow redirects. OAuth token exchanges use the separate **OAuth** class.

A hostname grant does not grant access to private addresses. On-premises plugin
services also need an explicit **Plugins** internal-destination allowlist entry.
Metadata and reserved address protections still apply. SOCKS with remote DNS is
refused for these calls because destination addresses must be checked locally.
Requests have a 15-second deadline and an 8 MiB request/response ceiling (OAuth
responses are limited to 64 KiB); only the supported HTTP
methods and protocol headers are accepted. Cookies and proxy credentials are
never forwarded to the upstream service.

The host no longer injects process-wide `MW_NEXTCLOUD_*` credentials into plugin
requests. The legacy credential-less Nextcloud guest no longer receives automatic
authentication; use the native gateway with each mailbox's verified Nextcloud link.
Provider credentials require
a host-bound account and its approved provider authority. This transport boundary
does not itself add provider-domain login or NTLM connection-affinity support.

<a id="endpoint"></a>
## Spam classifier address and test

For a spam classifier: where the plugin reaches its daemon (the host must be in its host list; empty uses
the component's default). **Test classifier** asks the loaded plugin for a verdict and shows its raw
answer; it is unavailable while the plugin is not loaded.

<a id="unsigned"></a>
## Running without a signature

A third-party plugin without a signature runs only when this switch is on. The server checks the file
against its approved digest either way. Turning the switch on asks first.

## Uninstall

Uninstalling deletes the plugin's stored data for every account, its allowlist pins, its registration and
grants; the component file stays on disk. The confirmation asks for the plugin id to be typed.

## Third-party allowlist

The components found in the third-party plugin directory, with the SHA-256 the server computed over their
bytes, and the stored pins (approved and revoked). Approving pins the exact bytes shown and asks for the
id to be typed; revoking removes the approval and disables the plugin; both ask first.

An EWS account endpoint is a separate administrator-approved host authority: changing or disabling that account endpoint revokes its requests. Removing the generic plugin hostname alone does not revoke the exact EWS endpoint grant. The network capability and explicit internal-destination permission remain required.
