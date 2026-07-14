# Bridge PIM through the plugin seam — WIT `plugin-pim` world (SPEC §6.5/§22)

> **Status:** scaffold (t10-e0). Host probing filled by t10-e1; bridge guests by
> t10-e2/e3/e4; engine routing by t10-e5; mounted by t10-e13.

The Graph / EWS / Gmail bridges already implement calendar, tasks, reactions,
voting, recall, and Focused-Inbox sync as pure functions. t10 **wires** those to the
plugin seam via an **additive, second-world** WIT extension — the bridges are not
rewritten.

## Why a second world (`plugin-pim`), not `@0.2.0` (the §5 fallback, taken)

The plan's first choice (§2.1) was to add the PIM exports to `world plugin` and bump
the package `0.1.0 → 0.2.0`. **That breaks backward-compat at the wasmtime linker:**
bumping the package renames every interface id to `…@0.2.0`, and wasmtime's
semver-aware component linker treats `0.1.x → 0.2.0` as INCOMPATIBLE — so the
committed `@0.1.0` bridge / LanguageTool / Nextcloud `.wasm` fixtures (which import
`mailwoman:plugin/host@0.1.0`) fail to link against a host that now provides
`host@0.2.0`. t10-e0 proved this (the `bridge-ews` jail-load tests broke) and took the
plan's **§5 fallback**: keep the package at `@0.1.0` and `world plugin` byte-unchanged,
add the three PIM interfaces, and add a **second world `plugin-pim`** that `include`s
`world plugin` and additionally exports the three. Committed fixtures keep loading;
PIM-capable guests target `plugin-pim`.

## Additive, backward-compatible

`types` + `account-backend` (and every other interface) are **byte-unchanged**, and
so is `world plugin`. The extension only **adds three OPTIONAL guest interfaces** and
exports them from the new `world plugin-pim`:

- `interface calendar` — `supports-calendar()`, `list-calendars`, `sync-events`,
  `find-rooms`, `get-schedule`. Records: `cal-info`, `room-info`, `event-info`,
  `event-delta`. iCalendar (RFC 5545) text crosses the seam.
- `interface tasks` — `supports-tasks()`, `list-tasks`, `sync-tasks`, `complete`.
- `interface bridge-parity` — `supports-{reactions,voting,recall,focused}()`;
  `set-reaction`/`get-reactions`, `cast-vote`/`tally`,
  `recall -> recall-outcome{requested|unsupported|failed}` (mirrors
  `mw_engine::v7::RecallOutcome`), `get-focused`/`set-focused`.

## The "host probes optional exports" contract (frozen — plan §5)

The host (`mw-plugin`, e1) **enumerates a component's exported interfaces at load and
binds only the ones present**. A shipped 0.1.0 component (LanguageTool / Nextcloud)
exports none of the three, so the host binds no PIM adapter and the account advertises
no PIM caps. A guest that targets @0.2.0 but does not support an interface exports a
trivial `supports-*() -> false` stub (its data funcs may return `unsupported`); the
host treats "interface absent" and "`supports-* == false`" identically.

`types.backend-caps` is **not** modified: calendar/tasks presence is advertised by the
new `supports-*` funcs; the coarse reactions/voting/recall/focused-sync bools already
in `backend-caps` stay the account-level advertisement.

Fallback if wasmtime resists optional-export probing: a **second world `plugin-pim`**
(documented, not silent) — see plan §5.

## Engine routing (byte-unchanged fallback — plan §2.2)

`mw-engine` gains symmetric `BridgeCalendar` / `BridgeTasks` traits alongside the
existing `BridgeReactions` / `BridgeVoting` / `BridgeRecall` / `BridgeFocusedSync`, and
`Engine::bridge_calendar` / `bridge_tasks` accessors (via
`BridgeCapabilitySource::calendar` / `::tasks`). When a bridge advertises a capability
the engine routes to it; **absence ⇒ the existing CalDAV/standards fallback runs
byte-for-byte unchanged** (the hard regression gate). e1 backs the trait objects with
through-the-jail WIT calls; e5 wires the routing preference in `mw-engine/src/pim/**`.

<!-- e1: probe-and-bind loader + PIM adapters. e2/e3/e4: wire each bridge guest.rs.
e5: engine routing + the byte-unchanged-fallback proof. -->
