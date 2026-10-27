# Composer ribbon

Send is at the top left. Save to Drafts sits immediately before Drafts at the top right. Discard account draft asks for confirmation; a persisted draft is removed using the revision held by this editor and verified account authority. A newer remote revision refuses removal and keeps the editor open. Cancelling the confirmation keeps the draft. Closing a workspace tab continues to use the draft-save and dirty-state guards. The standalone dialog retains its close control.

Format text, Insert, Table, Security and Delivery share the top ribbon. Named groups contain at most three rows and scroll horizontally on narrow screens. Controls use the native theme-colored icon set and retain accessible names. Undo and Redo each have their own history disclosure, capped at twenty grouped operations. Escape closes a history disclosure. Paragraph marks are a pressed-state toggle in the Paragraph group; markers and display zoom never enter the message HTML.

HTML source displays the generated rich-message markup. Source edits pass through the existing editor schema and render when returning to rich text. Plain text is a separate format. Switching to plain text intentionally uses the text projection; switching back constructs HTML from that text, rather than restoring discarded formatting.

Rich-editor controls are mounted into the composer ribbon without moving or recreating the editing document. Standalone signature editors retain local controls. Detached composers retain the same document, selection and history through reattachment. No send or upstream mailbox permission changes are introduced by this layout.
