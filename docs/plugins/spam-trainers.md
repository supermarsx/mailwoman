# Spam-trainer plugins: Rspamd + SpamAssassin (SPEC §10.8)

> **Status:** scaffold (t10-e0). Filled by t10-e6; live-E2E by t10-e14.

Two thin `spam-action` WASM components that classify and train ham/spam by talking
to an existing spam engine **over the network via the host `http-fetch` import**,
under a net allowlist. There is **no C linkage** — rspamd / SpamAssassin are network
services, not linked libraries — so the permissive license floor is unchanged.

| Plugin | Crate | Service | Endpoint |
| --- | --- | --- | --- |
| Rspamd | `plugins/spam-rspamd` | rspamd worker-controller | `POST /checkv2`, `/learnham`, `/learnspam` |
| SpamAssassin | `plugins/spam-spamassassin` | spamd (via HTTP shim) | CHECK / REPORT / learn |

## Hook

Both export `spam-action::classify(raw) -> verdict`. The verdict maps to the engine's
spam action (no action / add header / reject). The rest of the `plugin` world is
stubbed; the @0.2.0 PIM/parity interfaces advertise `false`.

> ⚠️ **Correction (26.19) — training does not exist.** This document said
> "training feeds the ham/spam learn endpoints" and the table above lists
> `/learnham` and `/learnspam` as if they were wired. **`SpamHook` has no
> train or learn method**, and grepping `learnham`/`learnspam`/`train` across
> `plugins/**/*.rs` returns zero hits, so nothing in the tree can call either
> endpoint. The trailing TODO in this file still asks for exactly that work,
> which is the tell.
>
> Read the endpoint column as *the protocol these plugins would speak once
> training is built*, not as a description of what they do. Classification is
> real; training is not. The user-visible consequence is described in SPEC
> §10.8: the Junk button relocates a message and teaches nothing, and there is
> no Not-junk button to undo a false positive.

## Jail posture

- Granted only `spam-action` + `net:host-allowlist` (the controller host).
- A request to any host outside the allowlist ⇒ `capability-denied` (proven by e14).
- Resource limits + epoch preemption apply as to every plugin.

## CI services

`docker-compose.ci.yml` carries commented `rspamd` + `spamassassin` services
(activated by e14). Both are network-only (Apache-2.0, mere aggregation), out of the
cargo-deny scope.

<!-- e6: fill the rspamd controller protocol (password header, /checkv2 JSON parse),
the spamd HTTP-shim protocol, and the train ham/spam paths. -->
