# Domain-selected EWS accounts

The EWS mail backend runs in engine mode through the bundled, digest-pinned
`bridge-ews` WASM component. Proxy mode refuses an EWS selection. This first
provider path supports HTTPS with HTTP Basic authentication. NTLM and the
Graph/Gmail OAuth enrollment flows are separate work; choosing EWS does not
enable them.

The domain's version 2 backend document uses an explicit provider endpoint:

```json
{
  "version": 2,
  "backends": [
    {
      "id": "exchange",
      "role": "mail",
      "protocol": "ews",
      "enabled": true,
      "preferred": true,
      "ews": {
        "endpointUrl": "https://exchange.example/EWS/Exchange.asmx",
        "authentication": "basic"
      }
    }
  ]
}
```

The endpoint must be HTTPS, without embedded credentials, a query or a fragment.
An optional `caPem` contains at most eight public CA certificates and 64 KiB.
Those roots trust only the configured origin, including the origin inside a
proxy tunnel; they do not change trust for the proxy itself. Existing username
formatting applies to the backend login. The first verified login address becomes
the account's displayed identity; later sign-ins to that same backend identity
do not silently rename or move the account to another domain.

The operator must approve and enable `bridge-ews` and grant its `net` and
`account-backend` capabilities. Global, domain and applicable account feature
policies still apply. Outbound requests use the current **Plugins** egress
route. An internal Exchange destination needs an explicit matching Plugins
destination permission. Loopback, link-local and metadata destinations remain
forbidden. A failed proxy route never falls back to a direct connection, and
the bridge cannot redirect credentials to another endpoint.

On first sign-in, a transient plugin host authenticates and lists mailboxes
before any durable account or credential is created. The saved account identity
uses the exact canonical endpoint and authenticated backend username. Two
endpoint paths on one HTTPS origin are separate realms, even with the same
username. Repeated sign-ins to the same realm preserve account IDs and enrolled
factors. The password is sealed with account, endpoint, username and domain
binding as associated data. The legacy socket credential cell contains no
usable password.

Activation publishes the authenticated folder metadata before starting message
indexing. Password login, native tokens and enrolled passkeys retain the normal
second-factor, session and account-policy checks. The provider adapter checks
the current selected authority before opening a credential or issuing an HTTP
request. Changing the selected endpoint or authentication policy refuses the
old binding until a fresh sign-in; a rejected credential revision cannot be
silently restored at startup.

The unauthenticated domain diagnostic does not claim EWS authentication or
mailbox access. Full verification requires a provider sign-in. Controlled TLS
fixtures exercise the bundled guest and routing boundary; they are not evidence
of compatibility with every deployed Exchange version or authentication policy.
