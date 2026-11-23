---
screen: true
---
# First sign-in and pre-added users

The first time someone signs in, Mailwoman creates their account. The **Sign-in policy** tab of the **Security centre** decides who may get an account that way: anyone whose mail server or identity provider accepts them, only addresses you added in advance (**Pre-added users**), or nobody new. People who already have an account are never affected, so tightening the rule does not lock anyone out.

![The Sign-in policy tab of the Security centre: the introduction and the start of the First sign-in rule by type table.](/screenshots/admin-access/preprovisioned-tab.webp){.mw-shot}

## Before you start

- Open **Security centre**, then the **Sign-in policy** tab. Changing the rules needs the settings write permission; adding, extending and revoking pre-added users needs the user create, write and delete permissions. An admin limited to some domains can manage pre-added users of those domains only.
- Under **Preprovisioned** or **Provisioned**, add the addresses you expect before you switch, or new people will be refused.

## The first sign-in rule

Each row of **First sign-in rule by type** takes one of these values:

![The First sign-in rule by type table: New users set to Open and the password, single sign-on and linked mailbox rows set to Inherit.](/screenshots/admin-access/preprovisioned-matrix.webp){.mw-crop}

| Value | Who may get a new account at first sign-in |
|---|---|
| **Open** | Anyone who authenticates against their mail server or identity provider. |
| **Preprovisioned** | Only an address added in advance as a pending user under **Pre-added users** (or with **Provision user** on **Users**), or covered by a domain-wide entry. |
| **Provisioned** | As **Preprovisioned**, and also pending users that came from an import or a directory sync. |
| **Closed** | Nobody new. Only existing accounts sign in. |
| **Inherit** | Use the next rule up. |

The rows are:

- **New users** ([`signin.newUsers`](/admin/settings#signin.newUsers)): the base rule, **Open** by default.
- **Password sign-in: new users** (`signin.newUsers.mail`): mailbox sign-in with a password (IMAP, POP3, Exchange).
- **Single sign-on: new users** (`signin.newUsers.sso`): OpenID Connect, OAuth, SAML and trusted-header sign-in.
- **Linked mailboxes: new users** (`signin.newUsers.linked`) and its IMAP, POP3 and Exchange rows: linking a mailbox that has no account here yet. A linked mailbox is judged by its own domain.

All type rows default to **Inherit**. The first value that is not **Inherit** wins, in this order: the type for the user's domain, the type for all domains, (for linked mailboxes, the provider row before the general linked row), **New users** for the domain, **New users** for all domains, then **Open**.

To set a rule for one domain, use **Domain overrides**: choose the **Type**, enter the **Domain**, choose the **Rule** and select **Add override**. **Remove** deletes an override.

## Add pre-added users

![The Pre-added users card: the address list, note and lifetime fields, the Add pending users button, and two pending entries for jordan.ellis@example.com and lena.brooks@example.com.](/screenshots/admin-access/preprovisioned-pending.webp){.mw-crop}

1. Under **Pre-added users**, enter the addresses in **Addresses (one per line, comma or semicolon separated)**. At most 500 at a time.
2. Optionally enter a **Note** (up to 200 characters).
3. Set **Valid for (days)**. The default comes from **Pre-added user lifetime (days)** (30); the maximum from **Longest pre-added user lifetime (days)** (90).
4. Select **Add pending users**. The screen reports how many were added; an address that already exists is not added twice.

Then tell the person to sign in with their mailbox password. There is no invitation email and no token.

Each entry shows its **State**: **Pending**, **Expired** or **Used**. The first successful sign-in uses the entry up and records the time under **Used at first sign-in**; two sign-ins at the same moment cannot both use one entry. **Extend** gives a pending entry a new lifetime of **Valid for (days)** from now; **Revoke** deletes it. A used entry can be neither extended nor revoked.

Expired entries are deleted after **Keep expired pending users (days)** (30). The deletion runs when the list of pre-added users is loaded, not on a schedule.

### Domain-wide entries

With **Allow domain-wide pre-added entries** checked (off by default), an entry `*@example.org` admits any address of that domain until its use limit (1 to 100 000) is reached. Such entries need the user create permission for all domains and a use limit, which the console form does not ask for: adding `*@example.org` there is refused. Add them through the console API with `domainWideMaxUses` on `POST <console>/preprovisioned-users`.

## Check an address

![The Would a new sign-in be refused? check: an address field, the sign-in type and the Check button.](/screenshots/admin-access/preprovisioned-check.webp){.mw-crop}

Under **Would a new sign-in be refused?**, enter an **Address**, choose the **Type** and select **Check**. The answer is "Admitted under the … rule." or "Refused under the … rule." It reads the saved rules and pending users and changes nothing. The same check is available as `GET <console>/signin-policy/preview?address=…&type=mail`.

## What a refused person sees

The rule is checked after the mail server or identity provider has accepted the person and before anything is stored, so a refusal leaves no account, password, session or mail connection behind.

**Refusal messages** ([`signin.refusalDetail`](/admin/settings#signin.refusalDetail)) decides the answer:

- **Explain (403, says the address is not provisioned)** (the default): the sign-in screen says "No account exists for this address yet, and new accounts are not created automatically. Ask your administrator to add you."
- **Uniform (401, same as a wrong password)**: the screen shows the same message as for a wrong password.

Every refusal is written to the security log as `signin-new-user-refused` with the rule, the sign-in type and the domain; no password is logged.

## Options

| Label as shown | What it does | Default | Who can change it |
|---|---|---|---|
| **New users** ([`signin.newUsers`](/admin/settings#signin.newUsers)) | Base first sign-in rule | **Open** | admin; per domain |
| **Password sign-in: new users**, **Single sign-on: new users**, **Linked mailboxes: new users** and the linked IMAP, POP3 and Exchange rows (`signin.newUsers.*`) | Rule for one kind of sign-in | **Inherit** | admin; per domain |
| **Refusal messages** ([`signin.refusalDetail`](/admin/settings#signin.refusalDetail)) | Explain or hide why a sign-in was refused | Explain | admin |
| **Allow domain-wide pre-added entries** (`signin.preprovision.domainWide`) | Allows `*@domain` entries with a use limit | off | admin |
| **Pre-added user lifetime (days)** (`signin.preprovision.ttlDays`) | Default lifetime of a new entry | 30 | admin, Settings |
| **Longest pre-added user lifetime (days)** (`signin.preprovision.maxTtlDays`) | Upper limit for a lifetime or extension | 90 | admin, Settings |
| **Keep expired pending users (days)** (`signin.preprovision.retentionDays`) | How long an expired entry stays listed | 30 | admin, Settings |

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| A new colleague gets "No account exists for this address yet…" | The rule is **Preprovisioned**, **Provisioned** or **Closed** and the address has no pending entry, or the entry expired | Add or extend the entry |
| An existing user cannot sign in after you changed the rule | Not caused by this rule, which never refuses existing accounts | Check the domain's sign-in lists, the user's access status and the sign-in logs |
| Adding `*@example.org` is refused or not added | **Allow domain-wide pre-added entries** is off, or no use limit was given (the form cannot give one) | Turn the option on and use the console API with `domainWideMaxUses` |

## Related

- [Domains](./domains.md) (per-domain sign-in allowlists and blocklists)
- [User access lifecycle](./user-access-lifecycle.md)
- [Single sign-on](./sso.md)
- [Security centre](./security-center.md)
