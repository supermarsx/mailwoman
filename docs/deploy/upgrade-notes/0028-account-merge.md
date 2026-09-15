# Upgrade note: migration 0028, one account per mail identity

Applies to every deployment that has run in **engine mode** (`MW_MODE=engine`,
logging in to an IMAP or POP3 server). Proxy-mode deployments have no engine-mode
account rows, so the migration only adds an index for them.

## What the migration does

Before this release, every engine-mode login created a new account row, even
when the same user had logged in to the same server before. A database that has
been in use therefore holds several account rows for one person.

Migration `0028_account_identity` runs once, the first time the new server
opens its store, and merges them:

- Rows count as one identity when kind (IMAP or POP3), server host, port and
  username match. Upper and lower case in the host and username, and a trailing
  dot on the host, are ignored.
- One account in each group is kept, the survivor. Everything the other rows
  owned is moved onto it: cached mail, mailboxes, notes, calendars, contacts,
  settings, sessions, API keys and the rest. **No mail is deleted.** When two rows
  cached the same upstream message, or the same folder, the survivor keeps one
  copy.
- The emptied rows are deleted, and a unique index then prevents a second row
  for the same identity from being created.

The migration runs in a single transaction. If it fails, nothing is changed and
the server does not start.

## Second factors: only the earliest enrolment is kept

The survivor is the account whose second factor was created first: a confirmed
TOTP authenticator, a passkey or a set of recovery codes. Ties go to the lowest
account id. If no account in the group has a second factor, the lowest account
id is kept.

If more than one of the merged accounts had its own 2FA enrolment, **only the
survivor's factors are kept**. TOTP secrets, passkeys and recovery codes on the
other accounts are removed, and so are TOTP enrolments that were started but
never confirmed.

Each removed factor is recorded in the audit log, with:

| field | value |
|---|---|
| `actor` | `migration-0028` |
| `actor_kind` | `system` |
| `action` | `twofa-factor-dropped-on-merge` |
| `target` | the surviving account's id |
| `detail_json` | `factor` (`totp`, `passkey` or `recovery-codes`), `duplicateAccountId`, `survivorAccountId`, `createdAt`; also `confirmed` for TOTP, `credentialId` for a passkey, and `count` / `unused` for recovery codes (one row per account) |

The rows contain no secrets: no TOTP secret, no key material and no recovery-code
hash.

To find them:

- **Admin panel:** the audit log (`GET /admin/audit?limit=…`, or the export at
  `GET /admin/audit/export`). The rows are timestamped at the moment of the upgrade.
- **Directly in the database:**

  ```sql
  SELECT ts, target, detail_json
    FROM audit_log
   WHERE action = 'twofa-factor-dropped-on-merge'
   ORDER BY target, ts;
  ```

  To see which login a surviving account belongs to:
  `SELECT kind, host, port, username FROM accounts WHERE id = '<target>';`

No rows means no account had competing enrolments. Nothing further is needed.

## Why, and what to review

In engine mode, the second-factor check looked factors up under the account row
of the current login. Every login made a new row, so the check found nothing and
the password alone was enough. With an admin require-2FA policy, every login was
sent to set up a new authenticator instead. **Before this fix, anyone who knew an
account's password could enrol their own authenticator in engine mode.**

That is why a later enrolment is not trusted to belong to the account's owner,
and why the migration keeps only the earliest one. The earliest is the best
evidence available, but it is not proof.

For each account named in the audit rows:

- Review whether the removed enrolments were expected, for example a user who
  set up a phone and later a laptop.
- **Consider requiring a password change.** In engine mode the password is the
  mail server's, so change it on the mail server, or through Mailwoman's
  password-change backend if one is configured
  ([`../../security/password-change.md`](../../security/password-change.md)).
- Consider revoking the account's existing sessions
  (`POST /admin/users/{account_id}/revoke-sessions`). The merge moves sessions onto
  the surviving account; it does not end them.
- Review the account's API keys and webhooks (`GET /admin/api-keys`,
  `GET /admin/webhooks`) and its OAuth tokens (the `oauth_tokens` table). Like
  everything else the removed rows owned, they now belong to the surviving account,
  including any created in a session that never passed 2FA.

## What users notice

- A user who enrolled a second device on one of the removed accounts is asked for
  the surviving factor at their next login, and has to enrol that device again.
- Recovery codes issued to a removed account no longer work. The survivor's codes
  still do.
- The first sync after the upgrade may refetch some messages. Mail already cached
  is kept.

## Moving to Postgres with `mailwoman migrate-store`

The unique index also exists in the Postgres schema. `migrate-store` copies the
SQLite file as it is and does not migrate it first. So open the SQLite store once
with this release (start the server, or run any command that opens the store)
before copying it. Otherwise the copy stops at the first duplicate account with a
unique-constraint error. The copy runs in a single transaction, so the target is
left unchanged.
