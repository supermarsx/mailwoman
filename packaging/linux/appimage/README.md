# AppImage tuning

Config: `apps/desktop/src-tauri/tauri.conf.json` → `bundle.linux.appimage`.

```jsonc
"appimage": {
  "bundleMediaFramework": false   // no gstreamer bundle — thin shell plays no media
}
```

`bundleMediaFramework: false` keeps the AppImage within the SPEC §16 thin-shell size
budget (< 10 MB); enabling it bundles gstreamer and inflates the image well past it.

Build: `pnpm -C apps/desktop exec tauri build` → `…/bundle/appimage/*.AppImage`.
Run: `chmod +x Mailwoman_26.8.0_amd64.AppImage && ./Mailwoman_26.8.0_amd64.AppImage`.

## In-app updates

The AppImage is the one Linux format the Tauri updater can self-update (deb/rpm defer
to the OS package manager). It consumes the `packaging/updater/` feed once a signing
key + feed host are provisioned (HUMAN-gated). Self-hosters can disable it — see
`packaging/updater/README.md`.

No HUMAN-gated input to build the AppImage itself.
