---
title: About settings
screen: true
---

# About settings

The **About** category shows facts about the software you are running and about the mail server. It has two sections, and neither has anything you can switch on or off.

![The Settings screen on About Mailwoman, with markers 1 and 2 on the two sections of the About category in the section list.](/screenshots/user-settings/about-overview.webp){.mw-shot}

1. **About Mailwoman**: version, build and software inventory.
2. **Server information**: annotations stored on the mail server.
{.mw-callouts}

## About Mailwoman

![The About Mailwoman section: version, source revision, release tag, build target, support end date and the software inventory downloads.](/screenshots/user-settings/about-product.webp){.mw-crop}

The section lists the build facts of this installation:

- **Version**, **Source revision**, **Release tag** and **Build target**.
- **Declared support end date**. It reads **Not declared** unless the distributor of this build declared one. Security fixes follow the latest tagged rolling release, and a fixed support period must be declared by the distributor.

These build facts and documents support CRA preparation. They do not establish conformity or determine which legal obligations apply.

**Software bill of materials** describes the software inventory. The backend inventory covers the Cargo workspace lock, including development and platform-specific packages; the frontend inventory covers bundled npm packages. Native libraries, containers, plugins and vendored assets need separate coverage. The card shows the number of components and the SHA-256 of each lock file, and offers three downloads: **Download backend inventory**, **Download frontend inventory** and **Download security policy**.

Below the downloads are links to **Report a vulnerability privately**, **Security advisories** and **Releases and updates**, and a **Third-party licences** list that you can open.

## Server information

![The Server information section, which says that annotations are read-only for this account.](/screenshots/user-settings/about-metadata.webp){.mw-crop}

**Server annotations** are metadata entries stored on the mail server. Changes here take effect only if your account is allowed to set them. When the server does not provide annotations, or loading fails, the section says **Could not load the annotations.** with a **Retry** button. When editing is not allowed for your account, it says **Editing is turned off for you. Annotations are shown read-only.**

## Troubleshooting

| What you see | Why | What to do |
|---|---|---|
| **Could not load the annotations.** | The mail server does not provide annotations, or the request failed. | Select **Retry**. If it keeps failing, the server may not support annotations. |

## Related

- [Personal settings](./index.md)
- [Server information for administrators](/admin/server-metadata)
