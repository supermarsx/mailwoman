# Account Drafts

The composer saves editable drafts into the account's mapped IMAP Drafts folder. Realtime mode waits for a typing pause; periodic mode saves at the configured interval; manual mode only saves on request. Global, domain and user settings and administrator locks apply. Successful saves use a quiet composer status; failures show an error toast.

This implementation requires native IMAP, a writable and unambiguous Drafts mapping, and UIDPLUS or IMAP4rev2 with a confirmed APPENDUID. POP3, proxy JMAP and plugin backends report unavailable. It never substitutes a local cache write or a message submission for a Drafts save.

## Resume and lifecycle

Open Compose → Drafts → Account drafts on another device to resume. Opening a Mailwoman-managed draft from the Drafts message list also resolves its editable account record. Drafts created by unrelated clients remain readable messages; this implementation does not guess their editable state. Each verified personal owner has an explicit maximum of 100 active drafts, all included in the account list.

Closing a composer keeps its draft. Explicit discard reserves the exact revision, removes the owned upstream copy, then clears the sealed content and releases the active quota. Small logical-ID tombstones remain to reject delayed requests that would otherwise recreate a discarded draft. Failed cleanup retains the record and reports pending cleanup.

A save creates and confirms the replacement before removing the previous copy. Attachments are rebound to the confirmed message's owned MIME parts. Partial recipient text is preserved in sealed editor state; only valid addresses appear in the MIME projection. The editable state is sealed at rest and bound to the verified personal owner, logical draft and revision.

Sending waits for any in-flight save and stops further autosaves. A queued or uncertain submission retains the saved draft. Confirmed delivery attempts exact-revision cleanup; cleanup failure is reported as a filing error without resending. A newer revision is never deleted as cleanup for an older send.

## Privacy and failure limits

Zero-access accounts and compositions containing decrypted quotations or selected encryption protection do not upload plaintext account drafts. Server-side sealing is not end-to-end encryption. Enabling protection cannot recall a copy already accepted by the upstream server.

Each save claims a unique operation before APPEND. An interrupted or unconfirmed APPEND remains uncertain and is not automatically retried, even after restart. There is no claim of crash-safe exactly-once IMAP delivery. Review the actual Drafts folder before taking further action. Browser termination cannot guarantee a remote flush; authority-bound local recovery is separate from confirmed account saving.

Requests require an authenticated verified personal subject and the expected storage-authority header. Raw JMAP account IDs do not establish ownership. Legacy ambiguous local records are not imported automatically. Editor state is bounded to 1 MiB, with at most 100 ordinary attachments, 32 inline parts and 32 MiB of actual aggregate attachment bytes; request concurrency and execution time are bounded.
