# MCP server

Mailwoman includes a Model Context Protocol (MCP) server so an AI agent can work
with a mailbox under authority the mailbox owner grants per tool and can revoke.
This page describes how calls are authorised, how sending is held for a person to
release, and what the server does and does not do about prompt injection.

## Availability and transport

- **`/mcp`** answers JSON-RPC 2.0 over Streamable HTTP. It exists only when the
  server runs in **engine mode**; in proxy mode the path answers `404`.
- **`mailwoman mcp-stdio`** forwards stdin/stdout JSON-RPC to a remote `/mcp`
  endpoint (`MW_MCP_SERVER`) with a key or token (`MW_MCP_TOKEN`), for MCP clients
  that only speak stdio.

## Authorisation

A tool call must present an API key (`mwk_…`) or an OAuth access token; see
[API keys and OAuth](./api-keys-oauth.md). For each call the server checks, in
order:

1. The key or token is valid, not revoked and not expired, and the account is not
   disabled.
2. The source address is on the key's IP allowlist and the key is within its rate
   limit.
3. **Audience:** an OAuth token must have been issued for this MCP endpoint. The
   endpoint's resource identifier is `MW_MCP_RESOURCE`, or else derived from
   `MW_PUBLIC_URL`. If neither is set, the audience is not checked. API keys are
   not bound to a resource.
4. **Tool grant:** the tool must be listed in the key's MCP tools, and the scope
   must hold the verb and surface the tool needs (table below).

Each attempt, allowed or refused, writes an audit entry. `initialize` and
`tools/list` do not need a credential.

| Tool | Needs | Output marked untrusted |
|---|---|---|
| `mail.search` | read, mail | yes (`mail-body`) |
| `mail.read` | read, mail | yes (`mail-body`) |
| `folders.list` | read, mail | no |
| `drafts.create` | send, mail | no; creates a draft, does not send |
| `mail.send` | send, mail | no; see [Sending](#sending) |
| `calendar.read` | read, pim | yes (`calendar`) |
| `calendar.propose` | send, pim | no |
| `tasks.read` | read, pim | yes (`task`) |
| `tasks.write` | send, pim | no |
| `contacts.read` | read, pim | yes (`contact`) |

## Sending

`mail.send` never transmits on an ordinary key. Which path a call takes depends on
the key:

| Key | Result |
|---|---|
| No `send` verb or no `mail.send` grant | Refused. |
| `send` and `mail.send`, no unattended send (the default) | The message is stored as a **Held** item in the account's **Outbox** and the call returns `{queued: true, sent: false}`. Nothing is handed to SMTP. |
| Unattended send requested, not approved | Refused with `403`. |
| Unattended send requested and approved by an administrator | The message is sent during the call; the result is `{queued: false, sent: true}`. |

The mailbox owner sees held items in the **Outbox** with "Held — created by API key
…" or "Held — created by the connected app …" and chooses **Release** or
**Discard**. The tool does not learn which they chose.

<!-- screenshot: id=security/mcp-outbox-held; screen=Mail > Outbox; state=one held message created by an API key; element=held row with Release and Discard -->

An administrator approves unattended send per key in **Integrations** → **API &
MCP keys** (column **Unattended send**: **Not requested**, **Requested, not
approved**, **Approved**). The approval is read from the database on every call,
so withdrawing it or revoking the key applies to the next call. OAuth tokens can
never send unattended. Approving does not ask the administrator for a fresh
confirmation of their identity.

<!-- screenshot: id=security/mcp-admin-unattended; screen=Admin console > Integrations > API & MCP keys; state=one key requested unattended send; element=Unattended send column with Approve -->

## Prompt injection

Mail is text written by whoever sent it, and an agent that reads a mailbox reads
instructions aimed at it. Mailwoman does not prevent an agent from following such
instructions. It limits what a steered agent can do:

- **Labels.** Tool results derived from message content are wrapped with
  `provenance: {trust: "untrusted", source: …}`, and the descriptions of the read
  tools say their output is untrusted data. A client can use the label to keep
  mail text apart from its instructions; whether it does is up to the client and
  the model.
- **No raw protocol.** Tools call the server's JMAP layer, the same path the web
  app uses. No tool builds IMAP or SMTP commands from its arguments.
- **Least authority.** An agent can do only what the key allows. A key with
  read-only mail access cannot send, delete or touch PIM data, whatever a message
  says.
- **Sending stays with a person** unless an administrator approved unattended send
  for that key.

What remains possible: an agent with read access can be steered into summarising
or quoting content in misleading ways, and an agent with send access on a held key
can fill the Outbox with messages a person might release without reading closely.
Grant read-only keys where you can, keep sending held, and treat a key with
approved unattended send as a privileged credential.

## Related

- [API keys and OAuth](./api-keys-oauth.md)
- [AI Assist privacy](../assist.md)
- [Request limits](/admin/request-protection)
