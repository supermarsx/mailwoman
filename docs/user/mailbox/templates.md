---
title: Templates
screen: true
---
# Templates

Templates are saved messages and snippets of text you use often: a full message with a subject and recipients, a canned reply, or a few lines to drop into any message. Choose **Templates** in the sidebar to keep them, and use them from the composer.

![The Templates screen with the Shared list holding Office hours and the Personal list holding Chase a print proof and Quick thanks.](/screenshots/user-mailbox/templates-screen.webp){.mw-shot}

1. **Shared** templates. Your administrator manages these. You can read and use them, but not change them.
2. **Personal** templates. Only you can see these.
{.mw-callouts}

## Create a template

1. Choose **Templates** in the sidebar.
2. In the **Personal** list, choose **Add template**.
3. Type a **Template name**.
4. To make a full message, fill in **Subject (optional)**, **To (optional)**, **Cc (optional)** or **Bcc (optional)**. Leave them empty for a snippet you want to reuse in any message.
5. Write the text in the editor. It has the same tools as the composer.
6. Choose **Back**, then **Save templates** at the foot of the list. Until you save, **Discard changes** drops what you changed.

![The editor for a new personal template: the Template name, the optional Subject, To, Cc and Bcc fields, the text editor, and the Back and Compose from template buttons.](/screenshots/user-mailbox/templates-editor.webp){.mw-shot}

1. **Template name**.
2. **Subject (optional)**. **To**, **Cc** and **Bcc** are beside it.
3. **Compose from template** opens a new message made from this template.
{.mw-callouts}

A template holds up to 8,000 characters of plain message text, and a name up to 80 characters. You can keep up to 20 templates, and the library may use up to 64 KiB. The count beside **Add template** shows how many you have, for example **2 of 20 templates**.

## Use a template

There are two ways.

**From the Templates screen.** Each template has a row with three buttons: **Compose from template**, **Edit** (shown as a view button for shared templates) and **Remove template**. **Compose from template** opens a new message with the template's subject, recipients and text filled in.

**From the composer.** Open the **Insert** tab and choose **Response templates**. Type in **Find a template** to narrow the list, and choose one. Each is shown with its name and where it comes from: shared templates are marked **Shared** and your own **Personal**.

![The Response templates picker with the shared template Office hours and the personal templates Chase a print proof and Quick thanks.](/screenshots/user-mailbox/compose-templates-picker.webp){.mw-crop}

Choosing a template shows a preview of its text with two buttons. **Insert text** adds the text to your message. **Replace message text** replaces the text you have written; it asks **Replace your message text?** first, and the recipients and the subject stay as they are. A notice confirms **Template inserted**.

If you have no templates yet, the list says **Create personal templates in Settings, or ask your administrator for shared templates.**

## Remove a template

Choose **Remove template** on a row, then **Save templates**.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Use response templates** ([`mail.templates.enabled`](/admin/settings#mail.templates.enabled)) | Turns templates on | on | you or your administrator |
| **Show Templates in the sidebar** ([`mail.templates.showInSidebar`](/admin/settings#mail.templates.showInSidebar)) | Shows the **Templates** entry in the sidebar | on | you or your administrator |
| Personal templates ([`mail.templates.allowPersonal`](/admin/settings#mail.templates.allowPersonal)) | Lets you keep personal templates | on | your administrator |
| Shared templates ([`mail.templates.shared`](/admin/settings#mail.templates.shared)) | The templates everyone can use | none | your administrator |

If your administrator turns off personal templates, the screen says so, and shared templates remain available while templates are enabled.

## Troubleshooting

- **The Templates entry is missing from the sidebar**: it appears once your settings have loaded, and only when **Show Templates in the sidebar** and **Use response templates** are on.
- **"Check template names, message text and library size. Each template needs a unique identifier, a name and message text."**: a template has no name or no text, or the library is over its limits. Fix the template and save again.
- **The picker is empty**: you have no templates. Create one on the **Templates** screen.

## Related

- [Writing messages](/user/mailbox/composing)
- [Mail settings](/user/settings/mail)
