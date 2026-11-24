# Upgrade note: console settings that now apply

Applies to every deployment. No action is required, but some values saved in the admin console before the upgrade had no effect until now, and they start to apply after it. Check them before you upgrade.

## What changes

Before this release the server saved the settings below but did not read them, or read only an environment variable. They now apply. Where a setting has an environment variable, the variable still wins when it is set, and the console value applies when it is unset.

| Setting | Scope | What it now does |
|---|---|---|
| `import.maxMessages` | global, domain | Most messages one import request may carry (1 to 80,000, default 20,000). A larger import is refused with `400`. |
| `import.maxMessageBytes` | global, domain | Largest message an import accepts (1 to 64 MiB, the request body limit of the import routes). A larger message is refused with `413`. A value saved above 64 MiB before this release acts as 64 MiB. |
| `webhooks.maxAttempts` | global | Delivery attempts per outbound webhook event, including breach notices (1 to 10, default 3). |
| `webhooks.timeoutSecs` | global | Time allowed for each delivery attempt (1 to 60 seconds, default 10). |
| `upstream.proxyTimeoutSecs` | global | Proxy mode: total time allowed for one request to the JMAP server (30 to 900 seconds, default 300). The server re-reads it every 30 seconds. |
| `signin.ldapBindPrecheck` | global | Checks each sign-in password with an LDAP bind before the mail server. `MW_LDAP_BIND_AUTH` wins when set. |
| `maskedEmail.domain` | global | Domain of new masked addresses. Existing addresses keep their domain. `MW_MASKED_EMAIL_DOMAIN` wins when set. |
| `abuse.reportAddress` | global, domain | Address that abuse reports from webmail (`POST /api/security/report`) are sent to, by the reporter's domain. `MW_ABUSE_ADDRESS` wins when set. |
| `health.diskFreeMinPercent` | global | The new `GET /readyz` answers `503` when the filesystem holding the data directory, `MW_UPLOAD_DIR` or the on-disk search index has less than this percentage free (1 to 50, default 5), and `200 ok` otherwise. A directory that cannot be measured is skipped. `/readyz` follows the same access rules as `/healthz`. |
| `about.securityContact`, `about.securityPolicyUrl` | global | The server answers `GET /.well-known/security.txt` (RFC 9116) with `Contact`, `Expires` (one year ahead) and `Policy`. Without a contact it answers `404`. |

A stored number outside a setting's range is treated as the nearest end of the range, so `0` cannot turn off webhook delivery or the proxy timeout.

## What to check

- **`signin.ldapBindPrecheck`.** If it was switched on in the console, every sign-in now needs a successful bind against the configured directory. A person the directory does not know, or any sign-in while the directory is unreachable, is refused. Switch it off, or set `MW_LDAP_BIND_AUTH=0`, if that is not what you want.
- **Import limits.** A domain value lower than the default refuses larger imports for that domain.
- **`abuse.reportAddress`.** In engine mode, the abuse report that the sender-control actions `report-phishing` and `report-junk` can send still goes to `MW_ABUSE_ADDRESS` only.
- **`/readyz`.** Before this release no `/readyz` route was served. Nothing shipped polls it. A load balancer or orchestrator you point at it takes the node out of rotation while a disk is below the floor; lower `health.diskFreeMinPercent` if 5 percent is too high for a large volume. Do not use it as a liveness probe.
- **security.txt.** If a contact was saved, the file is now public.

## The audit entry

On the first start after the upgrade the server writes one audit entry with the action `settings.now-applied`. It lists the global and domain values, saved before the upgrade, that differ from the defaults, for the settings above and for the settings in the other notes of this release. Per-user, profile and group values are counted, not listed. A value whose environment variable is set is left out, because the stored value still does not apply. Text longer than 120 characters is listed by its length. At most 200 values are listed; the entry gives the total.

The entry is written once per database. A server with no such values writes no entry.

## Settings removed from the console

These settings were listed in the console, but no part of Mailwoman acted on them. They are no longer registered. A value saved for one of them stays in the database and is ignored: it is not listed, exported, imported or applied. Nothing needs to be done.

| Setting | Why it was removed |
| --- | --- |
| `cap.user_css`, `cap.user_background` | No custom CSS or background feature exists. |
| `updates.check`, `updates.feedUrl`, `updates.apply` | The server has no update checker. The `updates.check` and `updates.apply` admin capabilities are unchanged. |
| `backup.retentionCount` | The server runs no backup job. `deploy.sh` makes the backups. |
| `config.snapshotsKept` | No configuration-snapshot feature exists. |
| `maintenance.announceMinutesDefault`, `maintenance.defaultMessage` | No maintenance announcement or banner exists. The maintenance-mode message is set on the Maintenance screen. |
| `observability.otlpMetrics` | The OTLP exporter is configured from the environment (`MW_OTLP_ENDPOINT`). |
| `usage.meterApi`, `usage.measureIntervalHours` | No API request counter or storage measurer exists. |
| `privacy.imageResizeFormat` | Both values produced PNG. Resized images are still PNG. |
| `restriction.mailDelivery` | The upstream mail server takes delivery, so this server cannot queue or reject mail. New restrictions record `keep`. |
| `jobs.historyDays` | It duplicated `retention.jobRuns.days` (default 90), which already prunes job history. Set that instead. |
| `metrics.drilldownTopN` | No drill-down limits its rows to a top-N count. |
| `consent.analytics.version`, `consent.analyticsTag.version`, `consent.remoteContent.version`, `consent.telemetry.version` | These purposes are not checked against the consent ledger, so the version never asked anyone again. Decisions for them are recorded with version 1. `cookies.policyVersion` still asks again for cookie consent. |
| `privacy.watermark.enabled`, `privacy.watermark.opacity` | Nothing draws a watermark. `MW_WATERMARK` and `MW_WATERMARK_OPACITY` still set what `GET /api/security/watermark` returns. |
| `about.projectLinks` | No About screen reads it. |
| `observability.errorForwardUrl`, `egress.probeUrl` | These are now set only by the environment (`MW_ERROR_FORWARD_URL`, `MW_EGRESS_PROBE_URL`), so an administrator with settings write access cannot point the server's outbound requests at another address. A value saved in the console never applied; set the variable to use one. |

## Other notes for this release

- [Send limits and the storage limit are enforced](limits-enforced.md)
- [Password policy and Assist settings from the console](password-policy-enforced.md)
- [Retention jobs are scheduled](retention-jobs-scheduled.md)
- [Seven more sections in data exports](dsr-export-sections.md)
