# Hosting panel connections

Open **Integrations → Hosting panels**. Each connection has its own create/edit page; domain and mailbox changes have separate detail pages. The supported providers are the Virtualmin remote API (Virtualmin is a Webmin module; a Webmin server without Virtualmin has no mail-domain model and is not supported) and HestiaCP's API. This integration does not provide panel-user login or single sign-on.

Use an HTTPS origin without a path, query, embedded login or fragment. Virtualmin uses `/virtual-server/remote.cgi` and its administrator login/password. Hestia uses `/api/`, an account owner and an access-key/secret-key pair. Grant only the commands required by the intended operations in the panel itself.

Credentials are sealed at rest and never returned by the API. Editing explicitly keeps, replaces or clears a credential. Changing its destination, provider identity, TLS authority or SSH tunnel requires replacing or clearing the saved credential. Enabling a connection requires a credential. Writes require an unscoped integration administrator, recent step-up and the current connection revision; a concurrent edit must be reloaded.

TLS certificate and hostname verification always remain enabled. A private CA, server-name override and TLS 1.2/1.3 minimum can be configured. Private or loopback destinations require explicit consent; unsafe metadata, link-local and reserved addresses remain blocked. An unsupported active proxy route fails closed. An unavailable SSH tunnel never falls back to a direct connection.

Inventory and mutations use fixed provider operations. Remote commands, paths and arbitrary provider output are not exposed. Available mailbox actions include creation, quota changes, password resets, enable/disable, removal and same-domain aliases. Domain creation and lifecycle actions are provider-specific. **Virtualmin domain lifecycle affects the entire virtual server, including websites, data and subservers**; it is disabled by default and requires a separate connection opt-in and target confirmation.

Virtualmin mailbox actions operate on its user account: disabling prevents its other logins as well, and deletion removes its home directory. Review the account before deleting or changing credentials. See the official [modify-user](https://www.virtualmin.com/docs/development/api-programs/modify-user/) and [delete-user](https://www.virtualmin.com/docs/development/api-programs/delete-user/) contracts. New users created here receive the default email-only access described by [create-user](https://www.virtualmin.com/docs/development/api-programs/create-user/).

Deletion requires typing the exact remote target. A request that was dispatched but did not produce a trustworthy completion response is reported as uncertain. It is never automatically repeated: inspect the remote panel before manually retrying. Removing a saved connection deletes its local credential and does not remove remote mailboxes or domains.

Saved hosting connections are deployment-local and intentionally excluded from `migrate-store`, together with their sealed credentials. Reconfigure them explicitly on the destination deployment.

## Local acceptance fixture

`node apps/web/e2e/fixtures/hosting-panel.mjs` runs a controlled Hestia-shaped HTTPS fixture on loopback port 42421 with the repository's public test certificate. It uses fixed fake credentials and must not be deployed. With a test Mailwoman server and built SPA running, set `PLAYWRIGHT_ADMIN_BASE_URL` and run `hosting-live.spec.ts` through `e2e/admin/playwright.admin.config.ts` from `apps/web`. The test exercises actual server-to-fixture HTTPS, connection persistence/redaction, domain/mailbox creation and quota updates in Chromium, Firefox and a phone viewport. Provider transport unit tests separately exercise Virtualmin HTTPS, CA/name failures and redirect/no-retry behavior.

These fixture checks do not establish compatibility with a particular production panel version or its local permissions. Verify that deployment's command permissions and behavior before operating on production data.
