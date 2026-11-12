# Governance registers

Administration → Compliance contains **Contracts**, **Data transfers**, **Control
mappings**. Reading and writing these areas require
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
