# Mailwoman — plugin strings (source locale: en).
#
# Two surfaces share this catalog, both lazily loaded via `loadCatalog('plugins')`:
# the SolidJS UI-plugin tier (src/plugins-ui/Tier.tsx) — the host-rendered chrome
# around sandboxed plugin frames — and the admin screens for engine plugins and UI
# plugins (src/screens/Admin/Plugins, src/screens/Admin/UiPlugins). Message ids are
# kebab-case and MODULE-PREFIXED (plugins-*).

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Unsigned UI plugin warning
plugins-unsigned-title = Unsigned plugins are running
plugins-unsigned-text = These plugins were admitted without a verified signature. They run sandboxed, but their code has not been checked against a trusted key.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Engine plugins run in a WebAssembly sandbox. A plugin runs only after it is registered, approved, granted at least one capability and enabled. It can use only the capabilities granted here.
plugins-admin-unsigned-banner = A third-party plugin without a signature is loaded. It runs because an administrator allowed it; the server checks its file against the approved digest only.
plugins-admin-load-error = Could not load the plugins.
plugins-admin-register-heading = Register a plugin
plugins-admin-register-component = Component
plugins-admin-register-third-party = Third-party component
plugins-admin-register-id = Plugin id
plugins-admin-register-id-hint = The server reads the component from the file named after this id in the third-party plugin directory. Approve that file's digest in the allowlist below first.
plugins-admin-register-name = Name
plugins-admin-register-version = Version
plugins-admin-register-capabilities = Capabilities the plugin declares
plugins-admin-register-hosts = Hosts the plugin may contact
plugins-admin-register-hosts-hint-first-party = Comma-separated host names. Leave empty to use the component's default hosts.
plugins-admin-register-hosts-hint-third-party = Comma-separated host names. Leave empty for no network access.
plugins-admin-register-submit = Register
plugins-admin-trust-first-party = First-party, digest built in
plugins-admin-trust-third-party = Third-party, digest approved
plugins-admin-status-loaded = Loaded
plugins-admin-status-not-loaded = Not loaded
plugins-admin-status-restart = Restart required
plugins-admin-restart-to-load = Restart required to load this plugin.
plugins-admin-restart-to-apply = Restart required to apply these changes. Some accounts of this bridge are not served until then.
plugins-admin-reason-not-approved = Not loaded: an administrator has not approved this plugin.
plugins-admin-reason-disabled = Not loaded: the plugin is not enabled.
plugins-admin-reason-unsigned-not-allowed = Not loaded: the plugin has no signature and has not been allowed to run without one.
plugins-admin-reason-no-grant = Not loaded: no capability it needs has been granted.
plugins-admin-reason-component-unavailable = Not loaded: no component file passed the digest check.
plugins-admin-reason-load-failed = Not loaded: the server could not load the component. See the server log.
plugins-admin-reason-proxy-mode = Not loaded: this server runs in proxy mode and does not process mail itself.
plugins-admin-reason-no-account-binding = Not loaded: no account is bound to this bridge.
plugins-admin-reason-no-host-caller = Not loaded: this server version does not call the hooks this plugin provides.
plugins-admin-reason-another-classifier-active = Not loaded: another spam classifier is loaded. Only one runs at a time.
plugins-admin-running-with = Running with: { $capabilities }
plugins-admin-hosts = Hosts: { $hosts }
plugins-admin-hosts-none = Hosts: none
plugins-admin-grants-heading = Granted capabilities
plugins-admin-grants-hint = The plugin can use only the capabilities ticked here. Saving replaces the earlier grant.
plugins-admin-grant-for = Grant { $capability } to { $name }
plugins-admin-grants-save = Save grants
plugins-admin-endpoint = Classifier address
plugins-admin-endpoint-for = Classifier address for { $name }
plugins-admin-endpoint-hint = Where the plugin reaches its daemon. The host must also be in the plugin's host list. Leave empty to use the component's default.
plugins-admin-endpoint-save = Save address
plugins-admin-test = Test classifier
plugins-admin-test-verdict = Test verdict: { $verdict }
plugins-admin-uninstall = Uninstall
plugins-admin-uninstall-confirm = Confirm uninstall
plugins-admin-uninstall-note = Uninstalling removes the registration, its grants and its stored settings.
plugins-admin-uninstall-detail = Uninstalling deletes the plugin's stored data for every account, its allowlist pins, and its registration and grants. The component file on disk is not deleted.
plugins-admin-revoke-detail = Revoking removes approval for this digest and disables the plugin. If the plugin is running, it is stopped at once.
plugins-admin-error-already-registered = This plugin is already registered. Uninstall it before registering it again.
plugins-admin-error-component-unavailable = No component file for this plugin matches the digest built into the server.
plugins-admin-error-digest-not-approved = No component file with an approved digest was found for this id. Approve the file's digest in the allowlist first.
plugins-admin-error-not-approved = Approve the plugin before enabling it.
plugins-admin-error-unsigned-not-allowed = This plugin has no signature. Allow it to run unsigned before enabling it.
plugins-admin-error-not-loaded = The plugin is not loaded, so it cannot be tested.
plugins-admin-error-classifier-error = The classifier call failed: { $detail }
plugins-admin-error-other = The server refused the request: { $detail }

# Rebuilt list + detail (f33, t32-cfg §4.12). `name` and `id` are isolated values.
plugins-admin-col-plugin = Plugin
plugins-admin-col-trust = Trust
plugins-admin-col-status = Running
plugins-admin-col-setup = Approval
plugins-admin-search = Search plugins
plugins-admin-no-match = No plugin matches the search or filters.
plugins-admin-filter-third-party = Third-party only
plugins-admin-filter-not-running = Not running
plugins-admin-not-approved = Not approved
plugins-admin-disabled = Disabled
plugins-admin-approved-done = { $name } is approved.
plugins-admin-enabled-done = { $name } is enabled.
plugins-admin-disabled-done = { $name } is disabled.
plugins-admin-registered-done = { $id } is registered. Approve it, grant what it needs and enable it to run it.
plugins-admin-uninstalled-done = { $id } was uninstalled.
plugins-admin-not-found = There is no plugin with the id { $id }.
plugins-admin-disable-title = Disable { $name }?
plugins-admin-disable-body = The plugin no longer runs; its registration, grants and stored settings are kept and it can be enabled again.
plugins-admin-panel-status = Status
plugins-admin-panel-network = Network and limits
plugins-admin-hosts-label = Hosts it may contact
plugins-admin-hosts-empty = None
plugins-admin-limit-memory = Memory
plugins-admin-limit-memory-value = { $mb } MiB
plugins-admin-limit-deadline = Time per call
plugins-admin-limit-deadline-value = { $ms } ms
plugins-admin-limit-fuel = Fuel per call
plugins-admin-network-note = The host list is set when the plugin is registered; uninstall and register it again to change it.
plugins-admin-panel-classifier = Spam classifier
plugins-admin-panel-unsigned = Running without a signature
plugins-admin-unsigned-help = The server checks the file against its approved digest either way; this switch only says that a missing signature is accepted.
plugins-admin-unsigned-title = Let { $name } run without a signature?
plugins-admin-unsigned-body = The server then loads this third-party plugin although it carries no signature, trusting the approved digest of its file.
plugins-admin-unsigned-confirm = Allow unsigned
plugins-admin-unsigned-allowed = { $name } may run without a signature.
plugins-admin-unsigned-withdrawn = { $name } needs a signature again.
plugins-admin-panel-remove = Remove
plugins-admin-uninstall-title = Uninstall { $name }?
plugins-admin-grants-saved = Grants of { $name } saved.
plugins-admin-endpoint-saved = Classifier address of { $name } saved.
plugins-admin-register-component-help = A first-party component built into this server, or a third-party component placed in the third-party plugin directory.
plugins-admin-register-id-pattern = Use lower-case letters, digits, hyphens and underscores, starting with a letter or digit.
plugins-admin-register-name-help = The name shown in this console.
plugins-admin-register-version-help = The version the component declares.
plugins-admin-register-capabilities-help = What the component may ask for; nothing is granted until you grant it on the plugin's page.
plugins-admin-register-hosts-help = Host names (optionally with a port) the plugin may contact; it cannot reach any other host.
plugins-admin-register-hosts-none-first-party = No host: the component's default hosts are used.
plugins-admin-register-hosts-none-third-party = No host: the plugin has no network access.
plugins-admin-register-host-invalid = { $host } is not a host name.

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI plugins
plugins-admin-ui-title = UI plugins
plugins-admin-ui-intro = UI plugins add controls to the web client. Each runs in a sandboxed frame and can use only the capabilities granted here. A plugin is offered to signed-in users once it is approved and enabled.
plugins-admin-ui-empty = No UI plugins are registered.
plugins-admin-ui-load-error = Could not load the UI plugins.
plugins-admin-ui-register-heading = Register a UI plugin
plugins-admin-ui-manifest = Manifest (JSON)
plugins-admin-ui-bundle = Bundle file
plugins-admin-ui-bundle-hint = Required for a signed plugin: the signature is checked against this file.
plugins-admin-ui-allow-unsigned = Allow this plugin to register without a signature
plugins-admin-ui-register = Register
plugins-admin-ui-manifest-invalid = The manifest is not valid JSON.
plugins-admin-ui-refused = The server refused the request: { $detail }
plugins-admin-ui-approve = Approve and enable
plugins-admin-ui-extension-points = Extension points: { $points }
plugins-admin-ui-grants-heading = Grant a capability
plugins-admin-ui-grants-note = The server does not report which capabilities are already granted. Granting a capability again replaces its earlier grant.
plugins-admin-ui-grant = Grant { $capability }
plugins-admin-ui-grant-for = Grant { $capability } to { $name }
plugins-admin-ui-grant-hosts = Hosts for { $capability } on { $name }
plugins-admin-ui-grant-hosts-hint = Comma-separated host names the plugin may fetch from.
plugins-admin-ui-granted = Granted { $capability }.
plugins-admin-ui-delete = Delete
plugins-admin-ui-delete-confirm = Confirm delete

# Rebuilt list + detail + registration (f33, t32-cfg §4.12).
plugins-admin-ui-search = Search UI plugins
plugins-admin-ui-col-signature = Signature
plugins-admin-ui-approved-done = { $name } is approved and enabled.
plugins-admin-ui-registered = { $id } is registered. Approve it to offer it to people.
plugins-admin-ui-registered-unsigned = { $id } is registered without a signature. Approve it to offer it to people.
plugins-admin-ui-deleted-done = { $name } was deleted.
plugins-admin-ui-disable-title = Disable { $name }?
plugins-admin-ui-disable-body = The plugin is no longer offered in the web client; its registration and grants are kept.
plugins-admin-ui-delete-title = Delete { $name }?
plugins-admin-ui-delete-body = The registration and its grants are removed and the web client no longer offers the plugin.
plugins-admin-ui-no-capabilities = The manifest declares no capability.
plugins-admin-ui-hosts-none = No host: the grant allows no fetch.
plugins-admin-ui-manifest-help = The plugin's ui-plugin.json; what it declares is shown before anything is sent.
plugins-admin-ui-manifest-needed = Read a manifest first.
plugins-admin-ui-manifest-file = Manifest file
plugins-admin-ui-manifest-address = Or the manifest's address on this server
plugins-admin-ui-manifest-address-help = The console can read only from its own server; for a file elsewhere, download it and choose it above.
plugins-admin-ui-manifest-read = Read
plugins-admin-ui-manifest-paste = Or paste the manifest
plugins-admin-ui-address-elsewhere = That address is on another server, which the console may not read from. Download the manifest and choose the file instead.
plugins-admin-ui-address-failed = The manifest could not be read from that address (HTTP { $status }).
plugins-admin-ui-manifest-not-object = The manifest must be a JSON object.
plugins-admin-ui-manifest-missing = The manifest has no { $key }.
plugins-admin-ui-manifest-lists = The capabilities and extension points must be lists of text.
plugins-admin-ui-manifest-capability = The manifest declares { $capability }, which this server does not know.
plugins-admin-ui-preview = What the manifest declares
plugins-admin-ui-preview-id = Id
plugins-admin-ui-preview-signature = Signature
plugins-admin-ui-preview-capabilities = Capabilities
plugins-admin-ui-preview-points = Extension points
plugins-admin-ui-bundle-needed = A signed manifest needs its bundle file: the signature is checked against it.
plugins-admin-ui-allow-unsigned-help = The manifest carries no signature; without this the server refuses it.
