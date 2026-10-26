# Mail import and export

Settings → Import and export mail opens a dedicated import editor. Choose an existing folder in the signed-in account and upload an EML, mbox, or uncompressed Maildir tar archive. Imports are limited to 64 MiB per request and 20,000 messages. Two imports may run concurrently per server instance. Administrator feature restrictions apply to both REST import and JMAP Email/import.

Import requires an active writable IMAP backend advertising UIDPLUS or IMAP4rev2 and a configured upload backend (`MW_UPLOAD_DIR`). POP3 and unconnected accounts are unavailable. Every successful message is acknowledged by upstream APPEND with its durable UID coordinates before it is reported as imported. Historical imports preserve supplied valid keywords and dates and do not execute arrival rules, vacation replies, or forwarding. A failed connection can leave an uncertain outcome: refresh the destination before retrying. Repeating an archive can create duplicates.

The message reader's Export action opens a dedicated editor for EML, mbox, text, Markdown, MSG, and DOCX. EML preserves original bytes; converted formats have their own representations. Export uses the existing authenticated server export route, administrator attachment/export restrictions, and security step-up. Export availability in proxy mode depends on the existing server export implementation; the editor reports unavailable responses explicitly.

Legacy cached IMAP date strings are normalized to UTC before mailbox queries and synchronization so old textual dates cannot sort above newly received messages. Updates are account-scoped, bounded in memory, and conditional on the original cached value.

These import changes do not establish durable upstream behavior for the separate Email/copy implementation. That operation remains a tracked follow-up; the transfer editor does not use it.
