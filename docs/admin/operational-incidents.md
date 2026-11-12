# Operational incidents and NIS2 preparation

Administration → Compliance → Operational incidents covers outages, suspected
account compromise, service disruptions and vulnerability incidents. It is
separate from the personal-data breach register: an event can belong in both.
All-domain `privacy.read` and `privacy.write` grants control access. Writes,
observations, exports and recorded submissions are audited.

## Automatic detection and recovery

`incidents.auto.enabled` defaults to true. The existing health probe cycle feeds
observations approximately once a minute; no additional network probes are
introduced. Three consecutive failed samples (configurable from 2 to 30) open
one incident per component episode. Samples less than 30 seconds apart are not
counted twice. Warnings, disabled and unknown states never count as healthy or
failed samples. Probe failure is a service-disruption candidate, not proof that
all users experienced an outage.

Two healthy observations record observed recovery and change the incident to
recovering. A further failure interrupts that recovery. Root cause and recovery
evidence are required before an operator can resolve or close an incident;
healthy samples never certify recovery. RTO and RPO targets can be recorded in
minutes. Observed intervals do not establish the amount of lost data.

Session-security audit signals for impossible travel, concurrent use and rotated
session-token reuse create suspected account-compromise candidates. References
to original audit events are retained instead of duplicating account names,
IP addresses or tokens. Existing session-security policies continue to govern
step-up and revocation. The incident worker does not introduce account lockout.

The security-event cursor persists both timestamp and ID, avoiding loss at
same-timestamp page boundaries. A fresh scanner starts from now without replaying
historical events. Deterministic IDs and database compare-and-swap deduplicate
observations and workers.

A heartbeat gap over three minutes creates a monitoring-gap candidate on the
next pass. It can result from planned downtime, a stopped monitor, database
unavailability or an outage; availability during that interval is unknown.
A process cannot observe its own complete outage. An external monitor can POST
`{ "component": "mail", "status": "fail" }` to `/admin/incidents/observations`
(use your configured console path). It needs an all-domain `privacy.write` grant.
States are `ok`, `fail`, `warn` or `disabled`. Names are bounded identifiers.
External components are namespaced separately from built-in probes. The server
stamps observations and applies the same debounce rules.

## Reporting and reminders

Automatic detections start unassessed and do not start a legal reporting clock.
An operator records significance and actual awareness time. For significant
incidents, the general NIS2 preparation clock shows 24 hours for early warning
and 72 hours for incident notification. `incidents.nis2.trustService` selects the
24-hour notification deadline for the relevant trust-service case.

Final/progress milestones use calendar-month arithmetic from the actual incident
notification submission, not a fixed 30-day interval. Ongoing incidents need a
progress report at that milestone. If handled by that date, the final report keeps
the original deadline. If still ongoing at that date, final reporting is due one
calendar month after handling. Record actual handling time separately from the
time recovery was verified; it defaults to verification time when omitted. Actual submissions
need an authority reference and timestamp. The application never submits to a
regulator or determines applicability, legal significance, DORA equivalence or
country-specific reporting rules.

Six-hour due reminders are recorded in the incident timeline and audit log.
Optional `incidents.notify.email` sends detection, observed/verified recovery and
reporting reminders to `incidents.notify.operatorAddresses`, through the compliance
SMTP service and its proxy policy. Persistent dispatch markers prevent repeating
old notices after email-history retention expires. Configure and test the relay
before relying on email; it is off by default.

JSON exports include incident details, reporting deadlines, actual submission
references and recovery history. They are preparation documents, not authority
submission forms. Concurrent edits require the current incident version.

Closed records expire after `incidents.retention.days` (1095 by default), subject
to the enforced long-class ceiling. Open records are retained. Subject legal
holds conservatively pause deletion. Monitor counters and cursors contain service
identifiers and timestamps. Timelines retain the latest 500 entries and reporting
history is bounded to 100 entries; the separate audit log has its own retention.
Avoid putting secrets or unnecessary personal data into operator notes.

References: [NIS2 Article 23](https://eur-lex.europa.eu/eli/dir/2022/2555/oj/eng),
[Commission FAQ](https://digital-strategy.ec.europa.eu/en/faqs/directive-measures-high-common-level-cybersecurity-across-union-nis2-directive-faqs).
