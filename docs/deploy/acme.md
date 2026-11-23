# TLS: ACME and external certificates

Mailwoman can terminate TLS itself, either with a certificate it obtains and renews
from Let's Encrypt over ACME, or with a certificate and key you provide. Use this
when no reverse proxy terminates TLS in front of Mailwoman, for example when it runs
alone on a host or behind a TCP balancer. If a proxy already handles TLS, keep
Mailwoman on plain HTTP behind it and see [Running behind a reverse
proxy](reverse-proxy.md) instead.

When TLS is on, the listener on `--bind` serves HTTPS only, with HTTP/2 and
HTTP/1.1. There is no separate plain-HTTP listener and no redirect from port 80.

## Requirements

For ACME:

- a public DNS name that points at this host;
- port 443 on that name reachable from the internet and delivered to Mailwoman's
  listener, because the `tls-alpn-01` challenge is answered on the TLS port;
- a writable directory for the certificate cache.

## Get a certificate with ACME

1. Start the server with the domain name and a listener on port 443:

   ```sh
   mailwoman serve \
     --bind 0.0.0.0:443 \
     --acme mail.example.org \
     --acme-contact admin@example.org \
     --acme-cache /var/lib/mailwoman/acme
   ```

   Or set `MW_BIND`, `MW_ACME`, `MW_ACME_CONTACT` and `MW_ACME_CACHE` in the
   environment instead. Several names go in one comma-separated value or a repeated
   `--acme`.
2. While you test, add `--acme-staging` (`MW_ACME_STAGING=true`). Staging has higher
   rate limits and issues certificates browsers do not trust. Remove it and restart
   when the setup works.
3. Watch the log. Each step of issuance and renewal is logged with the prefix
   `acme:`.

The account key and certificates are kept in the cache directory and renewed
automatically while the server runs. Keep the directory across restarts and
upgrades so the cached certificate is reused instead of requested again; Let's
Encrypt limits how often the same certificate can be issued.

Every `--acme` entry must look like a public domain name. The server refuses to start
when an entry is empty, an IP address, a name without a dot, a name ending in a
numeric label, or one of `off`, `no`, `none`, `false`, `disabled` or `0`.

::: warning
There is no value that turns ACME off. `--acme off` is refused, and an empty
`MW_ACME=` counts as set. To run without ACME, do not pass `--acme` and leave
`MW_ACME` unset.
:::

## Use your own certificate

1. Put the certificate chain and the private key in PEM files. The key may be PKCS#8
   (`BEGIN PRIVATE KEY`), SEC1 (`BEGIN EC PRIVATE KEY`) or PKCS#1
   (`BEGIN RSA PRIVATE KEY`).
2. Start the server with both files:

   ```sh
   mailwoman serve \
     --bind 0.0.0.0:443 \
     --tls-cert /etc/mailwoman/tls/fullchain.pem \
     --tls-key  /etc/mailwoman/tls/privkey.pem
   ```

   Or set `MW_TLS_CERT` and `MW_TLS_KEY`. Both are needed; with only one of them the
   server starts on plain HTTP.

## Replace a certificate without a restart

On Linux and other Unix systems:

1. Overwrite the PEM files at the same paths.
2. Send `SIGHUP` to the server process:

   ```sh
   kill -HUP "$(pidof mailwoman)"
   ```

The server reloads both files and uses the new certificate for new connections. The
log shows `reloaded TLS certificate on SIGHUP`. If the new files cannot be read or do
not form a pair, the log shows `TLS reload failed (keeping current cert)` and the old
certificate stays in use.

On Windows there is no `SIGHUP`; restart the server to load a new certificate.

## Options

| Flag | Variable | What it does | Default |
|---|---|---|---|
| `--bind` | `MW_BIND` | Address and port of the listener | `0.0.0.0:8080` |
| `--acme` | `MW_ACME` | Domain names to obtain certificates for; turns on HTTPS | unset |
| `--acme-contact` | `MW_ACME_CONTACT` | Contact email registered with the ACME account | unset |
| `--acme-cache` | `MW_ACME_CACHE` | Directory for the account key and certificates | `acme-cache` (relative to the working directory) |
| `--acme-staging` | `MW_ACME_STAGING` | Use the Let's Encrypt staging directory | off |
| `--tls-cert` | `MW_TLS_CERT` | Certificate chain (PEM) | unset |
| `--tls-key` | `MW_TLS_KEY` | Private key (PEM) | unset |

When both ACME and a certificate pair are configured, ACME is used. The TLS
handshake timeout and concurrency limits are on [Connection
limits](connection-limits.md).

## Cookies and HSTS

When TLS is configured through the environment (`MW_ACME`, or both `MW_TLS_CERT` and
`MW_TLS_KEY`), the server marks the session cookie `Secure` and sends
`Strict-Transport-Security` without further settings.

::: tip
If you pass `--acme` or `--tls-cert`/`--tls-key` on the command line instead, the
server does not detect that it serves HTTPS for these two purposes. Also set
`MW_PUBLIC_URL=https://mail.example.org`, or use the environment variables.
:::

## Behind a TCP balancer

A layer-4 balancer can pass connections through to Mailwoman's TLS listener and
announce the client address with the PROXY protocol. The header is read before the
TLS handshake, so this works with ACME and with your own certificate. See [PROXY
protocol](reverse-proxy.md#proxy-protocol-layer-4-balancers).

## Troubleshooting

**The server exits at start with an error about `--acme / MW_ACME`.** An entry is
not a domain name (see the rules above), or `MW_ACME` is set but empty. Fix the value
or unset it.

**No certificate is issued; the log shows `acme:` errors.** Let's Encrypt cannot
reach port 443 on the name, or the DNS name does not point at this host. Check the
DNS record and that port 443 reaches Mailwoman's listener directly, not through a
proxy that terminates TLS.

**The browser warns that the certificate is not trusted.** The server is still using
`--acme-staging`. Remove it and restart.

**A new certificate is not served after `SIGHUP`.** Look for `TLS reload failed` in
the log: the files are unreadable, the key does not match the certificate, or the
key format is not one of the three above.

## Related

- [Running behind a reverse proxy](reverse-proxy.md)
- [Connection limits](connection-limits.md)
- [Example proxy configurations](proxy/README.md)
