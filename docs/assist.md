# Assist (AI) — endpoints, privacy, and governance (V7)

V7 (release 26.8.0) adds Assist (`mw-assist`): optional AI features — summarize, draft,
grammar, dictation, semantic search, auto-tag, recap, and an assistant chat. Assist is
**bring-your-own-endpoint**: you point it at an AI endpoint you control or trust;
Mailwoman ships no built-in model and no default provider.

Assist is **off until configured.** With no endpoint configured, the gateway reports
`Disabled` and the web app hides every Assist surface — there is no Assist UI, no
prompts, nothing sent anywhere.

## Architecture: the engine is the only client

The browser **never** talks to an AI endpoint directly. All Assist traffic goes
through the engine-side **Assist gateway** (`mw-assist`), which the server proxies. The
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

## Capabilities and scoping

Each Assist feature is a capability, granted independently:

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
> embedding cache fills. A chat turn costs 1. `0` refuses every Assist request, which
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

**No Assist capability transmits, deletes, or accepts anything.** This is structural:
the capability enum has no send/delete/accept variant, so there is no code path for
Assist to send mail or act irreversibly. A drafted message goes to the Outbox for a
human to send. Assist adds no privileged path.

**One correction to how this used to be described.** The assistant chat is *not* yet
driven by the MCP tool surface. What ships is proposal **reporting**: the tool name in
a proposed action comes from the model and is **not validated against the `mw-mcp`
registry**. Because nothing is ever executed and every action is human-confirmed, the
consequence is that a proposal could display a tool name that does not correspond to
any real tool — a display-accuracy limit, not a privilege one. The send-gating above
is structural and unaffected. Do not describe the assistant as MCP-backed tool calling
until it is wired.

## What left the device (disclosure)

Assist shows a per-message **"what left the device"** disclosure so the user can see,
for each Assist action, what was sent and to which endpoint host. This is a
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
know that Assist was used and against which endpoint, the audit tells you; it will not
leak what was summarized or drafted.

## Privacy summary

- Off until configured; zero Assist UI when disabled.
- E2EE-decrypted content and attachments are excluded by default; including them is an
  explicit opt-in.
- The browser never contacts the AI host — the server proxies.
- The audit records capability + scope + host, never content.
- No Assist capability can send/delete/accept.
- `LocalProcess` keeps everything on-device.

## Admin governance

Admins control Assist tenant-wide (§19):

- an endpoint allowlist (which endpoints users may configure),
- per-capability locks (force a capability off, or restrict which capabilities exist),
- data-class ceilings (which accounts/folders, E2EE/attachment inclusion),
- a **kill switch** that disables Assist immediately.

Config is per-deployment and per-user, with admin locks taking precedence (migration
0008 `assist_config`).

## CI

The `assist-mock` job runs the gateway against a deterministic, offline
OpenAI-compatible (+ Anthropic) mock (`scripts/mock-assist/`, `docker-compose.ci.yml`
service `mock-assist`) so the scope/redaction/content-free-audit/no-send behavior is
tested reproducibly. A separate nightly, secret-gated `live-interop` job exercises real
Ollama/OpenAI/Anthropic endpoints when secrets are present; it is non-blocking and
never runs on pull requests.
