# AI assistant: data flows and controls

This page is for administrators and reviewers. It describes what the AI assistant
sends, where it sends it, which settings control that, and what is recorded. For how
to use the assistant, see [Using the AI assistant](/user/assist); for the admin
console screens, see [AI assistant administration](/admin/ai-assistant).

Mailwoman includes no AI model and no default provider. The assistant sends requests
to an endpoint you configure: an OpenAI-compatible API, the Anthropic Messages API, or
a program on the server. With no endpoint configured the assistant is off, the web
app shows no assistant controls, and nothing is sent.

## The server is the only client

The browser never contacts the AI endpoint. Every request goes to the Mailwoman
server (`/api/assist/*`), which sends it on and streams the reply back. The provider
therefore sees the server's address and your API key, not the user's browser.

Features (capabilities): `summarize`, `draft`, `grammar`, `dictation`,
`search-semantic`, `auto-tag`, `recap` and `assistant` (the chat). None of them can
send, delete or accept anything: there is no such capability. A message the chat
proposes is shown to the user as **Proposed action**; a proposal that would send
mail opens in the Outbox, and nothing leaves until the user sends it there. The
proposed tool name comes from the model and is not checked against a tool list; it is
display text only.

## What leaves the server

| Feature | What the server sends to the endpoint |
|---|---|
| Chat (the AI assistant panel) | A fixed system instruction, the user's prompt, and the subject and preview of the open message, labelled with its account and folder |
| Auto-tag | A fixed instruction and the subject and preview of the message being read. **This request is made automatically each time a message is opened** while the `auto-tag` capability is granted, without the user asking. |
| Composer tools (**Fix grammar**, **Rewrite**, **Adjust tone**, **Translate**) | A fixed system instruction, the tool's instruction, and the text of the draft |
| Dictation | The recorded audio |
| Semantic search | The search text, and the subject and body (no attachments, at most 6000 characters each) of up to 32 of the top search results that have not been embedded before |

Mail context (the message and draft text in the second column) passes through the
data limits below. The user's own prompt is always sent. The web app does not
currently call `summarize` or `recap`.

Embeddings are made only for results of a search the user ran with semantic search
turned on; mail is not embedded when it arrives. A saved search with semantic search
on repeats this each time it is opened. Auto-tag is the one feature that sends
message text without a user action: leave it unticked under **Capabilities** if
opening a message must not contact the endpoint.

## Controls, in the order they apply

1. **The feature is enabled for the user**: [`cap.ai_assistant`](/admin/settings#cap.ai_assistant)
   (admin, per domain or user) and [`assistant.enabled`](/admin/settings#assistant.enabled)
   (users can turn it off).
2. **Consent and restriction**: a user who has withdrawn consent for the feature, or
   whose processing is restricted, is refused.
3. **The assistant is running**: the deployment switch and **Stop the AI assistant
   now** in the admin console. Stopping takes effect at once for every user and
   persists across restarts.
4. **Capability granted**: only the capabilities ticked under **Capabilities** can be
   used.
5. **Data limits** (**Data limits** in the console):
   - **Account ids whose mail may be sent**: mail context is forwarded only for
     listed accounts. With no account listed, no mail content is sent; only prompts
     and audio go out.
   - **Folder ids to limit sending to**: empty means every folder of the listed
     accounts. Auto-tag, the chat panel and the composer tools label their context
     with the mailbox id (a draft counts as being in the Drafts mailbox), so a folder
     list applies to all three.
   - **Allow attachments to be sent**: off by default.
   - **Allow end-to-end-encrypted content to be sent**: off by default; decrypted
     text is withheld unless this is on.
   - Accounts in zero-access mode are removed from the scope of chat and composer
     requests whatever the limits say.

   Context that falls outside these limits is dropped whole; the server does not
   mask parts of a text (it does not remove names, numbers or addresses).
6. **Rate limit**: `MW_ASSIST_RATE_LIMIT_PER_MIN`, counted in outbound requests per
   account per minute. Unset means no limit; `0` refuses every request. A chat turn
   costs 1; a semantic search on a cold cache costs up to 33. The environment
   variable is read when the server starts. The console's **Rate limit** group shows a
   **Requests per minute** field (`assistant.rateLimitPerMin`) and says that a value
   saved there is not applied.
7. **Audit record** (below), then the request is sent.

::: warning What the data limits rely on
For chat and composer requests the web app supplies the text and labels each piece
with its account, folder and kind (`plain`, `e2ee-decrypted` or `attachment`). The
server enforces the limits on those labels. The composer tools label an ordinary
draft `plain`. A draft that quotes a decrypted message (a reply or forward of one) or
is set to be encrypted is labelled `e2ee-decrypted`; unless decrypted content is
allowed, the composer tools refuse to run on it with "Not sent to the AI assistant:
this draft may contain text from an encrypted message, and decrypted content is not
allowed to be sent." The chat panel and auto-tag send the subject and preview the
server already holds, labelled `plain`. The limits are a
policy for the supplied client, not a barrier against a user who sends their own text
to the endpoint.
:::

`limit.ai_calls_per_day` appears in the feature editor but is not enforced.

## Endpoints

### The deployment endpoint

Set under **Endpoint** in the admin console (`PUT /admin/assist`): **Endpoint type**
(**OpenAI-compatible**, **Anthropic**, **Local program** or **None**), **Base URL**,
**API key**, and the chat, embedding and speech-to-text models. The API key is
write-only and stored sealed with the server key (`MW_SERVER_KEY`); changing the type
or base URL drops a stored key.

Requests to the deployment endpoint use an HTTP client that ignores proxy environment
variables but does not apply the [egress address policy](/security/egress): the
administrator chooses the address, including local ones.

A **Local program** endpoint runs a program on the server for each request, passing
the request, including the message text, on standard input. Only programs listed (by
absolute path) in `MW_ASSIST_LOCAL_PROGRAMS` can be chosen; arguments are passed
without a shell. With a local program, nothing leaves the server.

### Provider profiles

A provider profile is a named endpoint, request format (`openai`, `azure-openai`,
`anthropic`), model, API key and options (`maxTokens`, `temperature` 0 to 2,
`timeoutMs` 1000 to 600000, default 60000). Administrators manage them under
**Admin → AI assistant → Provider profiles** and assign one with
[`assist.profile`](/admin/settings#assist.profile) at deployment, domain or user level
(users cannot choose their own). An empty value means the deployment endpoint. If the
assigned profile does not exist or its key does not open, the user has no assistant;
the request is not sent elsewhere.

- The key is write-only, sealed with the server key and bound to the profile id,
  request format and endpoint; changing the endpoint without entering the key again
  is refused. The admin API returns only whether a key is set; the key is not written
  to the audit log or the server log.
- Every profile request goes through the [egress address policy](/security/egress):
  only public addresses, no proxy, no redirects. A local endpoint (Ollama on
  `127.0.0.1:11434`, a LAN host) needs an egress grant for its exact `host:port`.
  Adding or removing a grant requires step-up authentication and is audited. A grant
  admits loopback and private addresses, never link-local, metadata, multicast or
  unspecified addresses.
- Semantic search always uses the deployment endpoint, because one search index needs
  one embedding model.
- **Test profile** sends the fixed prompt "Connection test from Mailwoman. Reply with
  OK." with no mail content and reports the time, the first 200 characters of the
  reply, or an error code. The provider's error message is not shown, because some
  providers echo part of the key in it. Tests are audited.

![The Data limits group of the AI assistant page: account ids whose mail may be sent, folder ids to limit sending to, and the switches that allow end-to-end-encrypted content and attachments, both off.](/screenshots/security/assist-admin-limits.webp){.mw-crop}

Presets for new profiles: OpenAI-compatible (any endpoint), Anthropic
(`https://api.anthropic.com`), OpenAI, Azure OpenAI, Google Gemini
(OpenAI-compatible endpoint), Mistral AI, Groq, Together AI, OpenRouter, DeepSeek,
Fireworks AI, Perplexity, and local Ollama, LM Studio, vLLM and llama.cpp server.
The OpenAI-compatible and Anthropic formats are tested against a local mock of each
format; no preset is tested against the provider's live service. Use **Test profile**
before assigning one.

## What is recorded

Each request writes an `assist_audit` row before the request is sent: time,
capability, a scope summary (the number of accounts and folders in scope and whether
encrypted content and attachments were allowed) and the endpoint host. It contains no
prompt, message text or reply, and it does not name the user or the accounts. Writing
the row is best effort: a failed write is logged and does not stop the request. Rows
are kept for [`retention.assistAudit.days`](/admin/settings#retention.assistaudit.days)
days (default 730; 0 keeps them forever).

The server does not store prompts or replies. Whatever the endpoint provider does
with the requests it receives is governed by your agreement with that provider.

## What users are told

The web app shows **What can leave this device** with the endpoint host and what is
withheld, and on each request reports what was sent and what was withheld before the
reply streams in. When auto-tag is granted, the disclosure adds: "Auto-tag is on: each
time you open a message, its subject and preview are sent to <host> to suggest labels,
without you selecting anything."

## Related

- [AI assistant administration](/admin/ai-assistant)
- [Using the AI assistant](/user/assist)
- [Outbound connections](/security/egress)
- [Zero-access storage](/security/zero-access)
