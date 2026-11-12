# Governance registers and email notifications

Administration → Compliance contains **Contracts**, **Data transfers**, **Control
mappings**, and **Email notifications**. Reading and writing these areas require
an all-domains administrator with `settings.read` or `settings.write`, respectively.
Writes and notification resolutions are audited. These records support an
operator's assessment; they do not establish certification or legal compliance.

## Contracts

Add the actual provider under Records of processing → Sub-processors first.
The guided contract form records the owner, counterparty, type (DPA, master
service agreement, SCC, other), linked sub-processor, purpose, signature and expiry
dates, review date, jurisdiction, HTTPS document link, clauses, and notes.

An active contract needs an owner, counterparty, signing date and document link.
Expiry cannot precede signing. A link records evidence; Mailwoman does not fetch
the document or decide whether its clauses are legally sufficient. Keep signed
documents and their version history in your controlled document repository.

## ICT contracts

The ICT service agreement type adds service description, supported business
functions, criticality, provider identifier, service/data locations, subcontractors
and their conditions, availability commitments, RTO/RPO, incident notice and
assistance, security requirements, access/audit rights, authority cooperation,
data access/return, termination notice, exit plans, resilience testing and supplier
concentration risk. Optional details are collapsed for ordinary contracts; saved
ICT detail is also readable by administrators without write permission.

Active ICT contracts require core service, location, security, data-return and
exit details. Critical/important services additionally require recovery targets,
incident assistance and notice, audit/authority rights, subcontracting conditions,
continuity testing and termination notice. These checks verify field presence and
numeric types, not contractual adequacy. They do not generate the prescribed DORA
register of information or replace a review of Article 30 terms.

## Internal and international transfers

The same register covers internal sharing and international transfers. Record
source/destination country codes, exporter/importer, data categories, purpose,
legal basis, owner, linked processor/contract, safeguards, assessment link and
review date. Use the international classification wherever the processing
requires international-transfer safeguards, including transfers within the same
corporate group. Country codes alone do not determine the legal assessment.

For an active international record, an international mechanism, safeguards and a
review date are mandatory. SCC, BCR and other mechanisms require an assessment
link; SCC transfers also require a linked contract. Choices include adequacy,
SCCs, BCRs, derogation and other mechanisms. Selecting one records the operator's
decision; it does not establish that the mechanism applies to this transfer.

Both registers have search, status filters and JSON/CSV/HTML exports. Review dates
in the past are flagged. CSV formula cells are escaped and HTML text is encoded.

## Control mappings

Fifteen starter controls link implementation references to GDPR articles, SOC 2
Trust Services Criteria and ISO/IEC 27001:2022 clauses/Annex A references. They
cover access, authentication, encryption, audit, retention, data requests,
consent, processing records, suppliers, transfers, incidents, risk assessment,
recovery, change management, and internal audit/management review.

Every starter begins **unassessed**. Adopt a starter or add a custom control and
record the owner, implementation, mappings, evidence, test date, review date and
notes. Statuses are unassessed, planned, implemented, verified by operator, and
exception. Verified requires an owner, evidence and a test date. Exceptions
require an owner, justification and review date. No code inspection, green build
or configuration setting automatically makes a control verified.

The starter catalog is a partial, editable cross-reference, not a complete
Statement of Applicability or an auditor-approved framework mapping. Validate
scope and references against your licensed standard and assessment criteria.
Sources: [EU GDPR](https://eur-lex.europa.eu/eli/reg/2016/679/oj),
[AICPA Trust Services Criteria](https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022),
[ISO/IEC 27001](https://www.iso.org/standard/27001).

## Outbound email setup

Outbound compliance email is **off by default**. Existing local-inbox, audit and
webhook channels remain independent.

1. Configure `compliance.mail.host`, `.port`, `.security` (`starttls` or `implicit`),
   `.from` and, when authentication is needed, `.username`.
2. Set `MW_COMPLIANCE_SMTP_PASSWORD` or `MW_COMPLIANCE_SMTP_PASSWORD_FILE` on the
   server. A password file takes precedence. The secret is never a settings
   value, returned by the admin API, or placed in a delivery audit entry.
3. Configure a global egress proxy route carrying **system_mail**. Proxy routing
   is required by default (`compliance.mail.requireProxy=true`). HTTP CONNECT,
   HTTPS CONNECT and SOCKS5 use pinned destination addresses. SOCKS5h is refused
   for this transport. A failed selected route never falls back to direct.
   Only explicitly setting `.requireProxy=false` allows an unproxied relay when
   no route applies. TLS certificate verification remains mandatory.
4. If the relay is on an operator-owned private network, explicitly enable
   `.allowPrivate`. Metadata, link-local and reserved addresses remain blocked;
   every DNS answer is checked before connecting.
5. Set `dsr.notify.operatorAddresses`, enable `compliance.mail.enabled`, and use
   **Send test email**. Check the delivery table before enabling event channels.

The separate channel switches are:

| Setting | Purpose |
| --- | --- |
| `dsr.notify.emailAdmins` | Incoming data requests, cancellations, due/overdue reminders and completion/blocking events. |
| `dsr.notify.emailRequester` | Requester progress/confirmation messages, independently of local-inbox delivery. |
| `breach.notify.email` | Existing breach operator events/reminders to `breach.notify.operatorAddresses`. |
| `breach.subjects.email` | Email to the affected audience selected through the breach notice preview/send workflow, including external addresses. |

This adds notifications when requests arrive through the existing data-request
workflow; it does not parse incoming email into new requests. Supervisory-authority
notices still require submission through the authority's own channel.

## Delivery evidence and recovery

Recipients, subjects and bodies are encrypted with authenticated, ID-bound
XChaCha20-Poly1305 in the durable queue. A database compare-and-swap claim prevents
two workers sending the same queued row. The worker runs every 30 seconds and
retries definite failures up to five attempts with increasing delays.

“Accepted by relay” means SMTP acceptance, not guaranteed final mailbox delivery.
Breach notice jobs and queued emails never count as evidence that people were
informed. The status and authority report require a successful individual channel
for every listed affected person, or an explicit “subjects-informed” timeline
record documenting notification outside Mailwoman. SMTP outcomes are appended to
the incident delivery ledger.
A timeout or loss of a worker after claiming a delivery is **uncertain** and is
not automatically retried. Check the recipient or relay records, then use the
confirmed Retry action (which may create a duplicate), or dismiss without
resending. Resolution is audited. Disabling an event channel before delivery
suppresses its queued messages; disabling the global mail switch pauses sending.

Completed, failed and dismissed history expires after
`retention.complianceMail.days` (30 days by default, subject to enforced retention
class ceilings). Pending/uncertain rows are kept for resolution. Automatic
history purging conservatively pauses while subject legal holds exist. The
requester export includes their delivery metadata, and erasure removes and
checks their queued/history rows. Message bodies and verification codes are not
included in the metadata export.

The SMTP relay is a processing recipient. Enabling it updates the generated
processing activity and policy facts. Add its actual operator, contract, location
and transfer safeguards to the registers before use.
