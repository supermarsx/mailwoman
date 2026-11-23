# What Mailwoman is

Mailwoman is a self-hosted webmail application. You run its server on your own machine or host, and your users sign in with a web browser, the desktop app or the Android app to read and send mail from a mail server you already have. This page explains what the parts are and how they fit together, so you can choose how to install it.

<!-- SCREENSHOT deploy/overview-mailbox: the mailbox after sign-in with demo data (folder list, message list, open message), full viewport -->

## What it does and does not do

- **It is the mail client, not the mail server.** Mailwoman does not receive mail from the Internet or deliver it to other domains. Each user signs in with the address and password of an account on a mail server you name in the configuration, and Mailwoman reads and sends through that server.
- **It runs in one of two modes**, chosen for the whole server with `MW_MODE`:
  - **Proxy mode** (`proxy`, the default): the mail server speaks JMAP. Mailwoman passes the browser's JMAP requests on to it.
  - **Engine mode** (`engine`): the mail server speaks IMAP or POP3, with SMTP for sending. Mailwoman keeps a synchronised copy of each account in its own database and offers the same JMAP interface to the web app. See [IMAP and POP3](../deploy/imap-pop3.md).
- **It keeps its own data** in one SQLite file by default, or in PostgreSQL (see [PostgreSQL](../deploy/postgres.md)): sessions, settings, the admin console's state and, in engine mode, the account copies. Stored secrets are sealed with a 32-byte server key, which you must back up together with the database.

## The parts

| Part | What it is |
| --- | --- |
| `mailwoman` | The server binary. `mailwoman serve` runs the web server, which also serves the web app it has embedded. Other subcommands handle maintenance, backups and the admin command line. |
| `mw-render` | A worker the server starts for one job at a time to process untrusted message content. On Linux it runs inside a kernel sandbox. The Docker image includes it at `/usr/local/bin/mw-render`. |
| Web app | The mailbox, settings and the admin console, served by the server. Besides mail, it has calendar, contacts, tasks and notes screens. |
| Admin console | The administration screens, at a private path that each installation generates. Admin accounts are separate from mailbox accounts. |
| Desktop and Android apps | Optional apps that open the same web app against the server they were built for. See [Desktop and mobile shells](../deploy/shells.md). |

## Ways to install

- **[`deploy.sh`](../deploy/script.md)** on one Linux host with Docker: it writes the configuration, builds or pulls the image, starts the app, optionally adds Caddy for automatic HTTPS, and handles updates, backups and restores. This is the shortest path: see [Quick install](./install-quick.md).
- **The Docker image** on your own container setup, or **the Helm chart** on Kubernetes: see [Packaging](../deploy/packaging.md).
- **The binaries under systemd** on a server without containers: see [Install and configure](../deploy/README.md#systemd).

## Related

- [Quick install](./install-quick.md)
- [First sign-in](./first-sign-in.md)
- [Install and configure](../deploy/README.md)
- [Security model](../security/README.md)
