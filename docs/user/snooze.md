---
title: Snooze messages
screen: true
---

# Snooze messages

Snoozing a message sets a time for it to come back to your attention. You pick
the time from a short menu of presets, or from a date and time of your own. You can
change which presets the menu offers in Settings.

![The mailbox with the message context menu open on Re: Spring catalogue: print proof and its Snooze submenu showing Choose date and time, Later today, Tomorrow and Next week.](/screenshots/user-features/snooze-orientation.webp){.mw-shot}

## Snooze a message

1. Right-click a message in the list, or open it and use its actions. Choose
   **Snooze**. The submenu lists your presets and **Choose date and time…**.

   ![The Snooze entry of the message menu with its submenu: Choose date and time, Later today, Tomorrow and Next week, each with a marker.](/screenshots/user-features/snooze-menu.webp){.mw-crop}

   1. **Snooze** opens the submenu.
   2. **Choose date and time…** opens a date and time field.
   3. **Later today** is a preset.
   4. **Tomorrow** is a preset.
   5. **Next week** is a preset.
   {.mw-callouts}

2. Choose a preset. The message is snoozed until the time the preset gives, and a
   **Snoozed** notice with an **Undo** button appears for about ten seconds.

3. To pick any other time, choose **Choose date and time…**. The dialog shows the
   time zone it uses (for example **Timezone: UTC**), taken from your Regional
   settings. Enter a time in the future and choose **Snooze**.

   ![The custom snooze dialog with a date and time field, the time zone and Snooze and Cancel buttons.](/screenshots/user-features/snooze-custom.webp){.mw-crop}

A time that is not in the future, or a clock time that does not exist because clocks
move forward that day, is refused with the message "Choose a valid date and time in
the future. A skipped daylight-saving time cannot be used." A clock time that happens
twice when clocks move back asks you to choose which occurrence you mean.

While a message is snoozed, its header shows a **Snooze:** badge with the date and
time, and the message menu offers **Unsnooze**. Choose **Unsnooze** to clear the
snooze before the time arrives. When the time arrives the snooze ends.

::: tip
Whether the **Snooze:** badge is always shown, shown only when notable, or never
shown is set under Message badges in Message display settings.
:::

## Change the snooze menu

Open **Settings**, then **Message display**, then the **Snooze** tab. The tab is
titled **Snooze options**.

![The Snooze tab of Message display settings: the list of snooze presets, each with a checkbox, the next time it would snooze to, and buttons to move or delete it.](/screenshots/user-features/snooze-settings.webp){.mw-shot}

- Tick a preset to show it in the menu. Only **Later today**, **Tomorrow** and
  **Next week** are ticked at first; the others are listed so you can turn them on.
- **Next snooze time** shows the time each preset would give right now.
- Use the arrows to move a preset up or down, and the bin button to delete it.
- **Add preset** creates a preset. A preset is one of these rules: hours from now,
  calendar days from now, tomorrow, next week, end of the week, end of the month,
  first day of next month, next weekday, an ordinal weekday next month, a day of
  the month, or next year. Calendar rules have a time of day.
- **Restore built-in presets** puts the original list back in the editor. Choose
  **Save** to apply it.
- Up to 32 presets are allowed, with names of up to 80 characters.
  **Choose date and time…** is always in the menu, even if you remove every preset.

Hours are elapsed time. Calendar rules use your Regional settings time zone and
week start. If an end-of-period time has already passed, the following period is
used. A calendar preset whose clock time does not exist on that day is left out of
the menu.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Snooze** presets ([`mail.snooze.presets`](/admin/settings#mail.snooze.presets)) | The presets in the snooze menu | Fifteen presets; three shown | You, unless an administrator locks it |
| Snooze capability ([`cap.snooze`](/admin/settings#cap.snooze)) | Whether snoozing a message is allowed | On | Administrator |

## Troubleshooting

- **Snoozing fails.** The administrator can switch off the snooze capability, and the
  server then refuses the change.
- **A preset is missing from the submenu.** It is not ticked in the Snooze tab, or
  its clock time does not exist on that day.

## Related

- [Settings: message display](/user/settings/mail)
- [Keyboard shortcuts](/user/keyboard-shortcuts)
- [Reading messages](/user/mailbox/reading)
