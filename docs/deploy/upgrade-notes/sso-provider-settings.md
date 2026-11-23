# Upgrade note: single sign-on provider settings and SAML signatures (26.20)

Applies to deployments with at least one single sign-on provider (**Single sign-on** in the admin console). Deployments without one need do nothing.

## What changes

**Providers saved from the console.** Before this release, the console's single sign-on editor sent the protocol settings under names the server did not read. A provider created or saved in the console was stored with an empty issuer URL, client ID, redirect URL and first-login policy (OpenID Connect), or with empty SAML settings (IdP SSO URL, signing certificates, entity ID, ACS URL, and **Require signed assertions** stored off). Such a provider cannot sign anyone in. From this release the server reads the editor's settings, shows stored settings in the editor, and refuses a save with a missing or invalid field, naming it.

**SAML signatures.** Before this release, a SAML provider stored with **Require signed assertions** off accepted a response that carried no signature at all. That included every SAML provider saved from the console, and rows written through the console API or directly in the database that left the setting out (a missing setting was read as off). A provider in that state that was enabled and had an IdP SSO URL (so that a sign-in could be started) could be used by anyone who could reach this server to sign in as any account the provider was allowed to sign in. Providers saved from the console had an empty IdP SSO URL, so no sign-in could be started with them.

From this release:

- a valid signature by one of the provider's certificates, on the assertion or on the whole response, is always required, whatever the setting says;
- a response that carries more than one assertion is refused;
- a provider without a signing certificate, an entity ID or an ACS URL signs nobody in;
- a provider cannot be saved with **Require signed assertions** off, and a stored provider without the setting reads it as on.

## What to do

1. **Find the affected providers.**
   - In the console's provider list API, `GET <console path>/sso` (signed in as an administrator with `sso.read`), each provider has a `problems` list. A non-empty list names the settings that stop it from signing in; an entry with `field` `config.wantAssertionsSigned` marks a SAML provider stored with the setting off. The console page does not display this list.
   - In the database, SAML providers stored with the setting off, or without it:

     ```sql
     SELECT id, display_name, enabled FROM sso_config
     WHERE kind = 'saml'
       AND (config LIKE '%"want_assertions_signed":false%'
            OR config NOT LIKE '%"want_assertions_signed"%');
     ```

     After the upgrade a row without the setting reads as on, but before the upgrade it was read as off, so include those rows when you review the past.
2. **Re-enter and save each provider** whose protocol settings are empty, in **Single sign-on** in the console. For SAML providers, turn **Require signed assertions** on; the server does not save the provider otherwise.
3. **Review past single sign-on sign-ins** for every SAML provider found in step 1 that was enabled and had an IdP SSO URL, for the time it was in that state. Two records exist:
   - **Logs** > **Authentication logs** in the console: each sign-in that created a mailbox session has a row with **Method** `sso`, the account, the time, the source IP and the browser. These rows do not say which provider was used. They are kept for [`logs.auth.retentionDays`](/admin/settings#logs.auth.retentionDays) (1095 days by default) and at most [`logs.auth.maxEntries`](/admin/settings#logs.auth.maxEntries) rows.
   - The database table `sso_login_audit`: one row per single sign-on attempt with the time (`ts`), the provider (`provider_id`), `kind` and `outcome` (`ok` or `error:<reason>`). It does not record the account or the source address; `subject_hash` is the SHA-256 (hexadecimal) of the subject the provider asserted, so you can compare it with the hash of a subject you expect. The console has no screen for this table. Rows are kept for [`retention.ssoLoginAudit.days`](/admin/settings#retention.ssoLoginAudit.days) (365 days by default).

   To match the two, take the `ok` rows of an affected provider in `sso_login_audit` and look for authentication log rows with **Method** `sso` at the same time. Treat a sign-in you cannot account for as a compromised session: end the account's sessions in **Security centre** > **Sessions** and check the account.

## Related

- [Single sign-on](/admin/sso)
- [Logs](/admin/logs)
- [Sessions](/admin/sessions)
