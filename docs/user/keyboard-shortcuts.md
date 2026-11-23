---
title: Keyboard shortcuts
screen: true
---

# Keyboard shortcuts

Mailwoman has keyboard shortcuts for the common mail actions. You choose one of four
sets, called presets, and you can change the key for any single action or turn
shortcuts off. Shortcuts apply on the device and browser where you set them.

![The Keyboard and display settings page with the Shortcut preset card at the top and the Offline cache card below it.](/screenshots/user-features/shortcuts-screen.webp){.mw-shot}

The **Shortcut preset** card:

![The Shortcut preset card with the presets Mailwoman, Gmail, Outlook, Vim and Off, and the table of preset bindings opened.](/screenshots/user-features/shortcuts-presets.webp){.mw-crop}

1. **Mailwoman** is the preset in use until you choose another.
2. **Off (no shortcuts)** turns every shortcut off, including your own.
3. **View preset bindings** opens the table of what each key does in the preset you
   have selected.
{.mw-callouts}

## Choose a preset

Open **Settings**, then **Keyboard & display**. In **Shortcut preset**, choose
**Mailwoman**, **Gmail**, **Outlook**, **Vim** or **Off (no shortcuts)**. The change
applies at once on this device.

## The keys in each preset

| Action | Mailwoman | Gmail | Outlook | Vim |
|---|---|---|---|---|
| Compose | `c` | `c` | `Ctrl+N` | `o` |
| Archive | `e` | `e` | `Backspace` | `x` |
| Reply | `r` | `r` | `Ctrl+R` | `r` |
| Next message | `j` | `j` | `↓` | `j` |
| Previous message | `k` | `k` | `↑` | `k` |
| Search | `/` | `/` | `Ctrl+E` | `/` |
| Move to trash | `Delete` | `Delete` | `Delete` | `Delete` |
| Zoom message in | `Ctrl+=` or `Ctrl++` | same | same | same |
| Zoom message out | `Ctrl+-` | same | same | same |
| Reset message zoom | `Ctrl+0` | same | same | same |
| Rename folder | `F2` | `F2` | `F2` | `F2` |
| Open message in its own tab | `Shift+Enter` | same | same | same |

On macOS, `Ctrl` means `Cmd`. These actions have no key in any preset, but you can give
them one: **Reply all**, **Forward**, **Refresh messages**, **Open settings**,
**Mark read**, **Mark unread** and **Mark as spam**.

How the keys work:

- **Compose** opens a new message. **Search** puts the cursor in the search box.
- **Next message** and **Previous message** move through the message list. **Open
  message in its own tab** opens the message that has focus.
- **Reply**, **Reply all** and **Forward** act on the open message.
- **Move to trash** acts on the selected messages, or on the focused one. Holding the
  key down does not repeat it.
- **Rename folder** works when focus is on a folder in the sidebar.
- **Zoom** changes the size of the message text only, never the rest of the app.

Shortcuts do nothing while you are typing in a field, in the message editor, or in an
open dialog, so letters you type are never taken as commands.

Other keys work only in one place:

- In the message list, `↓` and `↑` move between messages; with Shift they extend the
  selection when selecting with modifier keys is turned on.
- `Ctrl+Z` and `Ctrl+Shift+Z` undo and redo in the mailbox when **Enable undo and redo history** is on in Settings, outside text fields.
- In the message editor, `Ctrl+K` starts inserting a link, `Alt+H` folds or unfolds the
  message header, and `Ctrl+F1` pins or auto-hides the ribbon.

## Change one key

Under **Keyboard shortcuts** in the same settings page, the table lists each action
with its **Binding**. **inherit** means the key from your preset.

![The Keyboard shortcuts card with the switch Enable application shortcuts, a search field, and the table of actions with their bindings and an edit button on each row.](/screenshots/user-features/shortcuts-custom.webp){.mw-crop}

1. **Enable application shortcuts** turns all shortcuts on or off.
2. **Search shortcuts** filters the table.
3. The edit button on a row, **Edit shortcut**, changes that action's binding.
{.mw-callouts}

1. Choose the edit button on the action's row.
2. Enter **inherit** to use your preset's key, **disabled** for no shortcut, or a key
   such as `Ctrl+Shift+K`. A key can have the modifiers Ctrl, Alt and Shift.
3. Choose **Save shortcut**, or **Back to shortcuts** to leave without saving.

If two actions end up with the same key, neither runs and the page says "Two actions
use the same shortcut. Change one binding; conflicting shortcuts will not run." Some key
combinations are reserved by your browser and may not be available.

If your preset is **Off**, your own bindings do not run either; choose another preset
to use them.

The same page has **Show keyboard shortcut toasts**, off at first. When it is on, a
short notice shows the action and keys each time a shortcut is used. It confirms that the
key was recognised, not that the action finished.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Shortcut preset** | Which set of keys is used | **Mailwoman** | You |
| **Enable application shortcuts** ([`shortcuts.enabled`](/admin/settings#shortcuts.enabled)) | Turns all shortcuts on or off | On | You, unless an administrator locks it |
| Per-action binding (`shortcuts.<action>`, for example [`shortcuts.compose`](/admin/settings#shortcuts.compose)) | The key for one action | **inherit** | You, unless locked |

## Related

- [Reading messages](/user/mailbox/reading)
- [Settings: interface](/user/settings/interface)
