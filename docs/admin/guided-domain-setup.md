# Guided domain setup

Admin → Domains → guided setup has three connection choices:

- **Server defaults** creates the domain without an upstream override.
- **Configure a mail server** retains the compact single IMAP or POP3 endpoint plus optional SMTP setup (or an HTTPS JMAP URL in proxy mode).
- **Configure multiple backends and advanced options** builds a version-2 backend list. Changes stay in the wizard until the final review and Create domain request.

The advanced list supports up to 32 uniquely identified backends. Each has an explicit protocol/service role, enabled state, numeric priority and preferred selection. At most one enabled backend is preferred per service role. Lower priority values win after preferred selection; this does not authorize retrying rejected credentials against another backend.

Engine mode currently offers IMAP, POP3, SMTP and ManageSieve. Proxy mode offers JMAP session URLs. These modes cannot be mixed by this wizard. EWS, Microsoft Graph/365, Gmail APIs and MAPI are not selectable merely because an underlying plugin exists.

Socket backends expose full-email, local-user, domain\\user, user@realm and custom-template username formats, plus the existing server-side username preview. The wizard never collects account passwords. Authentication method choices come from the server's actual protocol matrix. Fixed selection uses one mechanism; ordered selection uses distinct supported mechanisms. Channel-binding requirements and rejection behavior remain enforced by the protocol runtime.

The shared transport editor supports fixed TLS/STARTTLS/plaintext selection or an explicitly ordered allowed list when the server reports that runtime feature ready. Plaintext is an explicit insecure choice. Certificate chain and hostname verification remain enabled. A custom CA and server-name override are available through the existing transport policy; they do not disable certificate verification. Private destinations require the existing explicit private-target policy. DNS hostnames are resolved on connection according to that runtime policy; setup does not pin an address or add an egress grant.

**Discover configuration** invokes the existing server-side discovery endpoint through its configured outbound route. It needs unscoped upstream configuration permission. The returned endpoints remain an unsaved proposal until reviewed. OAuth-required results show a persistent warning: importing endpoint information does not configure OAuth or make provider-native sign-in available.

This wizard does not activate the older stored automatic-discovery switch or domain DAV URL fields, which are not consumed by the current login/PIM runtime. Contacts and calendars use the managed, account-linked Nextcloud configuration. Connection/DNS diagnostics remain available on the created domain's dedicated diagnostics page.

The list and exclusive backend editor use the current admin theme tokens, wrapping tables and responsive field columns. Buttons retain intrinsic width and checkbox labels align alongside their inputs.
