---
title: Effective configuration
screen: true
---
# Effective configuration

**Effective configuration** (under **Configuration** in the console navigation) shows what this server is configured with: the environment variables that the process reads, a few build constants, and every setting of the settings registry with the value it resolves to now. It is a read-only report with secrets hidden. You change a value in the editor that owns it, not here.

![The Effective configuration page: the search and filter row and a table of settings with their reported value, source and how a change applies.](/screenshots/admin-ops/opsa-config-effective.webp){.mw-shot}

## Before you start

- You need the `config.read` permission without a domain restriction. A domain-only role does not see this page.
- Rows of an area are shown only if your role can also read that area: sign-in and administrator rows need `security.read`, integration rows `integrations.read`, mail rows `domains.read`, observability rows `observability.read`, and the settings registry rows `settings.read`.

## Find a setting

1. Open **Effective configuration**. The page lists every entry your role may read, 50 at a time; **Show more entries** loads the next 50.
2. Under **Find configuration**, type in **Search configuration** to match a setting's name, its key or environment variable, or its short description.
3. Choose an **Area** (**All areas** by default) to see one group of entries.
4. Choose a **Source** (**All sources** by default) to see only entries of one kind.
5. **Clear filters** removes all three. The line below the filters gives the number of matching entries, and **Refresh configuration** reads the report again.

![The search and filter row of Effective configuration with markers 1 to 3 on Search configuration, Area and Source.](/screenshots/admin-ops/opsa-config-filters.webp){.mw-crop}

1. **Search configuration**
2. **Area**
3. **Source**
{.mw-callouts}

## Read the table

Each row has four columns.

| Column | What it shows |
|---|---|
| **Setting** | The name and, below it, the key or the environment variable. |
| **Reported value** | The value this server resolved, or **Runtime value unavailable** when this process cannot report it, or **Not set**. A secret is shown only as **Secret configured (hidden)** or **Secret not configured**. Addresses are shown without user names, passwords, query strings and fragments. |
| **Source** | Where the value comes from, and a lock marker (**Policy locked**) when a lock prevents lower levels from changing it. |
| **How changes apply** | **Restart or rebuild required** or **See setting behavior**, the place a change is made, and, for registry settings, **Open settings editor**. |

The **Source** labels mean:

| Source | Meaning |
|---|---|
| **Captured at startup** | Read from the running process when it started. |
| **Build value** | A constant of this build. |
| **Default** | The setting's default; nothing overrides it. |
| **Stored policy** | A value saved in the settings by an administrator. |
| **Environment override** | An environment variable that pins the value. |
| **Environment fallback** | An environment variable that applies because no value is stored. |
| **Unsupported** | Not supported by this server. |

For an environment variable whose effective value this process cannot report, the row shows **Runtime value unavailable** and a **Configured input** disclosure with what the environment holds, redacted. The note under it says **This input is not proof of the value currently used by the subsystem.** The server never shows the configured input as if it were the effective value, and it never makes up a default.

An environment variable can be changed only by restarting the process. This page does not edit the environment and does not restart anything. Registry settings are changed in **Settings** (or in the dedicated editor of a feature), where validation, scope rules, locks, environment pins, the revision check, the impact preview and the audit apply.

The same data without values is the generated reference of [every setting and environment variable](/admin/configuration). The settings are also described in the [settings reference](/admin/settings).

## Options

This page has no settings. The controls are the filters and the refresh button.

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Search configuration** | Filters rows by name, key, environment variable and description. | Empty | Any administrator who can open the page |
| **Area** | Shows one group of entries. | **All areas** | Same |
| **Source** | Shows one kind of source. | **All sources** | Same |

## Troubleshooting

- **The page says "This server does not provide a configuration report."** The server did not answer the report request.
- **"No configuration entries match these filters."** Choose **Clear filters**.
- **An area is missing.** Your role cannot read that area; ask for the matching read permission.
- **A variable such as `MW_PUBLIC_URL` says "Runtime value unavailable."** This process does not report that value. Open **Configured input** to see what the environment holds, and check the subsystem that uses it, for example **Diagnostics → Connectivity**.
- **The area list contains names such as `accounts`, `build` or `cli` in lower case.** These are group names of the report that have no display name; they work as filters.

## Related

- [Settings reference](/admin/settings)
- [All settings and environment variables](/admin/configuration)
- [Diagnostics](/admin/diagnostics)
