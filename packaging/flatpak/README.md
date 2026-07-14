# Flatpak / Flathub recipe

Files:

- `com.mailwoman.Mailwoman.yml` — flatpak-builder manifest (thin shell, narrow sandbox).
- `com.mailwoman.Mailwoman.desktop` — desktop entry (mailto/mailwoman scheme handlers).
- `com.mailwoman.Mailwoman.metainfo.xml` — AppStream metainfo (store listing).

## Local build (autonomous, no account)

```sh
# Build the thin shell first so the manifest's binary source exists:
MW_SELF_CONTAINED=0 bash scripts/build-shells.sh
flatpak-builder --user --force-clean build-dir \
    packaging/flatpak/com.mailwoman.Mailwoman.yml
```

CI validates the manifest parses and (when the runtime is available) that
`flatpak-builder --show-manifest` accepts it — it does **not** publish.

## HUMAN-gated (before Flathub presence)

- **Flathub submission:** open a PR to `github.com/flathub/flathub` adding this
  app-id; a Flathub maintainer reviews the manifest, sandbox permissions, and the
  metainfo. Flathub then hosts and signs the build.
- **Reproducibility:** Flathub prefers building from source. Convert the binary
  `sources:` entries to a `git`/`archive` source pinned to the release tag, plus a
  cargo/pnpm vendoring step, before submission.
- **Screenshots + release URLs** in the metainfo must resolve to hosted assets.
