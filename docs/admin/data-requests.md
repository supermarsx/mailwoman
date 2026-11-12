# Data requests

People can ask what this server holds about them, ask for a copy, and ask for their data to be corrected, erased, restricted or no longer used. This page describes the controls the server provides for handling those requests. It does not say that using them meets any law; what a request must cover, who may make one and how quickly it must be answered is for your own legal advice.

A person files a request under **Settings > Privacy & data > Your data**. You work the queue under **Admin > Data requests** (needs the `data_export.run` capability, limited to the domains the grant covers). Every option below is a setting, set globally, per domain, or, where noted, per user. See [settings](settings.md) for the `dsr.*` keys; each has an admin default and lock.

## Request types

| Type | What it does here |
| --- | --- |
| Access | Builds a ZIP of the data the server holds about the person. |
| Download (portability) | The same ZIP; the two are separate types so they can be switched on and approved separately. |
| Correction | Recorded in the queue for an administrator to handle by hand. The page points the person to the places where they can edit their own details. |
| Erasure | Deletes the person's data after an approval step and a waiting period the person can cancel during. |
| Restriction | When approved, the person's accounts are restricted straight away (`restriction.autoApplyOnApproval`), or an administrator applies it by hand. The request stays in progress until an administrator closes it with a note. See [restrictions and consent](consent.md). |
| Objection, Other | Recorded in the queue for an administrator to handle by hand and close with a note. Nothing changes automatically. |

Each type has its own switch (`dsr.type.*`), and `dsr.enabled` turns the whole feature off.

## What happens to a request

1. **Filing.** The person picks a type. For access, download and erasure the server asks for a fresh password or second factor first (`dsr.reauth.required`). A person is limited to `dsr.rateLimit.perDay` requests a day and one open request per type.
2. **Confirmation.** With `dsr.confirmation.required` on, the server puts a message with a code in the person's own inbox, and the request waits until the code is entered (five wrong tries cancel it; an unconfirmed request is cancelled after seven days). A request filed by an administrator on the person's behalf skips this step.
3. **Identity check** (`dsr.identityVerification.required`). An administrator records how the identity was verified. Without a note the step cannot be recorded.
4. **Approval** (`dsr.approval.export`, `dsr.approval.erasure`, `dsr.approval.other`: `auto` or `admin`). With `auto` the request proceeds on its own.
5. **Carrying out.**
   * Access and download requests are queued, and a background worker builds the export.
   * An erasure is scheduled for `dsr.erasure.graceDays` after approval. The person can cancel until then. Setting 0 runs it at the worker's next pass.
   * Other types move to "in progress" until an administrator completes them with a note.
6. **Due date.** Each request is due `dsr.sla.days` after it was filed. An administrator can extend it with a recorded reason, up to `dsr.sla.maxExtensionDays` in total. The queue shows a countdown, flags requests due within the widest value of `dsr.sla.reminderDays`, and records "due soon" and "overdue" in the request's trail and as security events.
7. **Rejection and cancellation.** An administrator can reject with a reason; the person can cancel until the work has started. Both are kept in the trail.

Every change is a line in the request's trail (who, when, what). The person sees their own trail with administrator names replaced by "administrator". If `dsr.notify.admins` is on, new requests and state changes that need attention are also written to the audit log as security events (domain and request type only, not the address). With `dsr.notify.requester` on (a per-user setting with an admin default and lock), the person gets a message in their inbox when a request changes state. Local-inbox delivery remains available. With the separately configured compliance SMTP service, `dsr.notify.emailAdmins` emails `dsr.notify.operatorAddresses` about incoming requests and deadlines, while `dsr.notify.emailRequester` emails requester notices independently. These channels are off by default; see [Governance and email](compliance-governance.md).

## The export

The ZIP is built section by section and stored as it goes, so a restart continues where it stopped. The contents are switched per section (`dsr.export.include.*`):

* `profile.json` (always): the directory entry.
* Settings the person has set, identities and signatures, contacts (`contacts.vcf` and a JSON index), calendars (`.ics` per calendar), notes, filter scripts, sessions, remembered devices and sign-in history, cookie consent records, audit entries where the person is the actor or the target, and their data requests.
* Mail, only when `dsr.export.mailAllowed` is on and the person ticked it, as mbox or one `.eml` per message (`dsr.export.mailFormat`). Only messages in this server's own cache are included. When the server proxies to a mail provider, mail stays at the provider and is not part of the export; the manifest says so.
* `manifest.json` lists each section, its files, record counts and anything left out and why.

With `dsr.export.format` set to `jsonCsv`, list data is also written as CSV (cells that start with `=`, `+`, `-` or `@` are prefixed with `'`). `dsr.export.maxMegabytes` caps the total; a section that would exceed it is left out and listed in the manifest. `dsr.export.encrypt` seals the stored parts with the server key until the export is downloaded or expires.

Beyond those, sections built from the data map add mail state (pins, snoozes, tags, saved searches, trusted senders, read-receipt rules and status, remote-image grants, queued submissions), tasks, security metadata (key fingerprints and associations, API key prefixes, passkey labels, OAuth token metadata, password change history), integrations (external and linked mailboxes, bridges, webhooks), delegations granted and received, usage and quota figures, and records (consent ledger, restrictions, retention rules, tracking pixels and opens, uploads, plug-in data keys, DLP decisions). [Erasure and export coverage](erasure-coverage.md) lists every table and the section it goes to.

Secrets are not exported: no password, session token, second-factor secret, passkey key material or API key hash. Sealed items (authenticator secrets, recovery codes, stored passwords, sealed tokens, key material) appear in the export only as a marker saying the item is present and not exported.

### Download

When an export is ready the person asks for a download link (a fresh password is asked again when `dsr.reauth.required` is on). The link carries a random token that is stored only as a hash. It is valid for `dsr.download.linkMinutes`, only for the signed-in person, and, with `dsr.download.singleUse` on, only once. The export itself is deleted after `dataExport.linkHours`, along with its stored parts.

## Erasure

The erasure deletes the person's rows from the personal tables (second-factor and passkey records, push subscriptions, signatures, notification rules, sign-in events, remembered devices, masked addresses, API keys, filter scripts, identities, drafts, calendars, address books, notes, personal settings and saved searches, cookie consent records), the search index documents and message embeddings of the person's accounts, their sessions, and any native account rows (which removes the locally cached mail). It then closes the directory entry so the address cannot sign in again. The rows are found through the data map (`crates/mw-store/src/data_map.rs`), which also covers tables that are removed with a parent row (events, contacts, tasks, mail cache), delegations where the person is the delegate, linked mailboxes and the other tables listed in [erasure coverage](erasure-coverage.md). A second pass counts the rows every entry of the data map still finds, not only those the delete statements touch, so a table that is only mapped is verified too; if anything remains or any step failed, the request is retried and then marked failed for an administrator to rerun.

What is kept, and listed in the completion record:

* Audit log entries, for the audit retention period you set. With `scrub.autoRunOnErasure` on, the person's address and account ids in audit entries, alert and assistant history and the rotated log files are first replaced with a keyed pseudonym; the audit hash chain is re-sealed and an anchor record is written (see [erasure coverage](erasure-coverage.md#scrubbing-one-person-from-history)).
* The directory entry (address, domain, status, timestamps), so that the address is not enrolled again without an administrator.
* The request and its trail, for `dsr.retention.completedDays`.
* Mail held by an upstream mail provider. This server removes its link to it; it does not delete mail it does not hold.

Every completed erasure also writes a tombstone (a keyed hash of the address, no raw identifier). Backups record the tombstone watermark when they are made, and a restore replays the newer tombstones when `backup.propagateErasure` is on, so people erased after a backup are erased again after restoring it. The admin Erasure and scrubbing page marks backups that still contain people erased since.

The completion record also holds counts of rows removed per table and a digest of those counts, the request id and the time. The digest lets you check later that the record was not edited; it is not a signature.

### Legal holds

An address, or an `@domain` entry, listed in `dsr.legalHold.subjects` is under hold, and an administrator can also put a single request under hold. An erasure for a held person is not carried out: the request is shown as **blocked** with the source and reason of the hold, in the queue, the request details and the person's own list. When the hold is removed (the setting entry removed, or the request's own hold released) the request continues and the waiting period starts again.

## Requests on behalf of a person

With `dsr.onBehalf.allowed` on, an administrator can file a request for a person who has signed in to this server, for example one received by post. It is recorded as filed by that administrator, skips the confirmation message and follows the same approval steps.

## The queue

Filter by status, type, due state, assignee or search text; open a request to see its details and trail; export the filtered list as CSV. The cards show open requests, overdue and due-soon counts, the share answered by the due date and the average time to answer, and a chart of requests filed per day. Statistics cover all requests in the administrator's domains, not only the filtered ones.

An administrator's actions on a request: assign, record an identity check, approve, reject (reason needed), extend (reason needed), complete by hand (note needed), put under or release a hold, and run now (which also retries a failed request and skips an erasure's waiting period).

## Texts and contact details

`dsr.text.intro`, `dsr.text.confirmation` and `dsr.text.completion` take one text per language. The first is shown above the request form; the other two are added to the confirmation and completion messages. `dsr.contact.name`, `dsr.contact.email` and `dsr.contact.url` are shown to people on the same page. Fixed server messages and status names are available in English and Portuguese (Portugal); other languages fall back to English.

## Limits to know about

* Restriction, objection and correction are not applied automatically.
* Mail that stays at the upstream provider is neither exported nor deleted.
* Notices to people go into their inbox; there is no e-mail to an administrator or data protection contact.
* The worker runs every 30 seconds in each server process; several processes share the queue safely but a request is worked by whichever picks it up first.
* The queue lists at most the 5000 newest requests.
