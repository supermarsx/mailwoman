# Sieve transport and local execution

Mailwoman stores mail rules as Sieve scripts. It can upload them to the mail
server's ManageSieve service, and it can run an account's active local script
itself when new mail arrives in the Inbox. This page covers
the endpoint configuration, the network path, the policy settings and what the
local evaluator supports.

## Per-mailbox rules

In webmail, **Settings** > **Mail rules** applies to the selected mailbox,
including a verified linked mailbox. Visual rules, scripts and vacation replies
are kept separately for each account. Switching accounts clears the editor and
cancels pending operations from the previous account. The preferences
import/export tool does not include rules or scripts.

## Policy settings

These settings can be set globally, per domain or per account, and can be
locked. The selected mailbox's own policy applies; linking an account does not
bypass it.

| Setting | What it controls | Default |
|---|---|---|
| `cap.rules` | Whether the account may use mail rules | true |
| `cap.rules.forward` | Whether rules may forward | true |
| `sieve.maxScriptBytes` | Largest script, in bytes (1-1,048,576) | 65,536 |
| `sieve.maxScripts` | Number of scripts (1-100) | 5 |
| `sieve.maxRedirects` | Redirects per script (0-100) | 4 |
| `sieve.vacationAllowed` | Vacation replies | false |
| `sieve.allowedExtensions` | Sieve extensions the account may use | null |
| `cap.sieve.raw_edit` | Editing the raw script text | false |
| `cap.sieve.forward_external` | Redirects outside the account's domain | false |

See the [Settings reference](../admin/settings.md).

## Configure a ManageSieve endpoint

The endpoint comes from the domain's upstream configuration for the signed-in
account. A request cannot override the host, port, TLS mode, user name, password
or proxy. The account's sealed mail credentials are used to sign in.

```json
{
  "sieve": {
    "host": "sieve.example.org",
    "port": 4190,
    "tls": "starttls",
    "allowPrivate": false
  }
}
```

| Field | Values |
|---|---|
| `host`, `port` | The ManageSieve server (usually port 4190) |
| `tls` | `starttls`, `implicit` or `none` |
| `allowPrivate` | `true` allows a private (RFC 1918), IPv6 unique-local or loopback address. Default `false`. |

Link-local, cloud metadata, multicast and unspecified addresses, and IPv4
addresses embedded in IPv6 transition forms that point at a blocked address,
are always refused.

Without an endpoint, ManageSieve requests answer
`this account has no configured ManageSieve endpoint`.

## Network path

- An egress route can select the `sieve` traffic class. A domain route takes
  precedence over a global one.
- HTTP CONNECT, HTTPS CONNECT and SOCKS5 routes carry the already-resolved
  destination address. SOCKS5h is refused for Sieve, because it would resolve the
  name remotely.
- Every DNS answer is checked before any socket is opened.
- TLS runs end to end between Mailwoman and the Sieve server, using the
  configured host name.
- If a selected route fails, the connection fails; it never retries directly.
- With `tls: "none"` and a selected route, the route must allow plaintext.
- Connecting and signing in to the Sieve server has a 15-second timeout.

## Uploading a script

Before uploading, Mailwoman checks the account policy against the extensions
the server actually offers, checks the script count and available space, and
runs `CHECKSCRIPT`. If the policy check fails, no script content and no
`PUTSCRIPT` command is sent. A failed connection to a configured server does not
fall back to local execution.

## Local execution

An account's active local script runs when new messages are added to its Inbox,
so rules take effect as quickly as the account syncs.

**Actions:** branching (`if`/`elsif`/`else`), `stop`, `keep`, `discard` (moves
to Trash), `fileinto`, `:copy`, IMAP flags, and `redirect` with a loop marker.
Explicit flags on `keep` and `fileinto` are supported.

**Tests:** `header`, `address`, `envelope`, `size`, `hasflag` against the
current flags, `allof`/`anyof`/`not`, `date`/`currentdate`, `index` and
relational comparisons. Comparators: `i;octet`, `i;ascii-casemap` and
`i;ascii-numeric`.

**Extensions:** `fileinto`, `imap4flags`, `envelope`, `copy`, `subaddress`,
`vacation`, `vacation-seconds`, `date`, `index`, `relational` and
`comparator-i;ascii-numeric`.

Not supported locally, and not advertised: full MIME `body` matching,
`include`, `variables`, `regex`, `mailbox` and `duplicate`. A script that uses an
unsupported extension, command or tag is rejected even if that part would never
run. Unresolved `include` and unknown commands also fail validation for scripts
sent to a ManageSieve server.

External redirects need the account policy to allow them. The policy is checked
again when the message is sent, so revoking the permission takes effect without
changing the script.

### Vacation replies

- `vacation` and `vacation-seconds` need `sieve.vacationAllowed`.
- Replies use an empty SMTP sender (`MAIL FROM:<>`) and are not sent to
  automated, list, bulk or non-personal messages, to the account itself, or in
  loops.
- Before sending, Mailwoman records the sender and reply interval for the
  account. If the server crashes or SMTP fails after that, the reply for that
  interval can be lost.
- `:from` must be the account's own address.
- The vacation reply set up in the rules screen uses a 7-day interval; it cannot
  be changed there.

## Scripts over JMAP and API keys

The SieveScript get, set, query and validate methods report only the
capabilities of the local evaluator or the configured ManageSieve server,
limited by policy. Changes through MailRule and VacationResponse update the
managed blocks of a script and leave other handwritten parts alone. Raw editing
needs `cap.sieve.raw_edit`. A scoped API key needs the scripts grant for the
matching account and cannot reuse a browser's step-up proof.

ManageSieve changes and the local copy of a script are not updated in one
transaction.

## Related

- [Egress proxy](egress-proxy.md)
- [Domains](../admin/domains.md)
- [Settings reference](../admin/settings.md)
