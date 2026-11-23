# Upgrade note: the server chooses the mail server (26.20)

Applies to every deployment, in both proxy and engine mode.

## What changes

- The sign-in form asks for the email address and the password only. It has no server address field and no "enter server details manually" link, and it does not show a server it found for the address.
- The server decides which mail server an address signs in to: the domain's connection settings in the admin console, then the deployment default (`upstream.default`, otherwise `MW_UPSTREAM_DEFAULT`), then, in proxy mode, the single origin in `MW_JMAP_UPSTREAMS` if it lists exactly one. With none of these, sign-in answers `404` with code `addressNotConfigured`, and the form says "This address is not set up on this server. Contact your administrator." See [Which mail server users sign in to](../README.md#which-mail-server-users-sign-in-to).
- `POST /api/login` refuses a body that names a server (`jmapUrl`, `jmap_url`, `server`, `serverUrl`, `host`, `port`, `imapUrl`, `pop3Url`, `sessionUrl`, `upstream` or `url`, even empty) with `400` and code `clientServerNotAllowed`. A script that signs in sends only `username` and `password` (plus `clientType` for a native client).
- `POST /api/discover` answers `410 Gone`.
- With header authentication (`MW_HEADER_AUTH`) in proxy mode, the upstream is `MW_HEADER_AUTH_JMAP_URL`, otherwise the order above.
- Desktop and mobile apps built before this release send a server address to `/api/login` and are refused. Rebuild them from this release.

## What to do

1. Set the mail server your users used to type, for example in `app.env` or the service environment:

   ```ini
   MW_UPSTREAM_DEFAULT=https://jmap.example.org      # proxy mode
   MW_UPSTREAM_DEFAULT=imaps://imap.example.org:993  # engine mode
   ```

   With `deploy.sh`, set `UPSTREAM_DEFAULT` in `deploy.env` instead (not both). A proxy installation that already lists exactly one origin in `MW_JMAP_UPSTREAMS` keeps working without it.
2. If different domains use different mail servers, configure each domain in the admin console under **Domains**.
3. Rebuild and redistribute the desktop and mobile apps.
