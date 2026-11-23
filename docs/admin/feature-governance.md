---
screen: true
---
# Feature profiles

The **Feature profiles** screen of the console groups feature switches, limits and other settings into named profiles, and assigns them to domains, groups or single people. A profile can turn features off or on and set limits; it does not grant access to mailboxes, addresses or credentials.

![The Feature profiles screen with the editor of a new profile open: the profile name and the switches with their values and locks.](/screenshots/admin-privacy/features-overview.webp){.mw-shot}

## Before you start

- The screen needs `settings.read` on all domains, `settings.write` to change profiles, and `settings.user_values` to assign them. An administrator limited to some domains cannot use it.
- The values a profile holds are settings; see the [settings reference](/admin/settings) for each key.

## Create a profile

1. Open **Feature profiles** and select **New profile**.

   ![The top of a new feature profile: the profile name, the preset picker with Apply preset, Import JSON and Export JSON, and the feature search.](/screenshots/admin-privacy/features-preset.webp){.mw-crop}

   1. **Profile name**.
   2. **Apply preset**, after choosing a preset.
   3. **Import JSON**; **Export JSON** is beside it.
   {.mw-callouts}

2. Enter the **Profile name**. Under **Start from a preset**, choose **Full**, **Standard**, **Mail only** or **Minimal / kiosk** and select **Apply preset**; a preset fills the switches and is not stored itself.
3. For each switch, choose **Allowed**, **Not allowed** or **Inherit** (the profile holds no value, so the next profile, the domain or the server-wide value decides), and tick **Locked** to stop narrower scopes from changing it. Limits take **Inherit** or **Set a value**. **Find a feature** filters the list. **Allow all**, **Disallow all** and **Clear** change every switch at once. **Other settings a profile can hold** lists the other settings a profile may set.
4. Select **Create profile** (or **Save profile** for an existing one).

**Export JSON** and **Import JSON** move a profile between servers; an import is checked like a save. **Clone** copies a profile.

## Assign a profile

1. Open the profile and go to **Assign this profile**.
2. Choose **Applies to**: **Domain**, **Group** or **User**, the target, and a **Priority**.
3. Select **Assign**. **Remove** on a row ends the assignment.

## Check what a person gets

Under **Effective values**, choose **Preview for** and **Who**, then **Show**.
 The preview lists each switch with its effective value (**Effective**) and the scope that decided it (**Decided by**). It uses the same resolver as the server's checks, on saved values only: unsaved changes in the editor are not included.

A value is chosen like this:

1. an environment pin wins over everything;
2. otherwise a lock wins over every narrower scope, and the broadest lock wins;
3. otherwise the narrowest value that is set wins, from narrow to broad: the person's own choice (only for settings people may change), a value an administrator set for that person, a group, a profile, a domain, the server-wide value;
4. with nothing set, the built-in default (or an environment fallback) applies.

## What turning a feature off does

For most features, the server refuses the feature's requests with `403 feature_disabled` before anything runs, and the mail app hides it. Some switches do less, and the editor marks them:

| Badge | Feature | What happens |
| --- | --- | --- |
| **Hidden in the app only** | Original HTML of a message (`reading.original_html`), the attachments view (`views.attachments`) | The mail app hides the feature; the server does not refuse it. Messages stay reachable through search. |
| **Some limits not yet counted** | Sending (`compose.sending`) | The recipients-per-message limit is enforced; the hourly and daily sending limits (`limit.send_per_hour`, `limit.send_per_day`) are stored but not counted. |
| **Some limits not yet counted** | Storage (`storage`) | `limit.storage_bytes` is stored only; the mail server's own quota applies. |

Signing in, passwords, second factors and sessions are not controlled by profiles; they have their own [security policy](security-policy.md).

## Options

Feature profiles hold values of other settings, so they have no settings of their own.

## Related

- [Settings reference](/admin/settings)
- [Security policy](security-policy.md)
- [Restrictions and consent](consent.md)
