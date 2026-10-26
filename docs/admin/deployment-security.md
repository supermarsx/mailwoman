# Deployment security assessment

Open **Diagnostics → Deployment security assessment**. The read-only report requires
an unscoped `diagnostics.read` grant and is returned with `Cache-Control: private, no-store`.

The report describes the process serving the request: trusted effective HTTPS,
secure-cookie and HSTS decisions, strict CSRF configuration, enforced administrator
session limits, stored factor enrollment, sign-in pacing and credential-failure
locks, local country-policy readiness, and document worker configuration.
Administrator factor coverage includes enabled environment-bootstrap accounts and
recognizes both TOTP and passkeys. Disabled administrators are excluded.

Session limits are the clamped runtime values, not merely submitted settings.
Configuration links lead to the existing policy editor; they do not bypass its
permissions. Filtering and refreshing do not change deployment configuration.

This report does not probe the public TLS endpoint or firewall, authenticate to an
upstream service, execute the document worker, or certify the deployment. A worker
path being configured does not demonstrate that its isolation works. External
reverse proxies and other replicas can have different configuration; inspect them
separately. No secrets, raw environment values or filesystem paths are returned.
