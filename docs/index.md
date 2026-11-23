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
      text: Install Mailwoman
      link: /getting-started/install-quick
    - theme: alt
      text: Use the app
      link: /user/
    - theme: alt
      text: Documentation map
      link: /README

features:
  - title: Start here
    details: What Mailwoman is, a quick install and your first sign-in.
    link: /getting-started/what-is-mailwoman
    linkText: Get started
  - title: Using Mailwoman
    details: Reading, writing, searching and organizing mail; contacts, calendar, tasks and notes; personal settings.
    link: /user/
    linkText: Read the user guide
  - title: Administering
    details: Install and upgrade, accounts and domains, security policy, privacy and compliance, and day-to-day operations.
    link: /admin/index
    linkText: Run the server
  - title: Security model
    details: One page per surface, including what each control does not guarantee.
    link: /security/README
    linkText: Review the model
  - title: Integrations and protocols
    details: Mail provider bridges, Nextcloud, plugins and message export.
    link: /bridges/index
    linkText: Connect and extend
  - title: Reference
    details: The command line, environment variables, configuration, every setting and a glossary.
    link: /reference/cli
    linkText: Look something up
---

![The Mailwoman mailbox in the dark theme: the folder list with unread counts, a message list of demo messages and an open message in the reading pane](/screenshots/core/mailbox.webp){.mw-shot}

## Start with your role

| You want to… | Read |
| --- | --- |
| Read, compose and organize mail | [Using Mailwoman](./user/index.md) |
| Install or upgrade a server | [Quick install](./getting-started/install-quick.md) · [Install and configure](./deploy/README.md) |
| Configure users and policies | [The admin console](./admin/index.md) · [Settings reference](./admin/settings.md) |
| Understand privacy and security | [Security guide](./security/README.md) |
| Connect a mail provider | [JMAP discovery](./admin/jmap.md) · [IMAP/POP3](./deploy/imap-pop3.md) · [Graph](./bridges/graph.md) |
| Find all documentation | [Documentation map](./README.md) |

## Find your way

- **Sidebar**: the whole manual grouped by topic, with the current page highlighted. On small screens, open it with **Menu**.
- **Search**: press `Ctrl` `K` (`⌘` `K` on macOS) or `/`. Search runs in your browser from an index built with the site; no query leaves the page.
- **On this page**: each article has an outline, previous and next links, and an edit link to its source on GitHub.
- **Theme**: the manual starts in dark mode. The theme switch in the header changes it, and your choice is remembered in this browser.
