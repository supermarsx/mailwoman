# Pre-provisioned users and the first-sign-in rule

`signin.newUsers` decides whether a sign-in may create a NEW account. It never affects an
account that already exists: an identity with an account (or a directory record of an earlier
sign-in) keeps signing in under every value, so tightening the rule cannot lock anyone out.

| Value | Who may create an account at first sign-in |
|---|---|
| `open` (default) | Any identity that authenticates against its mail server or identity provider. |
| `preprovisioned` | Only an address an administrator added in advance as a pending user (or a capped `*@domain` entry, when `signin.preprovision.domainWide` is on). |
| `provisioned` | The same, and also pending entries of any origin: imported and directory-synced users as well as admin-added ones. |
| `closed` | Nobody new. |

`preprovisioned` and `provisioned` differ only in the origin of the pending entry they accept
(admin-added only, or admin-added plus imported/synced). Existing accounts are admitted by all
values, so there is no further difference.

## Types and precedence

Rules are set per type of sign-in: `mail` (IMAP, POP3, Exchange password sign-in), `sso`
(OIDC, OAuth, SAML and trusted-header sign-in) and linked mailboxes (`linked`, with `imap`,
`pop3`, `ews` refinements). A linked mailbox is judged by the linked mailbox's own domain.
The first value that is not `inherit` wins: type for the domain, type global, (linked: provider
kind before the type), `signin.newUsers` for the domain, `signin.newUsers` global, `open`.

## Pending users

* Add: Security, Sign-in policy, Pre-added users (single, or a pasted list; at most 500 per
  request), or `POST {console}/preprovisioned-users`. Domain-scoped administrators can add,
  extend and revoke only inside their domains. `POST {console}/users` provisioning counts too.
* Lifetime: `signin.preprovision.ttlDays` by default, at most `signin.preprovision.maxTtlDays`.
  An entry may carry a note (200 characters).
* The first successful sign-in uses the entry up (`first_signin_at` is set by a
  compare-and-swap, and a domain-wide entry's use counter is spent by a compare-and-swap on its
  cap, so two concurrent sign-ins cannot both use one). A consumed entry is listed as Used.
* Revoke removes a pending entry; a used one cannot be revoked. Expired entries are removed by
  cleanup after `signin.preprovision.retentionDays`.
* Domain-wide entries (`*@domain`, capped number of uses) are administrator-only and off by
  default.
* No tokens and no invitation email are involved. The server has no transactional mail path
  for this, so tell the person to sign in with their existing mailbox password.

## Refusals

The check runs after the password or identity provider has been accepted and before anything
is stored, so a refusal leaves no account row, stored credential, session or engine
registration. With `signin.refusalDetail` = `uniform` the answer is the same 401 as a wrong
password; with `explain` it is 403 `signinNotProvisioned`, and the login screen says that no
account exists for the address. Each refusal is written to the security audit log as
`signin-new-user-refused` with reason `new_user_refused`, the rule, the type and the domain
(never a credential).

## Read-only check

`GET {console}/signin-policy/preview?address=...&type=mail` reports the rule that applies and
whether a first sign-in would be refused. Nothing is written.
