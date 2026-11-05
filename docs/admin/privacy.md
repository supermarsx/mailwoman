# Privacy and attachment policy

Admin console sections: **Privacy** (`#/privacy`) and **Attachments** (`#/attachments`).

## Privacy tabs

Privacy has one tab per topic, each with its own address, so a tab can be bookmarked:

| Tab | Address | Needs | Contents |
| --- | --- | --- | --- |
| Overview | `#/privacy` or `#/privacy/overview` | `egress.read`, not domain-scoped | Status, key remote-content settings, key figures, history charts |
| Remote content and trackers | `#/privacy/remote` | `settings.read` | `privacy.remoteContent`, tracker blocking, link cleaning, direct loading, manual downloads, the loading toast, outgoing mailer headers |
| Image proxy | `#/privacy/image-proxy` | `egress.read` or `settings.read`, not domain-scoped | Worker history charts, tier readout, the capacity mode card, and every `imageProxy` setting grouped as request rate, cache, memory tier, disk tier, tiering |
| Fetch limits | `#/privacy/fetch-limits` | `settings.read` | Timeouts, redirects, parallel fetches, queue, retries, size and rate limits |
| Traffic | `#/privacy/traffic` | `egress.read`, not domain-scoped | Allowed, refused and failed requests per traffic class, cache size, tracking pixels replaced, traffic charts |

A tab you cannot open is not listed. Opening its address shows "You do not have access to this tab."

`#/privacy/policy/<key>` links from earlier versions still work: they open the tab that edits the key and move focus to its row. A `privacy.*` key that no tab owns opens the single-setting page as before.

## Editing in place

Every setting on these tabs is edited where it is listed. Each setting shows:

- its name, key and the source that decides the value now (default, stored, locked, pinned by an environment variable);
- the value that is in effect;
- the value editor for its type;
- "Users may change it" and the lock, where the setting has narrower scopes;
- **Save**, **Discard changes**, **Revert to default** and, behind "Per-domain values", the domain overrides.

Saving sends the stored version with `If-Match`. If someone else changed the setting first, the card keeps your edit and offers Reload or Keep mine. Locking a setting, or turning off user choice, over existing narrower values first shows a dry run of what would be overruled. Every write is recorded in the audit log like a write from the Settings page.

Without `settings.write`, values are shown and the controls are disabled with the reason. A setting pinned by an environment variable is read-only and names the variable. A setting another section owns links to that section.

While any card has unsaved edits, the page stops refreshing the statistics and the console asks before you leave.

## Image proxy history

`GET /admin/egress/privacy-stats` returns the current figures and a `history` object. `?range=15m|1h|6h|24h` picks the window; any other value reads as `1h`.

- The server takes one sample every 15 seconds and keeps 24 hours (5,760 samples) in a ring buffer in memory.
- **The history is empty after a restart** and is not shared between server processes. Each process answers for itself.
- A response carries at most 120 points per series. Longer ranges are averaged or summed over wider steps (`stepSecs`), so the size does not grow with the range.
- Series: images and bytes served per second, memory and disk hit rate, memory and disk size and entries, fetches in flight and waiting, average rate-limit wait, `429` and `503` answers, burst allowance and active accounts, tracking pixels replaced, and allowed, refused and failed requests per traffic class.
- A counter that goes backwards (a reset) leaves a gap in the chart. It never draws a negative rate.

The charts are SVG with the console's theme colours and no animation. Each has a summary sentence and a "View as table" switch; the table is always in the page for screen readers. The Image proxy and Overview tabs show a range selector, a Refresh button and the time of the last update.

## Remote-content links

The reader loads remote images, inline parts and downloads of linked files through `/api/rc/i/<handle>`, `/api/rc/p/<handle>`, `/api/rc/f/<handle>` and `/api/rc/d/<handle>`. A handle is 22 URL-safe characters. The claims behind it (account, session, message, target, expiry) are stored on the server, in the shared `settings` table under `rc.h.<handle>`, so any replica sharing the database can resolve it and the link length does not depend on the remote URL.

- Lifetimes are unchanged: one hour for an image, five minutes for a part or a file. A handle is checked against the same session, account, message and grant rules as before.
- The store holds at most 100,000 live handles. Expired rows are removed on the next signing request; if the cap is still exceeded, the rows closest to expiry are dropped first. A dropped handle answers `403 invalid or expired link` and the reader signs again.
- Links issued before this change (a long `payload.tag` token) keep working until they expire.
- A remote URL longer than 4096 bytes is not signed. `POST /api/rc/sign` lists it under `refused` with the reason `length` and a message, and `GET /api/image-proxy?url=` answers `414`. Download file names in `Content-Disposition` are cut to 100 characters.

## Attachment policy

The Attachments section has tabs: **Allow and block lists**, **Limits**, **Download and preview**, **Warnings** and **Test a file**. Every `attachments.*` setting is edited in place.

- **Lists** (allowed, blocked and warning extensions and MIME types) are chip lists. Extensions are normalised (`.PDF`, `*.pdf` and `pdf` all become `pdf`; `tar.gz` is allowed). MIME types are lowercased and may end in `/*` (`image/*`). Entries are checked with the same rules the server applies. Repeats, invalid entries and the limit of 100 rules per list are reported in words and nothing invalid is added.
- **Paste many** accepts rules separated by spaces, commas, semicolons or lines. **Import** reads a text file the same way; **Export** writes one rule per line.
- A list longer than eight rules gets a filter box.
- **Conflicts** between lists are shown under the field: a rule that is also blocked is blocked (blocked rules win); a blocked rule that also has a warning never shows the warning; allowed plus warning is allowed with a warning. `image/*` in one list overlaps `image/png` in another.
- **Maximum attachment size** takes an amount and a unit (B, KiB, MiB, GiB). Zero means no limit.
- Nested messages (`message/rfc822`, `.eml`) and TNEF files are refused whenever any restriction is on, because their contents cannot be listed.

### Test a file

`POST /admin/attachments/simulate` runs the server's own evaluator (`attachment_policy.rs`) on a file name, MIME type and size under the stored global policy and returns the decision (delivered, preview allowed, warning, reason code) and which rule lists matched. It needs `settings.read`, changes and stores nothing, and never reads a message or attachment. The request may carry a `draft` object with any of the `attachments.*` fields (`blockedExtensions`, `maxBytes`, ...) to try an unsaved policy; the console does not send one. Draft rules are validated like saved ones.

Old links keep working: `#/settings/attachments.<key>` opens the Settings page for the key, and `#/attachments/attachments.<key>` opens the owning tab on that row.
