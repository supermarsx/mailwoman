---
title: Logs
screen: true
---
# Logs

**Logs** (under **Operations** in the console navigation) has two tabs. **Authentication logs** lists sign-in attempts to webmail and to the administration console. **Application logs** lists the warnings and errors that this server process wrote recently. Use the first to see who tried to sign in and why an attempt failed, and the second to see what the server itself reported.

![The Authentication logs tab of Logs: the filter form and a table of sign-in attempts with area, backend, result, reason, method and source address.](/screenshots/admin-ops/opsa-logs-authentication.webp){.mw-shot}

## Before you start

- You need the `logs.read` permission, and it must not be limited to some domains. An administrator whose role covers only some domains does not see **Logs**.
- Both tabs are read-only.

## Look up a sign-in attempt

1. Open **Logs**. The **Authentication logs** tab opens first and lists the newest attempts.
2. Fill in any of the filters, then choose **Apply filters**.

   ![The filter form of the Authentication logs with markers 1 to 6 on the search box, Sign-in area, Result, Backend, Reason and Apply filters.](/screenshots/admin-ops/opsa-logs-filters.webp){.mw-crop}

   1. **Account, attempted username or IP** matches part of the account, the name typed, or the source address (up to 128 characters).
   2. **Sign-in area** is **All**, **Webmail** or **Administration**.
   3. **Result** is **All**, **Success**, **Failure** or **Blocked**.
   4. **Backend (exact identifier)** is the exact name of the mail backend the attempt used. Administrator sign-ins use `admin-local`.
   5. **Reason (exact code)** is one of the codes in the table below.
   6. **Apply filters** runs the search.
   {.mw-callouts}

3. Read the table. Each row shows **First attempt**, **Account or attempted username**, **Sign-in area**, **Backend (exact identifier)**, **Result**, **Reason (exact code)**, **Method**, **Source IP** and **Attempts**. Repeated failures of the same kind inside one minute are one row, and **Attempts** counts them. Webmail rows also have a **Device details** disclosure.
4. Choose **Older attempts** below the table to load the next 50 rows.

The page text says how long entries are kept (**Authentication metadata kept for up to 1,095 days, subject to the entry limit.**). Rows are removed when they are older than the limit or when the table exceeds the maximum number of entries.

### Reason codes

The **Reason** is set only when the server has typed evidence for it. A failure the server cannot classify has a generic code, and the history never claims the password was wrong unless the mail server said so.

| Code | Meaning |
|---|---|
| `auth-rejected` | The mail server (or, for the console, the administrator login) refused the credentials. |
| `not-configured` | No usable mail backend is configured for the address. |
| `upstream-policy` | A policy refused the connection or the lookup, for example the outbound address policy. |
| `account-disabled` | The account is disabled. |
| `new-user-refused` | The sign-in setting for new users refused to create the account. |
| `dns` | The mail server's name did not resolve. |
| `connect` | The connection to the mail server was refused, reset or could not be made. |
| `timeout` | The mail server did not answer in time. |
| `certificate` | The mail server's certificate was not accepted. |
| `tls` | The TLS handshake with the mail server failed for another reason. |
| `protocol` | The mail server answered in a way the server could not use. |
| `unsupported` | The mail server does not support what the sign-in needed. |
| `transport` | A network error without a more specific cause. |
| `storage` | The server could not read or write its own data. |

## Read the application logs

1. Open the **Application logs** tab.
2. Use **Search redacted details**, **Component** (the name of the part of the server that wrote the entry) and **Severity** (**All**, **WARN** or **ERROR**), then choose **Apply filters**.
3. Read the table: **First attempt**, **Severity**, **Component**, **Redacted details** and **Attempts**. Entries that are identical within one minute are one row.

![The Application logs tab: a note on what is captured, the count of discarded entries, search, component and severity filters, and warnings captured since the server started.](/screenshots/admin-ops/opsa-logs-application.webp){.mw-shot}

What the tab holds:

- Only warnings and errors from this server process, and only those that pass the process's log filter (`MW_LOG`, or `RUST_LOG` when it is set).
- At most 1,000 entries from the last 24 hours. **Entries discarded from this process buffer** counts the entries pushed out when the buffer was full.
- The list is kept in memory. It is empty after a restart and, with several servers behind one address, shows only the server that answered.
- Passwords, tokens and message content are redacted before an entry is stored, and fields that are too large are left out. Do not write secrets into log lines you add.

For anything older, or for the other servers, use your own log collection.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| [`logs.auth.retentionDays`](/admin/settings#logs.auth.retentiondays) | Days of sign-in history kept (1 to 1095). | 1095 | Administrators with settings access (global) |
| [`logs.auth.maxEntries`](/admin/settings#logs.auth.maxentries) | Maximum sign-in entries kept (1,000 to 100,000). | 50000 | Administrators with settings access (global) |

Both settings are read each time a sign-in is recorded and each time the tab is opened. Removing old history does not affect the sign-in security checks that count recent failures.

## Troubleshooting

- **The tab says "Authentication history is unavailable on this server."** The server does not provide the history. The same applies to **Application logs are unavailable on this server.** for the second tab.
- **A failed attempt shows `auth-rejected`, but the user says the password is right.** The mail server refused it. Check the mail server's own log and the domain's backend settings under **Domains**.
- **Many `dns`, `connect`, `timeout` or `certificate` rows for one backend.** The server cannot reach or trust that mail server. Run **Diagnostics** and look at **Mail backends**.
- **Application logs is empty after a restart.** It is meant to be: the list lives in memory.
- **The same warning appears on the first start:** `the admin console is at [admin-path], the first path a scanner tries` means the console is still at its default address. Give it its own path in **Security centre**, with `mailwoman admin console-path generate`, or with `MW_ADMIN_PATH`.

## Related

- [Diagnostics](/admin/diagnostics)
- [Observability](/admin/observability)
- [Sign-in locks](/admin/signin-locks)
- [Settings reference](/admin/settings)
