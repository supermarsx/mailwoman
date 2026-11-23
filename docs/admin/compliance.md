---
screen: true
---
# Compliance

The **Compliance** section of the console gathers the data-protection controls in one place: data requests, cookie consent, retention, restrictions, consents, the privacy policy, records of processing, impact assessments, the breach register, encryption, erasure, the governance registers, email notifications, operational incidents and the product's software inventory. Each tab is a setting or a record you keep. Nothing in the console decides whether a configuration meets a particular law or standard; that judgement is yours.

![The Compliance section of the console on its Overview tab: the row of tabs and one card per tab, with status lines on the Policy, Records of processing and Impact assessments cards.](/screenshots/admin-privacy/compliance-overview.webp){.mw-shot}

## Before you start

Each tab has its own permission. **Data requests** and **Restrictions** need `data_export.run` and work per domain. **Breach register**, **Erasure and scrubbing** and **Operational incidents** need `privacy.read` (and `privacy.write` to change). Every other tab needs `settings.read` (and `settings.write` to change), and all tabs except **Data requests**, **Retention** and **Restrictions** are only shown to an administrator who is not limited to some domains. A tab you cannot open shows "You do not have access to this tab."

## The tabs

| Tab | What it holds | Page |
| --- | --- | --- |
| **Data requests** | The queue of access, download, correction, erasure, restriction and objection requests | [Data requests](data-requests.md) |
| **Cookies** | The cookie consent control, the storage inventory and the consent log | [Cookie consent](cookies.md) |
| **Retention** | How long each store keeps data, the daily sweep and log rotation | [Retention](retention.md) |
| **Restrictions** | People whose processing is paused | [Restrictions and consent](consent.md) |
| **Consents** | Decisions per purpose and the settings of each purpose | [Restrictions and consent](consent.md#consents-by-purpose) |
| **Policy** | The generated privacy and cookie policies and their versions | [below](#privacy-policy-and-cookie-policy) |
| **Records of processing** | Processing activities and recipients | [below](#records-of-processing) |
| **Impact assessments** | A register of impact assessments | [below](#impact-assessments) |
| **Breach register** | Personal-data breaches and their notice clock | [Breach register](breach-register.md) |
| **Encryption** | What is encrypted on disk and how that was checked | [Encryption at rest](encryption.md) |
| **Erasure and scrubbing** | Scrubbing a person from history, the audit chain and backups | [Data requests](data-requests.md#erasure-and-scrubbing) |
| **Contracts**, **Data transfers**, **Control mappings** | Governance registers | [below](#governance-registers) |
| **Email notifications** | Outbound email for data requests and breaches | [below](#email-notifications) |
| **Operational incidents** | Outages and service incidents | [Operational incidents](operational-incidents.md) |
| **About Mailwoman** | Build facts and software inventories | [Product security information](product-security.md) |

The Overview shows one card per tab. Three cards carry a status line: **Policy** (blank operator fields, changes not yet in a version, or the current version), **Records of processing** (activities and recipients) and **Impact assessments** (assessments due or suggested). Opening the Overview records a policy version when **Automatic policy versions** is on and the policy has changed. The older addresses `#/data-requests`, `#/retention` and `#/privacy/cookies` open the matching tab.

## Privacy policy and cookie policy

The server writes a privacy policy and a cookie policy from the live configuration, in English and European Portuguese. The text covers who is responsible, what is processed, purposes and legal bases, cookies and browser storage (from the built-in inventory and your rules; see the gaps listed on [Cookie consent](cookies.md#check-the-inventory)), retention, data subject rights, remote content, recipients, transfers, security measures and contact details. Optional features such as the assistant or open tracking appear only while they are on.

### Fill in the operator fields

1. Open **Compliance** and select **Policy**.

   ![The Status section of the Policy tab: the current version and the buttons to publish a new version, open the public page and download the policy as Markdown.](/screenshots/admin-privacy/policy-status.webp){.mw-crop}

   1. **Publish a new version** records the current text as a version.
   2. **Open public page** opens `/privacy`.
   3. **Download Markdown** saves the policy as a Markdown file.
   {.mw-callouts}

2. Under **Operator fields**, fill in the facts the server cannot know.

   ![The top of the Operator fields section of the Policy tab, listing the five required fields that are still blank: controller name and address, legal bases, hosting provider and supervisory authority.](/screenshots/admin-privacy/policy-operator-fields.webp){.mw-crop}

   Five of them count as required and are reported while blank: **Controller name**, **Controller address**, **Legal bases**, **Hosting provider** and **Supervisory authority**. **Transfers outside the hosting country** is used in the text but never reported as blank. The data protection contact comes from the **Data protection contact** settings of [Data requests](data-requests.md#options).
3. Under **Preview and overrides**, choose the **Document** and **Language**. **Replace text** on a section replaces it with your own text for that language (plain text, blank lines between paragraphs, `- ` for list items, `**bold**`). **Use generated text** goes back to the generated text. A replaced section stops showing its own warnings, but the **Operator fields** list still counts the blank fields.
4. Select **Publish a new version**, or let versions be recorded automatically (see below).

### Versions

Each version stores a snapshot of the facts and overrides it was written from. The public pages show the last recorded version, not the live configuration.

- With **Automatic policy versions** on (the default), a version is recorded the next time the Policy tab, the Overview or a public page is opened after a change that affects the policy. These automatic versions are not written to the audit log; **Publish a new version** is.
- With it off, changes are listed as pending until you publish.
- A change counts when it is to a setting whose key starts with `incidents.`, `compliance.mail.`, `cookies.`, `dsr.`, `privacy.`, `logging.redaction.`, `assistant.` or `policy.`, or is `mail.openTracking`, `mail.readReceipts` or `observability.client.enabled`, or to retention values, your cookie rules or the overrides. Not counted: **Policy version**, the policy switches themselves, the cookie banner texts (`cookies.text.*`), image-proxy tuning, and the `dsr.notify.*` and `dsr.reauth.*` settings. Category names and descriptions of the cookie control do count.
- **Version history** shows the last 50 versions.
- With **Ask for cookie consent again on a new policy version** on, each version after the first raises the server-wide **Policy version** of the cookie control by one; a value set for one domain is not raised.

### Public pages

| Address | Content |
| --- | --- |
| `/privacy` | The privacy policy |
| `/cookies-policy` | The cookie policy |

Both are public. Add `?lang=pt-PT` or `?lang=en` to choose the language; without it the browser's language is used. Add `?format=md` for Markdown or `?format=json` for the sections as data. With **Public policy pages** off, both answer 404.

## Records of processing

The **Records of processing** tab lists **Processing activities** and **Recipients and sub-processors**.

![The Processing activities table of the Records of processing tab: the generated activities, two marked off, and one activity added by an administrator, Staff directory.](/screenshots/admin-privacy/records-activities.webp){.mw-crop}

- **Generated** activities come from a fixed list in the server: Operational incident management, Compliance email notifications, Mail service, Authentication and security records, Data subject requests, Cookie consent records, Remote content proxy, AI assistant, Open tracking and Service statistics. Each is marked "(off)" while the setting it depends on is off. The cookie consent activity follows **Keep a consent log**, which is on by default, so it is listed as on even while the cookie consent control is off. All generated activities share the legal-basis text of the **Legal bases** operator field, and they cannot be edited.
- **Added by you**: **Add activity** and **Add sub-processor** open a three-step form for processing the server cannot see and for the providers you use. **Delete** removes a row without asking for confirmation; the deletion is recorded in the audit log.
- The generated recipients are the classes of outbound traffic the server makes. The **Recipients** count on the Overview card counts only the sub-processors you added.
- **Export CSV**, **Export JSON** and **Export HTML** include rows that are off, with an `enabled` column, whatever the **Include in exports and the user view** box says. Signed-in people see only the rows that are on, in their own language, through `/api/privacy/processing`.

## Impact assessments

The **Impact assessments** tab is a register of data protection impact assessments. The register records your assessment; whether one is required is for you to decide.

![The Impact assessments tab: the register with one draft assessment for the AI assistant, and the New assessment form with its title, template and language.](/screenshots/admin-privacy/dpia-register.webp){.mw-crop}

1. Under **New assessment**, enter a title, choose a **Template** (Generic, AI assistant, Email tracking, Large-scale monitoring or New processor) and select **Create**. Or use **Start from template** on a suggestion.
2. Open the assessment and fill in its text fields, **Risks** and **Attachments (links or text)**. Each risk has a likelihood and a severity from 1 to 4; the score is their product: up to 3 low, up to 6 medium, up to 9 high, above that very high. The same scale applies to the likelihood and severity after mitigation.
3. Move it through the statuses: **Draft** to **In review**; **In review** to **Draft**, **Approved** or **Needs prior consultation**; **Approved** back to **In review**; **Needs prior consultation** to **In review** or **Approved**. Approval needs a score after mitigation on every risk, and none of 10 or more.

Every save is kept as a version, and a save made against an older version is refused. Approving sets the review date **Impact assessment review interval** × 30 days ahead, unless the editor already has a date. An approved assessment, or one needing prior consultation, is marked **Review due** from **Impact assessment reminder window** days before its review date and **Overdue** after it; drafts and assessments in review are never due. Nothing is sent: the marks are shown on the tab and counted on the Overview. Delete has no confirmation; it is recorded in the audit log. **Export HTML** and **Export JSON** save one assessment.

**Suggested assessments**, with **Suggest impact assessments** on, lists the assistant, open tracking and analytics while each is on and no assessment covers that feature. The analytics suggestion uses the Large-scale monitoring template, and an assessment started from it does not clear the suggestion.

## Governance registers

**Contracts**, **Data transfers** and **Control mappings** are registers you fill in. The server checks that the required fields are present and well formed (dates as YYYY-MM-DD, HTTPS links, upper-case country codes, whole numbers). It does not fetch linked documents, and it does not judge whether a contract, a transfer mechanism or a control is adequate.

![The Contracts register with its search, status filter, export buttons and one active data processing agreement.](/screenshots/admin-privacy/governance-contracts.webp){.mw-crop}

Each register has **Search records**, a status filter, **Add record**, which opens a three-step form, and **Export JSON**, **Export CSV** and **Export HTML**. A past review date is marked **Review overdue**. Records cannot be deleted from the console.

- **Contracts.** Types: data processing agreement, master service agreement, standard contractual clauses, ICT service agreement, other. An **Active** contract needs an owner, counterparty, signing date and document link, and its expiry cannot precede signing. The ICT service agreement adds service, location, security, recovery, audit, exit and subcontracting fields; an active one needs more of them filled in, and more again when its criticality is important or critical. The form does not produce a regulatory register of information.
- **Data transfers.** Internal or international. An **Active** transfer needs an owner, exporter, importer, purpose, data categories, legal basis and both countries. An active international transfer also needs a mechanism (adequacy decision, standard contractual clauses, binding corporate rules, derogation or other), safeguards and a review date; standard contractual clauses, binding corporate rules and other mechanisms also need an assessment link, and standard contractual clauses a linked contract.
- **Control mappings.** Fifteen starter controls, each linked to a documentation page and to GDPR articles, SOC 2 criteria and ISO/IEC 27001 references. They start **Unassessed**. Statuses: **Unassessed**, **Planned**, **Implemented**, **Verified by operator**, **Exception**. **Verified by operator** needs an owner, evidence and a test date; **Exception** needs an owner, a justification and a review date. The evidence is text you enter: nothing is computed from the server's configuration, and nothing marks a control verified by itself. The starter list is a partial cross-reference, not a complete statement of applicability.

## Email notifications

The **Email notifications** tab sets up outbound email for data requests and breaches. It is off by default; the notices written to people's own mailboxes, the audit log and webhooks work without it.

![The Email notifications tab of Compliance: the switch that turns compliance email on and the SMTP relay settings.](/screenshots/admin-privacy/compliance-notifications.webp){.mw-shot}

1. Choose the **Submission protocol**: SMTP (with **SMTP relay host**, **SMTP port**, **TLS mode** STARTTLS or implicit TLS) or JMAP over HTTPS (**JMAP session URL**). Enter the **Sender address** and, when needed, the **Submission username**.
2. Set the password on the server, not in the console: `MW_COMPLIANCE_SMTP_PASSWORD` or `MW_COMPLIANCE_SMTP_PASSWORD_FILE` for SMTP (the file wins when both are set), `MW_COMPLIANCE_JMAP_PASSWORD` or its `_FILE` form for JMAP.
3. With **Require proxy route** on (the default), sending needs an outbound proxy route for the `system_mail` traffic class (see [Outbound connections](/security/egress)). **Allow private relay** permits a relay on your own private network.
4. Enter **Operator recipients**, turn on **Enable email** and select **Send test email**. The test goes to the **Operator recipients** of data requests.
5. Turn on the channels you want: **Email operators about data requests**, **Email requester updates**, **Email breach operators** (to **Breach operator recipients**) and **Email affected people**.

Queued messages (recipient, subject and text) are stored encrypted with the server key. The worker runs every 30 seconds and tries a failed message up to five times with growing delays. "Accepted by relay" means the relay accepted the message, not that it reached the mailbox. A message whose sending was interrupted becomes **Acceptance uncertain** and is not retried automatically: check the relay's records, then retry (which may deliver a duplicate) or **Dismiss without resending**. Finished entries are deleted after **Completed email history retention (days)** (730 by default); the deletion pauses while any legal hold exists.

## Options

| Setting | What it does | Default |
| --- | --- | --- |
| **Public policy pages** ([`policy.publicPages`](/admin/settings#policy.publicPages)) | Serves `/privacy` and `/cookies-policy` | on |
| **Automatic policy versions** ([`policy.autoVersion`](/admin/settings#policy.autoVersion)) | Records a version when the policy changes | on |
| **Ask for cookie consent again on a new policy version** ([`policy.reconsentOnChange`](/admin/settings#policy.reconsentOnChange)) | Raises the cookie **Policy version** with each version | off |
| Operator fields (`policy.controller.*`, [`policy.legalBases`](/admin/settings#policy.legalBases), `policy.hosting.*`, [`policy.supervisoryAuthority`](/admin/settings#policy.supervisoryAuthority)) | Facts used in the policy and the records | empty |
| **Suggest impact assessments** ([`dpia.suggest`](/admin/settings#dpia.suggest)) | Lists suggestions | on |
| **Impact assessment review interval** ([`dpia.reviewMonths`](/admin/settings#dpia.reviewMonths)) | 30-day months to the review date on approval | 12 |
| **Impact assessment reminder window** ([`dpia.reminderDays`](/admin/settings#dpia.reminderDays)) | Days before the review date an assessment is marked due | 30 |
| **Enable email** ([`compliance.mail.enabled`](/admin/settings#compliance.mail.enabled)) and the relay settings (`compliance.mail.*`) | Outbound compliance email | off |
| **Require proxy route** ([`compliance.mail.requireProxy`](/admin/settings#compliance.mail.requireProxy)) | Sends only through a `system_mail` proxy route | on |
| **Completed email history retention (days)** ([`retention.complianceMail.days`](/admin/settings#retention.complianceMail.days)) | Keeps finished entries | 730 |

The `policy.*` settings can also be set per domain; the `dpia.*`, `compliance.mail.*` and retention settings are server-wide.

## Related

- [Data requests](data-requests.md)
- [Breach register](breach-register.md)
- [Retention](retention.md)
- [Security model](/security/README)
