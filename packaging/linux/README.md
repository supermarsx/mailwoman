# Linux packaging: deb / rpm / AppImage

These formats are produced by the Tauri bundler from
`apps/desktop/src-tauri/tauri.conf.json` (`bundle.targets: "all"`,
`bundle.linux.{deb,rpm,appimage}`). This directory holds the **tuning + per-format
notes**; the machine-readable config lives in `tauri.conf.json`.

## Build (autonomous, no account)

```sh
# Full bundles (deb + rpm + appimage) — needs the Linux webview toolchain:
pnpm -C apps/desktop exec tauri build
# artifacts under apps/desktop/src-tauri/target/release/bundle/{deb,rpm,appimage}/
```

`.github/workflows/packaging.yml` builds the thin shell + asserts the §16 size
budgets (thin < 10 MB, self-contained < 40 MB). Full-bundle deb/rpm/appimage
generation is exercised by the existing `desktop-shell` CI job.

## Per-format tuning (in `tauri.conf.json` → `bundle.linux`)

- **deb** — `section: mail`, `priority: optional`. `depends` is empty: the WebKitGTK
  runtime libs are resolved from the base system; add explicit `libwebkit2gtk-4.1-0`
  etc. here if targeting a minimal base. See [`deb/`](deb/).
- **rpm** — `release: "1"`, `epoch: 0`. Same runtime-dep note. See [`rpm/`](rpm/).
- **appimage** — `bundleMediaFramework: false` (no gstreamer bundle; the thin shell
  plays no media). Enabling it inflates the AppImage well past the §16 budget. See
  [`appimage/`](appimage/).

## HUMAN-gated (optional, for a hosted repo)

- Signing deb/rpm for an **apt/dnf repository** (GPG repo key) and hosting that repo
  is an **ops** step — not required for the loose `.deb`/`.rpm`/`.AppImage` artifacts,
  which install directly. The updater feed (`packaging/updater/`) covers in-app
  updates for the AppImage.
