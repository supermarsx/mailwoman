# Upgrade note: console settings that now apply

Applies to every deployment. No action is required, but some values saved in the admin console before the upgrade had no effect until now, and they start to apply after it. Check them before you upgrade.

## What changes

Before this release the server saved the settings below but did not read them, or read only an environment variable. They now apply. Where a setting has an environment variable, the variable still wins when it is set, and the console value applies when it is unset.

| Setting | Scope | What it now does |
|---|---|---|
| `import.maxMessages` | global, domain | Most messages one import request may carry (1 to 80,000, default 20,000). A larger import is refused with `400`. |
| `import.maxMessageBytes` | global, domain | Largest message an import accepts. A larger message is refused with `413`. The request body limit of the import routes stays 64 MiB, so a value above 64 MiB acts as 64 MiB. |
| `webhooks.maxAttempts` | global | Delivery attempts per outbound webhook event, including breach notices (1 to 10, default 3). |
| `webhooks.timeoutSecs` | global | Time allowed for each delivery attempt (1 to 60 seconds, default 10). |
| `upstream.proxyTimeoutSecs` | global | Proxy mode: total time allowed for one request to the JMAP server (30 to 900 seconds, default 300). The server re-reads it every 30 seconds. |
| `signin.ldapBindPrecheck` | global | Checks each sign-in password with an LDAP bind before the mail server. `MW_LDAP_BIND_AUTH` wins when set. |
| `maskedEmail.domain` | global | Domain of new masked addresses. Existing addresses keep their domain. `MW_MASKED_EMAIL_DOMAIN` wins when set. |
| `abuse.reportAddress` | global, domain | Address that abuse reports from webmail (`POST /api/security/report`) are sent to, by the reporter's domain. `MW_ABUSE_ADDRESS` wins when set. |
| `about.securityContact`, `about.securityPolicyUrl` | global | The server answers `GET /.well-known/security.txt` (RFC 9116) with `Contact`, `Expires` (one year ahead) and `Policy`. Without a contact it answers `404`. |

A stored number outside a setting's range is treated as the nearest end of the range, so `0` cannot turn off webhook delivery or the proxy timeout.

## What to check

- **`signin.ldapBindPrecheck`.** If it was switched on in the console, every sign-in now needs a successful bind against the configured directory. A person the directory does not know, or any sign-in while the directory is unreachable, is refused. Switch it off, or set `MW_LDAP_BIND_AUTH=0`, if that is not what you want.
- **Import limits.** A domain value lower than the default refuses larger imports for that domain.
- **`abuse.reportAddress`.** In engine mode, the abuse report that the sender-control actions `report-phishing` and `report-junk` can send still goes to `MW_ABUSE_ADDRESS` only.
- **security.txt.** If a contact was saved, the file is now public.

## The audit entry

On the first start after the upgrade the server writes one audit entry with the action `settings.now-applied`. It lists the global and domain values, saved before the upgrade, that differ from the defaults, for the settings above and for the settings in the other notes of this release. Per-user, profile and group values are counted, not listed. A value whose environment variable is set is left out, because the stored value still does not apply. Text longer than 120 characters is listed by its length. At most 200 values are listed; the entry gives the total.

The entry is written once per database. A server with no such values writes no entry.

## Other notes for this release

- [Password policy and Assist settings from the console](password-policy-enforced.md)
- [Retention jobs are scheduled](retention-jobs-scheduled.md)
- [Seven more sections in data exports](dsr-export-sections.md)
