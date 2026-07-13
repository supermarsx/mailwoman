# Max-security message opening

Some messages are worth reading with the blast radius turned all the way down.
V4 adds a **three-position switch** to the message toolbar that controls how far a
message is stripped before it is rendered:

1. **plain-text** — render the message as escaped plain text. No HTML, no remote
   content, no inline media. The safest, lowest-fidelity view.
2. **sanitized-no-media** — render sanitized HTML but with **all media stripped**
   (the render CSP drops images/media). Layout and links survive; nothing is
   fetched from a remote host.
3. **full-sanitized** — the normal V2 reading experience: sanitized HTML in the
   sandboxed iframe.

All three render inside the existing V2 viewer sandbox (`sandbox=""`, no
`allow-scripts`, no `allow-same-origin`). The switch changes the CSP the iframe is
given and the sanitize mode; it never grants the message more capability.

## Attachments

In any of the reduced modes, attachments open **only via the re-encode preview
jail** (the V2 viewer sandbox that renders a re-encoded preview), never the
original bytes. There is no "open original" path from the reader in these modes.

## Policy precedence

The effective mode for a message is chosen by, in order of precedence:

1. **Admin floor** (config) — an operator can set a *minimum* strictness that a
   user cannot relax.
2. **Per-sender policy** — a user's saved preference for that sender.
3. **Global default** — the user's global setting.

The admin floor only ever *raises* strictness; a user may make a message stricter
than the floor but never weaker. Per-sender overrides the global default within the
bounds the floor allows.

## Relationship to end-to-end encryption

Max-security opening and E2EE are independent and compose cleanly: a decrypted
end-to-end-encrypted body is sanitized **in the crypto worker** (never on the
server) and then rendered honouring the selected max-security mode — so a decrypted
message opened in `plain-text` shows escaped text, and in `sanitized-no-media` has
its media stripped, exactly like a cleartext message.
