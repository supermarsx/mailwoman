# LDAP / GAL directory (V7)

V7 (release 26.8.0) adds a read-only LDAP directory backend (`mw-directory`) for the
Global Address List (GAL): recipient auto-complete against your directory, expanding
a distribution group before send, S/MIME certificate lookup, and photo lookup. It
also provides an LDAP-**bind** login backend (authenticate a user against the
directory). Directory access is **read-only** in 1.0 — Mailwoman never writes to your
LDAP tree.

`mw-directory` uses `ldap3` with **rustls** only; no `native-tls`/OpenSSL enters the
build. It supports plaintext, StartTLS, and implicit LDAPS.

## What it does

| Capability | LDAP operation |
|---|---|
| GAL search (recipient fields) | paged subtree search over the configured base DN |
| Group expand-before-send | read `member` of a `groupOfNames`/`groupOfUniqueNames`, resolve each entry |
| S/MIME cert lookup | read `userCertificate;binary` for an address |
| Photo lookup | read `jpegPhoto` (or your mapped attribute) |
| LDAP-bind login | bind as the resolved user DN with the supplied password |

Results are cached through `mw-cache` (the `GalDirectory` cache class) with a
configurable refresh, so the GAL is usable offline between refreshes.

## Configuration

A directory is an **ordered list** of endpoints. Lower `priority` is queried first;
results from multiple directories are merged in priority order (first hit wins for a
given address). Each endpoint carries its own attribute mapping.

`DirectoryConfig` / `LdapEndpoint` fields:

| Field | Meaning |
|---|---|
| `url` | `ldap://host:389` or `ldaps://host:636` |
| `base_dn` | search base, e.g. `dc=example,dc=com` |
| `bind_dn` | optional service-account DN for the search bind (anonymous if omitted) |
| `tls` | `None`, `StartTls`, or `Ldaps` |
| `priority` | lower is queried first; used to merge multiple directories |
| `attr_map` | attribute-name overrides (below) |

### Attribute mapping

Directories vary in which attributes hold which data. `attr_map` overrides the
defaults per endpoint (any field left unset uses the default):

| Mapping field | Default attribute | Holds |
|---|---|---|
| `display_name` | `displayName` (falls back to `cn`) | shown name in the GAL |
| `mail` | `mail` | primary email address |
| `member` | `member` | group membership (for expand) |
| `user_cert` | `userCertificate;binary` | S/MIME certificate (DER) |
| `photo` | `jpegPhoto` | contact photo |

Active Directory example: `mail` is usually `mail`, groups use `member`, and the
S/MIME cert is `userCertificate;binary`. Some deployments store the photo in
`thumbnailPhoto` rather than `jpegPhoto` — set `photo = "thumbnailPhoto"` in that
case.

## Transport security

- **StartTLS** (`tls = "StartTls"`): connect on the plaintext port, then upgrade.
- **LDAPS** (`tls = "Ldaps"`): TLS from the first byte, typically on 636.
- **None**: plaintext — use only on a trusted network segment.

TLS is validated with the system/rustls root store. Point Mailwoman at an internal CA
bundle if your directory uses a private CA.

## Multiple directories & priority

List several endpoints to merge, for example, a corporate directory ahead of a
partner directory:

```
priority 10  ldaps://ad.corp.example    base dc=corp,dc=example
priority 20  ldap+starttls://partner…   base dc=partner,dc=example
```

GAL search queries both and merges by priority; the corporate directory's entry for a
given address wins over the partner directory's.

## S/MIME certificate lookup

A cert found in the directory feeds the existing S/MIME path (`mw-crypto`, §8.2):
Mailwoman can encrypt to a recipient whose certificate is published in the GAL without
the user importing it manually. No new crypto is introduced — the directory only
supplies the DER bytes.

## CI conformance

The `directory-vs-openldap` CI job runs `mw-directory`'s live tests against a real
seeded OpenLDAP (`docker-compose.ci.yml`, service `openldap`; seed LDIF at
`scripts/openldap/ldifs/`). The live legs are `#[ignore]`d and self-skip unless
`MW_TEST_LDAP_URL` is set, so `cargo test -p mw-directory` stays deterministic
locally. OpenLDAP is a **network-only CI service** (mere aggregation) and is out of
`cargo deny` dependency scope, the same posture as the Postgres/Valkey test services.

## Scope boundary (honest)

- **Read-only.** No LDAP write path ships in 1.0.
- **Login is LDAP-bind only.** LDAP-bind authenticates an existing user against the
  directory. Enterprise **OIDC/SAML single sign-on is not implemented** — it is a
  tracked 1.0 gap (see `docs/ROADMAP-1.0.md`), not part of V7's committed scope.
