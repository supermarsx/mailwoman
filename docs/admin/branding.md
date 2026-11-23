---
title: Branding
screen: true
---

# Branding

**Branding** sets the product name, the sign-in notice and six images that Mailwoman shows to everyone on this server: a logo, a browser icon, a sign-in background and three icons for the installed app. The settings are global. They apply to every domain, and no domain can override them.

Open **Branding** in the **Configuration** group of the console navigation.

![The Branding screen: the console navigation on the left, and on the right the product name, the sign-in notice and the first image section.](/screenshots/admin-ops/branding-screen.webp){.mw-shot}

## Before you start

- You need the capability `branding.read` to open the screen and `branding.manage` to change it. An administrator whose grants are limited to some domains cannot change branding, because it is not a domain setting. Without `branding.manage` the fields and buttons are disabled.
- Everything you enter or upload here is public. The name and the sign-in notice appear on the sign-in pages, which anyone can open, and a saved image is served without signing in. Do not put anything confidential in them.
- Use PNG or JPEG files of at most 1 MB.

## Set the name and the sign-in notice

1. Open **Branding**.
2. In **Name and sign-in notice**, type the **Product name**. It is required and can be up to 64 characters.
3. Optionally type a **Sign-in notice** of up to 500 characters. Leave it empty for no notice.

![The Name and sign-in notice section with marker 1 on the Product name field and marker 2 on the Sign-in notice field.](/screenshots/admin-ops/branding-text.webp){.mw-crop}

1. **Product name** appears in the mailbox sidebar, the console header, both sign-in pages and the browser tab title, and as the name of the installed app.
2. **Sign-in notice** appears as a paragraph on both sign-in pages.
{.mw-callouts}

Each field has a small information button that shows the value that is published now next to the unsaved one. Nothing changes for other people until you save.

## Add an image

Each of the six images has its own section with the same controls. The **Logo** section is the example.

1. In the section, choose **Choose File** under **Choose a local image** and pick a PNG or JPEG file. The server checks it and re-encodes it as PNG without its metadata. The section then says "Image uploaded privately. Save branding to publish it."
2. Check the preview beside **Current**. It shows the new image with the label **New (unsaved)**.
3. Choose **Save branding** at the bottom of the screen.

![The Logo section with marker 1 on the thumbnail of the current image, marker 2 on the file chooser and marker 3 on the Restore default button.](/screenshots/admin-ops/branding-logo.webp){.mw-crop}

1. The thumbnail of the current image, or **Built-in default** when none is set.
2. The file chooser. It accepts PNG and JPEG only.
3. **Restore default** clears the image and goes back to the built-in one. It is disabled when no image is set.
{.mw-callouts}

An upload stays private until you save. Only an image that the saved branding refers to is served, at an address under `/api/assets/` made of the image's content hash. Restoring the default stops new requests for that address, but browsers that already loaded the image can keep it, because the file is sent with `Cache-Control: public, max-age=31536000, immutable`.

| Section | Used for | Largest size |
| --- | --- | --- |
| **Logo** | Beside the product name in the mailbox sidebar, the console header and both sign-in pages | 512 px on each side |
| **Browser icon** | The tab icon and the Apple touch icon of the browser | 1024 px on each side |
| **Sign-in background** | A decorative background of the sign-in page | 2560 px on each side |

## Set the installed app icons

The three **Installed app icon** sections feed the web app manifest that browsers use when someone installs Mailwoman as an app. These images must have exactly the size in the section title: a square of 192 px, 512 px or 512 px.

![The Maskable installed app icon section, with its guidance that PNG or JPEG files must be exactly 512 by 512 pixels and keep important artwork inside the central 80% circle.](/screenshots/admin-ops/branding-app-icons.webp){.mw-crop}

- **Installed app icon (192 px)** and **Installed app icon (512 px)** are used as ordinary icons.
- **Maskable installed app icon (512 px)** is used where the system crops icons to a shape. Keep the important artwork inside the central circle whose diameter is 80% of the image.

The manifest is served at `/manifest.webmanifest`. It carries the product name as `name` and `short_name`, the three icons, and the fixed values `display: standalone`, theme and background colour `#111111`. Its start address and icon addresses are relative to the path the application is served from, and it contains nothing about any account. An icon you did not set falls back to the one that ships with Mailwoman. A browser or operating system refreshes an installed app's name and icon on its own schedule, so saving does not change apps that are already installed at once.

## Save

1. Choose **Save branding**. It is disabled until something has changed.
2. The screen says "Branding saved." The console updates at once. The mailbox and the sign-in pages use the new values the next time they load.
3. Choose **Reload saved branding** at any time to throw away your unsaved changes and read the saved values again.

The save is checked against the revision you loaded. If another administrator saved first, your save is refused with "Someone changed it in the meantime. Reload and try again." Choose **Reload saved branding**, then make your change again. The save is recorded in the audit log as `branding-changed`, and each upload as `branding-image-uploaded`.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Product name** ([`appearance.brandName`](/admin/settings#appearance.brandname)) | Product name shown to everyone | `Mailwoman` | Administrators with `branding.manage` on this screen, or any administrator who can edit the setting in **Settings** |
| **Sign-in notice** ([`login.notice`](/admin/settings#login.notice)) | Text on the sign-in pages | none | Same as above |
| **Logo**, **Browser icon**, **Sign-in background** | Optional images | none, the built-in mark and the theme background | Administrators with `branding.manage` |
| **Installed app icon (192 px)**, **Installed app icon (512 px)**, **Maskable installed app icon (512 px)** | Icons in the web app manifest | none, the icons that ship with Mailwoman | Administrators with `branding.manage` |

The product name and the sign-in notice are the same values as the two settings in the table, so a change made in **Settings** shows here and the other way round.

## Personal appearance customization

Separately from branding, people can restyle their own workspace. Two settings in **Settings** decide whether this is allowed:

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Allow personal appearance CSS** ([`appearance.customCssEnabled`](/admin/settings#appearance.customcssenabled)) | Lets people apply their own restricted CSS to their workspace | off | Administrators, globally only |
| **Allow personal appearance scripts** ([`appearance.customJsEnabled`](/admin/settings#appearance.customjsenabled)) | Lets people run a small appearance script | off | Administrators, globally only |

While a setting is off, people can still save a draft, but it does not run. The editor is the **Personal appearance customization** card in a user's **Settings**, in **Appearance**, and on the console page **My preferences** for administrators. The two kinds of owner have separate documents.

- **CSS** is limited to 16 KiB, 128 rules and 512 declarations. Selectors stay inside the person's own workspace. Imports, URLs, attribute selectors, `!important` and positioning are rejected. It pauses while an administrator holds any mailbox appearance setting.
- **Scripts** are limited to 16 KiB and run in an isolated frame for at most 500 ms and 32 updates. They can only call `appearance.set` with an `accent`, `density`, `font` or `mode`, and they receive only `context.dark` and `context.reducedMotion`. They cannot read mail, account data or the page, and they cannot reach the network. A script cannot change a field that an administrator locks.
- A person who locks themselves out with a bad customization can add `appearanceSafe=1` to the page address, before any `#`, to open the workspace without custom CSS or scripts and then reset it. This does not remove administrator locks.

## Troubleshooting

- **The screen says "Your role does not allow this action. Ask an admin with the right role."** The account lacks the capability, or only has grants limited to some domains. Ask for `branding.read` or `branding.manage` without a domain limit.
- **An upload fails with "The server refused a value. Check the fields and try again."** The file is not a PNG or JPEG, is over 1 MB (the request limit is about 1.4 million base64 characters), is larger than the size shown for that section, or is damaged. SVG files are refused. Resize the image and try again.
- **An installed app icon is refused although it is a PNG.** These three sections need exactly 192 by 192, 512 by 512 and 512 by 512 pixels. The server answers "Installed app icons must use the exact square dimensions shown".
- **An upload fails with "Too many requests. Wait a minute and try again."** At most two images are decoded at once on the server. Try again.
- **Save fails with "Branding changed or an image is not available for this purpose; reload".** An image was replaced or removed on the server, or it was uploaded to a different section than the one it is saved in. Reload, upload the image again and save.
- **The old logo still shows after you restore the default.** The browser cached it, see [Add an image](#add-an-image). A reload without the cache, or waiting, clears it.
- **An installed app still shows the old name or icon.** The browser or operating system decides when to refresh it. Remove and install the app again to see the new identity at once.

## Related

- [Settings](/admin/settings) for `appearance.brandName`, `login.notice` and the two customization settings
- [Fonts](/deploy/fonts) for the typefaces that ship with the web client
- [The admin console](/admin/) for how to open and move around the console
