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
