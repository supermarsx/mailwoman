# Admin console: UI plugins

The **UI plugins** page (`#/ui-plugins`) lists the sandboxed web-client plugins with their signature and
whether they are approved and enabled. A plugin opens at `#/ui-plugins/<id>`. **Approve and enable** and
**Enable** act at once; **Disable** and **Delete** ask first.

<a id="manifest"></a>
## Manifest

**Register a UI plugin** opens the form at `#/ui-plugins/+register`. The plugin's `ui-plugin.json` is read
from a file, from an address on this server, or from pasted text, and checked the way the server checks
it: `id`, `name` and `version` are text, `capabilities` and `extensionPoints` are lists, and every
capability is one the server knows (`ui:compose-action`, `ui:message-toolbar`, `ui:settings-panel`,
`net:host-allowlist`, `store:kv-scoped`). The page shows what the manifest declares before anything is
sent. The console's content security policy lets it read only from its own server, so a manifest on
another server has to be downloaded and chosen as a file.

<a id="bundle"></a>
## Bundle file

Required for a signed manifest: the server checks the signature against this file.

<a id="unsigned"></a>
## Registering without a signature

Shown for a manifest without a signature. Without it the server refuses the registration.

<a id="grants"></a>
## Granting capabilities

Each capability the manifest declares has a **Grant** button; `net:host-allowlist` takes the list of
hosts the plugin may fetch from. The server does not report which capabilities are already granted, so
the page does not show a grant state; granting again replaces the earlier grant.
