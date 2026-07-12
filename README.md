# Mailwoman 📬

An ultra-secure, high-performance webmail client and personal-information
manager. Inspired by [SnappyMail](https://github.com/the-djmaze/snappymail)'s
drop-it-anywhere ethos, rebuilt on **Rust + TypeScript** with a JMAP-first
architecture, verifiable end-to-end encryption (OpenPGP, S/MIME, post-quantum
ready), optional zero-access storage, full PIM modules (calendar, tasks,
encrypted notes, contacts/GAL), a strictly opt-in scoped AI subsystem, and
thin desktop & mobile clients around a web-first core.

**Status:** specification / pre-alpha. Read the full [technical spec](SPEC.md).

## Highlights

- **Web-first, thin shells** — the web client is the product; desktop and
  mobile apps are thin Tauri shells onto the same Rust server (with a
  self-contained local mode for serverless laptops).
- **Works with every server** — JMAP natively (Stalwart, Fastmail, Cyrus),
  full IMAP4rev2 + guaranteed POP3 + SMTP/Sieve, and first-party bridges for
  Microsoft Graph, on-prem Exchange (EWS), and the Gmail API.
- **Security first** — memory-safe parsing in sandboxed disposable workers,
  sandboxed HTML rendering with partial image loading, metadata & signature
  analysis, DLP, hardened `FROM scratch` containers, built-in Let's Encrypt,
  no telemetry.
- **Outlook-class features** — full calendar (sharing, conflicts, every view),
  tasks & My Day, encrypted notes, focused inbox, sweep, snooze, follow-ups,
  message recall, voting buttons, reactions, search folders, pins, and
  export to PDF/MSG/DOCX/Markdown.
- **PostgreSQL backend, optional Redis** — scope-configurable caching that is
  never load-bearing; extensive structured logging with self-hosted
  Sentry-compatible error reporting (off by default).
- **API, webhooks & MCP** — the JMAP surface is the API; scoped keys,
  human-gated sends, and a built-in MCP server for agents.
- **AI only if you bring it** — grammar review, dictation, recaps, semantic
  search, and auto-organization against endpoints *you* configure (Ollama,
  your own keys); permission-scoped, audited, invisible until configured.
- **MIT licensed** — permissive-only dependency tree, enforced in CI.

## License

[MIT](LICENSE)
