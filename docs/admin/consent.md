---
screen: true
---
# Restrictions and consent

Two tabs of **Compliance** record what a person agreed to and what they asked you to stop. **Restrictions** pauses chosen kinds of processing for one person while their data stays stored. **Consents** shows the decisions people record per purpose under **Settings > Privacy** and holds the settings for each purpose. Both are controls you configure; whether a purpose needs consent, and on which legal basis, is for you to decide.

![The Restrictions tab of Compliance with one restricted person, the kinds of processing paused for them and the reason.](/screenshots/admin-privacy/restrictions-list.webp){.mw-shot}

## Before you start

- **Restrictions** needs the `data_export.run` capability; an administrator limited to some domains can restrict only people of those domains.
- **Consents** needs `settings.read` (and `settings.write` to change a setting) on all domains.
- A person must have signed in at least once before you can restrict them.

## Restrict a person

1. Open **Compliance** and select **Restrictions**, then **Restrict a person**.

   ![The Restrict a person form: the address, the six kinds of processing that can be paused, the reason and the length.](/screenshots/admin-privacy/restrictions-apply.webp){.mw-crop}

2. Enter the **Address**.
3. Under **Pause**, keep the kinds of processing to pause. All six are ticked when the form opens.
4. Enter a **Reason**. It is required and is shown in the list.
5. In **Ends after (days, empty or 0 = until lifted)**, enter a number of days, or leave it empty. An empty field uses **Default restriction length (days)**, which is 0 (until lifted) unless you changed it.
6. Select **Apply restriction**.

A restriction is also applied automatically when you approve a **Restriction** data request and **Restrict when a request is approved** is on (the default). That restriction pauses the kinds set as defaults in the **Settings** tab of **Restrictions** (all six by default).

To end a restriction, select **Lift** on its row and give a reason. With **Tell the person** on (the default), the person is told when a restriction starts and when it is lifted. **Include lifted and expired** shows past restrictions.

### What each paused kind stops

| Pause | Where the server stops it |
| --- | --- |
| **Search indexing and embeddings** | New messages are not published to the search index, and semantic search is refused. Mail already in the index stays there. |
| **Assistant** | The assistant and the MCP endpoint are refused, and so is semantic search. |
| **Analytics and open tracking** | Usage counts are not sent, the analytics provider script is not served, and open tracking records nothing. |
| **Rules and automatic filing** | Filter rules are not run on incoming mail. |
| **Push notifications** | No push wake-ups are sent. |
| **Connected services and plugins** | UI plugins, package tracking and connected services (integrations) are refused. |

Whatever kinds are paused, an active restriction also:

- refuses permanent deletion of messages and deletion of folders for the person, so that the data stays as it was. Mail still arrives and the person can read it and move it to the trash;
- skips the person in the retention rules of their mailbox, unless **Run user rules for restricted accounts** is on (see [Retention](retention.md#user-retention-rules)).

A restriction acts only inside this server. It does not reach the upstream mail server, your other systems or backups.

## Consents by purpose

1. Open **Compliance** and select **Consents**.

   ![The Overview of the Consents tab: for each purpose with decisions, how many accounts currently say yes or have withdrawn, and the withdrawal rate.](/screenshots/admin-privacy/consents-overview.webp){.mw-crop}

2. Read the **Overview**. **Yes now** and **Withdrawn now** count accounts by their latest decision for each purpose. A yes given for an older text version still counts as yes here. **Withdrawal rate** is the share of those accounts whose latest decision is a withdrawal.
3. **Export ledger (CSV)** downloads every decision: time, domain, purpose, decision, text version, legal basis, source and account id. The export is recorded in the audit log.
4. Edit the settings of each purpose in the **Settings** section below the charts. This tab sets values for all domains; to set a value for one domain, use the **Settings** screen of the console.

People record decisions under **Settings > Privacy** in the mail app; the server does not ask them with a prompt or a dialog. A decision is stored with the purpose, the decision, the text version, the legal basis shown, the source (`settings`, or `dsr` when an approved objection withdrew it), the time and the account id. It holds no address, IP address or browser details. The ledger is deleted when the person is erased. It is not part of the person's data export: it belongs to an export section that has no export switch and is always left out (see [What an export contains](data-requests.md#what-an-export-contains)).

### Purposes

| Purpose (console name) | Default basis | What reads the decision |
| --- | --- | --- |
| **Assistant** (and each assistant feature) | contract | Checked on every assistant call and for the MCP endpoint. |
| **Push notifications** | contract | Checked before every push wake-up. |
| **Open tracking** | contract | Checked when an open is recorded. |
| **Connected services** | contract | Checked with the feature gate for UI plugins, package tracking and integrations. |
| **Usage counts** | consent | Not read. Usage counts follow the person's **Optional usage counts** preference; the ledger only keeps a receipt when it changes. |
| **Analytics provider script** | consent | Not read. The script follows the person's own consent switch for it; the ledger only keeps a receipt. |
| **Remote images** | consent | Not read. Listed for the person only; remote images follow the remote-content settings. |
| **Load-time measurements** | legitimate interest | Not read. Listed for the person only. |

For the four purposes the server checks:

- With basis **consent**, the purpose is off until the person grants the current text version.
- With any other basis, the purpose is on unless the person withdrew it.
- Raising the **Text version** asks the person again: a yes for an older version no longer counts.
- If the ledger cannot be read, a consent-based purpose is refused and the others are allowed.
- A refused feature answers with an error; the mail app does not prompt the person to decide.

For the other four purposes, the **Legal basis** setting changes only what the person is shown. They have no **Text version** setting; their decisions are recorded with text version 1.

::: warning Purpose consent off
With **Purpose consent** off, the server allows every purpose, including those a person has withdrawn, and refuses to record new decisions.
:::

**Automated processing notice** shows people a description of the assistant and search features with a switch per assistant feature (summaries, drafting, grammar, dictation, semantic search, tag suggestions, recaps, chat). An assistant feature that is switched off there stays off for that person whatever its basis.

## Options

| Setting | What it does | Default | Scope |
| --- | --- | --- | --- |
| **Restrictions of processing** ([`restriction.enabled`](/admin/settings#restriction.enabled)) | Allows restrictions | on | domain |
| **Restrict when a request is approved** ([`restriction.autoApplyOnApproval`](/admin/settings#restriction.autoApplyOnApproval)) | Applies a restriction when a Restriction request is approved | on | domain |
| **Default restriction length (days)** ([`restriction.defaultExpiryDays`](/admin/settings#restriction.defaultExpiryDays)) | Used when the length is left empty; 0 is until lifted | 0 | domain |
| **Tell the person** ([`restriction.notifyUser`](/admin/settings#restriction.notifyUser)) | Notice when a restriction starts and ends | on | domain |
| **Pause …** (`restriction.scope.*`) | Kinds paused by a restriction from a data request. The form on the **Restrict a person** tab ignores these and starts with all six ticked | all on | domain |
| **Purpose consent** ([`consent.enabled`](/admin/settings#consent.enabled)) | Turns the consent ledger on; off allows every purpose | on | domain |
| **Consent receipts kept (days)** ([`consent.receipts.retentionDays`](/admin/settings#consent.receipts.retentionDays)) | Older decisions are deleted, at most once an hour | 1825 | server |
| **Automated processing notice** ([`consent.art22.notice`](/admin/settings#consent.art22.notice)) | Shows the notice and the per-feature switches | on | domain |
| **Legal basis: …** (`consent.<purpose>.basis`) | consent, legitimate interest, contract or legal obligation | see table above | domain |
| **Text version: …** (`consent.<purpose>.version`) | Raising it asks again. Only the four checked purposes have this setting | 1 | domain |
| **Own wording: …** (`consent.<purpose>.text`) | Your text per language, shown instead of the built-in description | empty | domain |

## Related

- [Data requests](data-requests.md)
- [Cookie consent](cookies.md)
- [Usage analytics](analytics.md)
- [Retention](retention.md)
- [Privacy settings in the mail app](/user/settings/account#privacy)
