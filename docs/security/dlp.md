# Data-loss prevention (DLP)

Data-loss prevention checks outgoing mail against rules an administrator sets, and
can warn the sender, refuse the message, or tell an administrator. The checks run on
the server, in engine mode, when a message is submitted and again just before it is
handed to the SMTP server. In JMAP-proxy mode the upstream server sends the mail and
these rules do not apply.

## Where rules come from

The rules in force are the rules in the `MW_DLP_RULES` environment variable followed
by the rules stored in the admin security policy (**Security policy** in the admin
console, or `mailwoman admin security-policy set --dlp-rules-json`). When a stored rule
has the same `id` as an environment rule, the environment rule is used and the stored
one is ignored, so the console cannot replace or remove an environment rule. The
console lists environment rules under **Rules from MW_DLP_RULES** and marks a shadowed
stored rule.

<!-- screenshot: id=security/dlp-rules-editor; screen=Admin > Security policy; state=one stored rule expanded, one environment rule listed; element=the DLP rules group -->

`MW_DLP_RULES` is either inline JSON (a value that starts with `[`) or a path to a
JSON file holding the rule list. The file is read on every evaluation, so you can
change it without a restart. Limits: 2 MiB for the environment value or file, 8 MiB
for the stored policy row.

The rules fail closed. If `MW_DLP_RULES` names a file that cannot be read, or either
source does not parse, every submission is refused with a `serverFail` error whose
`code` is `dlpPolicyUnavailable`; the message is not sent. An unset or empty
`MW_DLP_RULES` with no stored rules means no rules, and mail is sent unchecked. The
admin API refuses stored rules that do not parse, whose `customRegex` does not
compile, or whose `action` is not one listed below.

`GET /api/security/dlp/config` returns the merged list with each rule marked
`source: "env"` or `source: "admin"`.

## Rule format

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

| Field | Meaning |
|-------|---------|
| `id` | Rule identifier, recorded in the audit row. |
| `name` | Label shown to the sender and recorded in the audit row. |
| `enabled` | Disabled rules are skipped. |
| `priority` | Order of the returned verdicts; higher numbers first. Every matching rule produces a verdict, so priority does not stop later rules. |
| `action` | `warn`, `block`, `require-encryption` or `notify-admin` (`notify` is accepted as a synonym). |
| `message` | Text shown to the sender when the rule matches. |

## How a rule matches

The server checks the message subject and its text body parts. It does not read the
contents of attachments; for attachments it sees only the name, the MIME type and the
size.

A rule first applies its recipient gate, then matches if **any** of its content
conditions hits:

- `recipientDomains` with `recipientDomainMode`: `"in"` applies the rule when any
  recipient's domain is in the list; `"notIn"` applies it when any recipient's domain
  is outside the list (so adding one internal recipient does not exempt a message that
  also goes outside). Domains compare case-insensitively. A rule with recipient
  domains and no content conditions matches on the gate alone.
- `detectors`, built-in checks over subject and body:
  - `pan`: a 13 to 19 digit run (single spaces or dashes allowed between digits) that
    passes the Luhn check.
  - `iban`: a single unbroken token of 15 to 34 letters and digits (two letters, two
    check digits, then the account part) that passes the mod-97 check. An IBAN written
    in spaced groups (`PT50 0002 …`) is not detected.
  - `ssn`: the US `NNN-NN-NNNN` shape; the separators may be dashes, spaces or absent.
  - `national-id`: any run of 9 or more consecutive digits. This is broad and will
    match order numbers and phone numbers.
  - `pt-nif`, `pt-niss`, `pt-citizen-card`: Portuguese tax number, social security
    number and citizen card number, with their check digits.
  - `credentials`: words such as "password", "api key", "private key", "recovery
    code" and their Portuguese equivalents, as whole words.
  - `custom-regex`: the rule's `customRegex` (Rust `regex` syntax).
- `customRegex`: also checked when set without listing `custom-regex`.
- `dictionaries`: a list of literal words or phrases, matched case-insensitively as
  whole words (each entry is a term, not the name of a word list).
- `classification`: a label such as `confidential`, matched as a whole word in the
  subject or body.
- `attachmentTypes`: matches an attachment whose MIME type equals an entry, or whose
  file name ends with it.
- `maxAttachmentSize`: matches an attachment larger than this many bytes.

These are pattern checks, not a guarantee that sensitive data is caught. Text inside
attachments, images, archives, or an end-to-end-encrypted body is not inspected,
because on the server an encrypted body is ciphertext.

## Actions

| Action | Label in the rule editor | What happens |
|---|---|---|
| `warn` | **Warn the sender** | Compose shows the warning (**Heads up**); the message can be sent. |
| `block` | **Block the message** | The server refuses the submission with a `dlpBlocked` error listing the matching rules; Compose shows **Sending blocked**. |
| `require-encryption` | **Require encryption** | Treated as a block on the server: a message whose readable content matches is refused. Compose shows **Encryption required**. Encrypting the message end to end turns the body into ciphertext, which the body detectors cannot match. |
| `notify-admin` | **Send and notify an administrator** | The message is sent. If [`dlp.adminAddress`](/admin/settings#dlp.adminaddress) (`MW_DLP_ADMIN`) is set, the server emails that address a notice naming the rules and detectors that matched, never the matched text or the message body. Sending the notice is best effort. |

Compose runs the same rules as a dry run (`Dlp/scan`) while you write, so warnings
appear before you press Send. The dry run is advisory: the server checks again at
submission and at dispatch, so a modified client cannot skip a `block`.

If the server cannot finish the inspection (the message is over 64 MiB, cannot be
parsed, or the inspection workers are busy), the submission is refused rather than
sent unchecked; a busy refusal is marked retryable.

## Audit records

The server writes one `dlp_audit` row per matching rule: time, account, rule id and
name, action, the detector names that matched, and whether the message was blocked.
The matched text is not stored; the verdict's excerpt is the fixed string
`•••• redacted (<detector names>)`. Rows are written when a submission is blocked
and, for every matching rule, when a message is dispatched. The compose dry run
writes nothing.

Rows are kept for [`retention.dlpAudit.days`](/admin/settings#retention.dlpaudit.days)
days (default 1825; 0 keeps them forever).

## Related

- [Admin panel: security policy](./admin-panel.md#security-policy)
- [Message encryption](./crypto.md)
- [Settings reference](/admin/settings)
