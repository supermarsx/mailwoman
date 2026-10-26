# IP bans

Open **Security centre → IP ban list** in the administrator console.

An active ban refuses application requests from that client IP, including requests
using an existing mailbox or administrator session. This is application admission,
not a TCP firewall. Existing connections are not forcibly disconnected; the next
HTTP request is checked. `/healthz` and `/readyz` remain accessible to deployment
probes. Fail2ban and firewall policies operate independently.

The source is the socket peer unless the existing trusted-proxy configuration
authorizes a forwarded address. Untrusted `X-Forwarded-For` cannot choose or evade
a banned source. Configure `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` correctly
before relying on individual client bans behind a reverse proxy. There is no
loopback or private-network exemption: banning a shared proxy or loopback address
can block every client represented by it. Use the local administration CLI to
remove a ban if the browser is locked out.

Ban management requires deployment-wide `bans.read` or `bans.write`; a grant
limited to domains cannot manage the shared source list. Adding or removing an entry also requires fresh administrator proof (`banManage`). The console opens a dedicated editor with an optional local-time expiry and an unsaved-change guard. Addresses must be single
IPv4 or IPv6 addresses. IPv4-mapped IPv6 is normalized to IPv4. Reasons are bounded
to 512 characters without control characters. Optional `expiresAt` must be a
future RFC3339 timestamp. At its exact deadline the ban stops applying; omitted
expiry means permanent. Invalid historical expiry is treated as active until an
administrator repairs or removes it. Listing retains expired entries for review. Bounded historical non-address identifiers can be deleted exactly, but cannot be created as new bans.

Concurrent updates use database compare-and-set, with bounded retries and a
10,000-entry ceiling. Expired entries are pruned when a new ban is added. Reads
check current database state, so unbanning takes effect on the next request across
replicas. A failed ban-store read refuses application requests with 503 rather
than silently bypassing enforcement.
