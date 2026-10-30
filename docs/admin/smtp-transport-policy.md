# SMTP transport policies

Mailwoman submits messages to a configured SMTP relay. These controls protect
that connection. They do not turn Mailwoman into a recipient-MX delivery server
or configure the relay's onward delivery policy.

In **Admin → Domains → Backends**, edit an SMTP backend and enable **Customize
transport security**. Select a fixed TLS or STARTTLS method, then configure:

- **DANE:** Off, enforce when securely validated TLSA records exist, or Required.
  The lookup names the actual SMTP hostname and port. Required mode refuses an
  endpoint without usable, DNSSEC-validated TLSA records.
- **MTA-STS:** Off, follow the published policy, or Required. Set the policy
  domain explicitly to the domain publishing a policy for this relay. Its MX
  patterns must authorize the configured hostname. Recipient domains are never
  substituted for this setting.

Both controls default to Off for existing configurations. Enabled policies
require a fixed encrypted connection method and do not permit a TLS server-name
override. Policy checks finish before SMTP authentication or message transfer.
JMAP proxy accounts delegate SMTP delivery to their upstream provider.

When usable, securely validated DANE records exist, DANE authentication takes
precedence over MTA-STS. A failed DANE match cannot fall back to MTA-STS or normal
certificate validation. Without usable DANE records, the configured MTA-STS
policy applies.

DANE supports DANE-TA (usage 2) and DANE-EE (usage 3), certificate and public-key
selectors, and full, SHA-256 and SHA-512 matching. The strongest supported digest
is preferred for each usage/selector pair. A securely signed set containing only
unsupported records still requires TLS and normal certificate validation in
opportunistic mode; Required mode refuses it. Other DANE certificate usages are
not implemented.

MTA-STS `testing` policies do not block delivery in automatic mode; Required mode
requires an enforcing policy when DANE does not take precedence. Mailwoman does
not currently generate aggregate TLS-RPT reports.

The persisted endpoint representation is:

```json
{
  "host": "smtp.example.org",
  "port": 465,
  "tls": "implicit",
  "security": {
    "mode": "fixed",
    "allowed": ["implicit"],
    "smtpPolicy": {
      "dane": "required",
      "mtaSts": "auto",
      "policyDomain": "example.org"
    }
  }
}
```

DNS wire requests use HTTPS through the `dns_auth` egress class. The default
resolver is `https://cloudflare-dns.com/dns-query`; operators can set
`MW_SMTP_POLICY_DOH_URL` to another public HTTPS DNS wire resolver. Enabling a
policy authorizes these lookups. DNSSEC validation is local: a resolver's AD flag
alone is not proof. A configured egress route must succeed; failures do not
trigger a direct-network fallback.

MTA-STS policies are stored in the application database and survive restarts.
The cache holds at most 1,024 entries of at most 64 KiB each, removes expired
entries, and refuses new entries rather than evicting unexpired protections.
An older concurrent fetch cannot replace a newer cached policy. An unexpired
enforcing policy remains applicable when a refresh fails.
Known policies are refreshed through HTTPS during submissions after one day,
even if their TXT discovery record is unavailable. Failed refreshes are retried
no more often than every five minutes and emit a bounded warning for enforcing
policies. A failed refresh never extends a policy's original expiry.

Standards: [SMTP DANE](https://www.rfc-editor.org/rfc/rfc7672.html),
[MTA-STS, including smart-host policy selection in section 3.4](https://www.rfc-editor.org/rfc/rfc8461.html).

Antivirus scanning is separate and is not provided by these controls.
