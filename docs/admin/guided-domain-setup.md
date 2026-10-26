# Guided domain setup

Admin → Domains → guided setup has three connection choices:

- **Server defaults** creates the domain without an upstream override.
- **Configure a mail server** retains the compact single IMAP or POP3 endpoint plus optional SMTP setup (or an HTTPS JMAP URL in proxy mode).
- **Configure multiple backends and advanced options** builds a version-2 backend list. Changes stay in the wizard until the final review and Create domain request.

The advanced list supports up to 32 uniquely identified backends. Each has an explicit protocol/service role, enabled state, numeric priority and preferred selection. At most one enabled backend is preferred per service role. Lower priority values win after preferred selection; this does not authorize retrying rejected credentials against another backend.

Engine mode currently offers IMAP, POP3, SMTP, ManageSieve and EWS HTTPS Basic. Proxy mode offers JMAP session URLs. These modes cannot be mixed by this wizard. Microsoft Graph/365 OAuth, Gmail APIs, NTLM and MAPI are not selectable: these are not activated domain transports. Microsoft 365 tenants that require OAuth cannot use this Basic-only EWS integration.

Socket backends expose full-email, local-user, domain\\user, user@realm and custom-template username formats, plus the existing server-side username preview. The wizard never collects account passwords. Authentication method choices come from the server's actual protocol matrix. Fixed selection uses one mechanism; ordered selection uses distinct supported mechanisms. Channel-binding requirements and rejection behavior remain enforced by the protocol runtime.

The shared transport editor supports fixed TLS/STARTTLS/plaintext selection or an explicitly ordered allowed list when the server reports that runtime feature ready. Plaintext is an explicit insecure choice. Certificate chain and hostname verification remain enabled. A custom CA and server-name override are available through the existing transport policy; they do not disable certificate verification. Private destinations require the existing explicit private-target policy. DNS hostnames are resolved on connection according to that runtime policy; setup does not pin an address or add an egress grant.

**Discover configuration** invokes the existing server-side discovery endpoint through its configured outbound route. It needs unscoped upstream configuration permission. The returned endpoints remain an unsaved proposal until reviewed. OAuth-required results show a persistent warning: importing endpoint information does not configure OAuth or make provider-native sign-in available.

This wizard does not activate the older stored automatic-discovery switch or domain DAV URL fields, which are not consumed by the current login/PIM runtime. Contacts and calendars use the managed, account-linked Nextcloud configuration. Connection/DNS diagnostics remain available on the created domain's dedicated diagnostics page.

The list and exclusive backend editor use the current admin theme tokens, wrapping tables and responsive field columns. Buttons retain intrinsic width and checkbox labels align alongside their inputs.

## Exchange Web Services

Choose advanced setup, add a mail backend and select **EWS**. The same fields are available on the dedicated Add backend and Edit backend pages. Enter the exact HTTPS endpoint, normally ending in `/EWS/Exchange.asmx`, and optionally a public CA bundle (1–8 PEM certificates, at most 64 KiB). The server validates certificate encoding; hostname and chain verification stay enabled. Credentials, URL query strings, fragments and private keys are rejected. CA trust applies to the configured origin, not the outbound proxy.

EWS uses the signing-in user's password through HTTPS Basic. Username formatting and preview remain available; the form stores no account password. Priority, preferred selection and enabled state use the same rules as other mail backends. When EWS is selected, active SMTP and Sieve backends must be disabled because EWS owns submission. Switching protocols removes incompatible socket, JMAP and EWS fields.

Saving validates configuration only. The approved `bridge-ews` plugin and its grants must be available, and a private Exchange destination requires an explicit **Plugins** outbound destination permission. No checkbox in this editor bypasses the egress policy. Unauthenticated domain diagnostics do not probe EWS credentials or establish provider readiness; the UI states that limitation rather than reporting success or a missing endpoint. Actual authenticated service operation remains a separate runtime check.
