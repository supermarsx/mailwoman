# LDAP directory configuration

Open **Admin → Integrations → LDAP directory configuration**. Viewing requires `integrations.read`; saving requires unscoped `integrations.write`.

The editor manages a priority-ordered set of LDAP/GAL servers: endpoint URL, search base DN, service bind DN, TLS mode, attribute mapping, search inclusion and connection policy. Changes are persisted atomically with an ETag. A stale editor must reload before saving. Removing a row removes its stored service credential too.

Configuration is loaded at server startup. The page compares the saved revision with the running directory's revision and reports when a restart is required. Saving does not test reachability and does not claim that a server is connected.

## Network consent

Every endpoint starts with **Block connections**. This also applies to pre-existing endpoints upgraded through migration 0037. Explicitly select **Allow guarded direct connections** to authorize connections. Private LAN and loopback destinations require the separate opt-in. Link-local addresses, cloud metadata endpoints, unspecified, multicast and reserved destinations remain refused. Every DNS answer must pass the policy; the connection is pinned to an accepted address while TLS verifies the original hostname.

LDAP proxy tunneling is not implemented in this transport. If an outbound proxy is active, LDAP connections fail closed, including connections from a directory that was already running before the proxy was activated. The editor reports this limitation; it never silently routes around the proxy.

Use `ldaps://` with LDAPS or `ldap://` with StartTLS. Plaintext supports anonymous directory access only. Service credentials and guarded LDAP authentication require TLS. URLs cannot contain credentials, queries, fragments or a search path; configure the search base separately.

## Service credentials

Passwords are sealed at rest using the server key and are never returned to the browser or audit log. Choose **Keep unchanged**, **Set or replace password**, or **Clear stored password**. Changing the endpoint or bind DN requires replacing or clearing its saved credential, preventing accidental credential forwarding to a new server.

`MW_DIRECTORY_BIND_DN` and `MW_DIRECTORY_BIND_PW` together pin the credential for that bind DN. The page shows the pin without revealing the password. Retargeting that endpoint or changing the pinned password requires changing the environment and restarting. Passwords are injected per endpoint, so another server using the same DN does not inherit the stored credential.

Directory configuration and credentials describe the deployment's network and remain excluded from `migrate-store`, consistent with the pre-existing directory configuration exclusion. Reconfigure them on the destination deployment.
