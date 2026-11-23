---
layout: home
title: Mailwoman manual
titleTemplate: false

hero:
  name: Mailwoman
  text: The manual
  tagline: Self-hosted webmail and personal workspace for the mail accounts you already use. Find the settings and workflows you need, from your first mailbox to running a deployment.
  image:
    src: /logo.svg
    alt: Mailwoman icon
  actions:
    - theme: brand
      text: Deploy Mailwoman
      link: /deploy/script
    - theme: alt
      text: Use the app
      link: /user/sync-offline
    - theme: alt
      text: Documentation map
      link: /README

features:
  - title: Using Mailwoman
    details: Sync and offline access, multiple windows, unified mail and snooze.
    link: /user/sync-offline
    linkText: Read the user guide
  - title: Installation & operations
    details: The deployment script, reverse proxies, TLS, PostgreSQL, backups, packaging and the desktop and Android shells.
    link: /deploy/README
    linkText: Deploy and operate
  - title: Administration
    details: One page per admin console area, plus the settings and configuration references.
    link: /admin/index
    linkText: Configure the server
  - title: Security & privacy
    details: The security model, one page per surface, including what each control does not guarantee.
    link: /security/README
    linkText: Review the model
  - title: Mail providers
    details: JMAP discovery, IMAP and POP3, and the Microsoft Graph, Exchange EWS and Gmail API bridges.
    link: /bridges/index
    linkText: Connect a provider
  - title: Integrations & plugins
    details: Nextcloud, WebAssembly plugins, spam trainers, masked email and message export.
    link: /integrations/nextcloud
    linkText: Extend Mailwoman
---

![The Mailwoman mailbox in the default dark theme: a folder sidebar, a message list of fictional demo messages, and an open message in the reading pane](/screenshots/mailbox-dark.png){.mw-shot}

## Start with your role

| You want to… | Read |
| --- | --- |
| Read, compose and organize mail | [Using Mailwoman](./user/sync-offline.md) |
| Install or upgrade a server | [Deployment guide](./deploy/script.md) · [Operations index](./deploy/README.md) |
| Configure users and policies | [Administrator settings](./admin/settings.md) |
| Understand privacy and security | [Security guide](./security/README.md) |
| Connect a mail provider | [JMAP discovery](./admin/jmap.md) · [IMAP/POP3](./deploy/imap-pop3.md) · [Graph](./bridges/graph.md) |
| Find all documentation | [Documentation map](./README.md) |

## Find your way

- **Sidebar**: the whole manual grouped by topic, with the current page highlighted. On small screens, open it with **Menu**.
- **Search**: press `Ctrl` `K` (`⌘` `K` on macOS) or `/`. Search runs in your browser from an index built with the site; no query leaves the page.
- **On this page**: each article has an outline, previous and next links, and an edit link to its source on GitHub.
- **Theme**: the manual starts in dark mode. The theme switch in the header changes it, and your choice is remembered in this browser.
