# Mailwoman 📬

**Self-hosted webmail and personal workspace for the accounts you already use.**

Mailwoman brings email, calendars, contacts, tasks, and encrypted notes into one
browser workspace. Connect a JMAP server or use Mailwoman's engine with an existing
mail provider. The web app pairs a Rust server with a TypeScript interface, and the
desktop and mobile shells share the web client.

[Features](#features) · [Webmail comparison](#webmail-comparison) · [Quick start](#quick-start) · [Documentation](#documentation)

> **Project status**
>
> Mailwoman is under active development. Feature availability depends on deployment
> mode, provider capabilities, and administrator policy. The [technical spec](SPEC.md)
> covers the broader design and planned work.

## Features

| Area | What you can do |
| --- | --- |
| **Mail workspace** | Open folders and messages in tabs, choose a reading pane layout, group related messages into conversations, and use bulk actions, follow-up flags, snooze, and saved searches. |
| **Compose and send** | Write in rich text, HTML, or plain text, use signatures for new messages and replies, resume drafts, and schedule delivery where supported. [Compose formats](docs/deploy/compose-formats.md) · [Draft storage](docs/deploy/account-drafts.md) |
| **Attachments** | Browse and filter attachments, preview PDFs and images, and inspect file details. Image controls include fit, zoom, rotate, and copy. Attachment thumbnails are opt-in. [Preview policy](docs/deploy/attachment-preview.md) |
| **Reader privacy** | View messages in a sandbox, control remote images and links per message or sender, and inspect authentication and delivery details. [Privacy controls](docs/admin/privacy.md) · [Delivery status](docs/deploy/delivery-status.md) |
| **Encryption** | Use OpenPGP or S/MIME with browser-side private-key operations. Optional zero-access storage has its own configuration and scope. [Message encryption](docs/security/crypto.md) · [Zero-access storage](docs/security/zero-access.md) |
| **Calendar and contacts** | Manage events and address books alongside mail and connect supported CalDAV and CardDAV services. [DAV setup](docs/deploy/caldav-carddav.md) |
| **Personal organization** | Keep tasks, My Day, and encrypted notes in the same workspace. Customize the ribbon, message list, and reader from settings. |
| **Mail connections** | Proxy to JMAP or connect through the engine using IMAP or POP3 with SMTP. Provider bridges cover Microsoft Graph, Exchange EWS, and the Gmail API. [JMAP](docs/admin/jmap.md) · [IMAP/POP3](docs/deploy/imap-pop3.md) · [Graph](docs/bridges/graph.md) · [EWS](docs/bridges/ews.md) · [Gmail](docs/bridges/gmail.md) |
| **Administration** | Set deployment and account policies, manage backups, and configure an optional AI assistant with explicit endpoint, capability, and data limits. [Settings reference](docs/admin/settings.md) · [Backups](docs/deploy/backups.md) · [AI assistant](docs/admin/ai-assistant.md) |

## Webmail comparison

The Outlook row covers **Outlook on the web**. Availability can vary with the
edition, account, server, extensions, and administrator policy.

| Web client | Hosting and mail connection | Workspace and workflow | Message encryption |
| --- | --- | --- | --- |
| **Mailwoman** | Self-hosted. JMAP proxy or engine with IMAP/POP3 and SMTP, plus provider bridges. | Mail, calendar, contacts, tasks, and notes. Workspace tabs, follow-up flags, and configurable actions. | Browser-side OpenPGP and S/MIME. [Connection guide](docs/admin/jmap.md) · [Crypto guide](docs/security/crypto.md) |
| **Outlook on the web** | Microsoft 365 / Exchange Online, with web access also available in Exchange Server deployments. | Mail, calendar, and People. Inbox tools include Sweep and Archive. | S/MIME or Microsoft Purview Message Encryption, depending on account and policy. [People](https://support.microsoft.com/en-us/outlook/using-contacts-people-in-outlook-on-the-web) · [Calendar](https://support.microsoft.com/en-us/outlook/calendar-sharing-in-microsoft-365) · [Inbox tools](https://support.microsoft.com/en-us/outlook/organize-your-inbox-with-archive-sweep-and-other-tools-in-outlook-on-the-web) · [Encryption](https://support.microsoft.com/en-us/outlook/mail/learn-about-securing-and-protecting-email-messages-in-outlook) |
| **RainLoop** | Self-hosted PHP webmail for IMAP/SMTP. The original repository is archived. | Email-focused interface with contacts and support for multiple accounts. | Client-side OpenPGP. [Features](https://www.rainloop.net/features/) · [Repository](https://github.com/RainLoop/rainloop-webmail) |
| **SnappyMail** | Self-hosted PHP webmail for IMAP/SMTP. A fork of RainLoop. | Mail, contacts, identities, and Sieve filtering. | OpenPGP. [Project](https://github.com/the-djmaze/snappymail) · [Feature comparison](https://snappymail.eu/comparison) |
| **Roundcube** | Self-hosted PHP webmail for IMAP/SMTP. | Mail and address books with a plugin ecosystem. | PGP support. [Project features](https://roundcube.net/about/) |
| **Bulwark** | Self-hosted JMAP webmail focused on Stalwart. A separate Legacy Proxy can connect an existing IMAP stack. | Mail, calendar, contacts, and files in one client. | OpenPGP through an extension. [Project](https://bulwarkmail.org/) · [OpenPGP extension](https://extensions.bulwarkmail.org/extension/openpgp) |

## Quick start

On a host with Bash, Docker, and Compose, run from the repository root:

```sh
bash deploy.sh --domain mail.example.org --mode engine
```

Point your domain at the deployment host and allow inbound ports 80 and 443 for
automatic HTTPS. Omit `--domain` for a local HTTP deployment. Omit
`--mode engine` to use JMAP proxy mode. The deployment runs the web app and
connects to your existing mail service. See the full prerequisites and upstream
configuration in the [deployment guide](docs/deploy/script.md).

## Documentation

| If you want to… | Start here |
| --- | --- |
| Install, update, or restore a deployment | [Deployment guide](docs/deploy/script.md) · [Backup and recovery](docs/deploy/backups.md) |
| Configure mail providers and accounts | [JMAP](docs/admin/jmap.md) · [IMAP/POP3 and SMTP](docs/deploy/imap-pop3.md) · [Domain providers](docs/admin/domain-providers.md) |
| Review security and privacy settings | [Hardening](docs/deploy/hardening.md) · [Privacy](docs/admin/privacy.md) · [Report a vulnerability](SECURITY.md) |
| Set up desktop or mobile clients | [Desktop](docs/deploy/desktop.md) · [Android](docs/deploy/mobile-android.md) · [JMAP client discovery](docs/admin/jmap.md) |
| Explore the architecture and design | [Technical specification](SPEC.md) |

## License

Mailwoman is released under the [MIT license](license.md).
