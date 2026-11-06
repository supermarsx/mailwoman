# Restrictions, consent and the Your data section

This page describes three controls that work together: a restriction of processing for one person (the Art.18 request type), a ledger of consent decisions per purpose, and the overview a person sees under **Settings > Privacy & data > What we hold about you**. They are configuration. Which legal basis is right for a purpose, and whether a setup meets a given law, is for you and your own legal advice to decide.

Admin screens: **Admin > Compliance > Restrictions** and **Admin > Compliance > Consents**. Settings: `restriction.*` and `consent.*` (see [settings](settings.md)); every key has an admin default and a lock and can be set per domain.

## Restriction of processing

A restriction pauses chosen kinds of processing ("scopes") for one account while the data stays stored. It is kept in the table `account_restriction`, one row per restriction, with the history of lifted and expired ones.

| Scope | What stops | Where it is enforced |
| --- | --- | --- |
| `indexing` | Adding new mail to the search index; semantic re-ranking and message embeddings; a full index rebuild skips the account | `mw-engine`: ingest, batched re-index, single re-index, rebuild, search |
| `assistant` | Every AI assistant route and the MCP surface | `require_feature` for `ai_assistant` and `mcp` (REST and JMAP) |
| `analytics` | The analytics counts and browser tag, open tracking of the person's sent mail (mint, bind and recording a load) | analytics routes and tag loader; the single open-tracking policy gate |
| `classification` | Local Sieve rules, mail rules and the spam verdict at ingest; mail is stored where it arrives | `mw-engine` ingest |
| `push` | Web push, UnifiedPush and APNs wakes | the push dispatcher and the wake policy |
| `thirdParty` | Nextcloud and other integrations, UI plugins, package tracking: anything that sends the person's data to another service | `require_feature` for `integrations.*`, `plugins.ui`, `packages` |

What continues: delivery to the mailbox, reading, deleting, and anything the person does by hand (search with the stored data, export, composing and sending). The person sees a banner in the webmail and a notice under Settings > Privacy & data while a restriction applies.

**Mail delivery.** `restriction.mailDelivery` (`keep`, `queue`, `reject`) records what you chose for mail that arrives during a restriction. This server does not receive mail itself: it works against your mail provider, which keeps delivering. The value is stored with the restriction and shown to the person, and your mail server is the one that has to act on `queue` or `reject`. Nothing here changes how the provider delivers.

**Applying.** Three ways:

* An administrator applies one under Admin > Compliance > Restrictions: address, scopes (all by default, from `restriction.scope.*`), a required reason and an optional end date (`restriction.defaultExpiryDays`; 0 means until lifted).
* A restriction request (Settings > Your data > Restrict use of my data, or the request form) is applied automatically when it is approved if `restriction.autoApplyOnApproval` is on. The request stays "in progress" until an administrator completes it with a note. With the setting off, the administrator applies the restriction by hand.
* A restriction can end by itself at its end date.

**Lifting.** An administrator lifts it under Restrictions with a required reason. With `restriction.notifyUser` on, the person gets a message in their inbox when a restriction is applied and when it is lifted. Applying and lifting are written to the audit log (`restriction-applied`, `restriction-lifted`), and a restriction that came from a request is noted in that request's trail. After a lift, new mail is indexed again; run the search index rebuild under Admin > Maintenance to bring older mail back into the index.

`restriction.enabled` switches the feature off for a domain. Restrictions follow the person: all accounts bound to their address are restricted together. A restriction row is erased with the person.

## Consent by purpose

Each purpose has:

* a **legal basis** you choose per domain (`consent.<purpose>.basis`: consent, legitimate interest, contract, legal obligation), shown to the person;
* a **text version** (`consent.<purpose>.version`). Raising it asks everyone again; a yes given for an older version stops counting;
* optional **own wording** (`consent.<purpose>.text`, per language) instead of the built-in description.

| Purpose | Default basis | How it is enforced |
| --- | --- | --- |
| `analytics` (usage counts) | consent | The existing analytics preference; the ledger holds the receipts. |
| `analyticsTag` (provider script) | consent | The existing browser-tag consent; the ledger holds the receipts. |
| `assistant` (and each feature) | contract | Ledger, checked on every assistant call. |
| `push` | contract | Ledger, checked before every wake. |
| `openTracking` | contract | Ledger, checked at the open-tracking gate. Sending with tracking also needs `cap.compose.open_tracking` and the sender's own `mail.openTracking`. |
| `integrations` | contract | Ledger, checked with the feature gate for connected services. |
| `remoteContent` | consent | Listed for the person; the switch is the existing remote-images setting. |
| `telemetry` | legitimate interest | Listed for the person; the switch is the existing load-time measurement setting. |

The defaults are starting values, picked so that current behaviour does not change when you upgrade; review each one. Marketing e-mail is not a purpose because the server sends none.

**How the ledger answers.** A purpose whose basis is **consent** is allowed only after a "granted" decision for the current version, so it is off by default. For any other basis the purpose is allowed unless the person withdrew it. Withdrawing is one call with no precondition and stops the processing on the next use. Decisions are rows in `consent_record` (append-only): purpose, decision, text version, basis shown, source (`settings`), time and the account id; no address, IP or user agent. `consent.receipts.retentionDays` bounds how long they are kept (0 keeps them until the person is erased). The person sees their receipts under their consents, and the ledger is part of their data export and is erased with them.

Cookie and browser-storage consent (Admin > Compliance > Cookies) is a separate log (`cookie_consent`) and is unchanged.

**Automated processing notice.** With `consent.art22.notice` on, the person sees a plain-language note on how the assistant and search features work, their significance and the human-review path (a data request), with a switch per assistant feature (summaries, drafting, grammar, dictation, semantic search, tag suggestions, recaps, chat). A switch-off holds whatever the basis. The text describes the features as built: they make suggestions, and none sends, deletes or files mail on its own. If you change what a feature does, change the version of its purpose.

**Admin overview.** Admin > Compliance > Consents shows, per purpose and per assistant feature, how many people currently have a yes or a withdrawal, the withdrawal rate, a graph of grants and withdrawals per day, and a CSV export of the ledger. The settings for each purpose are edited in place below.

## What the person sees: What we hold about you

`GET /api/privacy/overview` returns, for the signed-in person:

* stored data by category (mail copies and message index, contacts, calendars, notes, rules, sessions and devices, sign-in history, audit entries about them, assistant and search data, push subscriptions, open tracking, app keys, consent records, data requests) with counts and approximate sizes (sizes are `0` where the store does not track bytes);
* the retention that applies, from the `retention.*` settings (for example `retention.ipAddresses.days` for sign-in history). Per-user retention rules are read through one function, `user_rule_days` in `privacy_overview.rs`, which returns nothing until that source exists;
* who handles each category (your mail provider's server, the assistant provider, the push service), listed only when something is stored and the purpose allows it;
* their consents, restriction status, and shortcuts to download their data, restrict use or delete their account, which pre-select the request type in the request form below;
* links to the privacy policy (`/privacy`) and the processing records (`/api/privacy/processing`). These are published by the compliance policy feature; the page links to them without depending on them.

Sessions, remembered devices and app keys are listed with revoke buttons that use the existing session and key routes. Tracking protection and remote content stay in the Privacy section above.

## Limits

* Counts and sizes are what this server stores. Mail held by your mail provider is not counted.
* A restriction pauses processing in this server. It does not reach your mail provider, other systems of yours or backups.
* The restriction of `indexing` leaves already indexed mail in the index.
