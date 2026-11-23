# Upgrade note: deploy.sh files and admin variables (26.20)

Applies to installations created with `deploy.sh` before 26.20, and to any deployment that sets `MW_ADMIN_*` variables.

## deploy.sh keeps your existing files

`deploy.sh` generates `deploy.env` and `app.env` only for a new installation; later runs keep the values in them. Values that new installations get are therefore missing from older files:

- **Admin username.** New installations get a generated name such as `admin-7a93c284f016`. An existing `app.env` keeps the name it has; no administrator is renamed.
- **Admin console path.** New installations get a generated `MW_ADMIN_PATH` in `app.env`. An existing installation without one keeps `/admin`, and the server logs a warning at every start. See [the admin console path note](admin-console-path.md).
- **Caddy image.** New installations name Caddy by exact version and digest. An older `deploy.env` keeps `caddy:2-alpine`, a moving tag. To pin it, set `CADDY_IMAGE` to a version and digest (the value in `deploy.sh` is an example) and run `bash deploy.sh deploy --no-build`.
- **Resource limits.** New installations set `MEMORY_LIMIT=4g` and `CPU_LIMIT=4` in `deploy.env`. An older file has neither, so the app runs without limits, and every deploy says so in one line. Add the two keys to set limits.

## Removed admin variables stop the start

`MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`, `MW_ADMIN_THEME`, `MW_ADMIN_BRAND` and `MW_ADMIN_ACCENT` were never applied. From 26.20, setting any of them (even to an empty value) stops the start with an error that names what to use instead:

| Variable | Use instead |
| --- | --- |
| `MW_ADMIN_PORT` | Nothing: the console is served on the main listener. |
| `MW_ADMIN_LOG_LEVEL` | `MW_LOG` |
| `MW_ADMIN_OTLP_DSN` | `MW_OTLP_ENDPOINT` |
| `MW_ADMIN_METRICS` | `MW_METRICS_TOKEN` (`/metrics` is served when it is set) |
| `MW_ADMIN_THEME`, `MW_ADMIN_BRAND`, `MW_ADMIN_ACCENT` | The console's appearance settings. |

Remove them from the environment before you upgrade.

## Corrections to earlier documentation

- Earlier versions of the deployment pages suggested `--acme off` or `MW_ACME=off` to turn ACME off. `--acme` takes domain names only; the server refuses `off` (and `no`, `none`, `false`, `disabled`, `0`, or an empty `MW_ACME`) at start. To run without built-in ACME, remove the flag and leave `MW_ACME` unset.
- Earlier documentation mentioned a `mailwoman fcgi` command. It does not exist.
