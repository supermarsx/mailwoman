---
title: Account settings
screen: true
---

# Account settings

The **Account** category holds the settings that belong to you as a person: the mailboxes you have linked, the names you send as, your signatures, how you sign in, and what the server may do with your data. It has five sections, listed here in the order the Settings navigation shows them.

![The Settings screen on Linked mailboxes, with markers 1 to 5 on the five sections of the Account category in the section list.](/screenshots/user-settings/account-overview.webp){.mw-shot}

1. **Linked mailboxes**
2. **Identities**
3. **Signatures**
4. **Security centre**
5. **Privacy**
{.mw-callouts}

| Section | What it is for |
|---|---|
| [Linked mailboxes](#linked-mailboxes) | Open other mailboxes without signing out. |
| [Identities](#identities) | The names and addresses you send mail as. |
| [Signatures](#signatures) | Text added to the messages you send. |
| [Security centre](#security-centre) | Password, second factor, passkeys, sessions and sign-in history. |
| [Privacy](#privacy) | Remote images, trackers, links, diagnostics, your stored data and data requests. |

For what the Settings screen is and how a setting can be locked by an administrator, see [Personal settings](./index.md).

## Linked mailboxes

![The Mailbox preferences card of Linked mailboxes: Use linked mailboxes, Preload linked mailboxes and Show mailbox connection icons.](/screenshots/user-settings/account-linked-preferences.webp){.mw-crop}

Linked mailboxes lets you open another mailbox while you keep your primary sign-in. Each linked mailbox keeps its own preferences, identities, signatures, folders and drafts. For the day-to-day use, see [Linked mailboxes](/user/mailbox/linked-accounts).

### Before you start

Linking has to be allowed for your account. When it is not, the section shows **Linking mailboxes is disabled by your account policy.** and the **Link a mailbox** button is unavailable. Linking supports native IMAP, POP3 and EWS accounts that can be verified; accounts that sign in through a proxy or single sign-on cannot be linked. An administrator can also cap how many mailboxes you may link.

### Link a mailbox

1. Open **Settings**, then **Linked mailboxes**.
2. Select **Link a mailbox**.
3. Enter the **Username or email address** and **Password** of the other mailbox. **Display name (optional)** is a label for the list.
4. Select **Verify and link**. If that mailbox requires a second factor, you complete it in the next step. Your primary sign-in does not change.

![The form that links a mailbox: username or email address, password, optional display name, and the Verify and link button.](/screenshots/user-settings/account-linked-form.webp){.mw-crop}

Each linked mailbox in the list can be opened, renamed with a nickname that only you see, set as the primary mailbox that opens first after sign-in, and unlinked. **Unlink mailbox** removes access through your primary account; it does not delete the mailbox or its messages. When access to a mailbox expires the row shows **Access expired** and **Renew access**. To change the order, drag a mailbox by its handle, or press Alt and the Up or Down arrow on the focused row; the order is the same on every device.

### Provider connection

![The Provider connection card: a provider choice between Microsoft Graph and Gmail API, an application client ID, and Connect securely.](/screenshots/user-settings/account-provider-connection.webp){.mw-crop}

The **Provider connection** card connects a linked mailbox to Microsoft Graph or the Gmail API using your own app registration. An administrator must enable the bridge first, and you authorize the same email address as the mailbox. Choose the **Provider**, enter the **Application client ID** (and an optional **Microsoft tenant**), then select **Connect securely** or **Use a device code**. Both buttons stay unavailable until a mailbox is linked.

### Options

Every setting in this section can be locked by an administrator. A locked control is dimmed and shows **Set by your administrator**.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Use linked mailboxes** | Allows access to your linked mailboxes when your administrator permits it. Turning it off does not delete the links or their mailboxes. | On | [`accounts.useLinked`](/admin/settings#accounts.useLinked) |
| **Preload linked mailboxes** | Keeps a small session cache of linked mailbox folders and recent messages for faster switching. Mail caching must also be on. | On | [`accounts.preloadLinked`](/admin/settings#accounts.preloadLinked) |
| **Show mailbox connection icons** | Shows a small status icon beside mailbox names. Connection status is always available in this section. | Off | [`accounts.showConnectionStatus`](/admin/settings#accounts.showConnectionStatus) |

These preferences belong to the account you are signed in with. While a linked mailbox is open, the section tells you to switch to your primary account to change them.

## Identities

![The Identities list with two identities, the first marked Default, and markers 1 to 4 on the buttons of the second.](/screenshots/user-settings/account-identities.webp){.mw-crop}

1. **Make default** makes this identity your default sender.
2. **Create variant** adds a variant of this identity.
3. **Edit** opens the identity in the form below.
4. **Delete** deletes the identity after a confirmation.
{.mw-callouts}

An identity is a name and address you send mail as, with its own reply-to address, automatic Bcc, signature and OpenPGP key. Sending addresses must be allowed by your mail provider or administrator. The list starts empty; one identity is your default sender.

### Add an identity

1. Select **New identity**.
2. Under **Basics**, enter **Display name** and **Email address**; **Organization** is optional.
3. Under **Reply handling**, optionally set a **Reply-to address (optional)** (replies go there instead of to the sender address) and **Automatically Bcc**, up to 20 addresses separated by commas.
4. Under **Signature**, choose the signature for **New messages** (the account signature by default) and for **Replies** (the new-message signature by default).
5. Under **Sending options**, choose an **OpenPGP key**, or leave **Use the default key**.
6. Select **Save**.

![The New identity form with Basics, Reply handling, Signature and Sending options groups.](/screenshots/user-settings/account-identity-editor.webp){.mw-crop}

A variant is a second identity that shares its base identity's address. It has a **Variant label** (for example Newsletters), and any field you leave empty uses the value from the base identity. Deleting an identity that has variants asks whether to delete the variants too or keep them. Deleting your default identity makes your oldest remaining identity the default. Drafts and sent messages are not affected.

### Options

Identities are stored as records, not as individual settings, so they have no entry in the settings reference. Which addresses you may send from is decided by your mail provider or administrator.

## Signatures

![The Signatures list with two signatures, one marked as the default.](/screenshots/user-settings/account-signatures.webp){.mw-crop}

A signature is text appended to messages you send. One signature can be the default. Identities can use a signature other than the default.

1. Select **New signature**.
2. Enter a **Name**. Names are unique; a name that already exists is refused.
3. Choose the **Signature format**: **Plain text** or **Rich text (HTML)**. Rich text opens the visual editor, with an **Edit HTML source** view.
4. Write the signature. For rich text, **Plain-text alternative** is the text plain messages use; rich edits update it until you customize it.
5. Tick **Use as default** if this should be your default signature, then select **Save**.

![The New signature form with a name field, a format choice and the signature text.](/screenshots/user-settings/account-signature-editor.webp){.mw-crop}

HTML signatures are limited to 2 MiB including images, and are sanitized before saving and sending: remote images and active content are removed. Images you add are stored in the signature and sent as inline images of each message. Deleting a signature makes identities that used it fall back to the account signature; messages already sent are not affected.

Where the signature goes in a reply is a composing setting, **Signature position**, described under [Composing and replies](./mail.md#composing-and-replies).

## Security centre

The Security centre groups everything about signing in and your sessions into six tabs.

### Sign-in & passkeys

![The Change password card: current password, new password and confirmation fields, and the password rule.](/screenshots/user-settings/account-security-signin.webp){.mw-crop}

**Change password** asks for the **Current password**, a **New password** and **Confirm new password**. The new password must be at least 12 characters. See [Change your password](/security/password-change) for what a change does to stored credentials.

![The Two-factor authentication, Authenticator app and Passkeys cards, before anything is set up.](/screenshots/user-settings/account-security-2fa.webp){.mw-crop}

- **Two-factor authentication** shows how many recovery codes remain and offers **Generate new recovery codes**. Recovery codes are shown once and each works one time.
- **Authenticator app**: **Set up an authenticator app** gives you a secret to add to your app, then asks for the **Authenticator code** to confirm. **Remove authenticator** takes it off.
- **Passkeys**: **Add a passkey** registers one, with an optional name. The card lists the rules your organization's policy sets for passkeys (for example that they check your PIN, fingerprint or face). Passkeys can sign you in without a password in engine mode; in proxy mode they verify a second factor after your mail password. A passkey added before a stricter policy may be listed as not meeting it; add a new one and remove the old one.

Your organization may require a second factor, and may refuse removing your last one.

![The Sign-in country policy card with its policy choice, allowed countries and allowed addresses, and the Countries seen card.](/screenshots/user-settings/account-security-country.webp){.mw-crop}

**Sign-in country policy** can refuse new sign-ins to your account from certain countries. It never affects sessions that are already signed in, and your password and second factor are still required. Country detection uses an IP geolocation database that can be wrong, so VPNs, mobile carriers and travel can block a legitimate sign-in.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Policy** | **Off**; **Block countries I have not signed in from**; or **Block countries outside my allowed list**. | Off | [`security.signin.accountGeoMode`](/admin/settings#security.signin.accountGeoMode) |
| **Allowed countries (two-letter codes, comma separated)** | Countries that always pass the check. | Empty | [`security.signin.accountGeoAllowCountries`](/admin/settings#security.signin.accountGeoAllowCountries) |
| **Allowed IP addresses or networks (one per line)** | Addresses and networks that always pass the check. | Empty | [`security.signin.accountGeoAllowIps`](/admin/settings#security.signin.accountGeoAllowIps) |

**Add my current IP** and **Add my current country** copy your present address or country into the lists; **Save** refuses a policy that would block your current address or country. **Countries seen** lists the countries you have signed in from successfully, kept for 365 days; only country codes, dates and counts are stored. **Forget countries seen** clears the list.

### Sessions & access

![The Session security notices card, empty, above the Alert preferences card.](/screenshots/user-settings/account-security-sessions.webp){.mw-crop}

**Session security notices** lists warnings about unusual use of your signed-in sessions. Each notice says what the server saw and what it did (alerted you, asked the session to confirm your password and second factor, or signed it out), with **This wasn't me** and **Dismiss** actions. **Alert preferences**:

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Warn me when a session is used from a new country** | Turns the new-country warning on or off. If your administrator does not allow muting, the server keeps alerting when this is off. | On | [`security.alerts.countryChange`](/admin/settings#security.alerts.countryChange) |
| **Where to receive warnings** | **In the app**, **In my mailbox**, or **In the app and in my mailbox**. | In the app and in my mailbox | [`security.alerts.channels`](/admin/settings#security.alerts.channels) |
| **Sensitivity** | **Low** (location findings only, no forced sign-out), **Normal**, or **High** (stricter travel check, also compares networks). | Normal | [`security.alerts.sensitivity`](/admin/settings#security.alerts.sensitivity) |

**Use administrator defaults** removes your own choices for these three.

![The Active sessions card: the linked-mailbox origin choice, a search box, the session table and the button that signs out the other sessions.](/screenshots/user-settings/account-security-active.webp){.mw-crop}

**Active sessions** lists the sessions signed in to this account, with the device and client, the IP address and last known country, and the activity. Search the list, tick sessions and select **Sign out selected sessions**, or use the button that signs out all other sessions. The current session cannot be selected here. **Show where linked mailbox access comes from** controls how the table names the account a linked-mailbox session comes from (**Full address**, **Address with the first part masked** or **Hidden**); only you see it. Its setting is [`privacy.sessions.showLinkedOrigin`](/admin/settings#privacy.sessions.showLinkedOrigin) and its default is **Full address**. **Applications using your identity** lists applications that hold identity consent, so you can revoke it; this does not remove separate mail API permissions.

### Sign-in history, Remembered devices and Account activity

![The Sign-in history card with a result filter, Refresh and Export history.](/screenshots/user-settings/account-security-history.webp){.mw-crop}

**Sign-in history** lists sign-in attempts with their time, address, country and client. Filter by **All results**, **Successful**, **Failed** or **Blocked**, select **Refresh** to reload, or **Export history** to download it.

![The Remembered devices card with a device name field and the Remember this device button.](/screenshots/user-settings/account-security-devices.webp){.mw-crop}

**Remembered devices**: if your administrator permits it, **Remember this device** remembers the browser after you verify your identity. Sensitive changes still require fresh verification.

**Account activity** lists recent account and security events; each entry is kept separately and lists its details.

### Encryption

![The Encryption tab: the Zero-access storage card with its state, the list of what the server still sees, the caveat, and the key setup choice between passphrase and passkey.](/screenshots/user-settings/account-security-encryption.webp){.mw-crop}

**Zero-access storage** is **Off** or **On**. When it is on, the server keeps this account's mail out of its search index and keeps text taken from it out of shared caches. Turning it on also deletes the stored passwords of linked mailboxes. It does not change how mail is stored: message bodies and envelopes are sealed with the server's key whether the mode is on or off, and the server can open them. The card lists what the server still sees (bodies and envelopes it can open, mail in transit, sizes, timestamps, routing metadata) and states that zero-access does not defend your mail against the server or whoever runs it.

To turn it on, choose a key source, **Passphrase** or **Passkey (passwordless)**, and select **Enable zero-access**. A recovery phrase is shown once; keep it offline. Stored mail is not encrypted with that key. **Disable zero-access** turns the mode off again and lets the server add the account's mail to its search index again. See [Zero-access mode](/security/zero-access) for the details.

## Privacy

The Privacy section has five tabs.

### Message content

![The Privacy card of Message content: remote content choice, tracking switches, image retry and unsafe HTML handling.](/screenshots/user-settings/account-privacy-content.webp){.mw-crop}

Remote images can reveal when you open a message. These settings decide when they load and whether the server fetches them for you. Every row has a reset button that removes your own choice, so the administrator's or built-in value applies again; it reads **Use inherited setting** in this tab.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Remote content** | **Block remote content**; **Ask before loading through the server**; **Load through the server**; or **Load directly from the sender's servers**. Direct loading needs your administrator's permission and shows your network address to the image host. | Ask before loading through the server | [`privacy.remoteContent`](/admin/settings#privacy.remoteContent) |
| **Block tracking images and beacons** | Blocks tracking images and beacons in messages. | On | [`privacy.trackers`](/admin/settings#privacy.trackers) |
| **Remove tracking parameters from links** | Strips tracking parameters from message links. | On | [`privacy.cleanLinks`](/admin/settings#privacy.cleanLinks) |
| **Manual remote-image downloads** | When an image does not display, lets you download a checked copy through the server. The server verifies that the image belongs to the message and still applies its privacy and security rules. Not available for client-decrypted images. | Off | [`privacy.remoteImageDownloads`](/admin/settings#privacy.remoteImageDownloads) |
| **Show a notification while remote images load** | Shows a notice while remote images load. | On | [`privacy.remoteImagesToast`](/admin/settings#privacy.remoteImagesToast) |
| **Remove outgoing mailer headers** | Native SMTP and EWS sends remove top-level client fingerprint headers. It does not cover upstream JMAP proxy sends or headers added later by a relay, and a message that is already signed over a removed header needs this off before sending. | On | [`privacy.stripMailerHeaders`](/admin/settings#privacy.stripMailerHeaders) |
| **Automatic image retry attempts** | Retries only temporary image failures, after the current images finish loading. 0 turns automatic retries off; manual retry stays available. | 3 (0 to 8) | [`privacy.imageRetryAttempts`](/admin/settings#privacy.imageRetryAttempts) |
| **Image retry delay (seconds)** | Wait before an automatic retry. | 5 (0.5 to 60) | [`privacy.imageRetryDelayMs`](/admin/settings#privacy.imageRetryDelayMs) |
| **Potentially unsafe message HTML** | **Ask before showing sanitized HTML**, **Always show safe text**, or **Always show sanitized HTML**. Sanitized HTML removes active content; remote resources, links and automatic receipts stay blocked for these messages. | Always show sanitized HTML | [`mail.reader.unsafeHtmlMode`](/admin/settings#mail.reader.unsafeHtmlMode) |
| **Allow a per-message tracker choice** | Offers an explicit action to load blocked trackers for one message. An administrator can require protection. | On | [`privacy.trackerOverrideAllowed`](/admin/settings#privacy.trackerOverrideAllowed) |

![The Original message HTML card and the Trusted link domains card, with its domain list and subdomain option.](/screenshots/user-settings/account-privacy-links.webp){.mw-crop}

**Original message HTML** has one switch, **Allow Original HTML per message** ([`mail.reader.allowOriginalHtml`](/admin/settings#mail.reader.allowOriginalHtml), off by default). When on, an opened message offers **Show Original HTML**, which keeps the sender's formatting. Scripts and forms stay disabled, and images and links still follow your privacy controls; sanitized HTML stays the default.

**Trusted link domains** lists domains whose links open straight from **Review links** without the destination step ([`links.trustedDomains`](/admin/settings#links.trustedDomains), empty by default). Enter one domain per line; matching is exact, so `example.com` does not trust `examples.com`. Use the ASCII (`xn--`) form for international names. Save with **Save trusted domains**. **Also trust subdomains** ([`links.trustedSubdomains`](/admin/settings#links.trustedSubdomains), off) extends trust to any name ending in `.example.com`. Links that use plain HTTP or an IP address are always reviewed. Next to each of these controls, **Use administrator default** removes your own choice.

![The Trusted link senders card: a list of sender addresses and a list of sender domains, each with a save button.](/screenshots/user-settings/account-privacy-link-senders.webp){.mw-crop}

**Trusted link senders** allows message links automatically for exact sender addresses (**Sender addresses**, [`links.trustedSenders`](/admin/settings#links.trustedSenders)) or sender domains (**Sender domains**, [`links.trustedSenderDomains`](/admin/settings#links.trustedSenderDomains)). Link destinations are still validated, and this does not load images. Both lists are empty by default.

### Cookies and consent

![The Optional usage counts card with one checkbox, off.](/screenshots/user-settings/account-privacy-consent.webp){.mw-crop}

**Allow optional screen usage counts for this account** is off by default. If your administrator enables a collector, you may share fixed screen names such as Mail or Calendar. Messages, email addresses, searches, route details and browser identifiers are never sent. These are event counts, not unique visitor statistics.

### Diagnostics

![The Load times card with two measuring switches and the Error reports card with one switch.](/screenshots/user-settings/account-privacy-diagnostics.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Measure how long folders take to open** | Reports the time from choosing a folder until its message list is shown, and whether the data came from cache or the network. No folder, message, sender or subject is included. | On | [`observability.clientTiming.folderLoads`](/admin/settings#observability.clientTiming.folderLoads) |
| **Measure how long messages take to open** | Reports the time from opening a message until its headers and text are shown, in the same way. | On | [`observability.clientTiming.messageLoads`](/admin/settings#observability.clientTiming.messageLoads) |
| **Send error reports from this browser** | When part of Mailwoman fails, the browser sends your administrator a short report: error type, where in the app it happened, app version and browser type. It never holds message content, addresses, attachment names or what you typed. Off keeps reports on your device; the error panel still shows a reference you can copy. | On | [`privacy.errorReports`](/admin/settings#privacy.errorReports) |

### Your data

![The What we hold about you table: each kind of data this server keeps, how many items, how long it is kept and who handles it.](/screenshots/user-settings/account-privacy-data.webp){.mw-crop}

**What we hold about you** is a table of what this server stores about you: the number of items, the approximate size, how long each kind is kept, and who handles it. Mail itself stays on your mail provider's server; the counts are what this server keeps. **Your choices** shows each purpose with its legal basis, and a switch where you can change it (for example the assistant, push wake-ups and open tracking of your sent mail); other purposes point to the setting that controls them. **Automated processing** lets you switch each assistant feature off separately. **Where you are signed in** lists your sessions with **Sign out / revoke**. **Privacy documents** opens the privacy policy and the processing records.

![The Take your data or leave card with Download my data, Restrict use of my data and Delete my account.](/screenshots/user-settings/account-privacy-data-actions.webp){.mw-crop}

**Download my data**, **Restrict use of my data** and **Delete my account** do not act at once. Each opens the request form on the **Data requests** tab with that request type chosen.

### Data requests

![The data request form: a request type, optional details and Submit request, above the list of your requests.](/screenshots/user-settings/account-privacy-requests.webp){.mw-crop}

Choose a **Request type**: **Access to my data**, **Download my data** (a ZIP of your data in JSON, vCard and iCalendar), **Correction**, **Erasure**, **Restriction**, **Objection** or **Other request**. Add **Details (optional)** and select **Submit request**; you are asked for your password first. Each request in **Your requests** shows when it was filed and when an answer is due. An erasure cannot be undone after approval; there is a waiting period in which you can cancel, and mail kept by your mail provider is not deleted by this server. If data requests are switched off on the server, the tab says so.

## Troubleshooting

| What you see | Why | What to do |
|---|---|---|
| **Linking mailboxes is disabled by your account policy.** | Linking is not allowed for your account. | Ask your administrator to allow linked mailboxes. |
| A control is dimmed and shows **Set by your administrator**. | An administrator has fixed the value. | Ask your administrator; you cannot override it. |
| **This would block sign-in from your current address or country.** | The country policy would lock you out. | Add your current IP address or country to the allowlist, then save again. |
| **Your organization does not allow removing your last second factor.** | Policy requires a second factor. | Add another passkey or authenticator app first. |

## Related

- [Personal settings](./index.md)
- [Linked mailboxes](/user/mailbox/linked-accounts)
- [Encryption and keys](/user/encryption-keys)
- [Settings reference for administrators](/admin/settings)
