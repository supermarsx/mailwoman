---
screen: true
---
# Data requests

People can ask what this server holds about them, ask for a copy, and ask for their data to be corrected, erased or restricted, or object to a use of it. The **Data requests** tab of **Compliance** is the queue where you handle those requests. This page describes what the server does with each request. It does not say that handling requests this way meets any law: what a request must cover, who may make one and how quickly it must be answered is for your own legal advice.

![The Data requests tab scrolled to the filters and the queue, with the status filter set to Any: three requests from alex@example.com, an access request, a correction and an erasure.](/screenshots/admin-privacy/dsr-queue.webp){.mw-shot}

## Before you start

- You need the `data_export.run` capability. An administrator limited to some domains sees and handles only the requests of those domains.
- People file requests themselves under **Settings > Privacy > Data requests** in the mail app (see [Account settings](/user/settings/account#data-requests)). You can also file one for them (see [File a request for someone](#file-a-request-for-someone)).
- Which request types are offered, who approves them and what an export contains are settings (see [Options](#options)). Most of them can be set per domain.

## Request types

| Type (as the person sees it) | What the server does |
| --- | --- |
| **Access to my data** | Builds a ZIP file of the data the server holds about the person (see [What an export contains](#what-an-export-contains)). |
| **Download my data** | Builds the same ZIP file. The two types exist so they can be offered and approved separately. |
| **Correction** | Nothing automatic. The request waits for you to make the correction by hand and close it with **Mark complete**. |
| **Erasure** | After approval and a waiting period, deletes the person's data and closes their account (see [What erasure removes](#what-erasure-removes-and-what-it-leaves)). |
| **Restriction** | On approval, applies a restriction of processing to the person's accounts when **Restrict when a request is approved** is on (the default), using the default paused kinds. See [Restrictions and consent](consent.md). |
| **Objection** | On approval, records a withdrawal of consent for each purpose the person named. See [Restrictions and consent](consent.md#consents-by-purpose). |
| **Other request** | Nothing automatic. You handle it by hand and close it with a note. |

## What happens to a request

1. **Filing.** For access, download and erasure, the person confirms their password or second factor first (**Re-authentication for requests**). Each person can file at most **Requests per person per day** requests in 24 hours and one open request per type.
2. **Confirmation.** With **Confirmation message** on, the server puts a message with a code in the person's own mailbox and the request shows **Waiting for your confirmation** until the code is entered. Five wrong codes cancel the request, and an unconfirmed request is cancelled after seven days.
3. **Identity check.** With **Identity verification step** on, the request shows **Waiting for identity check** until an administrator records how the identity was checked, with a note. **Record identity check** is also offered while a request waits for the person's confirmation code; using it skips the code.
4. **Approval.** Each approval setting is **auto** or **admin**. With **admin**, the request shows **Waiting for approval** until someone uses **Approve**. With **Confirm again before approving** on (the default), approving an erasure, and **Run now** on an erasure, ask for your password again.
5. **Carrying out.** Access and download requests are queued and a background worker builds the export. An erasure is scheduled for **Erasure grace period** days after approval (14 by default); 0 runs it at the worker's next pass. The person can cancel an erasure while it waits. Correction, objection and other requests move to **In progress** until you use **Mark complete**.
6. **Due date.** A request is due **Days to answer a request** days after it was filed (30 by default). The server counts whole days; it does not use calendar months. **Extend due date** adds days, up to **Longest extension** in total, and needs a reason.

The background worker runs every 30 seconds. Every change is added to the request's trail with who made it and when. The person sees the same trail, with administrator names replaced by "administrator".

## Work through the queue

1. Open **Compliance** and select the **Data requests** tab. The **Queue** tiles count open and overdue requests and those due within 7 days. The 7 days are fixed; they do not follow **Due date flags**.
2. Narrow the list with the **Filters**: **Status**, **Type**, **Due**, **Assigned to** and **Search by address or id**. **Export log (CSV)** downloads the requests that match.

   ![The filter bar of the Data requests tab and the queue below it, with markers 1 to 3.](/screenshots/admin-privacy/dsr-filters.webp){.mw-crop}

   1. **Status**, set to **Open (not finished)** when the tab opens.
   2. **Export log (CSV)**.
   3. The queue: person, type, status, the days left to the due date, and who the request is assigned to.
   {.mw-callouts}

3. Select **Open** on a row. The **Request details** panel shows who filed it, the trail, and the actions the request's status allows.

   ![The Request details panel of an erasure request waiting for approval, with markers 1 to 3.](/screenshots/admin-privacy/dsr-detail.webp){.mw-crop}

   1. **Note or reason**. **Reject**, **Extend due date**, **Mark complete**, **Record identity check** and **Put under legal hold** need one.
   2. The actions for this request.
   3. **Trail**: every step of the request so far.
   {.mw-callouts}

4. Choose the action. Select several rows to use **Bulk actions** on up to 200 requests at once; a request the action does not apply to is refused and stays selected.

| Action | Shown when | What it does |
| --- | --- | --- |
| **Record identity check** | waiting for identity check or for the confirmation code | Records the note and moves the request on. |
| **Approve** | waiting for approval | Starts the work (see step 5 above). |
| **Reject** | open, not being prepared | Closes the request with your reason. |
| **Extend due date** | open | Moves the due date by **Extend by (days)**. |
| **Mark complete** | in progress | Closes a request you carried out by hand. |
| **Put under legal hold** / **Release hold** | an open erasure | Stops the erasure from running while the hold is in place. |
| **Run now** | failed, queued or scheduled | Runs the work at once. For a scheduled erasure this skips the rest of the waiting period. |
| **Assign** | any | Records who is working on it. |

### Legal holds

A legal hold stops an erasure. **Legal holds** ([`dsr.legalHold.subjects`](/admin/settings#dsr.legalHold.subjects)) lists addresses that are always held; **Put under legal hold** holds one request. A held erasure shows **The erasure is blocked by a legal hold and will not run. It continues when the hold is removed.** The hold is checked again just before the erasure runs.

## What an export contains

The export is a ZIP file with a `manifest.json` that lists the sections, the files, the record counts and the reason for each part that was left out. It is built section by section, so a server restart continues where it stopped.

| File | Contents | Switch |
| --- | --- | --- |
| `profile.json` | The directory entry: address, domain, status, display name, dates | always |
| `settings.json` | Settings the person has set | **Export: settings** |
| `identities.json` | Sending identities, with their signature text | **Export: identities** |
| `contacts.vcf`, `contacts.json` | Contacts | **Export: contacts** |
| `calendars/*.ics`, `calendars.json` | One file per calendar | **Export: calendars** |
| `notes.json` | Notes, with their text | **Export: notes** |
| `rules/*.sieve`, `rules.json` | Filter scripts | **Export: filters and rules** |
| `mailstate.json` | Tags, saved searches, masked addresses, focused senders, remote-image grants, read-receipt rules and receipt status, queued submissions, pins, snoozes and follow-up dates, sender actions; a marker when drafts are saved on the server | **Export: tags, saved searches and sending records** |
| `tasks.json` | Tasks, with their iCalendar text | **Export: tasks** |
| `security.json` | Passkey labels and credential ids, sign-in addresses, password change history, API key prefixes, OAuth token metadata, sign-in consents given to applications, encryption key metadata and key associations; markers for authenticator secrets, recovery codes and zero-access keys | **Export: security records** |
| `integrations.json` | External mail (EWS) and DAV connections, added and linked mailboxes, webhooks and inbound webhooks; markers for sealed credentials | **Export: connected accounts and webhooks** |
| `delegation.json` | Delegations the person granted and received, the shared mailbox row when the account is a shared mailbox, group memberships | **Export: delegations and shared mailboxes** |
| `usage.json` | Last activity, usage counters, storage measurements, quotas | **Export: usage and quota records** |
| `records.json`, `records/compliance-notifications.json` | Restrictions, the consent ledger, retention rules, settings and runs, contact and calendar retention, uploaded file metadata, open tracking and opens, data-loss-prevention decisions, plug-in data keys and sizes; compliance emails sent to the person (event, status, time and attempts, no content) | **Export: restrictions, retention rules and other records** |
| `sessions.json` | Sessions (at most 200) and sign-in history (at most 500 entries per account) | **Export: sessions, devices and sign-in history** |
| `consent.json` | Cookie consent records | **Export: consent records** |
| `audit.json` | Audit entries where the person is the actor or the target (at most 5000 per account) | **Export: audit entries about the person** |
| `data-requests.json` | The person's data requests | always |
| `mail/mail.mbox` or `mail/*.eml` | Mail (see below) | **Mail in exports** and the person's choice |

The caps on sessions, sign-in history and audit entries are applied without a note in the manifest.

**Mail** is included only when **Mail in exports** is on (off by default) and the person ticked the box when filing. It covers the messages this server has in its message list for the person's accounts, up to 100,000 per account. A body that is not cached is fetched again from the upstream mail server, so the export includes mail that lives upstream. Attachments are inside the messages. The manifest always carries the sentence "Mail that stays on the upstream mail server is not part of this export", whatever the export contains. The console form for filing a request on someone's behalf never asks for mail; a request filed through the console API (`POST /dsr` with `includeMail`) can include it when **Mail in exports** is on.

The seven sections from `mailstate.json` to `records.json` are read from the server's data map (see [Erasure coverage](erasure-coverage.md#export-sections)) and are written as JSON only; **Export formats** adds no CSV copies of them. Each row names its source table. They leave out:

- **Secrets.** A sealed or secret row (authenticator secrets, recovery codes, zero-access keys, stored passwords and OAuth credentials of connected accounts, bridge tokens, linked-mailbox credentials, Nextcloud links, server-saved drafts) appears as a marker with the table name, the number of rows and the status `present, not exported`. Key values, token values, hashes, webhook signing secrets, passkey public keys, private key backups (only a yes/no flag is exported) and plug-in values are never selected. A test fails when an exported column is named like a secret, token, password, hash or key and is not on its reviewed allow list.
- **Parts of URLs.** A webhook URL and an EWS endpoint URL are exported as `scheme://host[:port]`: the user name, password, path, query string and fragment are removed, because they can hold tokens (many hook services put the token in the path).
- **Other people.** Rows are found by the person's account ids, personal owner keys, address and identity principals only. A delegation the person granted names the delegate's account id, since that says who can read the person's mail; a delegation the person received does not name the owner's account id. Rows of other accounts are not read through joins, delegations or shared mailboxes.

**Not exported**: signatures saved apart from an identity, and mail flags and folders (the mail itself is in `mail/` when included).

**Size**: an export stops at **Export size cap** (500 MB by default). A section that would go over it is left out and the manifest says so; this can leave out all mail.

**Format**: **Export formats** set to JSON with CSV adds CSV copies of some sections. Cells that start with `=`, `+`, `-`, `@`, a tab or a carriage return get a leading `'`.

**Storage and download**: the parts are stored on the server while they wait to be downloaded. **Seal exports at rest** (off by default) encrypts the stored parts with the server key; the ZIP file the person downloads is not encrypted. The person downloads the file from **Settings > Privacy > Data requests**, after confirming their password if **Re-authentication for requests** is on. The download link works for **Download link lifetime** (24 hours by default), only for the signed-in owner, and once when **Single-use download link** is on. The stored parts are deleted **Data export download window** hours after the export was ready (24 by default). Administrators cannot download an export.

## What erasure removes and what it leaves

When an erasure runs, the server:

1. Checks for a legal hold.
2. Sends the person a notice that the erasure is about to run.
3. Removes the person's documents from the search index, ends their sessions, and deletes message embeddings and compliance email records.
4. Deletes uploaded files, then runs every delete rule of the erasure registry, ending with the account rows, whose deletion also removes cached mail, calendars, contacts and notes.
5. Marks the directory entry as deleted, so the address can no longer sign in and the entry cannot be reactivated.
6. Counts the remaining rows of every deleted table and, when none remain, writes an erasure tombstone (a keyed hash of the address, used to repeat the erasure after a restore).
7. With **Scrub after erasure** on (off by default), replaces the person's identifiers in the audit history and log files (see [Erasure and scrubbing](#erasure-and-scrubbing)).
8. Deletes stored exports for the same address.

If rows remain or a step fails, the erasure is scheduled again 60 seconds later; after three attempts in total the request is marked **Failed, will be retried**, and **Run now** starts it again. The tables the registry covers, and what each rule does, are listed in the generated [Erasure coverage](erasure-coverage.md) page.

The completion record lists the per-table counts, the number of export parts deleted, and a digest of the record. The digest is an unkeyed hash, not a signature.

Erasure does **not** remove:

- **Mail on the upstream mail server.** Only this server's copies are deleted.
- **Backups.** They are not changed. Restoring a backup with the `mailwoman erasure restore` command repeats the erasures recorded since the backup, when **Replay erasures on restore** is on.
- **The request itself.** The request, its free-text details and its trail, including identity-check notes, are kept for **Finished request retention** (1825 days by default).
- **The audit log and log files.** Entries that name the person stay unless the person is scrubbed.
- **Records the registry keeps on purpose**: single sign-on, assistant and alert records and the breach trail and delivery log (kept for their retention periods; a scrub replaces the person's identifiers in them), breach incidents themselves, including any affected addresses entered in them, bans, the identity record, client error reports, group definitions, administrator accounts, and the retention archive, which keeps archived rows until each archive file expires. The generated [Erasure coverage](erasure-coverage.md) page lists every table and why it is kept.

### Erasure and scrubbing

The **Erasure and scrubbing** tab of **Compliance** replaces one person's address and account ids in history with a pseudonym, and shows the records that make an erasure repeatable after a restore.

![The Scrub person section of the Erasure and scrubbing tab with an address entered, and markers on the address and Preview.](/screenshots/admin-privacy/erasure-scrub.webp){.mw-crop}

1. **Address** of the person to scrub.
2. **Preview** counts the rows and log lines that would change and changes nothing. **Scrub person** is available after a preview.
{.mw-callouts}

**Scrub person** rewrites the person's address, account ids and user names in the audit log, the assistant, alert, breach, sign-in and single sign-on records, and, with **Scrub log files** on, the address and account ids in the rotated log files. Each is replaced with `scrubbed-<generation>-<hex>`, an HMAC of the address under a key stored sealed with the server key. It cannot be undone. A preview does not count audit entries written since the chain was last sealed; the run seals them first and then rewrites them.

The audit chain is an unkeyed SHA-256 chain kept in the same database. Anyone who can write to the database can rewrite entries and recompute it, a missing entry is not reported as a failure, and entries are not part of the chain until they are sealed (by **Seal new entries** or a scrub). Audit retention deletes entries, and a scrub rewrites them. The chain detects accidental damage; it does not make the audit log tamper-proof.

**Register backup** records a backup's label and location with the current tombstone watermark; nothing is done to the backup itself. **Erased since** counts the people erased or scrubbed after that watermark, whose data the backup still holds.

## File a request for someone

Use this for a request received by post or in person.

1. On the **Data requests** tab, go to **File a request for a user**.

   ![The File a request for a user form with the Person, Type and Details fields.](/screenshots/admin-privacy/dsr-behalf.webp){.mw-crop}

2. Enter the **Person**'s address, choose the **Type**, and add **Details (optional)**.
3. Select **File request**.

The person must have signed in to this server at least once; otherwise the server answers "No account of this address has signed in to this server." The request skips the confirmation code and is recorded as filed by you. **Requests on behalf of a user** turns this off.

## Options

All keys are in the [settings reference](/admin/settings). Scope "domain" means the value can also be set per domain.

| Setting | What it does | Default | Scope |
| --- | --- | --- | --- |
| **Data requests** ([`dsr.enabled`](/admin/settings#dsr.enabled)) and **Accept … requests** (`dsr.type.*`) | Offer the feature and each type | on | domain |
| **Approval of access and download requests** ([`dsr.approval.export`](/admin/settings#dsr.approval.export)) | `auto` or `admin` | auto | domain |
| **Approval of erasure requests** ([`dsr.approval.erasure`](/admin/settings#dsr.approval.erasure)) | `auto` or `admin` | admin | domain |
| **Handling of other requests** ([`dsr.approval.other`](/admin/settings#dsr.approval.other)) | `auto` or `admin` | admin | domain |
| **Re-authentication for requests** ([`dsr.reauth.required`](/admin/settings#dsr.reauth.required)) | Password or second factor to file access, download and erasure requests and to download | on | domain |
| **Confirmation message** ([`dsr.confirmation.required`](/admin/settings#dsr.confirmation.required)) | Code sent to the person's mailbox | on | domain |
| **Confirm again before approving** ([`dsr.admin.stepUp`](/admin/settings#dsr.admin.stepUp)) | Password again to approve or run an erasure | on | domain |
| **Identity verification step** ([`dsr.identityVerification.required`](/admin/settings#dsr.identityVerification.required)) | Adds the identity check | off | domain |
| **Erasure grace period** ([`dsr.erasure.graceDays`](/admin/settings#dsr.erasure.graceDays)) | Days between approval and erasure | 14 | domain |
| **Days to answer a request** ([`dsr.sla.days`](/admin/settings#dsr.sla.days)) | Due date | 30 | domain |
| **Longest extension** ([`dsr.sla.maxExtensionDays`](/admin/settings#dsr.sla.maxExtensionDays)) | Total extension allowed | 60 | server |
| **Due date flags** ([`dsr.sla.reminderDays`](/admin/settings#dsr.sla.reminderDays)) | Only the largest value is used: one "due soon" entry is recorded that many days before the due date | 1, 3, 7 | server |
| **Mail in exports** ([`dsr.export.mailAllowed`](/admin/settings#dsr.export.mailAllowed)) / **Mail format in exports** ([`dsr.export.mailFormat`](/admin/settings#dsr.export.mailFormat)) | Offer mail in exports; mbox or eml | off / mbox | domain |
| **Export: …** (`dsr.export.include.*`) | Include each section | on | domain |
| **Export size cap** ([`dsr.export.maxMegabytes`](/admin/settings#dsr.export.maxMegabytes)) | Largest export | 500 MB | server |
| **Export formats** ([`dsr.export.format`](/admin/settings#dsr.export.format)) | JSON, or JSON with CSV copies | JSON | domain |
| **Seal exports at rest** ([`dsr.export.encrypt`](/admin/settings#dsr.export.encrypt)) | Encrypt stored parts with the server key; the download is not encrypted | off | domain |
| **Download link lifetime** ([`dsr.download.linkMinutes`](/admin/settings#dsr.download.linkMinutes)) | Minutes a download link works | 1440 | domain |
| **Single-use download link** ([`dsr.download.singleUse`](/admin/settings#dsr.download.singleUse)) | One download per link | on | domain |
| **Data export download window** ([`dataExport.linkHours`](/admin/settings#dataExport.linkHours)) | Hours the stored export is kept | 24 | server |
| **Finished request retention** ([`dsr.retention.completedDays`](/admin/settings#dsr.retention.completedDays)) | Days a finished request, its details and trail are kept | 1825 | server |
| **Record new requests for admins** ([`dsr.notify.admins`](/admin/settings#dsr.notify.admins)) | Security events in the audit log for new and overdue requests | on | server |
| **Progress messages to the requester** ([`dsr.notify.requester`](/admin/settings#dsr.notify.requester)) | Message in the person's mailbox at each change | on | user |
| **Requests on behalf of a user** ([`dsr.onBehalf.allowed`](/admin/settings#dsr.onBehalf.allowed)) | Allow filing for someone | on | domain |
| **Requests per person per day** ([`dsr.rateLimit.perDay`](/admin/settings#dsr.rateLimit.perDay)) | Rolling 24 hours | 5 | domain |
| **Legal holds** ([`dsr.legalHold.subjects`](/admin/settings#dsr.legalHold.subjects)) | Addresses whose erasure never runs | empty | server |
| **Data protection contact name**, **address**, **page** (`dsr.contact.*`) | Shown to people on their data requests page and in the privacy policy | empty | domain |
| **Data requests introduction**, **Confirmation message text**, **Completion message text** (`dsr.text.*`) | Your own wording per language | empty | domain |

The server's own notices about requests (confirmation, erasure, completion) are written in English whatever the person's language; only the texts you set in `dsr.text.*` are shown in other languages. Email delivery of request notices to operators and requesters is set on the **Email notifications** tab (see [Compliance](compliance.md#email-notifications)). The settings of the **Erasure and scrubbing** tab are:

| Setting | What it does | Default |
| --- | --- | --- |
| **Person scrub** ([`scrub.enabled`](/admin/settings#scrub.enabled)) | Allow **Scrub person** | on |
| **Scrub after erasure** ([`scrub.autoRunOnErasure`](/admin/settings#scrub.autoRunOnErasure)) | Scrub automatically after every erasure | off |
| **Scrub log files** ([`scrub.includeLogs`](/admin/settings#scrub.includeLogs)) | Also rewrite rotated log files | on |
| **Pseudonym length** ([`scrub.pseudonymAlgorithm`](/admin/settings#scrub.pseudonymAlgorithm)) | 16 or 32 hexadecimal characters | 16 |
| **Pseudonym key retention** ([`scrub.keyRetentionDays`](/admin/settings#scrub.keyRetentionDays)) | Days a retired key is kept after **Rotate key**; 0 keeps it. Changing it does not affect keys retired earlier | 0 |
| **Replay erasures on restore** ([`backup.propagateErasure`](/admin/settings#backup.propagateErasure)) | Read only by the `mailwoman erasure restore` command | on |
| **Erasure tombstone retention** ([`backup.tombstoneRetentionDays`](/admin/settings#backup.tombstoneRetentionDays)) | Days tombstones are kept; an erasure older than this is not repeated when a backup is restored | 3650 |

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| **File request** answers "No account of this address has signed in to this server." | The person has never signed in, so there is no account to attach the request to. | Ask the person to sign in once, or handle the request outside the queue. |
| An erasure stays **On hold** | A legal hold applies, from **Legal holds** or from the request. | Release the hold when it no longer applies; the erasure then continues. |
| The person cannot file a second request of the same type | One open request per type is allowed. | Finish or reject the open one. |

## Related

- [Restrictions and consent](consent.md)
- [Erasure coverage](erasure-coverage.md)
- [Retention](retention.md)
- [Compliance](compliance.md)
- [Data requests in the mail app](/user/settings/account#data-requests)
