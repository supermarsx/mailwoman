# Quick install

This page takes you from an empty Linux host to a running Mailwoman with HTTPS, using the `deploy.sh` script in the repository. It covers one path: Docker on a single host, a public hostname, and Caddy for automatic certificates. For other setups (your own reverse proxy, PostgreSQL, Kubernetes, no containers), see [Install and configure](../deploy/README.md).

## Before you start

You need:

- A Linux host with Git, Bash 4 or newer, `curl` and `tar`, and Docker running Linux containers with the Compose plugin 2.30 or newer. The user that runs the script must be able to use Docker.
- A hostname for the webmail, for example `mail.example.org`, whose A/AAAA records point at this host.
- TCP ports 80 and 443 free on the host and reachable from the Internet (UDP 443 too, if you want HTTP/3).
- An existing mail server and its address:
  - a **JMAP** server, for example `https://jmap.example.org` (proxy mode), or
  - an **IMAP** or **POP3** server with SMTP for sending, for example `imaps://imap.example.org:993` (engine mode).

Mailwoman does not receive or deliver mail itself; see [What Mailwoman is](./what-is-mailwoman.md).

## 1. Get the code

```sh
git clone https://github.com/supermarsx/mailwoman.git
cd mailwoman
```

## 2. Create the configuration

For a JMAP server:

```sh
bash deploy.sh init --domain mail.example.org
```

For an IMAP or POP3 server, add `--mode engine`:

```sh
bash deploy.sh init --domain mail.example.org --mode engine
```

`init` writes two private files in `.deploy.local/` and does not start anything:

- `deploy.env`: the deployment settings (hostname, mode, images, backup options).
- `app.env`: the server's environment, including the generated server key, the admin username and password, and the admin console path.

The script prints the admin username and the admin console path. The password is only in `app.env`, as `MW_ADMIN_PASSWORD`.

::: warning
`app.env` holds the server key that seals stored secrets. Keep it, and back it up together with the data: without it, stored secrets cannot be opened.
:::

## 3. Name your mail server

Open `.deploy.local/deploy.env` and set `UPSTREAM_DEFAULT` to your mail server:

```ini
# proxy mode (JMAP)
UPSTREAM_DEFAULT=https://jmap.example.org
```

```ini
# engine mode (IMAP or POP3)
UPSTREAM_DEFAULT=imaps://imap.example.org:993
```

Engine mode accepts `imaps://`, `imap://` (STARTTLS), `pop3s://` and `pop3://`. Mail is sent through SMTP on the same host by default; to use another SMTP server, uncomment and set `MW_SMTP_HOST`, `MW_SMTP_PORT` and `MW_SMTP_SECURITY` in `app.env`.

The sign-in screen asks only for an address and a password. Until a mail server is set here or for the domain in the admin console, every sign-in answers "This address is not set up on this server".

## 4. Check and deploy

```sh
bash deploy.sh doctor
bash deploy.sh deploy
```

`doctor` checks the prerequisites and the configuration without starting anything. `deploy` builds the Mailwoman image from the checkout (or, with `--prebuilt`, pulls the latest published image instead of compiling), starts the app and Caddy, waits until the app is healthy and prints its address and the admin console address.

## 5. Sign in

1. Open `https://mail.example.org` and sign in with the address and password of an account on your mail server. See [Sign in](./first-sign-in.md).
2. Open the admin console address that `deploy` printed (`https://mail.example.org/<console path>/`). Sign in with the **Admin username** and **Password** from `app.env`.

![The admin console sign-in form with empty Admin username and Password fields and the Sign in button.](/screenshots/deploy/admin-signin.webp){.mw-crop}

The first sign-in with these credentials creates the first admin account, with the Super admin role. Set up two-factor sign-in for it, and create named admin accounts for the people who will administer the server. See [The admin console](../admin/index.md).

![The top of the admin console after signing in: the navigation on the left and the dashboard's setup checklist and account counts.](/screenshots/deploy/admin-first-dashboard.webp){.mw-crop}

## 6. Keep it running

- Run `bash deploy.sh deploy` again to update: it pulls the checkout, takes a backup, rebuilds and restarts, and rolls back automatically if the new version does not become healthy.
- Run `bash deploy.sh backup` to take a backup, and `bash deploy.sh timers` to get systemd timers (and the equivalent cron lines) for scheduled backups and health checks.
- If you forget the console path, run `docker exec <container> mailwoman admin console-path show`.

See [Deploy with deploy.sh](../deploy/script.md) for every command and option, and [Backups](../deploy/backups.md) for restore and recovery.

## Related

- [Install and configure](../deploy/README.md)
- [Deploy with deploy.sh](../deploy/script.md)
- [Reverse proxy](../deploy/reverse-proxy.md), if you already run one
- [IMAP and POP3](../deploy/imap-pop3.md)
