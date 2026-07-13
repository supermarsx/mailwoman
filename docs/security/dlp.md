# Data-loss prevention (DLP)

V4 adds an engine-side **outbound** DLP pipeline. Rules are evaluated on the
`EmailSubmission/set` path (before the message is handed to the submitter) and,
for a pre-send heads-up, on the compose-time `Dlp/scan` dry-run. Every evaluation
writes a **redacted** audit row (the matched detector + rule, **never** the matched
content).

In V4 the rules are **configuration/environment-sourced** — the admin panel proper
is a later milestone (V6). Both the engine (enforcement) and `mw-server`'s
`GET /api/security/dlp/config` (read-back) load the *same* value, so configuration
and enforcement cannot drift.

## `MW_DLP_RULES`

`MW_DLP_RULES` is either:

- a **path** to a JSON file containing an array of rules, or
- an **inline JSON** array (the string starts with `[`).

Unset, empty, or unparseable → no rules (allow-all). It is read per evaluation, so
you can update the file without restarting.

Each rule is a `DlpRule` (camelCase over the wire):

```json
[
  {
    "id": "rule-pan",
    "name": "Block card numbers",
    "enabled": true,
    "priority": 10,
    "conditions": {
      "detectors": ["pan"],
      "customRegex": null,
      "dictionaries": [],
      "attachmentTypes": [],
      "maxAttachmentSize": null,
      "recipientDomains": [],
      "recipientDomainMode": null,
      "classification": null
    },
    "action": "block",
    "message": "This message appears to contain a payment card number."
  }
]
```

### Fields

| Field | Meaning |
|-------|---------|
| `id` | Stable rule identifier (also recorded in the audit row). |
| `name` | Human-readable label shown in the compose warning + audit. |
| `enabled` | Disabled rules are skipped. |
| `priority` | Lower numbers evaluate first. |
| `action` | `warn` \| `block` \| `require-encryption` \| `notify-admin`. |
| `message` | The text surfaced to the sender / recorded with the audit. |

### Conditions

A rule matches when its configured conditions match the outbound message. All
configured facets must hold (an empty/`null` facet is not evaluated).

- `detectors` — built-in content detectors:
  - `pan` — payment card number (13–19 digit run, separators allowed,
    **Luhn-validated** to cut false positives).
  - `iban` — IBAN (format + **mod-97** check).
  - `ssn` — US SSN (`NNN-NN-NNNN`).
  - `national-id` — a generic 9+ digit identifier run.
  - `custom-regex` — matches `customRegex` (a Rust `regex`).
- `customRegex` — the pattern used by the `custom-regex` detector.
- `dictionaries` — named keyword dictionaries to match.
- `attachmentTypes` — MIME types / extensions to match on attachments.
- `maxAttachmentSize` — flag attachments larger than this many bytes.
- `recipientDomains` + `recipientDomainMode` — gate on recipient domain;
  `recipientDomainMode` is `"in"` (match if a recipient domain is in the list) or
  `"notIn"` (match if a recipient domain is **not** in the list).
- `classification` — match a message classification/label string.

## Actions

- **`warn`** — surfaced inline in Compose (via `Dlp/scan`) as a soft, dismissible
  warning; the send proceeds. This is the recommended default over `block`.
- **`block`** — `EmailSubmission/set` fails with a structured
  `notCreated: { type: "dlpBlocked", description, verdicts: [DlpVerdict] }` and the
  message is **not** sent. Compose also pre-gates the send.
- **`require-encryption`** — surfaced as a confirm gate encouraging the sender to
  enable end-to-end encryption before sending.
- **`notify-admin`** — records the audit row (and, if configured, an abuse-address
  notice) without blocking.

## Audit

Every matched rule writes a `dlp_audit` row: timestamp, account, rule id + name,
action, matched detector names, and a `blocked` flag. **The matched content is
never stored** — redaction goes through `mw-store`'s `redact.rs`, and an engine
test asserts a seeded card number appears nowhere in the audit row.
