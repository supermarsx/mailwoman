# Selected-message actions

Select messages in the list with Ctrl/Command-click, Shift-click, the Select messages button, or the keyboard: Space toggles the focused row, Shift+Up/Down extends the selection, Ctrl/Command+A selects every loaded message, and Esc clears the selection.

While one or more messages are selected, a floating selection bar sits over the bottom centre of the list pane. It does not push the rows down or reflow them, and it has no backdrop or focus trap. Clicking, Ctrl/Command-click, Shift-click, the arrow keys, Space and scrolling all keep working on the list underneath. The list gains bottom padding equal to the bar's height, so the last row can always be scrolled clear of the bar. The bar disappears when the selection is empty.

The bar shows the count (announced politely to screen readers), Select all / Select none, and the actions as icon buttons with tooltips: archive, delete, move to folder, mark read or unread, flag or remove flag, spam, labels, forwarding and download. Actions that do not fit on one line go into the bar's More menu, so the bar never wraps, including at phone width. × clears the selection. Select all selects the messages loaded in the list; there is no "select every message in the folder" action yet. Folder tabs use the same bar with the actions their own list supports: read, unread, archive, delete (in Trash, a permanent delete that asks for confirmation first) and move.

The bar does not take focus when it appears. It is one tab stop after the list. F6 moves focus from the list to the bar, the arrow keys, Home and End move between its buttons, and Esc or F6 returns focus to the list. Without reduced motion, the bar fades in from just below its final position.

Label changes use one JMAP update for the selection. A refused message is restored locally and reported; undo preserves labels that were already present. Labels are JMAP keywords, distinct from mailbox membership and system flags.

Forwarding opens a reviewable compose tab with the originals attached as `message/rfc822` files. It does not automatically address or send the draft, interpolate original HTML, or decrypt encrypted messages. At most 100 messages can be selected for this action. The engine checks the actual aggregate non-inline attachment bytes against a 32 MiB composition limit, independently of client size metadata. Originals are marked forwarded after submission acceptance; undo-send restores their previous forwarded marks.

Download prepares one mbox archive containing up to 100 messages and 32 MiB. Each message passes the existing server export authorization, fresh-proof and attachment-policy checks. A refusal, account switch, or size overrun prevents a partial archive from being downloaded. This export endpoint requires engine mode; a proxied JMAP account currently receives an unsupported response. Account label definitions synchronize separately from message keywords.
