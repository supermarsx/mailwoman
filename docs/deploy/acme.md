# TLS: ACME (Let's Encrypt) and external certificates

V2 lets `mw-server` terminate TLS directly, either by acquiring a certificate
over **ACME** or by loading an **external certificate** that hot-reloads on
`SIGHUP`. This is an alternative to terminating TLS at a reverse proxy — use one
or the other, not both, for a given listener.

> If you already terminate TLS at nginx/Traefik/Caddy (see
> [`README.md`](./README.md) and [`nginx.conf`](./nginx.conf)), you do **not**
> need this — keep `mw-server` on plaintext `127.0.0.1:8080` behind the proxy.

## ACME (automatic certificates)

```sh
mailwoman serve \
  --acme mail.example.org \
  --acme-contact admin@example.org \
  --acme-cache /var/lib/mailwoman/acme
```

- Uses the **tls-alpn-01** challenge on the TLS port, so **:443 must be reachable
  from the internet** and public DNS for the domain must point at this host. No
  separate http-01 listener is required.
- Certificates are cached under `--acme-cache` and renewed automatically before
  expiry.
- Start against the Let's Encrypt **staging** environment while validating
  (higher rate limits, untrusted certs) with `--acme-staging`, then drop the flag
  for production certificates.

Because ACME needs public DNS + a real CA, it **cannot run in CI** — CI covers
the TLS wiring and hot-reload with a self-signed pair. Treat a first live
issuance as a manual/nightly step.

## External certificate + hot-reload

Bring your own PEM cert/key (e.g. from an internal CA or an external ACME
client):

```sh
mailwoman serve \
  --tls-cert /etc/mailwoman/tls/fullchain.pem \
  --tls-key  /etc/mailwoman/tls/privkey.pem
```

To roll a renewed certificate **without downtime**, replace the PEM files and
signal the process:

```sh
kill -HUP "$(pidof mailwoman)"     # Linux/Unix
```

The resolver swaps the live `CertifiedKey` atomically; a malformed replacement
is rejected and the previous certificate stays in service (the reload fails
loudly in the log but the listener keeps serving).

> **Windows:** signal-driven reload is a no-op — restart the process to pick up a
> new certificate.

## Notes

- Set `MW_COOKIE_SECURE=true` whenever the browser reaches Mailwoman over HTTPS
  (direct TLS or via a proxy), so the session cookie is HTTPS-only.
- ACME and `--tls-cert/--tls-key` are mutually exclusive on one listener; pick
  the model that matches your environment.
