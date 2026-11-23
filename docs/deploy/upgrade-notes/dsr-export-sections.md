# Upgrade note: seven more sections in data exports

Applies to every deployment that handles data requests (**Data requests** in the admin console).

## What changes

- **Exports carry seven more sections.** Earlier releases built the `mailstate`, `tasks`, `security`, `integrations`, `delegation`, `usage` and `records` sections but always left them out, because their switches were not registered and read as off. Each now has a switch (`dsr.export.include.mailstate` and so on, **Export: tasks** and so on in the console), on by default. See [What an export contains](../../admin/data-requests.md#what-an-export-contains) for the files and what they hold.
- **Exports get larger.** The new sections are JSON metadata (tags, tasks, key and token metadata, connected accounts, delegations, usage counters, restrictions, retention rules and similar), so most exports grow by a small amount. Tasks carry their iCalendar text. A section that would go over **Export size cap** is left out and listed with reason `sizeCap`, as before.
- **What is not exported.** Secrets and sealed values appear only as markers (`present, not exported`). Webhook URLs and EWS endpoint URLs are exported as `scheme://host/path`, without user name, password, query string or fragment. A delegation the person received does not name the owner's account id.

## Existing requests

An export that is already ready, downloaded or expired is not rebuilt and does not gain the new sections. An export that is still being prepared when the server is upgraded keeps the sections it has already stored; sections it had already recorded as switched off stay switched off for that request.

## What to do

Nothing, if the new sections should be in exports. To keep one out, switch it off in the console settings, at server or domain level: for example **Export: security records** ([`dsr.export.include.security`](../../admin/settings.md#dsr.export.include.security)).
