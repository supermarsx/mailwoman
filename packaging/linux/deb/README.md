# deb tuning

Config: `apps/desktop/src-tauri/tauri.conf.json` → `bundle.linux.deb`.

```jsonc
"deb": {
  "section": "mail",       // Debian archive section
  "priority": "optional",
  "depends": []            // system-resolved WebKitGTK; pin explicitly for minimal bases
}
```

For a **minimal-base** target, pin the runtime deps rather than relying on the build
host having them installed:

```jsonc
"depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0", "libayatana-appindicator3-1"]
```

Build: `pnpm -C apps/desktop exec tauri build` → `…/bundle/deb/*.deb`.
Install: `sudo apt install ./Mailwoman_26.8.0_amd64.deb`.

No HUMAN-gated input to build the `.deb`. Signing it for a hosted **apt repo** (repo
GPG key) is an ops step — see `../README.md`.
