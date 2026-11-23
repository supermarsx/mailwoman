# Packaging and distribution

This page lists the ways Mailwoman is packaged: the server as a container image, a Helm chart or a binary under a service manager, and the desktop and mobile apps for each store. For each one it says what the repository provides, what CI checks, and what you must provide yourself (accounts, certificates, store submissions). CI never signs a desktop or mobile build and never submits anything to a store.

## Server

### Container image

CI (`.github/workflows/supply-chain.yml`) builds the Dockerfile target `runtime-static` on every push to `master` and publishes it to `ghcr.io/supermarsx/mailwoman` with two tags: `master` and `sha-<short commit>`. It then generates a CycloneDX SBOM, fails on vulnerabilities of high severity or above (grype), signs the image by digest with keyless cosign and attaches the SBOM as an attestation. No version tags are published. `deploy.sh --prebuilt` and `--image` use this image; see [Deploy with deploy.sh](script.md#using-a-published-image).

To build an image yourself, pass the full commit:

```sh
docker build --target runtime-static --build-arg MW_BUILD_REVISION="$(git rev-parse HEAD)" -t mailwoman:local .
```

A build without `MW_BUILD_REVISION` (40 hex characters) fails. `deploy.sh` supplies it from the checkout; when you build with your own Compose file, set `MW_BUILD_REVISION` to the full commit first.

| Target | Base | Contents |
|---|---|---|
| `runtime-static` (default, last stage) | `scratch` | Statically linked x86_64 `mailwoman` and `mw-render`, the web client at `/usr/lib/mailwoman/web`, first-party plugins at `/usr/lib/mailwoman/plugins`, CA certificates. Runs as UID 65532. |
| `runtime` | distroless `cc-debian12:nonroot` | The same programs, web client and plugins, dynamically linked. |

Both images listen on `0.0.0.0:8080`, keep the database at `/data/mailwoman.db` and run `mailwoman healthcheck` as their health check.

### Helm chart

`packaging/helm/mailwoman` deploys one server replica to Kubernetes 1.25 or newer.

1. Choose an image your cluster can pull. The chart's default repository is `ghcr.io/supermarsx/mailwoman`, where CI publishes the `master` and `sha-<short commit>` tags. The default tag is the chart's `appVersion`, a release number that CI does not publish as a tag, so set `image.tag` (for example `master`).
2. Create the server key as a Secret:

   ```sh
   kubectl create secret generic mailwoman-key \
     --from-literal=MW_SERVER_KEY=$(openssl rand -hex 32)
   ```

3. Install:

   ```sh
   helm install mailwoman packaging/helm/mailwoman \
     --set image.repository=ghcr.io/supermarsx/mailwoman \
     --set image.tag=<tag> \
     --set serverKeySecret.name=mailwoman-key
   ```

| Value | What it does | Default |
|---|---|---|
| `image.repository`, `image.tag` | Image to run. | `ghcr.io/supermarsx/mailwoman`, chart `appVersion` |
| `serverKeySecret.name`, `.key` | Secret holding `MW_SERVER_KEY`. Without it the key is not supplied by the chart. | empty, `MW_SERVER_KEY` |
| `env` | Plain environment variables for the container. | `MW_BIND`, `MW_DB_PATH`, `MW_RENDER_BIN`, `MW_PLUGIN_DIR`, `RUST_LOG` |
| `resources` | Requests and limits per replica. | 4 CPU and 4 GiB, requests equal to limits |
| `persistence` | Volume for `/data`. | enabled, `5Gi`, `ReadWriteOnce` |
| `ingress.enabled` | Create an Ingress. | `false` |
| `ingress.controller` | Annotation preset: `nginx`, `traefik` or `none`. | `nginx` |
| `tmpSizeLimit` | Size of the in-memory `/tmp`. | `64Mi` |

The Deployment uses the `Recreate` strategy, because SQLite on a `ReadWriteOnce` volume allows one writer. Liveness and readiness probes run `mailwoman healthcheck` inside the pod. The pod runs as UID/GID 65532 with a read-only root filesystem, all capabilities dropped, no privilege escalation, the `RuntimeDefault` seccomp profile and no service-account token.

When you enable the ingress:

- Set `env.MW_COOKIE_SECURE=true` when the ingress serves HTTPS; the chart does not set it.
- Set `env.MW_FORWARDED_MODE` and `env.MW_TRUSTED_PROXIES` to the controller's pod addresses. Otherwise every request appears to come from the controller's private address, and `/healthz` is open to everyone.
- The `nginx` preset raises the body size to 50 MB, turns off request and response buffering and sets 3600-second read and send timeouts, for uploads, server-sent events and the WebSocket. For Traefik, the timeouts are entrypoint settings in Traefik's own configuration; `values.yaml` shows them.

The chart creates no admin account and has no backup job. `helm lint` and `helm template` run in CI.

### Binary under systemd

[`mailwoman.service`](https://github.com/supermarsx/mailwoman/blob/master/docs/deploy/mailwoman.service) is a hardened systemd unit that runs `mailwoman serve` on `127.0.0.1:8080` with the database in `/var/lib/mailwoman`. Supply `MW_SERVER_KEY` through a drop-in or an `EnvironmentFile` with mode `0600`. See [Hardening](hardening.md).

`serve` listens on a TCP address only: there is no Unix-socket listener, no systemd socket activation and no FastCGI mode. The server must run as a persistent process, so a shared host that cannot run one cannot run Mailwoman.

### Hosting panels

The repository ships no panel packages. The outlines below describe how to run the server under each panel; they are not tested in CI. In every case you run `mailwoman serve` as a persistent service on a loopback port and let the panel's reverse proxy and certificate manager handle the domain and TLS.

Common settings for these setups:

- `--bind 127.0.0.1:8801` (or `MW_BIND`), `--db-path <data dir>/mailwoman.db` (or `MW_DB_PATH`), and `MW_SERVER_KEY` from a private environment file.
- Do not pass `--acme` and leave `MW_ACME` unset: `--acme` takes domain names and turns on HTTPS on the bind address, which a panel proxy sending plain HTTP cannot reach. Do not pass `--tls-cert` or `--tls-key` either. See [ACME](acme.md).
- `MW_COOKIE_SECURE=true`, and `MW_TRUSTED_PROXIES` / `MW_FORWARDED_MODE` for the panel's proxy.
- The proxy forwards WebSocket upgrades and the `X-Forwarded-*` headers; [`nginx.conf`](https://github.com/supermarsx/mailwoman/blob/master/docs/deploy/nginx.conf) shows the header set. See [Reverse proxy](reverse-proxy.md).
- To serve the app under a path such as `/mail`, set `MW_BASE_PATH=/mail`. The app also stays reachable at the origin root. Read the sub-path section of [Reverse proxy](reverse-proxy.md) before using it.

| Panel | How to run it |
|---|---|
| cPanel | A per-user systemd service (`~/.config/systemd/user/mailwoman.service`, `systemctl --user enable --now mailwoman`), with a reverse proxy from the domain to the loopback port added in WHM's Apache include editor. AutoSSL handles TLS. |
| Plesk | A system service, and an **Additional nginx directives** block under the domain's **Apache & nginx Settings** proxying `/` to the loopback port. Plesk's Let's Encrypt extension handles TLS. |
| CloudPanel | A system service and a **Reverse Proxy** site pointing at the loopback port; add the WebSocket upgrade lines in the vhost editor if they are missing. |
| ISPConfig | A system service, and the reverse proxy and WebSocket headers in the website's nginx or Apache directives. |
| Cloudron | A package based on the container image: HTTP port 8080, a `localstorage` volume at `/data` (the image keeps `MW_DB_PATH=/data/mailwoman.db`) and `MW_BIND=0.0.0.0:8080`. |
| YunoHost | An app repository with `manifest.toml` and install scripts that install the binary, a data directory, a service based on `mailwoman.service`, and an nginx configuration. |
| runtipi | An app with `config.json` and a Compose fragment that uses the container image, exposes port 8080, mounts `${APP_DATA_DIR}/data:/data` and sets `MW_BIND=0.0.0.0:8080`. |

The cPanel unit, for example:

```ini
# ~/.config/systemd/user/mailwoman.service
[Service]
ExecStart=%h/mailwoman/mailwoman serve --bind 127.0.0.1:8801 --db-path %h/mailwoman/data/mailwoman.db
EnvironmentFile=%h/mailwoman/mailwoman.env
Restart=on-failure
```

Listing an app in a panel's catalogue (Cloudron App Store, YunoHost-Apps, the runtipi app store) is a submission to that project's review.

## Desktop and mobile apps

The desktop app (`apps/desktop`) and the Android app (`apps/mobile`) are Tauri shells around the same web client. How to build them for your server is described in [Desktop and mobile apps: building and provisioning](shells.md).

| Channel | Recipe | What CI checks | What you provide |
|---|---|---|---|
| Windows (winget) | [`packaging/winget/`](https://github.com/supermarsx/mailwoman/tree/master/packaging/winget) | The three manifests parse and carry the workspace version. | An Authenticode certificate (`bundle.windows.certificateThumbprint`), the signed installer's URL, SHA-256 and product code, and a pull request to `microsoft/winget-pkgs`. |
| macOS | [`packaging/macos/notarize.md`](https://github.com/supermarsx/mailwoman/blob/master/packaging/macos/notarize.md) | The shell builds (best effort; failures do not fail CI). | Apple Developer Program membership, a Developer ID Application certificate and notarization credentials. |
| Linux deb, rpm, AppImage | [`packaging/linux/`](https://github.com/supermarsx/mailwoman/tree/master/packaging/linux) | The shell builds. | A GPG key and hosting only if you run an apt or dnf repository. |
| Flatpak (Flathub) | [`packaging/flatpak/`](https://github.com/supermarsx/mailwoman/tree/master/packaging/flatpak) | `flatpak-builder --show-manifest` accepts the manifest; the desktop entry validates. | A pull request to `flathub/flathub`, and source entries that build from the tagged source instead of a prebuilt binary. |
| F-Droid | [`packaging/fdroid/`](https://github.com/supermarsx/mailwoman/tree/master/packaging/fdroid) | The metadata parses and carries the workspace version. | A merge request to `fdroiddata`; F-Droid builds and signs from source. |
| Google Play | `apps/mobile` | An unsigned APK builds (allowed to fail). | A Play Console account, the upload key, the store listing and review. |
| Apple App Store | `apps/mobile` | Not built in CI. | An App Store Connect account, distribution signing and a macOS host with Xcode. |

CI builds the desktop shell on Linux, Windows and macOS with `--no-bundle`, so it produces no installers. It checks the size of the thin shell (under 27 MB) and of the self-contained shell with the bundled server (under 204 MB). To build installers (deb, rpm, AppImage, NSIS, dmg), run `tauri build` yourself:

```sh
pnpm -C apps/desktop exec tauri build
```

On Linux this needs the WebKitGTK development packages that CI installs (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`, `patchelf`).

### Versions

`[workspace.package] version` in the root `Cargo.toml` is the single source. `scripts/stamp-version.sh` writes it into the winget manifests, the Flatpak metainfo, the F-Droid metadata, the Helm chart's `appVersion` and README, both `tauri.conf.json` files and both shell `package.json` files. CI fails when any of them differs.

### Updates

The desktop app can update itself only when its build is given an update feed (`MW_SHELL_UPDATER_URL`) and the minisign public key (`MW_SHELL_UPDATER_PUBKEY`); the private key and its password stay outside the repository in `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. See [Desktop updates](shells.md#desktop-updates) for the feed layout and what the app does with it. The committed configuration has no updater, and CI fails if one is added.

The mobile app has no updater: Android and iOS apps are updated through the store they came from.

## Related

- [Deploy with deploy.sh](script.md)
- [Desktop and mobile apps: building and provisioning](shells.md)
- [Reverse proxy](reverse-proxy.md)
- [Hardening](hardening.md)
- [Deployment overview](README.md)
