# Breach register

A register for personal-data breaches: the incident record, a notice clock that runs from the moment you became aware, a document for the supervisory authority, a message to the people affected, automations that open draft incidents from detections, and containment checklists. This page describes the controls the server provides. It does not say that using them meets any law; whether an incident must be notified, to whom and by when is for your own legal advice. Mailwoman never sends anything to an authority.

You work the register under **Admin > Compliance > Breach register** (needs the `privacy.read` capability to read and `privacy.write` to change; both are for all-domain grants). The settings below are the `breach.*` keys in [settings](settings.md), set globally or per domain where noted, each with an admin default and lock.

## An incident

| Field | Meaning |
| --- | --- |
| Reference | `BR-` and the first eight characters of the id, to quote in notes and messages. |
| Detected at | When the incident was first noticed. |
| Aware at | When the organisation became aware. This starts the notice clock. It can be changed, but only with a written reason, and the change is kept in the incident's trail and in the audit log. |
| Source | Entered by hand, automatic detection, data request or third party. |
| Type | Credential compromise, mailbox access, lost device, misdirected message, insider or administrator misuse, malware, processor or third party, other. The type selects the checklist. |
| Description, people and data | What happened, categories of people and of data, approximate number of people and records, likely consequences, measures taken and proposed. |
| Affected addresses | Optional list of addresses. Addresses that have signed in to this server are linked to their accounts, which the message and the playbook actions use. |
| Timeline | Detection, containment, closure, note and "people informed" entries with the time and the person. |
| Evidence | Text or links (`http://` or `https://` only). |
| Assignee | Who is handling it. |

Nothing is deleted from an incident: every change is an entry in its trail, and also a `breach-changed` security event in the audit log.

## Status

`detected`, `assessing`, `contained`, `notified-authority`, `notified-subjects`, `closed` and `not-notifiable`. Some moves have conditions that the server enforces:

* **Not notifiable** needs a "no risk" assessment and written reasoning (at least ten characters).
* **Authority notified** needs a recorded authority notice (below). Recording one moves the incident there by itself.
* **Data subjects notified** needs a data-subject message on record, or a timeline entry of the type "people informed" for a notice made outside Mailwoman.
* **Closed** needs an assessment. A risk or high-risk incident that has no authority notice on record needs a closing reason.
* **Reopening** a closed or not-notifiable incident needs a reason.

## Risk assessment

A guided questionnaire (who could have seen the data, special categories or credentials, identifiability, number of people, vulnerable people, whether the data was unreadable to whoever got it, containment and seriousness of the consequences) gives a score and a suggested level: **no risk**, **risk** or **high risk**. The suggestion is an aid. You choose the level; when it differs from the suggestion, or the questionnaire is incomplete, you must write the reasoning. The answers, the suggestion, the chosen level and who set it are kept.

## The notice clock

The deadline is *aware at* plus `breach.noticeHours` (72 by default; per domain if you set it). The list and the incident show the time left, "under 24 hours" in the list's due-soon filter, and the time past the deadline once it has passed. The clock stops when the first authority notice is recorded; it is shown as on time or late. Closing without a notice shows it as not required.

**Reminders.** `breach.reminders.hours` lists hours before the deadline (default 6, 24 and 48). The worker, which runs every minute, records each reminder once in the trail and tells the operators through the configured channels. If several thresholds passed since the last pass only the tightest is announced. Once the deadline has passed it reminds at once, then every `breach.reminders.overdueRepeatHours` (24; 0 reminds once). Recording an authority notice ends the reminders. **Check deadlines now** in the console runs the same pass on demand.

**Reasons for delay.** When the deadline has passed with no authority notice recorded, the incident asks for the reasons for the delay, and a late initial notice cannot be recorded without them.

**Operator channels.** `breach.notify.audit` (security events in the audit log), `breach.notify.webhookUrls` (signed POST to each URL, using the same address checks as other outbound webhooks; treat the URLs as secrets) and `breach.notify.operatorAddresses` (the text goes into the inbox of a local account for each address; an address nobody has signed in with is logged as not delivered). Outbound SMTP email is available through the separately configured, default-off compliance mail service: `breach.notify.email` sends operator notices, and `breach.subjects.email` adds email to the explicit affected-person preview/send workflow. See [Governance and email](compliance-governance.md).

## Notice to the supervisory authority

The incident page produces a document with the content grouped as the items of a notification: the nature of the breach (description, categories and approximate numbers of people and records), the data protection contact, the likely consequences and the measures taken or proposed, plus the incident times, the assessed risk and the reasons for delay when late. It is offered as a printable HTML page (English or Portuguese) and as JSON. The authority name, page and contact (`breach.authority.*`) and the contact person (`breach.dpo.*`) are settings that a domain can override.

You submit the document through the authority's own channel. Then **Record submission** with the time, the reference the authority gave and, if late, the reasons for delay. Later notices are recorded as an update or a final notice; each keeps a copy of the document as it stood. Generating a document is recorded in the trail; nothing is sent.

## Message to the people affected

Pick an audience (all affected addresses, selected addresses, or one domain), a language (or each person's own language), and optional advice. **Preview** is a dry run: it lists each recipient with the channels that would be used and shows the rendered subject and text, and flags variables it does not know. **Send** then delivers in the background and writes one delivery-log row per recipient and channel (sent, failed or skipped, with the reason).

Channels, each a setting: a notice inside the person's account (`breach.subjects.inApp`; it shows under `GET /api/privacy/breach-notices` and can be acknowledged), a message in the person's own mailbox by the same path as the data-request confirmations (`breach.subjects.inbox`), and the webhook addresses (`breach.subjects.webhook`, one payload per notice with the text and the number of recipients, not their addresses). An address with no account on this server is skipped for the first two.

The wording is `breach.template.subject` and `breach.template.body`, one text per language; an empty one uses the built-in English or Portuguese text. Variables are written in double braces: `incident`, `reference`, `title`, `date`, `description`, `categories`, `consequences`, `measures`, `advice`, `dpo`, `dpoName`, `dpoEmail`, `dpoPhone`, `authority`, `domain`, `recipient`. An unknown variable is left as written.

## Automations

Off until `breach.auto.enabled` is on. Each rule has a threshold in a `breach.auto.<rule>.threshold` setting (0 is off) and counts matching audit events over `breach.auto.windowMinutes`:

| Rule | Counts |
| --- | --- |
| theft | `session-security-alert` events for impossible travel, concurrent use, rotated-session reuse and client mismatch |
| credentialStuffing | `signin-spray-source` and `signin-spike-heightened` |
| massForward | any audit action whose name contains `forward` |
| roleEscalation | `admin-roles-changed`, `admin-role-updated`, `admin-overrides-changed` |
| exportVolume | `audit-export`, `cookie-consent-exported` |
| auditChain | audit actions naming a failed or broken audit chain |
| backupAnomaly | audit actions naming a failed backup or a restore |
| dsrExport | `dsr-export-ready` |

When a rule reaches its threshold it opens a **draft** incident (source "automatic"), assigned to `breach.auto.assignee`, and tells the operators. A draft has no clock: it starts when someone confirms it with the time they became aware, unless `breach.auto.startClock` is `onDetect`. A rule that matches again while its draft is still open adds to the draft instead of opening another. A draft or unassigned automatic incident older than `breach.auto.escalateAfterHours` is escalated once: assigned to `breach.auto.escalateTo` and the operators are told.

The worker reads the audit log from a cursor, so enabling automation does not replay history, and counts events in memory; with several server processes each counts what it sees. A rule only fires for events the server records: the audit-chain, backup and forwarding rules match by action name and fire once such events are written.

## Checklists and containment

Each type has a checklist (preserve evidence, identify the data, revoke sessions, review forwarding rules, decide on notifying the authority and the people, document the measures, root cause, lessons) with who ticked each step and when. Two steps have one-click actions on the affected accounts: **End sessions** and **Require password change**. Each has a preview that lists the accounts and, for sessions, how many would end; applying needs the preview's confirmation, is recorded in the trail and the audit log, and ticks the step.

## Register export and retention

Under **Register export**, download one year (by detection date) as CSV, JSON or HTML. Each incident also has a report (HTML, JSON or CSV) with its trail. Exports are recorded in the audit log.

Closed and not-notifiable incidents are removed, with their trail and delivery log, after `breach.retention.years` (10 by default). Open incidents are never removed.

## API

Admin routes under `/admin`: `GET/POST /breaches`, `GET /breaches/meta`, `GET /breaches/export`, `POST /breaches/run`, `GET /breaches/{id}`, `GET /breaches/{id}/report`, and `POST /breaches/{id}/` `details`, `assign`, `confirm`, `aware-at`, `assess`, `status`, `timeline`, `evidence`, `subject-notice`, `playbook`, `lockdown`, plus `GET` and `POST /breaches/{id}/authority-notice`. People: `GET /api/privacy/breach-notices` and `POST /api/privacy/breach-notices/ack`.
