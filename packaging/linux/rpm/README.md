# rpm tuning

Config: `apps/desktop/src-tauri/tauri.conf.json` → `bundle.linux.rpm`.

```jsonc
"rpm": {
  "release": "1",
  "epoch": 0,
  "depends": []            // system-resolved WebKitGTK; pin explicitly for minimal bases
}
```

For a minimal base, pin the Fedora/openSUSE runtime deps:

```jsonc
"depends": ["webkit2gtk4.1", "gtk3", "libappindicator-gtk3"]
```

Build: `pnpm -C apps/desktop exec tauri build` → `…/bundle/rpm/*.rpm`.
Install: `sudo dnf install ./Mailwoman-26.8.0-1.x86_64.rpm`.

No HUMAN-gated input to build the `.rpm`. Signing it for a hosted **dnf repo**
(repo GPG key) is an ops step — see `../README.md`.
