---
screen: true
---
# Security policy

The **Security policy** tab of the **Security centre** (`#/security/policy`) holds a
few deployment-wide rules: the lowest TLS version, whether every account needs a
second factor, the password hashing cost, the reading floor, screen capture
protection and the stored data loss prevention (DLP) rules.

![The Security policy tab of the Security centre: the Connections group with the lowest TLS version, the Sign-in group with the second-factor requirement and the Argon2 cost, the Reading group, and the Save bar.](/screenshots/admin-access/security-policy-tab.webp){.mw-shot}

## Before you start

- Viewing needs **security.read**; saving needs **security_policy.write**.
- Two fields can also be set by an environment variable, whose name is shown beside
  the label: **Lowest TLS version** (`MW_ADMIN_MIN_TLS`) and **Require a second
  factor for every account** (`MW_ADMIN_REQUIRE_2FA`). When the variable is set, the
  field cannot be changed here; the server refuses the change.

## Change the policy

1. Open **Security centre** > **Security policy**.
2. Change the fields. The bar at the bottom shows that there are unsaved changes.
3. Select **Save**, or **Discard changes**.

The server that receives the save applies it at once. Every other server process
that shares the database reads the policy again within 30 seconds. **History**
opens the change history of the policy. An out-of-range value is refused with the
reason, and nothing is saved.

## Connections

**Lowest TLS version**: **TLS 1.2 and later** (default) or **TLS 1.3 only**. It
applies to the HTTPS listener (when Mailwoman terminates TLS itself) and to new
outgoing IMAP, SMTP, POP3, ManageSieve and outbound-proxy connections. Connections
that are already open keep their version. It does not apply to TLS terminated by a
reverse proxy in front of Mailwoman.

## Sign-in

![The Sign-in group of the Security policy.](/screenshots/admin-access/security-policy-signin.webp){.mw-crop}

| Field | Default | What it does |
|---|---|---|
| **Require a second factor for every account** | off | The same rule as **Require two-factor for the whole deployment** on [Two-factor & sign-in requirements](./security-center.md#two-factor-sign-in-requirements). |
| **Argon2 memory, in KiB** | 19456 | Memory for each new hash, 19456 to 1048576. With a single pass at least 47104. |
| **Argon2 passes** | 2 | 1 to 10. |
| **Argon2 parallelism** | 1 | Threads per hash, 1 to 16. |

The Argon2 cost applies to new hashes of console passwords, locally stored passwords
and recovery codes. A higher cost makes each sign-in use more memory and time on the
server.

With **Require a second factor for every account** on, a mailbox account that has
no second factor is asked to set one up at its next sign-in to the web app (see
[Two-factor & sign-in requirements](./security-center.md#two-factor-sign-in-requirements)).

::: warning
The setup is done by whoever next signs in with the account's password. If a
password may be known to someone else, have the account holder set up a second
factor in their own settings before you turn this on.
:::

## Reading

![The Reading group of the Security policy.](/screenshots/admin-access/security-policy-reading.webp){.mw-crop}

| Field | Default | What it does |
|---|---|---|
| **Reading floor** | **No floor** | The least restrictive way the Mailwoman reader may show a message: **Sanitised HTML with media**, **Sanitised HTML without remote media** or **Plain text only**. A person can still choose a stricter view. |
| **Screen capture protection** | **Off** | **Protect** asks the Mailwoman desktop and Android apps to keep their window out of screenshots and screen recordings. |

Both are applied by Mailwoman's own apps after they read the policy. Other mail
clients that connect to the same mailboxes do not see them, and a browser cannot
block screenshots. See [Message view modes](/security/max-security) and
[Screen capture](/security/screen-capture) for what each app does.

## Data loss prevention

**Stored rules** are checked on outgoing mail sent through this server in engine
mode, beside any rules in `MW_DLP_RULES` (listed read-only under **Rules from
MW_DLP_RULES**; an environment rule with the same id replaces a stored one). Each
rule has a **Name**, **Rule is on**, a **Priority**, an action under **When a
message matches** (**Warn the sender**, **Block the message**, **Require
encryption**, **Send and notify an administrator**), an optional **Message to the
sender**, and what to **Look for**: built-in detectors, **A pattern of my own**,
**Recipient domains**, **Attachment types** and **Largest attachment, in bytes**.

What the rules inspect, how they behave in proxy mode and what they do not cover is
described in [Outbound DLP](/security/dlp).

## Related

- [Security centre](./security-center.md)
- [Admin console security](/security/admin-panel#security-policy)
- [Outbound DLP](/security/dlp)
- [Settings reference](./settings.md)
