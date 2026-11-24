---
title: Encryption and mail security server configuration
---

# Encryption and mail security server configuration

Mailwoman supports OpenPGP and S/MIME encryption and signing, a message security panel, outbound data-loss rules, and sender controls such as reporting a message as phishing or junk. Private-key operations happen in the user's browser and need no server configuration. The server only publishes public keys, loads data-loss rules, and forwards abuse reports. This page lists the environment variables and HTTP endpoints involved. For the model behind them see [Security model](/security/README); for the data-loss rules see [Data loss prevention](/security/dlp).

All of these are off until you set them.

## Environment

| Variable | Default | What it does |
|---|---|---|
| `MW_WKD_DIR` | Not set (publishing off) | Directory of public keys to publish over Web Key Directory. Two layouts are accepted, described below. |
| `MW_DLP_RULES` | Not set (no rules) | A path to a JSON file holding an array of data-loss rules, or the JSON array itself. See [Data loss prevention](/security/dlp). The server reads it when it evaluates a message. |
| `MW_ABUSE_ADDRESS` | Not set (reporting off) | The address abuse reports are sent to when a user reports a message as phishing or junk. |
| `MW_ABUSE_SPOOL` | Not set | A directory where each generated report is written as `<uuid>.eml` for pickup by another program. When it is not set, the report is built and logged, and not delivered. |
| `MW_WATERMARK` | `false` | Sets the value that `GET /api/security/watermark` reports as `enabled`. It does not draw anything; see below. |
| `MW_WATERMARK_OPACITY` | `0.08` | The opacity that same endpoint reports, limited to 0 to 1. |

The setting `abuse.reportAddress` (**Abuse report address**) applies when `MW_ABUSE_ADDRESS` is not set; when the variable is set, it wins. The watermark has no setting in the console: only the variables configure it.

### The watermark variables have no visible effect

The web app does not draw a watermark and does not call `GET /api/security/watermark`. Setting `MW_WATERMARK=true` changes only what that endpoint answers (the flag, the opacity, the signed-in user's name, the server time and a note that a browser cannot prevent or detect screenshots). Nothing in the web app, desktop app or mobile app reads it, so no overlay appears in any version of the interface. See [Screen capture](/security/screen-capture).

## Endpoints

| Method and path | Who can call it | Purpose |
|---|---|---|
| `GET /.well-known/openpgpkey/hu/{hash}` | Anyone | Web Key Directory, direct method: a key by its hash, for the domain in the `Host` header |
| `GET /.well-known/openpgpkey/{domain}/hu/{hash}` | Anyone | Web Key Directory, advanced method: the domain is in the path |
| `GET /.well-known/openpgpkey/policy`, `GET /.well-known/openpgpkey/{domain}/policy` | Anyone | Policy probe: an empty `200` tells a client the domain publishes keys |
| `POST /api/security/report` | Signed-in user | Builds an abuse report for a message (`{emailId, kind: "phishing" or "junk", note}`) and sends or spools it. Engine mode only. Answers `501` when `MW_ABUSE_ADDRESS` is not set, `400` for another `kind`, `404` when the message is not found, and `202` with `relayed: true` when the report was written to the spool. |
| `GET /api/security/dlp/config` | Signed-in user | The active data-loss rules, from `MW_DLP_RULES` and from the rules stored in the console, each marked with its source |
| `GET /api/security/watermark` | Signed-in user | The watermark values described above |

The web app's report-phishing and report-junk actions go through the mail interface rather than `POST /api/security/report`, and also use `MW_ABUSE_ADDRESS`. The web app does not call `GET /api/security/dlp/config` either.

Web Key Directory answers are served as `application/octet-stream`. The hash is 32 z-base-32 characters, and a domain containing `..` or `/` is refused.

### Layout of `MW_WKD_DIR`

Either layout works:

- **Files named after the address**: `alice@example.org`, with an optional `.asc`, `.pgp`, `.gpg`, `.key` or `.pub` extension, binary or armored. Armored keys are converted to binary when served.
- **The standard `gpg-wks` tree**: `<domain>/hu/<hash>`, where `<hash>` is the z-base-32 encoding of the SHA-1 of the lower-cased local part.

Only keys placed in this directory are served. When a reverse proxy terminates TLS, forward the `Host` header unchanged so the direct method finds the right domain; an `openpgpkey.` prefix on the host name is removed before the lookup.

## Troubleshooting

- `POST /api/security/report` answers **abuse reporting is not configured (set MW_ABUSE_ADDRESS)**: set `MW_ABUSE_ADDRESS` and restart.
- A report is accepted but never arrives: with `MW_ABUSE_SPOOL` unset the report is only built and logged. Set the spool directory and have a program deliver the files.
- The server logs **MW_DLP_RULES failed to load**: the path does not exist or the JSON does not parse. Fix the value. `GET /api/security/dlp/config` answers `500` with **DLP rules are misconfigured** until you do.
- A Web Key Directory lookup answers `404`: the key is not in `MW_WKD_DIR`, or the file name or hash does not match the address.

## Related

- [Security model](/security/README)
- [Data loss prevention](/security/dlp)
- [Screen capture](/security/screen-capture)
- [Settings reference](/admin/settings)
