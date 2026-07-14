# Auto-update: signing + staging recipe (Tauri updater)

The desktop shell wires `tauri-plugin-updater`. The config lives in
`apps/desktop/src-tauri/tauri.conf.json` under `plugins.updater` and
`bundle.createUpdaterArtifacts: true`. Auto-update is **signed + staged** per SPEC
§16 ("Auto-update signed + staged; self-hosters can pin/disable").

Mobile (`apps/mobile`) intentionally has **no self-updater**: Android/iOS updates are
delivered by the stores (Play / App Store / F-Droid). Shipping a self-updating mobile
binary would violate store policy. See `docs/deploy/packaging.md`.

## What is already wired (autonomous)

- `plugins.updater.active: true`
- `plugins.updater.endpoints`: a **stable** and a **staging** channel, templated with
  Tauri's `{{target}}/{{arch}}/{{current_version}}` variables.
- `plugins.updater.windows.installMode: passive`
- `bundle.createUpdaterArtifacts: true` (emits the `.sig` + updater bundle at
  `tauri build` time).

## HUMAN-gated inputs (required before real updates)

```
# HUMAN: provide signing key
#   1. Generate a minisign keypair (Tauri updater uses minisign):
#        pnpm -C apps/desktop exec tauri signer generate -w ~/.mailwoman/updater.key
#      -> prints the PUBLIC key and writes the PASSWORD-protected PRIVATE key.
#   2. Paste the PUBLIC key into apps/desktop/src-tauri/tauri.conf.json
#        plugins.updater.pubkey   (currently the HUMAN_PROVIDE_... placeholder).
#   3. Keep the PRIVATE key + its password OUT of the repo. In CI/release, export:
#        TAURI_SIGNING_PRIVATE_KEY           (the key file contents or base64)
#        TAURI_SIGNING_PRIVATE_KEY_PASSWORD  (the password)
#      as protected secrets. `tauri build` then signs the updater artifacts.
```

- `# HUMAN:` host the update feed. Point the two `endpoints` at a real host that
  serves the Tauri v2 updater JSON per platform/arch/version (`static.json` or a small
  service). The `staging/` endpoint is for a canary channel; promote a build by copying
  its artifacts from `staging/` to `stable/`.
- `# HUMAN:` (optional) Windows Authenticode + macOS Developer ID certs so the
  downloaded update binary itself is OS-trusted — see `../macos/notarize.md` and the
  `bundle.windows.certificateThumbprint` placeholder in `tauri.conf.json`.

## Self-hoster opt-out (SPEC §16: "can pin/disable")

Distro/self-hosted builds may set `plugins.updater.active: false` (or ship without a
`pubkey`) to disable in-app updates entirely and rely on the OS package manager.
Document this switch in the release notes for downstream packagers.

## Staging → stable promotion (feed layout)

```
releases.mailwoman.example/updater/
  staging/{target}/{arch}/{current_version}   <- canary; promote after soak
  stable/{target}/{arch}/{current_version}    <- general availability
```

Nothing here is signed or published by CI — the private key is a human secret.
