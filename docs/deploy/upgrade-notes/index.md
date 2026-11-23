# Upgrade notes

Each note describes a change that needs action, or a check, when you update an existing installation. Read the notes for every release between the one you run and the one you are installing, then update as usual (with `deploy.sh`, run `bash deploy.sh deploy`; it takes a backup first and rolls back if the new version does not become healthy).

To see which release you run, check the image tag or run `mailwoman --version`.

## 26.20

- [The server chooses the mail server](mail-server-chosen-by-server.md): the sign-in form has no server field; set `MW_UPSTREAM_DEFAULT` or configure domains.
- [Proxy-mode upstreams are checked](proxy-upstream-checked.md): a JMAP server on a private address must be named in the configuration.
- [The admin console answers only at its configured path](admin-console-path.md).
- [Feature profiles cover every gateable feature](feature-profiles-everywhere.md).
- [deploy.sh files and admin variables](deploy-script-and-environment.md): older `deploy.env`/`app.env` files, and `MW_ADMIN_*` variables that now stop the start.
- [Single sign-on provider settings and SAML signatures](sso-provider-settings.md): re-enter providers saved from the console, and review past SAML sign-ins if a provider accepted unsigned responses.
- [Sign-in locks and two-factor enrolment](sign-in-locks-and-two-factor.md): IPv6 sign-in locks cover the /64, a full lock table no longer refuses new sources, and accounts without a second factor set one up at sign-in.
- [OAuth client redirect addresses](oauth-client-redirects.md): self-registered apps need `https` (or `http` to a loopback IP) redirect addresses; unsafe addresses get no authorization code.

## 26.19

- [Forwarded headers and public address](forwarded-headers.md): set `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` behind a proxy.

## Engine-mode accounts

- [Migration 0028, one account per mail identity](0028-account-merge.md): applies to databases that have run in engine mode.
