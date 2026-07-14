# Packaging & store presence (SPEC §16, §18.1)

Two distinct distribution stories:

1. **Client shells** — the Tauri desktop/mobile apps (winget / notarized-macOS /
   AppImage / deb / rpm / Flatpak / F-Droid / Play / App Store). Recipes live in
   [`../../packaging/`](../../packaging/).
2. **Server** — the single Rust binary (`mailwoman serve` / `mailwoman fcgi` /
   container / systemd), deployable behind a reverse proxy or via a **hosting panel**.
   Reverse-proxy snippets are in this directory (`nginx.conf`, `caldav-carddav.md`,
   `websocket.md`, …); **hosting-panel recipes** are below.

Nothing here submits to a store or signs an artifact — every account/cert/submission
step is a `# HUMAN:` gate. CI (`.github/workflows/packaging.yml`) only proves the
artifacts **build** and meet the §16 size budgets.

---

## 1. Client shells — recipe index

| Channel | Recipe | Autonomous (CI) | HUMAN-gated |
|---|---|---|---|
| Windows / winget | [`packaging/winget/`](../../packaging/winget/) | YAML manifests build + validate | Authenticode cert; signed installer URL + SHA-256; `winget-pkgs` PR |
| macOS notarized | [`packaging/macos/`](../../packaging/macos/) | unsigned `.app`/`.dmg` builds + size gate | Apple Developer ID cert; notarization Apple ID / API key |
| Linux AppImage/deb/rpm | [`packaging/linux/`](../../packaging/linux/) | bundles build + size gate | (only for a hosted apt/dnf repo GPG key) |
| Flatpak / Flathub | [`packaging/flatpak/`](../../packaging/flatpak/) | manifest parses / `--show-manifest` | Flathub PR review + hosting; reproducible-source conversion |
| F-Droid | [`packaging/fdroid/`](../../packaging/fdroid/) | metadata parses | `fdroiddata` MR; reproducible build review |
| Google Play | (Android APK/AAB via `apps/mobile`) | APK builds (`android-apk` CI job) | Play Console account; Play **upload key**; store listing + review |
| Apple App Store | (`apps/mobile` iOS) | — (needs macOS + Xcode) | App Store Connect account; distribution cert; listing + review |
| Desktop auto-update | [`packaging/updater/`](../../packaging/updater/) | config wired; `createUpdaterArtifacts` | minisign keypair; hosted stable+staging feed |

### Version

Both `tauri.conf.json` files are synced to the current shipped workspace version
**26.8.0** (was stale at 26.6.0). The `1.0`/`26.9` release tag + any manifest bump is
a human-gated release decision (see `.orchestration/plans/t8-onedotzero.md` §3) — the
manifests here pin 26.8.0 so they build against today's tree.

### Mobile has no self-updater — by design

`apps/mobile/src-tauri/tauri.conf.json` has **no** `plugins.updater` block. Android/iOS
updates ship through the stores (Play / App Store / F-Droid); a self-updating mobile
binary would violate store policy. Only the **desktop** shell self-updates
(`packaging/updater/`).

### The complete HUMAN-gated list (accounts / certs / submissions)

Before any store presence a human must provide:

- **Windows:** an Authenticode code-signing certificate (EV recommended) →
  `bundle.windows.certificateThumbprint`; then a PR to `microsoft/winget-pkgs`.
- **macOS:** Apple Developer Program membership; a *Developer ID Application*
  certificate; notarization credentials (Apple ID + app-specific password **or** an
  App Store Connect API key). For the Mac App Store: an *Apple Distribution* cert +
  App Store Connect submission.
- **Auto-update:** a minisign keypair (public key → `plugins.updater.pubkey`; private
  key + password → CI secrets `TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]`); a host serving
  the stable + staging update feeds.
- **Google Play:** a Play Console account; the Play **upload/signing key**; a store
  listing + content rating + review.
- **Apple App Store (iOS):** an App Store Connect account; distribution signing;
  listing + review (requires a macOS build host with Xcode).
- **Flathub:** a PR to `flathub/flathub` + maintainer review (and reproducible-source
  conversion of the manifest).
- **F-Droid:** an MR to `fdroiddata` + reproducible-build review.

CI provides none of these secrets; all builds it runs are unsigned/unsubmitted.

---

## 2. Server hosting-panel recipes (§18.1)

The server is one binary with embedded assets + embedded ACME. Panels either run it
as a **long-lived service** (reverse-proxied) or, on shared PHP hosts, via
**FastCGI** (`mailwoman fcgi`). All recipes below are community-maintainable and
CI-smoke-testable; none needs a proprietary account.

Common prerequisites: a `mailwoman` binary (from a release, the container image, or
`cargo build --release --bin mailwoman`), a data dir, and either a public port behind
the panel's reverse proxy or a Unix socket. See [`hardening.md`](hardening.md) and
[`mailwoman.service`](mailwoman.service) for the base systemd unit these adapt.

### cPanel (with WHM / "Application Manager")

cPanel proxies to a persistent app via **Passenger/Application Manager** or a raw
reverse proxy. Since Mailwoman is a native binary (not Node/Python/Ruby), use the
reverse-proxy path:

1. As the cPanel user, install the binary under `~/mailwoman/` and create
   `~/mailwoman/data/`.
2. Run it as a per-user systemd **user** service (or `cpanel`/`supervisord` job)
   bound to a loopback port:
   ```ini
   # ~/.config/systemd/user/mailwoman.service  (systemctl --user enable --now mailwoman)
   [Service]
   ExecStart=%h/mailwoman/mailwoman serve --bind 127.0.0.1:8801 --data %h/mailwoman/data
   Restart=on-failure
   ```
3. In **WHM → Apache → Include Editor** (or cPanel's "Domains → proxy"), add a
   reverse proxy from the domain to `127.0.0.1:8801`, passing `X-Forwarded-*` and
   WebSocket upgrade headers (see [`nginx.conf`](nginx.conf) for the header set).
4. TLS is terminated by cPanel/AutoSSL; disable Mailwoman's embedded ACME
   (`--acme off`).

> **Shared-hosting fallback (no persistent process):** use FastCGI —
> `mailwoman fcgi` behind cPanel's FastCGI handler, the closest analog to
> SnappyMail's PHP deployability (§18.1).

### Plesk

1. Install the binary under `/opt/mailwoman/` + data dir; add a hardened systemd unit
   (adapt [`mailwoman.service`](mailwoman.service)) bound to `127.0.0.1:8801`.
2. In **Plesk → Domains → <domain> → Apache & nginx Settings**, add an
   **Additional nginx directive** reverse-proxying `/` to `http://127.0.0.1:8801`
   with the WebSocket `Upgrade`/`Connection` headers and `X-Forwarded-*`.
3. Let Plesk's Let's Encrypt extension own TLS; run Mailwoman with `--acme off`.
4. Optional subpath hosting (`/mail`) — Mailwoman supports it; set the proxy location
   and `MW_BASE_PATH=/mail`.

### CloudPanel

CloudPanel is nginx-based. Create a **Reverse Proxy** site:

1. Systemd unit as above on `127.0.0.1:8801`.
2. CloudPanel → **+ Add Site → Create a Reverse Proxy**, target
   `http://127.0.0.1:8801`; CloudPanel writes the nginx vhost. Ensure the generated
   vhost carries the WebSocket upgrade block (add via **Vhost Editor** if absent).
3. CloudPanel manages Let's Encrypt; run with `--acme off`.

### ISPConfig

1. Binary + data dir + systemd unit on `127.0.0.1:8801`.
2. In ISPConfig, create the website, then under **Options → nginx Directives** (or
   Apache `Directives`) add the reverse proxy + WebSocket headers to `127.0.0.1:8801`.
3. ISPConfig's Let's Encrypt checkbox owns TLS; `--acme off`.

### Cloudron

Cloudron packages are Docker images with a manifest. Recipe outline (community app):

- Base the app on the Mailwoman **container image** (`Dockerfile` → `runtime` stage).
- `CloudronManifest.json`: expose `httpPort` 8080, request a `localstorage` volume
  mounted at `/data` (matches the image's `MW_DB_PATH=/data/mailwoman.db`), set
  `MW_BIND=0.0.0.0:8080`, and let Cloudron terminate TLS + provide the domain.
- Health check → the image's `HEALTHCHECK` (`mailwoman healthcheck`).
- **`# HUMAN:`** publishing to the Cloudron App Store is a submission to Cloudron
  (community app review). The package itself is self-hostable without that.

### YunoHost

YunoHost apps are packaged as a git repo with `manifest.toml` + install scripts:

- `manifest.toml`: id `mailwoman`, a `main` permission, an internal port
  (`__PORT__`), a `data_dir`.
- `scripts/install`: fetch the release binary (or build), create the data dir + a
  system user, install a hardened systemd unit (adapt
  [`mailwoman.service`](mailwoman.service)), and register the nginx reverse-proxy conf
  (`conf/nginx.conf` from [`nginx.conf`](nginx.conf)) with SSOwat.
- YunoHost owns the domain + Let's Encrypt; run with `--acme off`.
- **`# HUMAN:`** listing in the YunoHost app catalog is a PR to `YunoHost-Apps` +
  a CI/level review. Self-hosting from the repo needs no catalog entry.

### runtipi

runtipi apps are a `config.json` + a `docker-compose.yml` fragment:

- Reuse the Mailwoman **container image**; compose service exposes 8080, mounts
  `${APP_DATA_DIR}/data:/data`, sets `MW_BIND=0.0.0.0:8080`.
- `config.json` declares the port, name, and a single exposed http port; runtipi's
  Traefik front handles TLS + domain.
- **`# HUMAN:`** inclusion in the runtipi app store is a PR to the runtipi
  app-store repo.

### CI smoke (autonomous)

The container-based panels (Cloudron / runtipi) reuse the repo `Dockerfile`; the
reverse-proxy panels (cPanel / Plesk / CloudPanel / ISPConfig) reuse
[`nginx.conf`](nginx.conf). `.github/workflows/packaging.yml` validates the container
image builds and (where a compose fragment is provided) that `docker compose config`
parses. Panel-store **publication** is HUMAN-gated in every case.
