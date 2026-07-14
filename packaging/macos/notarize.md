# macOS: code-signing + notarization flow (SPEC §16, "macOS universal, notarized")

The Tauri config (`apps/desktop/src-tauri/tauri.conf.json` → `bundle.macOS`) is wired
for a hardened-runtime, notarized universal build:

- `hardenedRuntime: true`
- `entitlements: ../../../packaging/macos/entitlements.plist`
- `minimumSystemVersion: 10.15`
- `signingIdentity` + `providerShortName` — **HUMAN placeholders**.

Everything below needs a paid Apple Developer account and is **HUMAN-gated** — CI
builds an **unsigned** `.app`/`.dmg` and checks its size only; it never signs or
notarizes (no Apple secrets on the runner).

## HUMAN-gated inputs

```
# HUMAN: provide Apple credentials
#   - Apple Developer Program membership (Team ID).
#   - "Developer ID Application" certificate in the CI keychain.
#   - Notarization credentials: an Apple ID + app-specific password (or an
#     App Store Connect API key: issuer id + key id + .p8).
```

1. **Fill the config:** set `bundle.macOS.signingIdentity` to
   `Developer ID Application: <Name> (<TEAMID>)` and `providerShortName` to the Team
   ID. Replace `TEAMID` in `entitlements.plist` `keychain-access-groups`.

2. **Build + sign** (universal):
   ```sh
   export APPLE_SIGNING_IDENTITY="Developer ID Application: <Name> (<TEAMID>)"
   pnpm -C apps/desktop exec tauri build --target universal-apple-darwin
   ```
   Tauri signs the `.app` with the hardened runtime + these entitlements and produces
   a `.dmg`.

3. **Notarize + staple** (Tauri does this automatically when notarization env is set):
   ```sh
   # Apple ID + app-specific password:
   export APPLE_ID="you@example.com"
   export APPLE_PASSWORD="app-specific-password"
   export APPLE_TEAM_ID="TEAMID"
   # …or an App Store Connect API key:
   export APPLE_API_ISSUER="…"; export APPLE_API_KEY="…"; export APPLE_API_KEY_PATH="AuthKey_XXXX.p8"
   ```
   Tauri submits to `notarytool` and staples the ticket to the `.dmg`. Verify:
   ```sh
   spctl -a -vvv -t install "Mailwoman.app"     # -> accepted, source=Notarized Developer ID
   xcrun stapler validate "Mailwoman_26.8.0_universal.dmg"
   ```

## App Store (separate track)

The Mac App Store needs a **different** signing identity (`Apple Distribution` /
`3rd Party Mac Developer`), a sandbox entitlement set, and an App Store Connect
submission — a distinct HUMAN-gated flow from the Developer-ID/notarized DMG above.
