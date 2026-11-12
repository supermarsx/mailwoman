# Compliance

The **Compliance** section of the admin console (`#/compliance/<tab>`) collects the data-protection controls in one place. Every control here is a setting or a record you maintain. Nothing in the console states that a configuration satisfies a particular law; that judgement is yours.

The sidebar entry opens an Overview with one card per tab. A card shows a one-line status where the tab has one (for example, how many operator fields are still blank). The tabs are:

| Tab | Id | What it holds |
| --- | --- | --- |
| Data requests | `data-requests` | Export and erasure requests ([data-requests.md](data-requests.md)). |
| Cookies | `cookies` | Cookie inventory, categories, consent banner and log ([cookies.md](cookies.md)). |
| Retention | `retention` | Retention per store, the sweep and log rotation ([retention.md](retention.md)). |
| Policy | `policy` | The generated privacy and cookie policy (below). |
| Records of processing | `processing-records` | Processing activities and sub-processors (below). |
| Impact assessments | `dpia` | The DPIA register (below). |

Contracts, internal/international transfers, control mappings and outbound email
notifications are described in [Governance registers and email](compliance-governance.md).
The Compliance hub also contains restrictions, consents, breaches, erasure coverage
and encryption.

The tab registry lives in `apps/web/src/screens/Admin/Compliance/tabs.ts`.

The older addresses keep working: `#/data-requests`, `#/retention` and `#/privacy/cookies` open the matching Compliance tab.

## Privacy policy and cookie policy

The server writes the policy from the live configuration, in English and European Portuguese. The text covers who is responsible, what is processed, purposes and legal bases, cookies and browser storage (from the built-in inventory and your classification rules), retention per store (from the `retention.*` settings), data subject rights (from the `dsr.*` settings), remote content handling, recipients (the outbound traffic classes), transfers, security measures and contact details. Optional features such as the assistant or open tracking appear only while they are switched on.

### Operator fields

Some facts the server cannot know. These settings hold them and the Policy tab lists the ones still blank as a warning, with an editor next to each:

- `policy.controller.name` and `policy.controller.address`
- `policy.legalBases`
- `policy.hosting.provider` and `policy.hosting.transfers`
- `policy.supervisoryAuthority`

The data protection contact comes from `dsr.contact.name`, `dsr.contact.email` and `dsr.contact.url`.

### Overrides

Any section can be replaced with your own text, separately for each language. A replaced section stops showing its warnings. Removing the override returns to the generated text. Overrides are plain text with blank lines between paragraphs, `- ` for list items and `**bold**`.

### Versions and publishing

Each recorded version stores a snapshot of the facts and overrides it was written from. The public pages show the last recorded version, not the live configuration.

- With `policy.autoVersion` on (the default), the first view of the Policy tab or of the public page after a policy-relevant setting changes records a new version automatically. Version-only settings (`cookies.policyVersion`, the two `policy.*` switches) and cosmetic ones (cookie banner wording, image-proxy tuning) are not counted.
- With it off, changes are listed as pending and wait for **Publish a new version**.
- The version history lists every version with the settings that changed, their old and new values.
- With `policy.reconsentOnChange` on, recording a version after the first also raises `cookies.policyVersion` by one. Visitors are then asked for their cookie choices again if `cookies.repromptOnVersion` is on.

### Public pages

| Address | Content |
| --- | --- |
| `/privacy` | The privacy policy. |
| `/cookies-policy` | The cookie policy. |

Both need no sign-in. Add `?lang=pt-PT` (or `en`) to choose the language; without it the browser's `Accept-Language` is used. Add `?format=md` for Markdown or `?format=json` for the sections as data. The pages are linked from the sign-in page, the Settings privacy section and the cookie banner (the banner uses `cookies.policyUrl` when it is set). `policy.publicPages` switches them off; they then return 404.

## Records of processing

The tab lists processing activities. Built-in activities (mail service, security records, data requests, cookie consent records, the remote content proxy, the assistant, open tracking, service statistics) are generated from the configuration and are marked off while their feature is off. Each shows purpose, legal basis (from `policy.legalBases`), categories, recipients, transfers, retention and security measures. Add your own activities for processing the server cannot see. The recipient list is built from the outbound traffic classes; add the names of the providers you use as sub-processors.

Export as CSV, JSON or HTML (print-ready, landscape A4). CSV cells that start with a formula character are defused.

### What we process about you

`GET /api/privacy/processing` (signed-in users) returns, in the user's language: the controller and contact, the enabled activities with purpose, categories, recipients, retention and legal basis, the recipients, and the policy address. The privacy dashboard in Settings reads it.

## Impact assessments

The register holds data protection impact assessments. Each one has a description, purposes, necessity, proportionality, risks, mitigations, the data protection officer's opinion, consultation notes, attachments (links or text), a review date and a status.

- **Risk scoring.** Each risk has a likelihood and a severity from 1 to 4. The score is their product: up to 3 low, up to 6 medium, up to 9 high, above that very high. The same scale applies to the residual likelihood and severity after mitigation. A matrix shows which risks sit in which cell.
- **Status.** `draft`, `review`, `approved`, `needs-prior-consultation`. A draft goes to review; review goes to approved or needs-prior-consultation; an approved assessment returns to review. Approval requires residual scores on every risk and none in the very high band; a very high residual risk belongs under needs-prior-consultation.
- **Versions.** Every save is kept. The editor refuses a save made against an older version.
- **Templates.** Generic, AI assistant, email tracking, large-scale monitoring and new processor, in English and European Portuguese.
- **Suggestions.** With `dpia.suggest` on, the tab lists assistant, open tracking and analytics that are switched on with no assessment covering that feature.
- **Reviews.** Approving sets the review date `dpia.reviewMonths` months ahead. An assessment is listed as due from `dpia.reminderDays` days before that date, and as overdue after it.
- **Export.** HTML (print-ready) or JSON per assessment.

The register records your assessment. Whether one is required for a given processing is for you to decide.

## Settings

`policy.publicPages`, `policy.autoVersion`, `policy.reconsentOnChange`, `policy.controller.name`, `policy.controller.address`, `policy.legalBases`, `policy.hosting.provider`, `policy.hosting.transfers`, `policy.supervisoryAuthority`, `dpia.suggest`, `dpia.reviewMonths`, `dpia.reminderDays`. See [settings.md](settings.md).

## Data stored

Migration `0096_compliance_documents` adds `compliance_policy_version`, `compliance_policy_override`, `compliance_record`, `dpia` and `dpia_version`. They hold operator-maintained records and configuration snapshots.

## Permissions

Reading needs `settings.read` and writing `settings.write`, for all domains. Changes are audit entries (`policy-override-saved`, `policy-published`, `compliance-record-saved`, `compliance-record-deleted`, `dpia-created`, `dpia-saved`, `dpia-deleted`).
