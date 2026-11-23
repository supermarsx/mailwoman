---
screen: true
---
# Operational incidents

The **Operational incidents** tab of **Compliance** records outages, service disruptions, suspected account compromise and vulnerabilities, with the time you became aware, the recovery, and the reports you made to an authority. The server opens candidate incidents from its own health checks and session-security signals. It never submits anything to an authority, and it does not decide whether an incident is significant or which reporting rules apply. Personal-data breaches have their own [Breach register](breach-register.md); one event can belong in both.

![The Operational incidents tab of Compliance with one active incident, and the buttons to add an incident and check signals now.](/screenshots/admin-privacy/incidents-list.webp){.mw-shot}

## Before you start

Reading the tab needs `privacy.read`, changing it `privacy.write`, both on all domains. Creating, editing, recording a report and exporting are recorded in the audit log.

## Record an incident

1. Open **Compliance** and select **Operational incidents**, then **Add incident**.

   ![The form for a new operational incident with its title, category, status, significance, awareness time, owner, impact and recovery fields.](/screenshots/admin-privacy/incidents-editor.webp){.mw-crop}

2. Enter the **Title**, **Category** (Outage, Suspected account compromise, Service disruption or Vulnerability), **Status**, **Significance**, **Awareness time**, **Owner** and **Impact**.
3. As work goes on, fill in **Root cause**, **Measures taken**, **Recovery plan**, **Recovery evidence** and, if you track them, the recovery time and recovery point in minutes. Resolving or closing an incident by hand needs a root cause and recovery evidence.
4. Under **Regulatory reporting**, use **Record authority submission** for each report you made, with the time and the authority's reference.
5. **Export report** saves the incident as JSON.

## Automatic detection

With **Automatically detect incident candidates** on (the default), a worker runs every minute and opens candidates from:

- **Health checks.** The server's health probes run about once a minute. **Failed health samples before detection** (3 by default, 2 to 30) consecutive failures of one component open one service disruption. Samples less than 30 seconds apart are ignored, and a warning or a disabled state resets the count. Two healthy samples set the incident to **Recovering**.
- **Session security.** Session alerts for impossible travel, concurrent use and reuse of a rotated session open a suspected account compromise, with a reference to the audit entry. It is not a confirmed compromise.
- **Monitoring gaps.** A gap of more than three minutes in the server's own heartbeat opens a candidate; the service's state during the gap is unknown.
- **External monitors.** `POST /admin/incidents/observations` (under your console path) with a component name and a state (`ok`, `fail`, `warn` or `disabled`) counts like a health sample. It needs `privacy.write` on all domains and is refused while automatic detection is off.

**Check signals now** runs the worker at once. The settings are under **Automation and notifications** on the tab:

![The Automation and notifications settings of the Operational incidents tab, opened.](/screenshots/admin-privacy/incidents-automation.webp){.mw-crop}

::: warning Short disruptions are closed automatically
With `incidents.auto.resolveTransient` on (the default), a service disruption from the health checks that recovers within `incidents.auto.transientMinutes` (15) with at least `incidents.auto.recoverySamples` (2) healthy samples, while still unassessed, unowned and without reports, is set to **Resolved** by the server. Its recovery is recorded as verified, with the owner "Automated health monitor" and generated recovery evidence, and no root cause. These three settings are not shown on the tab; change them on the **Settings** screen.
:::

## Deadlines and reminders

Deadlines apply only to an incident marked **Significant** that has an awareness time: an early warning 24 hours after awareness, the incident notification 72 hours after (24 hours with **Use trust-service reporting deadline**), and progress and final reports one calendar month later. The worker records a reminder in the timeline when a deadline is less than six hours away. With **Email incident updates and reminders** on, each step is also sent once to the **Operator email recipients** through the compliance email relay (see [Compliance: Email notifications](compliance.md#email-notifications)); short disruptions that may be closed automatically are not emailed.

## Retention

Closed incidents are deleted `incidents.retention.days` (2555 by default) after closing, held to the long retention class when classes are enforced. Deletion pauses while any legal hold exists. A timeline keeps its latest 500 entries and the reporting history 100. See [Retention](retention.md).

## Options

| Setting | Default |
| --- | --- |
| **Automatically detect incident candidates** ([`incidents.auto.enabled`](/admin/settings#incidents.auto.enabled)) | on |
| **Failed health samples before detection** ([`incidents.auto.failureSamples`](/admin/settings#incidents.auto.failureSamples)) | 3 |
| [`incidents.auto.resolveTransient`](/admin/settings#incidents.auto.resolveTransient), [`incidents.auto.transientMinutes`](/admin/settings#incidents.auto.transientMinutes), [`incidents.auto.recoverySamples`](/admin/settings#incidents.auto.recoverySamples) | on, 15, 2 |
| **Use trust-service reporting deadline** ([`incidents.nis2.trustService`](/admin/settings#incidents.nis2.trustService)) | off |
| **Email incident updates and reminders** ([`incidents.notify.email`](/admin/settings#incidents.notify.email)) | off |
| **Operator email recipients** ([`incidents.notify.operatorAddresses`](/admin/settings#incidents.notify.operatorAddresses)) | empty |
| **Closed incident retention (days)** ([`incidents.retention.days`](/admin/settings#incidents.retention.days)) | 2555 |

All are server-wide.

## Related

- [Breach register](breach-register.md)
- [Compliance](compliance.md)
- [Retention](retention.md)
