# Composition formats and local drafts

Plain-text composition creates a `text/plain` MIME body. Literal angle brackets, ampersands and line breaks remain text; switching to rich text creates HTML through the editor schema. Plain-text and armored cryptographic messages do not receive an open-tracking image. Cc and Bcc recipients enter the submission envelope, while the transmitted message omits Bcc headers.

Local unsent drafts are scoped to the authenticated account. They retain the selected identity, body format, inserted-signature identifiers, recipients, threading and uploaded attachment references. Resuming and sending removes that account's original draft. Account changes invalidate pending autosaves, uploads, previews and pre-dispatch tracking work. Already accepted upstream submissions cannot be recalled by changing the current account.

Legacy draft rows without an account identifier are not assigned to whichever user next signs in. Local drafts are plaintext browser storage, cleared on logout; they are not a cross-device server draft synchronization feature. Quoted decrypted content is never autosaved. Stored upload references may expire; resuming does not grant access to another account's blob.

A named signature selected for a native identity resolves the account-owned template at Identity/get time, including edits and removal. Existing templates are literal text: rich composition escapes them and plain composition uses the literal text. Existing provider HTML/text signatures and Sent-folder metadata survive local profile edits. Explicit HTML/plain template authoring is a subsequent schema/editor feature and is not established by this checkpoint.
