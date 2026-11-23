---
screen: true
---
# Aggregate and adaptive protection

Request protection decides whether the server answers a request at all, before the request reaches sign-in, the mailbox or the admin console. It has four parts: an immediate block for requests to recognized scanner paths, request budgets per address, mailbox and administrator, adaptive protection that reacts to repeated unusual requests, and rules for the user-agent a client claims. Only the scanner-path rule and the user-agent rules are on by default. This page describes the console tab that configures them and what each part does to a request.

![The Aggregate and adaptive protection tab of the Security centre: a notice that country features have no country database, the seven sub-tabs, and the Current policy card showing request budgets and adaptive protection both off.](/screenshots/admin-access/request-protection-overview.webp){.mw-shot}

The notice at the top appears on every Security centre tab while features that need a country are on and no [country database](./country-database.md) is installed.

## Before you start

- The tab is in **Security centre** > **Aggregate and adaptive protection**, at `<console path>#/security/protection`. You need a role that can read IP bans to see it, and one that can change settings to edit its rules.
- The client address every rule uses is the address the server accepts as the client. Behind a reverse proxy, set `MW_TRUSTED_PROXIES` first (see [Reverse proxy](../deploy/reverse-proxy.md)); otherwise every request appears to come from the proxy, and a block on the proxy's address refuses everyone.

## The sub-tabs

![The request protection sub-tabs Overview, Aggregate, Adaptive, Protocol guard, Probing, Object state and Decisions, numbered 1 to 7.](/screenshots/admin-access/request-protection-tabs.webp){.mw-crop}

1. **Overview**: the current policy, live counters, the summary **What happens today** and the presets.
2. **Aggregate**: the request budgets and the user-agent rules (**Browsers and crawlers**).
3. **Adaptive**: unusual-request protection, including the **Recognized scan paths** rule, the actions and the exceptions, and the simulator **What would happen?**.
4. **Protocol guard**: checks on malformed HTTP requests. See [HTTP protocol guard](./protocol-guard.md).
5. **Probing**: probing protection, which slows and then blocks a source that keeps looking for the admin console.
6. **Object state**: the subjects (addresses, mailboxes, administrators) that currently have counters or an action.
7. **Decisions**: the recent throttle and block decisions.
{.mw-callouts}

Each sub-tab has its own address, for example `#/security/protection/adaptive`.

## Console addresses and scanner paths

Every console screen is an address after a `#`: `https://mail.example.org/<console path>#/security/protection`. The part after `#` is never sent to the server. An address without the `#`, such as `<console path>/users`, is not a screen: it is either a console API route, which answers with data or `401`, or an unknown path, which answers with a "page does not exist" page. Bookmark the `#` form.

The **Recognized scan paths** rule ([`security.protection.probePathMode`](/admin/settings#security.protection.probePathMode)) is **Enforce** by default. A request whose path is in the built-in list of scanner targets (paths of other web applications, administration panels and sensitive files, such as `/wp-admin`, `/.env`, `/.git/config` or `/admin/users`) blocks its source address on the first request:

- The first response is `429` with the JSON code `requestProtectionLimited`, `retryAfterSecs` and a `Retry-After` header.
- For the next [`security.protection.blockSecs`](/admin/settings#security.protection.blockSecs) seconds (300 by default) every request from that address is refused the same way: the mail app, sign-in and the admin console included, even for an administrator who is already signed in. Static assets, `/healthz`, `/readyz`, sign-out and `/api/public-config` are still answered.
- The path of your own console is excluded. Below it, only foreign scanner paths count: `<console path>/users` is not a scanner path on its own server, but `/admin/users` on a server whose console is somewhere else is. A path that ends in `/console` is on the list too, so `<console path>/security/console` (the Console address tab typed without `#`) blocks.
- Addresses in **Exempt addresses** ([`security.protection.exemptClients`](/admin/settings#security.protection.exemptClients)) are never blocked by this rule.
- The rule works on its own: it does not need request budgets or adaptive protection, and **Observe** records the decision without refusing. Some explanatory text on this tab says the block happens "with adaptive client protection enforcing"; in fact it applies whatever **Unusual request protection** is set to.
- The block is not written to the server log as a fail2ban line. Only probing protection (below) writes one.

The list and how paths are matched are described in [Scanner-path catalog](./scan-path-catalog.md).

![The Recognized scan paths rule on the Adaptive sub-tab, with its setting key security.protection.probePathMode, its explanation and the value Enforce.](/screenshots/admin-access/request-protection-scan-paths.webp){.mw-crop}

::: warning
A mistyped address can lock you out of the console for five minutes. If your own address is blocked, wait for the time in `retryAfterSecs`, or open the console from another address and clear the block on **Object state** (see below). Add office or VPN networks to **Exempt addresses** if staff share one public address.
:::

## Overview

The **Current policy** card states whether request budgets and adaptive protection are on, and the storage limits: at most 20,000 subject records and 10,000 decisions, decisions kept for 30 days, and changes reaching other servers of the same deployment within 30 seconds. Below it are the protocol guard counters, request counts for the chosen **Range**, a summary of probing protection and the session-watch card.

**What happens today** restates the rules in force in one list:

![The What happens today card: request budget off, recognized scan paths enforced, unusual request protection off, and the slow-down, block and clean-record durations.](/screenshots/admin-access/request-protection-summary.webp){.mw-crop}

### Presets

![The Presets card with the choices Relaxed, Balanced and Strict.](/screenshots/admin-access/request-protection-presets.webp){.mw-crop}

Choosing a preset lists the rules it would change, with their current and new values, before you apply it. A preset never changes the exemption lists or **If the counters cannot be read**.

| Rule | Relaxed | Balanced | Strict |
|---|---|---|---|
| **Request budget** | Observe | Enforce | Enforce |
| **Unusual request protection** | Observe | Enforce | Enforce |
| **Recognized scan paths** | Observe | Enforce | Enforce |
| **Requests per address (not signed in)** | 1200 | 600 | 300 |
| **Requests per mailbox** | 3600 | 1800 | 900 |
| **Requests per administrator** | 2400 | 1200 | 600 |
| **Unusual requests in the short window** | 40 | 20 | 10 |
| **Share of unusual requests, short window** | 70 % | 50 % | 30 % |
| **Unusual requests in the long window** | 200 | 100 | 50 |
| **Share of unusual requests, long window** | 40 % | 25 % | 15 % |
| **Slow-down** | 3 s | 3 s | 10 s |
| **Block** | 120 s | 300 s | 1800 s |

The server's defaults are those of **Balanced** except for the three modes: request budgets and unusual-request protection are **Off** until you change them.

## Request budgets

On **Aggregate**, the **Request budget** group limits how many requests one subject makes in a fixed window.

![The Request budget group on the Aggregate sub-tab: the Request budget switch set to Off, the requests per address, per mailbox and per administrator, and the budget window, numbered 1 to 5.](/screenshots/admin-access/request-protection-budget.webp){.mw-crop}

1. **Request budget** ([`security.protection.rateMode`](/admin/settings#security.protection.rateMode)): **Off** (default), **Observe** (records when a budget runs out, refuses nothing) or **Enforce** (answers `429`).
2. **Requests per address (not signed in)**: 600 per window. Applies to requests without a valid session, and to every sign-in, second-factor, password and step-up request whatever its session.
3. **Requests per mailbox**: 1800 per window, counted across the mailbox's sessions, devices and addresses.
4. **Requests per administrator**: 1200 per window.
5. **Budget window**: 60 seconds.
{.mw-callouts}

A request with a valid mailbox or console session is charged to that mailbox or administrator, not to its address, so people behind one shared address do not share one budget. Zero turns a dimension off. A refusal is `429` `requestProtectionLimited` with `Retry-After`.

The **Browsers and crawlers** group on the same sub-tab is described in [Crawler identification](./crawler-identification.md).

## Unusual requests

On **Adaptive**, **Unusual request protection** ([`security.protection.adaptiveMode`](/admin/settings#security.protection.adaptiveMode), **Off** by default) counts typed unusual requests per subject and compares them with that subject's normal requests in two rolling windows.

| Rule | Default |
|---|---|
| **What counts as unusual** | malformed requests, oversized requests, rejected credentials. Also selectable: permission denied, body deadline, admin route miss |
| **Count per client**, **Count per mailbox**, **Count per administrator** | all on |
| **Short window** | 300 s; triggers at 20 unusual requests, or at 50 % unusual after 30 requests |
| **Long window** | 7 days; triggers at 100 unusual requests, or at 25 % unusual after 100 requests |
| **Limit on normal requests counted** | 1000 per window bucket |

Signals are set by the code that recognizes the problem, not derived from status codes, so a `429` refusal is never counted against the client that received it. Findings of the protocol guard always count, whatever **What counts as unusual** says.

![The What happens group on the Adaptive sub-tab: Slow-down 3 seconds, Block 5 minutes and Clean record after 1 day.](/screenshots/admin-access/request-protection-actions.webp){.mw-crop}

The first time a subject crosses a threshold it is slowed down for **Slow-down** (3 s). A later crossing before the record is clean blocks it for **Block** (300 s). After **Clean record after** (1 day) without unusual requests, the next crossing starts again with a slow-down. The deadlines are fixed: requests during a slow-down or block do not extend it. **Block** is also the length of a scanner-path block.

**Exceptions and failures**:

- **Exempt addresses**, **Exempt mailboxes**, **Exempt administrators**: each exempts only its own dimension. An exempt address still has its mailbox budget, and the other way round. Mailboxes and administrators are listed by their server account id, not by an email address.
- **If the counters cannot be read** (`security.protection.storageFailure`): **closed** (default) refuses a request that an enforcing rule cannot check, with `503`; **open** lets it through and logs a warning.

**What would happen?** runs a request description (subject, signal, counts, optionally a draft of your changes) through the server's own rules and shows the action it would take, without changing anything.

## Probing

**Probing** configures probing protection, a separate counter for sources that look for the admin console. It is **On** by default.

![The Probing protection card: its explanation, the warning that a block also refuses the mail app from that address, and the Off and On buttons with On selected.](/screenshots/admin-access/request-protection-probing.webp){.mw-crop}

Four kinds of miss count, one point each, per client address (IPv6 per /64): **A known scanner path** (a path on the **Probe paths** list outside the console), **A guess at the console address** (a `404` at the console path's depth), **A failed admin sign-in** and **An admin request without a session**. The first 10 misses in 10 minutes are forgiven; the score drains at that rate. After that each miss is answered with a 1.5 second delay, and 10 more misses block the source for 15 minutes. A block that starts within a day of the previous one doubles in length, up to one day. A blocked source gets `429` (or `403`, **Blocked answer status**) with `Retry-After` for every path. A successful admin sign-in clears its source.

| Setting | Key | Default |
|---|---|---|
| **Probing protection** | `security.adminProbe.enabled` | on |
| **Grace allowance (misses per window)** | `security.adminProbe.graceAllowance` | 10 |
| **Grace window (seconds)** | `security.adminProbe.windowSecs` | 600 |
| **Misses beyond grace before a block** | `security.adminProbe.blockAfter` | 10 |
| **Tarpit delay (ms)** | `security.adminProbe.tarpitDelayMs` | 1500 |
| **First block (seconds)** | `security.adminProbe.blockSecs` | 900 |
| **Longest block (seconds)** | `security.adminProbe.maxBlockSecs` | 86400 |
| **Repeat window (seconds)** | `security.adminProbe.repeatWindowSecs` | 86400 |
| **Blocked answer status** | `security.adminProbe.blockStatus` | 429 |
| **Never-blocked networks** | `security.adminProbe.allowlist` | empty |
| **Source address in the audit log** | `security.adminProbe.auditIp` | masked (IPv4 /24, IPv6 /48) |
| **fail2ban log line on block** | `security.adminProbe.fail2banLog` | on |

Probing state is kept in each server process; active blocks are saved and read back at start. With several servers behind one balancer, each keeps its own scores. The **Countries and trust tiers** group needs a [country database](./country-database.md). The fail2ban line is described in [fail2ban](../deploy/fail2ban.md).

## Object state and Decisions

**Object state** lists each subject with counters or an action: the subject kind and a shortened fingerprint, the last source address, the country, unusual / normal counts in the short and long windows, and the action deadline. **Decisions** lists recent slow-downs and blocks with the reason in words, the action, whether it was enforced or only observed, and the evidence. Both are newest first, paged.

To lift a block early, use **Clear state** on the subject's row in **Object state** and confirm your identity. This removes the subject's counters and action; it does not change the policy or remove manual IP bans. Clearing needs a role that can manage IP bans.

## Other limits

These answer `429` too, but are separate from this tab:

- **Sign-in**: a 3-second pause after a failed attempt ([`signin.retryCooldownSecs`](/admin/settings#signin.retryCooldownSecs)), shared by webmail and the console, and [temporary sign-in locks](./signin-locks.md).
- **JMAP requests**: at most 8 at once per mailbox (`ratelimit.jmap.perAccountConcurrent`) and 256 per server (`ratelimit.jmap.globalConcurrent`); a request waits up to `ratelimit.queueMs` (3000 ms) for a slot before it is refused.
- **Manual and automatic IP bans**: see [IP bans](./ip-bans.md).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Every page, including the console, answers `429` with `requestProtectionLimited` for five minutes | The address requested a recognized scanner path, for example a console address typed without `#` on a server whose console is not at that path | Wait for `retryAfterSecs`, or clear the subject on **Object state** from another address. Use the `#` form of console addresses. |
| All users are blocked at once | The server sees every request as coming from the reverse proxy | Set `MW_TRUSTED_PROXIES` to the proxy's address. |
| A crawler or script gets `403` `automatedClientBlocked` | Its user-agent claims an AI crawler class set to block | See [Crawler identification](./crawler-identification.md). |

## Related

- [Scanner-path catalog](./scan-path-catalog.md)
- [HTTP protocol guard](./protocol-guard.md)
- [Crawler identification](./crawler-identification.md)
- [fail2ban](../deploy/fail2ban.md)
- [Security centre](./security-center.md)
