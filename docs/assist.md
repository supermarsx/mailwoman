# AI assistant — endpoints, privacy, and governance (V7)

V7 (release 26.8.0) adds the AI assistant (`mw-assist`): optional AI features — summarize, draft,
grammar, dictation, semantic search, auto-tag, recap, and an assistant chat. The AI assistant is
**bring-your-own-endpoint**: you point it at an AI endpoint you control or trust;
Mailwoman ships no built-in model and no default provider.

The AI assistant is **off until configured.** With no endpoint configured, the gateway reports
`Disabled` and the web app hides every AI assistant surface — there is no AI assistant UI, no
prompts, nothing sent anywhere.

## Architecture: the engine is the only client

The browser **never** talks to an AI endpoint directly. All AI assistant traffic goes
through the engine-side **AI assistant gateway** (`mw-assist`), which the server proxies. The
browser's CSP keeps `connect-src 'self'` — the AI host is contacted only by the
server, never by the page. This mirrors the existing server-proxied surfaces.

## Endpoints (adapters)

Three adapters, all hand-rolled JSON over the in-tree `reqwest` (rustls) — no vendor
SDK:

| Adapter | Talks to |
|---|---|
| `OpenAiCompatible` | any OpenAI-compatible API: chat completions, embeddings, `/v1/audio/transcriptions` (works with Ollama, vLLM, LiteLLM, OpenAI, etc.) |
| `Anthropic` | the Anthropic Messages API |
| `LocalProcess` | a local binary you configure; JSON over stdio (fully on-device) |

Chat, embeddings, and speech-to-text are independent slots — you can, for example, run
a local embeddings model and a hosted chat model.

## Provider profiles

A **provider profile** is one endpoint, model, API key and set of request options
(t39 N8). Admins manage them under **Admin → AI assistant → Provider profiles**
(`/admin/assist/profiles`, `ai_assistant.read` / `ai_assistant.write`).

- **Fields.** Name, preset, request format (`openai`, `azure-openai`, `anthropic`),
  endpoint URL, model, the API key, and options: `maxTokens`, `temperature`
  (0 to 2), `timeoutMs` (1000 to 600000, default 60000; the whole request, connect
  to the last byte of the reply), `apiVersion` (Azure), `anthropicVersion`,
  `transcriptionModel` (dictation through the profile; unset means no dictation).
- **The key** is write-only. It is sealed with the server key (migration 0073,
  `assist_profile.api_key_sealed`) and bound to the profile id, request format and
  endpoint, so a key does not follow a profile to a new endpoint: changing the
  endpoint without entering the key again is refused (`422`). The admin API returns
  only `hasApiKey`; the key is not written to the audit log or to the server log.
- **Assignment** is the settings key `assist.profile` (a profile id): a deployment
  value, per-domain values and per-user values, with the registry's locks. A user
  value wins over the domain value, which wins over the deployment value, unless a
  wider value is locked. An empty value means the single deployment endpoint
  configured above the profiles (`PUT /admin/assist`). A user assigned a profile
  that no longer exists, or whose key does not open, has no AI assistant: the
  request is refused, not sent to another endpoint.
- **What still applies.** The `assist_config` on/off switch and kill switch, the
  capability grants, the data ceilings, redaction, the rate limit and the
  content-free audit run in front of the profile exactly as for the deployment
  endpoint. The audit row records the profile's endpoint host. Semantic search keeps
  the deployment endpoint, because one search index needs one embedding model.
- **Zero-access accounts.** Context from an account in zero-access mode is never
  forwarded, by either endpoint, whatever the data ceiling allows: the server drops
  those accounts from the request scope before redaction.
- **Usage.** The repository does not meter AI assistant usage per call
  (`limit.ai_calls_per_day` is registered, but no server code reads it), so profiles add no
  metering. The content-free audit rows remain the per-call record.

### Outbound addresses

Every profile request is resolved once through the `mw-egress` address gate and
the connection is pinned to the address that passed, with no proxy and no
redirects. Without a grant only public addresses pass. A local endpoint (Ollama on
`127.0.0.1:11434`, a LAN host) needs an **egress grant** for its exact `host:port`
(`/admin/assist/egress-grants`; adding or removing one needs a step-up and is
audited). A grant admits loopback and private addresses; link-local and cloud
metadata addresses (`169.254.169.254`), multicast and unspecified addresses are
refused even with a grant. The single deployment endpoint (`PUT /admin/assist`)
predates profiles and is not routed through this gate.

### Test profile

**Test profile** (`POST /admin/assist/profiles/{id}/test`) sends one fixed prompt
with no mail content ("Connection test from Mailwoman. Reply with OK.") and reports
the time to the end of the reply, the first 200 characters of it, or an error code:
`egress-refused`, `timeout`, `connect`, `unauthorized` (401/403), `not-found`,
`rate-limited`, `rejected` (other 4xx), `upstream` (5xx), `bad-response`,
`declined`, `unsupported`. With an HTTP error the status and the provider's own
short error code (`invalid_api_key`, `authentication_error`) are shown; the
provider's error message is not, because some providers echo part of the key in it.
The test runs whether or not the AI assistant is switched on, and is audited.

### Presets

| Preset | Format | Endpoint | Checked |
|---|---|---|---|
| OpenAI-compatible (any endpoint) | openai | (you enter it) | local mock of the format |
| Anthropic | anthropic | `https://api.anthropic.com` (`claude-opus-5-5`, `claude-sonnet-5-5`, `claude-haiku-5-5`) | local mock of the format |
| OpenAI | openai | `https://api.openai.com/v1` | not tested |
| Azure OpenAI | azure-openai | `https://<resource>.openai.azure.com/openai/deployments/<deployment>` | not tested |
| Google Gemini (OpenAI-compatible endpoint) | openai | `https://generativelanguage.googleapis.com/v1beta/openai` | not tested |
| Mistral AI | openai | `https://api.mistral.ai/v1` | not tested |
| Groq | openai | `https://api.groq.com/openai/v1` | not tested |
| Together AI | openai | `https://api.together.xyz/v1` | not tested |
| OpenRouter | openai | `https://openrouter.ai/api/v1` | not tested |
| DeepSeek | openai | `https://api.deepseek.com/v1` | not tested |
| Fireworks AI | openai | `https://api.fireworks.ai/inference/v1` | not tested |
| Perplexity | openai | `https://api.perplexity.ai` | not tested |
| Ollama (local) | openai | `http://127.0.0.1:11434/v1` | not tested |
| LM Studio (local) | openai | `http://127.0.0.1:1234/v1` | not tested |
| vLLM (local) | openai | `http://127.0.0.1:8000/v1` | not tested |
| llama.cpp server (local) | openai | `http://127.0.0.1:8080/v1` | not tested |

"Local mock of the format" means `crates/mw-server/tests/t39_assist_profiles.rs`
ran the request format against an in-process server that answers in the documented
streaming format; it is not a run against the provider's service. No preset has been
run against a live provider. Model names are suggestions and change; check them with
the provider and use Test profile.

## Capabilities and scoping

Each AI assistant feature is a capability, granted independently:

`summarize`, `draft`, `grammar`, `dictation`, `search-semantic`, `auto-tag`, `recap`,
`assistant`.

> **`search-semantic` now re-ranks (new in 26.19).** This note previously said the
> re-rank was not wired; it is. A query marked `semantic` has its top lexical hits
> re-ordered by embedding similarity, and every degradation — no capability, no
> endpoint, a dimension mismatch — lands back on lexical ranking rather than on an
> error.
>
> **This makes `search-semantic` a second egress path, and it is the one users are
> least likely to expect.** Running a semantic search sends the text of the messages
> being re-ranked (up to 32 per cold query) to the configured endpoint, not just the
> query. It is opt-in per query — with one qualification: **a saved-search folder
> whose stored filter carries `semantic: true` re-runs the re-rank every time the
> folder is opened.** Still user-initiated, so the egress bound holds, but the opt-in
> becomes persistent rather than per-query, and an operator sizing a rate limit
> should assume repeat traffic from saved searches.
>
> Bound it with `MW_ASSIST_RATE_LIMIT_PER_MIN` (per account, counted in **outbound
> endpoint requests**): a cold-cache semantic search costs up to 33 — one for the
> query plus one per document embedded on demand — converging toward 1 as the
> embedding cache fills. A chat turn costs 1. `0` refuses every AI assistant request, which
> is a usable kill switch.
>
> Embeddings are produced **only** for messages a user's own opt-in search surfaced.
> Mail is deliberately **not** embedded at ingest: that would send every message a
> deployment receives to the configured endpoint as a side effect of enabling a
> search feature, which is an egress posture nobody opted into.

Every invocation is enforced by the gateway in this order:

1. **Capability granted?** If not, denied.
2. **Data-class ceiling.** Which accounts/folders may be used, and two defaults that
   matter: end-to-end-encrypted decrypted content is **excluded by default**
   (`include_e2ee = false`) and attachments are **excluded by default**
   (`include_attachments = false`).
3. **Redaction** runs before anything leaves the engine.
4. **Rate-limit.**
5. **Audit** — a content-free record (below).

## Send is always human-gated

**No AI assistant capability transmits, deletes, or accepts anything.** This is structural:
the capability enum has no send/delete/accept variant, so there is no code path for
the AI assistant to send mail or act irreversibly. A drafted message goes to the Outbox for a
human to send. The AI assistant adds no privileged path.

**One correction to how this used to be described.** The assistant chat is *not* yet
driven by the MCP tool surface. What ships is proposal **reporting**: the tool name in
a proposed action comes from the model and is **not validated against the `mw-mcp`
registry**. Because nothing is ever executed and every action is human-confirmed, the
consequence is that a proposal could display a tool name that does not correspond to
any real tool — a display-accuracy limit, not a privilege one. The send-gating above
is structural and unaffected. Do not describe the assistant as MCP-backed tool calling
until it is wired.

## What left the device (disclosure)

The AI assistant shows a per-message **"what left the device"** disclosure so the user can see,
for each AI assistant action, what was sent and to which endpoint host. This is a
transparency surface, not a legal disclaimer — it reflects the actual gateway
behavior.

**Corrected in 26.19.** The standing disclosure copy described the `invoke` path only
and did not mention semantic search, so a user reading a sentence about "the selected
message" would not have expected a search to send message text to the same endpoint.
It now names both paths. In the same release the data-class ceiling became **enforced**
on the embed path rather than merely recorded on the audit row — before that, an
attachment-exclusion the audit reported as honoured was not applied to the text being
embedded, and an account outside the ceiling's allowlist was dispatched anyway. The
copy and the behaviour now agree; they did not.

One gap is named rather than left implied: `transcribe` still clamps its scope, uses
the result only to label the audit row, and does not refuse on an empty account set.
The payload there is user-supplied audio rather than mailbox content, so the allowlist
means something different and the fix needs a deliberate decision — but it is the same
shape as the defect fixed on the embed path.

## Audit is content-free

Every invocation writes an `assist_audit` row carrying the **capability**, a **scope
summary**, and the **endpoint host** — and **never the message content**. This is
asserted in tests: audit rows are checked to contain no mail content. If you need to
know that the AI assistant was used and against which endpoint, the audit tells you; it will not
leak what was summarized or drafted.

## Privacy summary

- Off until configured; zero AI assistant UI when disabled.
- E2EE-decrypted content and attachments are excluded by default; including them is an
  explicit opt-in.
- The browser never contacts the AI host — the server proxies.
- Content from zero-access accounts is never forwarded.
- Provider profiles reach only public addresses unless an admin grants a local `host:port`.
- The audit records capability + scope + host, never content.
- No AI assistant capability can send/delete/accept.
- `LocalProcess` keeps everything on-device.

## Admin governance

Admins control the AI assistant tenant-wide (§19):

- an endpoint allowlist (which endpoints users may configure),
- per-capability locks (force a capability off, or restrict which capabilities exist),
- data-class ceilings (which accounts/folders, E2EE/attachment inclusion),
- a **kill switch** that disables the AI assistant immediately.

Config is per-deployment and per-user, with admin locks taking precedence (migration
0008 `assist_config`).

## CI

The `assist-mock` job runs the gateway against a deterministic, offline
OpenAI-compatible (+ Anthropic) mock (`scripts/mock-assist/`, `docker-compose.ci.yml`
service `mock-assist`) so the scope/redaction/content-free-audit/no-send behavior is
tested reproducibly. A separate nightly, secret-gated `live-interop` job exercises real
Ollama/OpenAI/Anthropic endpoints when secrets are present; it is non-blocking and
never runs on pull requests.
