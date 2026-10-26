# Selected-message actions

Select messages in the list with Ctrl/Command-click, Shift-click, or the keyboard selection actions. The selection toolbar offers archive, delete, move, spam, read/unread, flag, add/remove labels, forwarding and download.

Label changes use one JMAP update for the selection. A refused message is restored locally and reported; undo preserves labels that were already present. Labels are JMAP keywords, distinct from mailbox membership and system flags.

Forwarding opens a reviewable compose tab with the originals attached as `message/rfc822` files. It does not automatically address or send the draft, interpolate original HTML, or decrypt encrypted messages. At most 100 messages can be selected for this action. The engine checks the actual aggregate non-inline attachment bytes against a 32 MiB composition limit, independently of client size metadata. Originals are marked forwarded after submission acceptance; undo-send restores their previous forwarded marks.

Download prepares one mbox archive containing up to 100 messages and 32 MiB. Each message passes the existing server export authorization, fresh-proof and attachment-policy checks. A refusal, account switch, or size overrun prevents a partial archive from being downloaded. This export endpoint requires engine mode; a proxied JMAP account currently receives an unsupported response. Account label definitions synchronize separately from message keywords.
