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

Signatures and saved searches use separate `personal_signature` and
`personal_saved_search` tables from migration 0062. Their `po1_` owner derives
from the same verified realm tuple, independently of the server encryption key;
the browser still supplies its session-bound `msa1_` authority on writes.
The native engine derives its owner only from an actual native account. Proxy
sessions cannot enter that namespace merely by returning a matching account ID.
Named signatures and virtual search folders read the new tables. Legacy
`signatures` and `saved_searches` rows remain intact and are never implicitly
attributed, even if a raw legacy ID happens to equal a new owner string.

Database migration copies both new tables and their explicit HTML/plain formats.
The existing migration tool copies sealed credentials without resealing: use the
same server key as required by that tool. A key-independent personal owner is
not a claim that arbitrary encryption-key rotation is supported automatically.

Table-backed identities and notification rules, and the separate feature
assignment/settings subject model still require their own ownership audit.
