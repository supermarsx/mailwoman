# Message view modes and the reading floor

The reader can show a message in three ways, from most to least content. The
choice controls how much of the sender's HTML reaches your screen. An
administrator can set a **Reading floor** that stops anyone from choosing a looser
view than the deployment allows.

![The View mode menu open from the View: Full button in the reader toolbar, offering Full, No media and Plain text.](/screenshots/security/reader-view-mode-menu.webp){.mw-crop}

## The three view modes

The reader toolbar has a view mode button with an icon for the current mode; its
accessible name is **View: Full**, **View: No media** or **View: Plain text**.
Selecting it opens the **View mode** menu.

| Mode | What is rendered | Images and media | Remote content |
|---|---|---|---|
| **Full** | Sanitized HTML with styling. | Inline (`cid:`) images and, depending on the remote-image policy, proxied or direct remote images. | Allowed only as the [remote-image policy](./egress.md) permits. |
| **No media** | Sanitized HTML with layout and links. | None. The frame's Content Security Policy is `default-src 'none'; style-src 'unsafe-inline'`, so no image, media or font source is allowed. | None. |
| **Plain text** | The message's text, escaped. No HTML is rendered. | None. | None. |

**Full** is the default. When the reader is offline, it renders the cached body as
**No media** whatever mode is selected.

## How the mode is chosen

The mode applies per sender. Choosing a mode in the **View mode** menu saves it for
the sender's address (compared in lower case), and every later message from that
address opens in the same mode. Messages from senders without a saved choice open
in **Full**.

These choices are stored in the browser's local storage (key `mw.maxsec.v1`). They
are not saved on the server: another browser or device does not see them, and
clearing site data removes them.

The **Reading floor** then raises the result if it is below the floor. Modes looser
than the floor are disabled in the **View mode** menu. In short: floor first, then
the per-sender choice, then **Full**.

## Setting a reading floor (administrators)

![The Reading group of the Security policy tab, with the Reading floor set to No floor and Screen capture protection set to Off.](/screenshots/security/admin-reading-floor.webp){.mw-crop}

1. **Reading floor**
{.mw-callouts}

In the admin console, open **Security centre** → **Security policy** and set
**Reading floor** in the **Reading** group (or run
`mailwoman admin security-policy set --max-security-floor <none|full-sanitized|sanitized-no-media|plain-text>`):

| Option | Effect |
|---|---|
| **No floor** | Users may use any mode (default). |
| **Sanitised HTML with media** | Same as no floor in practice: **Full** is the loosest mode. |
| **Sanitised HTML without remote media** | Users can choose **No media** or **Plain text**, not **Full**. |
| **Plain text only** | Every message opens as **Plain text**. |

The policy is saved through `PUT /admin/security-policy`. Each server process reads
it again within 30 seconds, and signed-in clients receive it as `maxSecurityFloor`
from `GET /api/policy`.

::: warning The floor is enforced by the Mailwoman client
The server serves the floor to the client, and the client applies it. The server
does not withhold HTML bodies or images when a floor is set. A modified client, or
a direct JMAP or API request, can still fetch the full message. Use the floor to
set what the Mailwoman apps display, not as an access control.
:::

## How the message frame is isolated

All three modes render inside sandboxed frames with opaque origins:

- The outer reader document is served by `/api/rc/frame` with a Content Security
  Policy whose `script-src` lists only the SHA-256 hashes of two fixed scripts: the
  reader bridge and the address context handler. It also sets `base-uri 'none'`
  and `form-action 'none'`.
- The message itself sits in a nested frame with `sandbox="allow-scripts"`
  (plus `allow-popups` when you have allowed links in that message), never
  `allow-same-origin`, so it cannot read the app's cookies, storage or DOM.
- Before rendering, the body is sanitized: scripts, event handler attributes,
  embedded documents, forms and executable URLs are removed. The address handler
  reports a right-click on an email address back to the app through a
  per-document channel, and the app checks the source, a random token, the
  document revision and the coordinates before it opens the contact menu.

Link consent is separate from the view mode: links stay inert until you allow them
for that message, or until you trust the sender or the sender's domain for links.

## Attachments

::: warning View modes do not restrict attachment downloads
The reader's attachment download and preview actions do not check the view mode.
In **No media** and **Plain text**, attachments can still be downloaded as their
original bytes (subject to the [`mail.allowDownload`](/admin/settings#mail.allowdownload) setting and the attachment
risk policy). The view mode limits how the message body is displayed, not what
files you can save.
:::

The draft preview is the exception: it hides the download button for a draft
attachment unless the effective mode for the sending address is **Full**.

## Encrypted messages

The view mode applies to decrypted OpenPGP messages too. Decryption and
sanitizing happen in the browser (see [OpenPGP and S/MIME](./crypto.md)). A
decrypted message opened in **Plain text** shows escaped text, and in **No media**
shows sanitized HTML without images, the same as an unencrypted message.

## What the view modes protect against

- **Tracking pixels and remote images**: **No media** and **Plain text** load no
  images, so the sender learns nothing from image requests.
- **Rendering tricks in HTML**: **Plain text** shows no layout, so hidden text,
  look-alike buttons and styled phishing content are visible as plain text.

They do not protect against:

- Malicious attachments you download and open in another application.
- Links you copy or follow by hand.
- Reading by anyone who can fetch the message another way (see the warning above).

## Related

- [Remote content and outbound requests](./egress.md)
- [PDF preview](./pdf-preview.md)
- [Security policy (admin)](/admin/security-policy)
- [Security model overview](./README.md)
