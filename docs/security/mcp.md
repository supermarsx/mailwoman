# MCP server security & prompt-injection posture (V6)

Mailwoman ships a **Model Context Protocol** server so an AI agent can work with a
mailbox under explicit, revocable, per-tool authority. Two design commitments make this
safe to expose: **every tool goes through the existing engine/JMAP surface** (never raw
IMAP/SMTP), and **mail content is treated as untrusted input** end-to-end.

## Transport & auth

- **Streamable HTTP at `/mcp`** for network clients, and **`mailwoman mcp-stdio`** for
  a local stdio client that proxies to a configured server.
- **MCP keys are API keys.** Authentication is an `mw-oauth` `mwk_<prefix>.<secret>`
  key or an OAuth 2.1 access token — the same scoping, expiry, IP-allowlist,
  rate-limit, and audit as any other key (see [`api-keys-oauth.md`](./api-keys-oauth.md)).
  The set of callable tools is the key's `mcp_tools` scope; a tool the key does not
  name is denied.

## Tools

Ten tools, each mapped to a required scope fragment and grantable individually:

| Tool | Requires | Notes |
|---|---|---|
| `mail.search` | read · mail | results carry untrusted provenance |
| `mail.read` | read · mail | results carry untrusted provenance |
| `folders.list` | read · mail | |
| `drafts.create` | send · mail | writes a draft; does not transmit |
| `mail.send` | send · mail | **gated** — see below |
| `calendar.read` | read · pim | untrusted provenance |
| `calendar.propose` | write · pim | proposes, does not auto-commit |
| `tasks.read` | read · pim | untrusted provenance |
| `tasks.write` | write · pim | |
| `contacts.read` | read · pim | untrusted provenance |

## Send is disabled by default, human-in-the-loop when enabled

Unattended sending is the highest-risk capability (SPEC §7.1: over-privileged
automation), so the gate is deliberate:

1. **No `send` scope** → `mail.send` is denied outright (the safety test asserts the
   transmit path is never reached).
2. **`send` scope, no `unattended_send`** (the default) → `mail.send` returns
   `{ queued: true, outboxId }`: the message lands in the **Outbox** and waits for an
   in-app human confirmation. It is **not** transmitted by the tool call.
3. **`send` + `unattended_send` + an admin countersignature on the key** → the message
   may transmit immediately.
4. **`send` + `unattended_send` but no countersignature** → **403**. Requesting
   unattended send without the admin countersign is refused, never silently downgraded
   in a way that would surprise the operator.

**Status (26.7):** the admin-countersign resolver is not yet wired, so in practice
every `mail.send` currently lands in the Outbox (case 2) or is refused (case 4) — the
safe default. Unattended transmission is not reachable until an operator provisions a
countersigned key. This is the intended conservative posture; it is documented rather
than hidden.

## Prompt-injection posture

Mail is attacker-controlled text. An agent that reads a mailbox will read whatever a
sender wrote, including instructions aimed at the agent. Mailwoman does not claim to
"solve" prompt injection; it constrains the blast radius:

- **Provenance labels.** Every tool result derived from mail content is wrapped in an
  untrusted envelope — `mail.search` / `mail.read` results are labelled
  `untrusted:mail-body`, calendar/task/contact reads carry their own source labels, and
  the `tools/list` metadata marks which tools produce untrusted output. A well-behaved
  client can therefore distinguish operator instructions from mail-borne text.
- **Tool descriptions declare mail untrusted.** The tool schemas themselves state that
  mail bodies are untrusted input, so the model sees the warning in-band.
- **No raw protocol composition.** No tool builds or forwards raw IMAP/SMTP commands.
  Tools call the engine/JMAP surface, which is the same validated path the mailbox UI
  uses — a malicious mail body cannot smuggle a protocol command through a tool.
- **Least authority.** Because the callable tool set and the send capability are per-key
  scopes, an injected instruction can only do what the key was already granted — it
  cannot escalate. Grant read-only, no-send MCP keys unless sending is genuinely needed.

### The honest boundary

Provenance labels and least-authority reduce, but do not eliminate, prompt-injection
risk: a client that ignores the untrusted labels, or a key over-granted with
`unattended_send`, can still be steered by hostile mail. The defense is the **scope you
grant** plus a client that honors provenance. Grant narrowly, keep send human-in-the-loop,
and treat any MCP key with `unattended_send` as a privileged credential.
