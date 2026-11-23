---
screen: true
---
# Breach register

The **Breach register** tab of **Compliance** keeps a record of personal-data breaches: what happened, when you became aware, a notice clock that runs from that moment, a document to submit to the supervisory authority, a message to the people affected, and checklists. The register records and reminds; it never contacts an authority itself. Whether an incident must be notified, and to whom, is for you to decide.

![The breach register scrolled to its filters and list, with one incident whose notice clock is running and one draft.](/screenshots/admin-privacy/breaches-list.webp){.mw-shot}

## Before you start

- Reading the register needs `privacy.read`, changing it `privacy.write`, both on all domains.
- Enter the **Supervisory authority** details and the **Data protection contact** under **Authority and data protection contact** on the same tab. They are copied into the authority document.
- Incident records, their trail and the delivery log are stored in the database without encryption (see [Encryption at rest](encryption.md)).

## Record an incident

1. Open **Compliance** and select **Breach register**.
2. In the new incident form, enter a title and select **Open incident**. The notice clock starts at once, unless you tick **Open as a draft (clock not started)**. A draft, including one opened by an automation, has no clock until someone confirms it with the awareness time.
3. Select **Open** on the incident's row.

   ![The top of an opened breach incident: its reference and title, the notice clock, and the status and assignment controls.](/screenshots/admin-privacy/breach-detail.webp){.mw-crop}

4. Fill in the details: type, description, the categories of people and data, the approximate numbers, the likely consequences and the measures taken and proposed, and, if you know them, the affected addresses. Addresses that have signed in to this server are linked to their accounts.
5. Answer the questions under **Risk assessment**. The answers give a **Suggested level** (no risk, risk or high risk). You choose the **Risk level**; when it differs from the suggestion, or questions are unanswered, you must give the **Reasoning** (at least 10 characters).
6. Add **Timeline** entries and **Evidence** (text, or `http://` and `https://` links) as you go, and **Assign** the incident.

Each incident has a reference, `BR-` and the first eight characters of its id. Changing the awareness time needs a reason of at least five characters. Every change is added to the incident's trail and recorded in the audit log. When you edit the details, the new values replace the old ones and the trail records which fields changed, not their earlier values.

### Statuses

**Detected**, **Assessing**, **Contained**, **Authority notified**, **Data subjects notified**, **Closed** and **Not notifiable**. The server enforces these conditions:

- **Not notifiable** needs a "no risk" assessment and reasoning of at least ten characters.
- **Authority notified** needs a recorded authority notice; recording one moves the incident there.
- **Data subjects notified** needs, for every affected person, a successful delivery of the in-app notice or the mailbox message, or an email accepted by the relay; or a "people informed" timeline entry for a notice made outside the server.
- **Closed** needs an assessment. A risk or high-risk incident without a recorded authority notice also needs a closing reason.
- Reopening a closed or not-notifiable incident needs a reason.

## The notice clock

The deadline is the awareness time plus **Hours to notify the authority** (72 by default). The list and the incident show the time left, and the time past the deadline once it has passed. The clock stops when the first authority notice is recorded.

A background worker runs every minute. It records a reminder in the trail at each of the **Reminder times** before the deadline (6, 24 and 48 hours by default), once each, and tells the operators. After the deadline it reminds at once and then every **Overdue repeat** hours (24; 0 reminds once). **Check deadlines now** runs the same pass immediately. When the deadline has passed with no authority notice, the incident asks for the reasons for the delay, and a late notice cannot be recorded without them.

**How operators are told** (each reminder, overdue notice, automatic draft and escalation):

- security events in the audit log, with **Record notices in the audit log** on (the default);
- a signed request to each of the **Webhook addresses**;
- a message in the mailbox of each of the **Operator mailboxes** that has signed in to this server. This channel has no switch of its own: it is used whenever addresses are listed;
- email through the compliance relay, with **Email breach operators** on (see [Compliance: Email notifications](compliance.md#email-notifications)).

## Notify the supervisory authority

1. On the incident, generate the authority notice. The document groups the description, the categories and numbers of people and records, the data protection contact, the likely consequences, the measures, the incident times, the assessed risk and, when late, the reasons for delay. It is offered as a printable HTML page and as JSON. Generating it is recorded in the audit log.
2. Submit it through the authority's own channel. The server sends nothing.
3. Use **Record submission** with its time and the reference the authority gave. Later notices are recorded as an update or a final notice; each keeps a copy of the document as it stood.

## Notify the people affected

1. Choose the audience (all affected addresses, selected addresses or one domain), the language (or each person's own) and optional advice.
2. Use **Preview (dry run)**. It lists each recipient with the channels that would be used, and shows the subject and text. Nothing is sent.
3. Select **Send**. Delivery runs in the background and writes one row per recipient and channel to the delivery log.

| Channel | Setting | Default |
| --- | --- | --- |
| A notice in the person's account | **Notice in the person's account** | on |
| A message in the person's own mailbox | **Message to the person's mailbox** | on |
| A request to each webhook address | **Also send to webhooks** | off |
| Email through the compliance relay, also to addresses with no account here | **Email affected people** | off |

The first two need an account on this server. An email counts as delivered when the relay accepts it, which does not prove it reached the mailbox. The webhook payload carries the text and the number of recipients; the text is filled in with the first recipient's values, so a template that uses `{{recipient}}` sends that person's address to the webhook. Notices shown in people's accounts are not removed when the incident is removed from the register.

The wording is **Message subject** and **Message text**, per language; empty uses the built-in text. Variables in double braces: `incident`, `reference`, `title`, `date`, `description`, `categories`, `consequences`, `measures`, `advice`, `dpo`, `dpoName`, `dpoEmail`, `dpoPhone`, `authority`, `domain`, `recipient`. An unknown variable is left as written.

## Automations

With **Automations** on (off by default), each rule counts matching audit events over **Counting window in minutes** (60) and opens a draft incident when its threshold is reached. Every threshold is 0 (off) until you set it.

| Rule (console name) | Counts |
| --- | --- |
| **Session theft alerts** | session security alerts for impossible travel, concurrent use, reuse of a rotated session and client mismatch |
| **Sign-in spray or spike** | sign-in spray and heightened sign-in spike events |
| **Forwarding rules in bulk** | audit actions whose name contains `forward` |
| **Administrator role changes** | changes to administrator roles and overrides |
| **Unusual exports** | audit log exports and cookie consent log exports |
| **Audit chain failures** | audit actions that name a failed audit chain |
| **Backup or restore failures** | audit actions that name a failed backup or a restore |
| **Many data request exports** | finished data request exports |

A draft is assigned to **Assign drafts to** and has no clock until someone confirms it with the awareness time, unless **Draft clock** is `onDetect`. A rule that matches again while its draft is open adds to the draft. An automatic incident still a draft or unassigned after **Escalate after hours** (4) is assigned to **Escalate to** once and the operators are told. The worker counts in memory from the time automation was turned on; with several server processes each counts what it sees.

## Checklists and containment

Each incident type has a checklist (preserve evidence, identify the data, revoke sessions, review forwarding rules, decide on notifying, document the measures, root cause, lessons), recording who ticked each step and when. **End sessions** and **Require password change** act on the affected accounts: each shows a preview first, and applying it is recorded in the trail and the audit log.

## Export and retention

Export the register for one year (by detection date) as CSV, JSON or HTML, or one incident's report with its trail. Exports are recorded in the audit log.

Closed and not-notifiable incidents are deleted, with their trail and delivery log, when they were last changed more than **Register retention in years** ago (10 by default; counted as 365-day years). The check runs every hour and does not consider legal holds. Open incidents are not deleted.

::: warning Erasure does not remove breach records
When a person is erased, breach incidents keep any affected addresses entered in them, and the trail and delivery log are kept as well; a scrub of the person replaces their identifiers in the trail and the delivery log, but not in the incident itself. The records stay until the register retention above removes the incident. See [Data requests](data-requests.md#what-erasure-removes-and-what-it-leaves).
:::

## Options

| Setting | Default | Scope |
| --- | --- | --- |
| **Breach register** ([`breach.enabled`](/admin/settings#breach.enabled)) | on | domain |
| **Hours to notify the authority** ([`breach.noticeHours`](/admin/settings#breach.noticeHours)) | 72 | domain |
| **Reminder times** ([`breach.reminders.hours`](/admin/settings#breach.reminders.hours)) | 6, 24, 48 | domain |
| **Overdue repeat** ([`breach.reminders.overdueRepeatHours`](/admin/settings#breach.reminders.overdueRepeatHours)) | 24 | domain |
| **Supervisory authority**, **Authority notification page**, **Authority contact** (`breach.authority.*`) | empty | domain |
| **Data protection contact name**, **email**, **phone** (`breach.dpo.*`) | empty | domain |
| **Message subject**, **Message text** (`breach.template.*`) | built-in text | domain |
| **Record notices in the audit log** ([`breach.notify.audit`](/admin/settings#breach.notify.audit)) | on | domain |
| **Webhook addresses** ([`breach.notify.webhookUrls`](/admin/settings#breach.notify.webhookUrls)) | empty | server |
| **Operator mailboxes** ([`breach.notify.operatorAddresses`](/admin/settings#breach.notify.operatorAddresses)) | empty | server |
| **Notice in the person's account**, **Message to the person's mailbox**, **Also send to webhooks** (`breach.subjects.*`) | on, on, off | domain |
| **Email breach operators** ([`breach.notify.email`](/admin/settings#breach.notify.email)), **Email affected people** ([`breach.subjects.email`](/admin/settings#breach.subjects.email)) | off, off | server |
| **Register retention in years** ([`breach.retention.years`](/admin/settings#breach.retention.years)) | 10 | server |
| **Automations** and its rules (`breach.auto.*`) | off, thresholds 0 | server |

## Related

- [Compliance](compliance.md)
- [Data requests](data-requests.md)
- [Operational incidents](operational-incidents.md)
- [Audit log and retention](retention.md)
