# Monitoring a deploy.sh installation

Two things are available without extra software: Prometheus metrics at
`/metrics`, and `bash deploy.sh check`, a health check that can call a script of
yours when it fails. [backups.md](backups.md#scheduling-timers) shows how to run
the check every five minutes with systemd or cron.

## Metrics

`/metrics` serves the Prometheus text format and requires
`Authorization: Bearer <MW_METRICS_TOKEN>`. Without the token, or with a wrong
one, it answers 401; with `MW_METRICS_TOKEN` unset it answers 401 to everyone, so
metrics are never open. No mail content is a metric name or label
([observability.md](../security/observability.md)).

`bash deploy.sh init` writes a random 64-hex-digit `MW_METRICS_TOKEN` into
`app.env` for new installations. `bash deploy.sh status` says whether the applied
`app.env` has one, and whether `app.env` was edited since.

### Installations created before the token was generated

Add the token to `app.env` and apply it:

```sh
printf 'MW_METRICS_TOKEN=%s\n' "$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')" >> .deploy.local/app.env
bash deploy.sh deploy --no-build
bash deploy.sh status        # Metrics: enabled at <URL>/metrics ...
```

To turn metrics off, delete the line and run `deploy --no-build` again.

### Scrape configuration

Give Prometheus the token as a file (keep it readable only by Prometheus):

```yaml
scrape_configs:
  - job_name: mailwoman
    scheme: https                     # http for a local installation
    metrics_path: /metrics
    authorization:
      type: Bearer
      credentials_file: /etc/prometheus/mailwoman-metrics-token
    static_configs:
      - targets: ['mail.example.org']  # localhost:8080 for a local installation
```

With `--proxy-network`, a Prometheus container attached to the same Docker
network can scrape `http://<project>:8080/metrics` directly (`scheme: http`,
target `<project>:8080`) instead of going through the public URL.

## Health check: `check`

```sh
bash deploy.sh check
```

exits 0 only when:

1. the app container is running and Docker reports it `healthy` (its own
   `/healthz` check), and
2. `/healthz` answers at the public URL from `deploy.env`; for `network`
   exposure, whose public URL is served by a proxy this script does not manage,
   it answers at `http://<project>:8080/healthz` on the proxy network instead
   (checked from a short-lived container on that network).

Otherwise it prints `CHECK FAILED (<check>): <message>`, runs `ALERT_HOOK`, and
exits 1. `<check>` is `container`, `public-url` or `proxy-network`.

`check` only reads: it does not take the state directory's lock and changes
nothing, so it can run while a backup or deploy is in progress. During a deploy
or backup the app is briefly stopped, and a check at that moment fails.

### `ALERT_HOOK`

`ALERT_HOOK` in `deploy.env` is the absolute path of an executable that `check`
runs on failure with two arguments, the check's name and the message. It runs as
the user that runs `deploy.sh`, with no input. Its exit status is reported but
does not change `check`'s own (always 1 on failure). The hook runs on every
failing check, every five minutes with the timer, so it should do its own
de-duplication if that matters. For example:

```sh
#!/bin/sh
# Send the failure to an ntfy topic (or any webhook).
exec curl -fsS -m 10 -d "mailwoman: $1: $2" https://ntfy.example.org/mailwoman-alerts
```

```sh
#!/bin/sh
# Mail it through the host's MTA.
printf 'Subject: mailwoman check failed (%s)\n\n%s\n' "$1" "$2" | exec sendmail ops@example.org
```

Like the backup settings, `ALERT_HOOK` is read from `deploy.env` as saved; no
deploy is needed after changing it. `doctor` checks that it is an executable
file.

The check proves that the app answers over HTTP(S). It does not sign in to a
mailbox or send mail.
