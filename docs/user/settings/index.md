---
title: Personal settings
screen: true
---

# Personal settings

Settings is where you change how Mailwoman looks and behaves for your own account: who you send as, how messages are displayed, what notifies you, and which integrations are connected. Nothing here changes how other people's accounts work.

![The Settings screen: the section list grouped by category on the left, with a search box above it, and the Identities section on the right.](/screenshots/user-settings/settings-overview.webp){.mw-shot}

1. **Search settings** finds sections and individual settings.
2. The section list is grouped into five categories. Select a section to open it on the right.
{.mw-callouts}

## Open Settings

Select the **Settings** button at the bottom of the sidebar, next to **Log out**.

![The two buttons at the bottom of the sidebar: Settings, with the sliders icon, and Log out.](/screenshots/user-settings/settings-open-button.webp){.mw-crop}

Settings opens as its own tab in the workspace tabs, so your mailbox stays open in the **Mail** tab.

Above the section the path is shown, for example Account then Identities. Some sections are split into tabs, such as the six tabs of the Security centre; the tabs keep what you have typed, so you can move between them before saving.

## Categories and sections

![The section list: Search settings on top, then the Account, Mail, Interface, Integrations and About groups with their sections.](/screenshots/user-settings/settings-nav.webp){.mw-crop}

1. **Search settings** filters the list.
2. Each category heading, here **Account**, groups its sections; select a section to open it.
{.mw-callouts}

There are 23 sections in five categories. Each category has its own page, with one heading per section.

| Category | Sections | Page |
|---|---|---|
| **Account** | Linked mailboxes, Identities, Signatures, Security centre, Privacy | [Account settings](./account.md) |
| **Mail** | Message display, Composing and replies, Folders and sidebar, Retention, Labels, Mail rules, Saved searches, Read receipts, Import and export mail | [Mail settings](./mail.md) |
| **Interface** | Appearance, Keyboard & display, Notifications, Regional settings | [Interface settings](./interface.md) |
| **Integrations** | Nextcloud accounts, Package tracking, API access | [Integrations settings](./integrations.md) |
| **About** | About Mailwoman, Server information | [About settings](./about.md) |

The list always shows every section. When a feature is switched off for your account, the section says so instead of disappearing.

## Find a setting

Type in **Search settings**. The list keeps the sections whose name or keywords match every word you typed, and, under them, lists individual settings whose label, help text or card title match. Select a setting to open its section at that card.

![The Settings screen after typing "image" in Search settings: the list keeps only the matching sections, followed by the matching setting rows.](/screenshots/user-settings/settings-search.webp){.mw-shot}

When the Settings area is narrow (760 pixels or less), the section list is replaced by a button under the search box. It shows the current section; select it to choose another.

![The Settings screen in a narrow window: the section list is replaced by a button that shows the current section, Identities.](/screenshots/user-settings/settings-narrow.webp){.mw-shot}

## How settings are saved and who controls them

Most switches and choices are saved as soon as you change them. Lists and forms that have a **Save** button, such as the trusted link domains or an identity, are saved when you select it.

Every setting has a built-in default. An administrator can set a different value for the whole server, a domain, a group or a single user, and can lock it. What you see is:

- **Your own choice**, when you have changed the setting and it is not locked.
- **The administrator's value**, when you have not. The reset button beside a setting removes your own choice so the administrator's or built-in value applies again. It is labelled **Use administrator default** in most sections and **Use inherited setting** in the Privacy section.
- **A locked control.** The control is dimmed and the text **Set by your administrator** appears beside it. You cannot change it until an administrator unlocks it.

A few settings are limits the administrator sets for you, for example how long a send can be delayed; they appear as locked controls or as notes such as **What this server allows**.

While a linked mailbox is open, a setting may show **Inherited from the primary account**. Sections that belong to the sign-in account show **Managed by the sign-in account** with a button that opens the primary account's settings. Your linked mailboxes keep their own preferences otherwise.

Each setting table in these pages names the setting key. The [settings reference](/admin/settings) describes the same keys from the administrator's side.

## Related

- [Account settings](./account.md)
- [Mail settings](./mail.md)
- [Interface settings](./interface.md)
- [Settings reference for administrators](/admin/settings)
