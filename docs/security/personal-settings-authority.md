# Personal settings and verified realms

UI preferences, mailbox appearance, and mailbox CSS/script documents use
`personal-v2` keys bound to the verified `storageAuthority`. The upstream
`accountId` remains unchanged. A settings mutation must send
`X-Mailwoman-Storage-Authority`; a missing header returns 428 and a different
authority returns 409. Existing CAS and appearance policy locks still apply.
Admin appearance documents retain their separate admin ownership model.

The old `ui-preferences:<account>`, `appearance:<account>`, and
`appearance-customization:mailbox:<account>` rows are preserved but are no longer
automatically loaded. A raw account ID cannot prove which upstream realm wrote
them, even when a native account now has that ID. Existing browser tabs should
reload before saving settings after this upgrade.

Users can recreate preferences through the normal settings pages. For recovery,
an administrator can export the legacy rows from a database backup, establish
the original owner's realm from independent evidence, and have that verified
user reapply the reviewed values through the authenticated settings APIs. There
is no automatic raw-ID recovery or blanket reassignment command. Preserve the
original rows until ownership has been established.

Analytics consent starts unset in each new namespace. Event admission and the
queued export recheck the same verified authority and its saved consent; another
realm with the same raw account ID cannot grant consent for that export.

This change covers these personal KV documents. Table-backed identities,
signatures, saved searches and notification rules, and the separate feature
assignment/settings subject model require their own ownership audit. It does
not claim that those older raw-ID consumers have been migrated.
