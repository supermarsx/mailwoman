# Upgrade note: sign-in locks and two-factor enrolment (26.20)

Applies to every deployment. No action is required; this note says what behaves differently after the upgrade.

## Temporary sign-in locks

**Before:** a lock and its failure counter belonged to one exact source address, IPv6 included. At most 10,000 address-and-surface records were kept; when the table was full, every sign-in from an address that was not already in it was refused, before the password was checked. Failed sign-ins from enough different addresses (easily done from one IPv6 network) could therefore stop new sources from signing in at all.

**After:**

- An IPv6 source is its /64 network. Rejected passwords from any address in the /64 count together, and a lock covers the whole /64. The **Temporary sign-in locks** list shows the start of the network (for example `2001:db8:1:2::`). IPv4 sources are unchanged.
- When the 10,000 records are full, a new source replaces the record that failed least recently and is not locked. Active locks are never dropped. Only if every record is an active lock are failures from a new source not counted until a lock ends; that source can still sign in and is still slowed by the sign-in retry cooldown.
- Locks recorded for an exact IPv6 address before the upgrade still apply, and can still be unlocked, until they expire.

See [Temporary sign-in locks](/admin/signin-locks).

## Required two-factor and accounts without a second factor

**Before:** when two-factor sign-in was required for an account (deployment-wide, for its domain, or for the account) and the account had no second factor, the web app stopped at "This account requires two-step verification, but no method is set up yet", and the person could not sign in.

**After:** the web app's sign-in screen asks the person to set up a second factor (an authenticator app, or a passkey where passkeys are allowed and the browser supports them; the desktop and Android apps offer the authenticator app), shows the one-time recovery codes, and then signs in. The setup is done by whoever signs in next with the account's password; if a password may be known to someone else, have the account holder set up a factor in their own settings first.

See [Two-factor & sign-in requirements](/admin/security-center#two-factor-sign-in-requirements) and [Security policy](/admin/security-policy).
