# SMTP transport policies

In engine mode, Mailwoman sends mail by submitting it to the SMTP server configured for the account's domain. Two optional checks protect that connection: DANE (TLS certificates published in DNSSEC-signed DNS) and MTA-STS (a policy the mail domain publishes over HTTPS). They apply only to the connection from Mailwoman to that SMTP server. They do not make Mailwoman deliver mail to recipients' servers, and they do not change how that SMTP server delivers onward. In proxy mode the JMAP provider submits mail itself, and these checks do not apply.

## Before you start

- The SMTP connection must use a fixed encrypted method (TLS or STARTTLS) and no **TLS server name override**. The console refuses to save a policy otherwise.
- DANE needs TLSA records for the SMTP host and port, signed with DNSSEC. MTA-STS needs a domain that publishes a policy whose `mx` lines name the SMTP host.

## Turn the checks on

1. Open **Domains**, choose the domain, and on **Connections and discovery** open its SMTP backend, or choose **Add backend** and set **Protocol and service** to SMTP.
2. Under **Advanced transport security**, tick **Customize transport security**. Keep **Security selection** at **Fixed method** with TLS or STARTTLS, and leave **TLS server name override** empty.
3. Set **DANE verification** and **MTA-STS verification**, and for MTA-STS enter the **SMTP policy domain**.

   ![The DANE verification, MTA-STS verification and SMTP policy domain fields of an SMTP connection, with MTA-STS set to Follow the published policy and example.org as the policy domain.](/screenshots/admin-access/smtp-transport-policy-fields.webp){.mw-crop}

4. Save the connection.

Both checks are **Off** for a new or existing connection until you set them.

## Options

| Field | Values | Effect |
|---|---|---|
| **DANE verification** | **Off**, **Enforce when secure TLSA records exist**, **Required** | Looks up the TLSA records for the SMTP host name and port and validates their DNSSEC signatures. With usable records, the server's certificate must match them. **Required** refuses to send when no usable, validated records exist. |
| **MTA-STS verification** | **Off**, **Follow the published policy**, **Required** | Fetches the MTA-STS policy of the **SMTP policy domain**. An enforcing policy must list the SMTP host name in its `mx` lines, or sending is refused. A policy in `testing` mode does not block sending; **Required** refuses to send unless an enforcing policy applies. |
| **SMTP policy domain** | a domain name | The domain whose MTA-STS policy covers this SMTP server, usually your own mail domain. Recipient domains are never used. |

The checks run before Mailwoman authenticates or sends any part of a message. When usable DANE records exist, DANE decides and MTA-STS is not consulted; a certificate that does not match the DANE records is refused, with no fall back to MTA-STS or ordinary certificate checks.

DANE details: certificate usages DANE-TA (2) and DANE-EE (3), with full certificate or public-key selectors and full, SHA-256 or SHA-512 matching. Other usages are ignored. With **Enforce when secure TLSA records exist**, a signed record set that contains only unsupported records falls back to ordinary certificate validation (and to MTA-STS, if it is on); **Required** refuses it.

## DNS lookups and cached policies

- DNS queries are sent as DNS-over-HTTPS to `https://cloudflare-dns.com/dns-query`, or to the URL in `MW_SMTP_POLICY_DOH_URL`. DNSSEC signatures are validated by Mailwoman itself; there is no fall back to the operating system's resolver. These lookups and the MTA-STS policy fetches follow the server's outbound connection rules for the DNS class (see [Outbound connections](../security/egress.md)).
- MTA-STS policies are stored in the database, so they survive a restart: at most 1,024 policies of at most 64 KiB each. A stored policy is refreshed over HTTPS when a message is sent and the policy is more than a day old; a failed refresh is retried at most every five minutes, and an unexpired enforcing policy keeps applying meanwhile (with a warning in the log).
- Mailwoman does not send TLS-RPT reports.

These checks do not scan messages for viruses.

## Related

- [Domains](./domains.md)
- [Internationalized SMTP submission](./smtp-internationalization.md)
- [Outbound connections](../security/egress.md)
