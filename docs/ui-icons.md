# Action icons

Every icon is hand-written path data in `apps/web/src/components/Icon.tsx`: 16 px grid, 2 px stroke, `currentColor`, no external assets. Which icon an action wears is decided in one place, `apps/web/src/actionIcons.ts` (`ACTION_ICONS`); the menus import it. `ACTION_MENUS` lists what each menu shows at once, and `apps/web/src/icons.actions.test.ts` fails when two of those share an icon, when an icon id is missing, or when a glyph leaves the set's grid, stroke or colour rules.

Directional glyphs (arrows) are mirrored under `[dir=rtl]` through the `DIRECTIONAL` set.

## Before: actions that shared an icon

| Menu | Actions sharing one icon | Icon |
| --- | --- | --- |
| Folder context | Clean up, Empty folder, Delete folder (More) | `delete` |
| Folder context | Open, Open in tab, Arrange | `folder` |
| Folder context | Mark read (selected), Mark read, This folder, With subfolders | `mailRead` |
| Folder context | Copy all messages (More) vs message Copy | `copy` |
| Trash toolbar | Empty trash | `delete` |
| Message context | Reply, Go to reply, Mark answered | `reply` |
| Message context | Forward, Go to forward, Mark forwarded | `forward` |
| Message context | Download attachments, Download selected, Export message | `download` |
| Message context | Copy, Copy to folder, Copy sender, Copy subject | `copy` |
| Message context | Search sender, subject, folder | `search` |
| Message context | Flag, Follow-up | `flag` |
| Message context | Snooze, each preset, Unsnooze | `clock` |
| Message row cluster | Snooze (and its presets: text only) | `clock` |
| Reader toolbar | Export | `download` (same as attachments) |
| Bulk bars | Move to folder | `folder` (same as the folder list rows) |
| Outbox menu | Open, Send now, Send again | `mail` |
| Outbox menu | Recall to drafts, Edit message | `compose` |

## After: icon per action

| Action | Icon id | Where |
| --- | --- | --- |
| Empty folder / Empty trash | `emptyFolder` (open tray, contents leaving) | folder menu, Trash toolbar |
| Clean up folder | `cleanup` (sparkles) | folder menu |
| Delete / Delete folder | `delete` (trash can) | all menus |
| Delete permanently | `deletePermanently` (can with cross) | reserved |
| Clean up this sender | `sweep` (broom) | reader toolbar |
| Archive | `archiveDown` (box with arrow) | message, row, reader, bulk, folder bulk |
| Retention (folder) | `archive` (plain box) | folder menu |
| Move to | `moveTo` (folder with arrow) | reader, bulk, folder bulk, Move all |
| Copy to | `copyTo` (folder, arrow and plus) | message Copy menu, Copy all |
| Duplicate | `duplicate` (sheets with plus) | reserved |
| Copy | `copy` | message Copy parent |
| Copy sender address | `copyAddress` (sheets with person) | message Copy menu |
| Copy subject | `copySubject` (sheets with lines) | message Copy menu |
| Copy link / id / text / path / headers | `copyLink`, `copyId`, `copyText`, `copyPath`, `copyHeaders` | reserved |
| Open in tab | `openTab` (box, arrow up-right) | folder menu |
| Open in window / side panel | `openWindow`, `openSide` | reserved |
| View source | `fileCode` | reserved |
| Download | `download` | bulk bar, linked files |
| Download as ZIP | `downloadZip` (zipper) | attachments, bulk attachments |
| Export message | `exportFile` (sheet, arrow out) | message menu, reader toolbar |
| Forward as attachment | `forwardAttachment` | bulk Forward |
| Redirect | `redirect` | reserved |
| Mark read / unread | `mailRead` / `markUnread` (envelope with dot) | message, reader, bulk |
| Mark all read | `markAllRead` (double check) | folder menu |
| Pin / Unpin | `pin` / `pinOff` | message, row |
| Star / Unstar (folder) | `star` / `starOutline` | folder menu |
| Flag / Unflag | `flag` / `flagOff` | message, bulk |
| Follow-up | `followUp` (flag with clock) | message menu, row cluster |
| Go to reply / Go to forward | `goReply`, `goForward` | message menu |
| Mark answered / forwarded | `markAnswered`, `markForwarded` | message menu |
| Mute, Block sender, Report spam, Not spam, Report phishing, Create filter | `mute`, `block`, `shieldAlert`, `notSpam`, `phishing`, `createFilter` | reserved |
| Properties | `info` | reserved |
| Expand / collapse all, Select all, Clear selection | `expandAll`, `collapseAll`, `selectAll`, `clearSelection` | reserved |
| Search sender / domain / subject / folder | `person`, `domains`, `mail`, `folder` | message menu |
| Outbox: Send now, Send again, Recall, Edit | `sent`, `refresh`, `undo`, `edit` | outbox menu |
| Snooze | `snooze` (clock with Z) | message menu, row cluster |
| Later today | `snoozeLater` (sun) | snooze presets |
| This evening | `snoozeEvening` (sun setting) | reserved |
| Tomorrow | `snoozeTomorrow` (sunrise) | snooze presets |
| This weekend | `snoozeWeekend` | reserved |
| Next week | `snoozeNextWeek` (calendar, arrow) | snooze presets |
| In a month | `snoozeMonth` (calendar, dots) | reserved |
| Pick date and time | `snoozePick` (calendar, clock) | reserved |
| Unsnooze | `unsnooze` (clock, slash) | message menu, row cluster |
| Snoozed folder | `snoozedFolder` (folder with Z) | reserved |
| Remind if no reply | `remindNoReply` | reserved |

"Reserved" icons are drawn and registered (and tested to be distinct) but no menu shows that action yet.

## Not audited into ACTION_ICONS

Compose ribbon, formatting toolbars, settings section glyphs, admin sidebar, mailing-list actions (already one icon each) and status chips keep their own icons; none of them shows two different actions with one icon in the same group.
