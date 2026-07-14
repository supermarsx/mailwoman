# TypeScript UI-plugin tier (SPEC §22.2)

> **Status:** scaffold (t10-e0). The web sandbox host is filled by t10-e10, the
> server registry + admin approval by t10-e11, and the tier is mounted by t10-e13.
> This page is a skeleton those executors complete.

The UI-plugin tier lets a signed, admin-approved TypeScript bundle add UI into
enumerated extension-point slots — **without** the ability to reach the host DOM,
cookies, storage, or network beyond an explicit allowlist. It is a **separate
sandbox** from the WASM engine-hook plugins (`plugins/*`); it does **not** open
third-party on-disk WASM loading (that stays deny-by-default).

## Security model (frozen — `apps/web/src/plugins-ui/`)

- **Cross-origin sandboxed iframe.** A plugin renders only inside
  `<iframe srcdoc sandbox="allow-scripts">` — **no `allow-same-origin`**, so the
  iframe has an opaque origin (no host cookies / `localStorage` / DOM / `window.parent`).
- **Locked CSP.** The per-plugin CSP is set host-side; the iframe's `connect-src` is
  `'none'` — all network is host-proxied under the `net:host-allowlist` grant only.
- **RPC broker.** Guest→host calls use a MessageChannel envelope
  `{v,id,cap,method,args}` → `{v,id,ok|err}`. The broker rejects any request whose
  capability is not granted or whose method is not in that capability's allowlist
  (`CAP_METHOD_ALLOWLIST`).
- **Deny-by-default + admin approval.** A plugin is disabled with no grants until an
  admin approves it. Unsigned bundles require an explicit `allow_unsigned` + a banner.

## Manifest (`ui-plugin.json`, §2.3)

See `apps/web/src/plugins-ui/types.ts` (`UiPluginManifest`): `id`, `name`, `version`,
`signature` (detached, over the bundle; `null` = unsigned), `extensionPoints`
(enumerated allowlist), `capabilities` (`ui:compose-action` / `ui:message-toolbar` /
`ui:settings-panel` / `net:host-allowlist` / `store:kv-scoped`), `csp`.

## Persistence (0010)

`ui_plugins` + `ui_plugin_grants` (`mw_store::UiPluginRow` / `UiPluginGrantRow`).
Signatures are verified with `ed25519-dalek` (e11). Routes: `crates/mw-server/src/ui_plugins.rs`.

<!-- e10/e11: fill the guest SDK shim, the extension-point registry, the admin
approval flow, the signature-verify path, and the escape-attempt test matrix. -->
