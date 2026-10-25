# Sieve transport and local execution

ManageSieve endpoints are selected from the signed-in account's configured domain. Requests cannot override the host, port, transport security, username, password, or proxy. Credentials come from the account's sealed mail credentials.

An operator endpoint can use this shape inside the domain upstream configuration:

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

`allowPrivate` defaults to false. An operator may explicitly enable it for an RFC1918, IPv6 unique-local, or loopback endpoint. Link-local, cloud metadata, multicast, unspecified, and IPv4 transition addresses embedding a blocked destination remain blocked.

Egress routes can select the `sieve` traffic class. A domain route takes precedence over a global route. HTTP CONNECT, HTTPS CONNECT, and SOCKS5 carry the pinned destination address. All DNS answers are validated before opening either the endpoint socket or proxy socket. The protocol client performs end-to-end TLS using the configured endpoint hostname. A configured route failure is final; it never retries directly. SOCKS5h is refused for this transport because it cannot preserve the local DNS pin. Plaintext Sieve requires both an explicitly plaintext endpoint and a selected route permitting plaintext.

Before an upload, the checked ManageSieve client validates account policy against the actual upstream extension inventory, checks script count and available space, and runs CHECKSCRIPT. Policy failures send no script content or PUTSCRIPT command.

Accounts with an active local script execute it when new INBOX messages are ingested. Execution latency therefore follows account sync latency. The local evaluator supports branching, stop, keep, discard-to-Trash, fileinto, copy, IMAP flags, and redirects with a loop marker. Tests include header/address/envelope/size, hasflag against the current flag set, boolean combinations, date/currentdate, index, and relational comparisons. String comparisons support i;octet, i;ascii-casemap, and i;ascii-numeric. Explicit keep/fileinto flags are supported. External redirects require the current account policy to permit them; policy is checked again at delivery, so revocation takes effect without rewriting the script.

Local vacation and vacation-seconds require the vacation permission. Replies use a null SMTP reverse path, suppress automated/list/bulk/self/loop/nonpersonal messages, and reserve a persistent per-account sender/response interval before sending. A crash or SMTP failure after reservation can suppress a reply for that interval. Vacation :from is restricted to the account identity. The current GUI uses a seven-day interval; configurable days and addresses remain pending.

Unsupported extensions, commands, and tags are rejected even in an unmatched branch. Full MIME body matching, include, variables, regex, mailbox, and duplicate-state execution remain unavailable locally and are not advertised. Unresolved include and unknown commands also fail policy validation for remote scripts.

Public SieveScript get/set/query/validate handlers expose account capabilities only for the available local evaluator or configured ManageSieve service, intersected with policy. MailRule and VacationResponse writes update protected managed blocks while preserving unrelated handwritten source. Raw editing has a separate permission. Scoped API keys require the scripts grant and matching account; they cannot inherit browser step-up proof. Failed configured upstream connections do not fall back to local execution. ManageSieve mutations and local snapshot persistence are not a distributed transaction.
