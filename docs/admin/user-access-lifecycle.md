# User access lifecycle

Admin **Users** distinguishes a directory access block from the older per-account disabled flag. Block and remove actions require a current revision, the matching domain permission, and fresh administrator proof. Removing local access requires typing the directory address.

- **Block login** holds Mailwoman access and revokes matching mailbox sessions.
- **Unblock login** removes the directory hold. Other authentication and security policies continue to apply.
- **Clear legacy login block** is a separate explicit action. It uses compare-and-swap on the stored flags and changes only `disabled`; password-change, zero-access, and other flags are retained. Clear this flag before changing the directory hold.
- **Remove Mailwoman access** tombstones the directory record. It does not erase upstream mailboxes, messages, account settings, or history. There is no restore operation in this interface.

Fresh sign-in, existing mailbox sessions and account-token admission consult the directory hold. Successful upstream authentication cannot automatically re-enroll a blocked or tombstoned directory identity. Provisioned users can be removed before first sign-in. Legacy mixed-case and Unicode/IDNA address spellings are matched using the same canonical identity for holds and revocation; a folded duplicate cannot re-enroll around the hold. A parent-domain hold also applies to its configured login aliases; matched alias sessions are revoked. Unrelated usernames sharing an upstream account ID are not revoked solely because of that ID.

This lifecycle governs mailbox access. It does not modify administrator accounts, roles, credentials, or the independent last-superadministrator safeguards. Deleting a mailbox access record is not a way to delete an administrator.

The endpoints are `GET/PUT/DELETE /admin/users/{id}/lifecycle` and `GET/DELETE /admin/users/{id}/legacy-block` (under the configured admin base). Writes require a concrete `If-Match`: missing/wildcard is 428, changed state is 412, and attempts to restore a tombstone are 409. Read and write authorization uses the resolved directory policy domain, never an arbitrary client-supplied domain.

Held-address matching uses the existing status index and a maximum of 5,001 rows to detect an excessive hold set. More than 5,000 held records refuses admission rather than allowing an incomplete lookup.
