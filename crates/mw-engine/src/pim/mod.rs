//! The engine's PIM surface (plan §0, §1.1, §2.1/§2.2): the Mailwoman-native
//! calendar / tasks / notes / contacts method families, dispatched over the same
//! `handle_jmap` envelope the mail surface uses (result references, per-account
//! state, cookie auth, the WS/SSE push channel) but under Mailwoman capability
//! URNs (`urn:mailwoman:{calendars,tasks,notes,contacts}`).
//!
//! ## Scaffolder note (e0)
//! e0 freezes the [`types`] (the §2.1 object shapes) + the method-dispatch arms
//! (`todo!()` bodies) wired into `handle_jmap` via [`Engine::dispatch_pim`], and
//! the `session_json` capability-URN additions. **e8** fills every `todo!()`:
//! the full method families, the CalDAV/CardDAV sync orchestration (driving
//! `mw-dav`/`mw-carddav`), RRULE expansion + `event_instances` materialization
//! (`mw-ics`), free/busy, iTIP/iMIP, sealed-at-rest notes, contact merge +
//! autocomplete, and the PIM `pim_changes` state tokens. No logic yet.

pub mod dispatch;
pub mod types;
