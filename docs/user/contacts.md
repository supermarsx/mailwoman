---
title: Contacts
screen: true
---

# Contacts

**Contacts** is your address book. It stores names, email addresses and phone numbers,
supports groups and favorites, imports and exports vCard and CSV files, finds
duplicates, and feeds address suggestions in the composer. Open it from **Contacts** in
the sidebar.

![The Contacts screen: address books and groups on the left, the contact list in the middle, and the card of Priya Raman on the right.](/screenshots/user-features/contacts-screen.webp){.mw-shot}

## Find your way around

The left pane lists the address books and the groups.

![The address book pane with markers on All contacts, Favorites, the group membership button, the new group button and Import.](/screenshots/user-features/contacts-books.webp){.mw-crop}

1. **All contacts** lists every contact. The number is the count.
2. **Favorites** lists the contacts you starred.
3. The group membership button opens the **Group membership** page.
4. The plus button creates a new group.
5. **Import…** opens the import screen.
{.mw-callouts}

Below these, each address book and each group is listed as its own entry. Choose one
to show only its contacts.

The middle pane is the contact list.

![The top of the contact list with markers on the search field, New contact, Find duplicates, a contact's selection checkbox and its favorite star.](/screenshots/user-features/contacts-list.webp){.mw-crop}

1. **Search contacts** filters the list as you type.
2. **New contact** opens an empty contact form.
3. **Find duplicates** looks for contacts that share an email address or a full name.
4. The checkbox selects a contact.
5. The star adds the contact to **Favorites**, or removes it.
{.mw-callouts}

At the foot of the list, **Export vCard** and **Export CSV** download the contacts
currently listed as `contacts.vcf` or `contacts.csv`.

## Read a contact card

Click a contact to see its card on the right.

![The card of Priya Raman with markers on the Favorited, Share contact, Edit and Delete buttons.](/screenshots/user-features/contacts-card.webp){.mw-crop}

1. **Favorited** (or **Favorite**) toggles the star.
2. **Share contact** opens a menu.
3. **Edit** opens the contact form.
4. **Delete** deletes the contact.
{.mw-callouts}

The card shows the email addresses, phone numbers, notes and group memberships the
contact has. Clicking an email address starts a new message to that person.

**Share contact** offers **Send contact by email**, **Copy as vCard** and
**Download .vcf**.

![The Share contact menu on a contact card: Send contact by email, Copy as vCard and Download .vcf.](/screenshots/user-features/contacts-share.webp){.mw-crop}

## Add or edit a contact

Choose **New contact**, or **Edit** on a card.

![The Edit contact form for Priya Raman, with Name and work, Email and phone, and Notes and preferences sections; markers on Full name, Organization, Add email and Add phone.](/screenshots/user-features/contacts-editor.webp){.mw-crop}

1. **Full name**. The form also has **Given** and **Surname**.
2. **Organization**, with a **Title** field below it.
3. **Add email** adds another address row, each with a label such as "work".
4. **Add phone** adds another number.
{.mw-callouts}

Further down, **Notes** holds free text, **Favorite** stars the contact, and **Security
key** stores text for display only. Mailwoman does not use that text to encrypt or
sign anything; to use encryption keys see [Encryption keys](/user/encryption-keys).
If saving fails, the form keeps your changes so you can try again.

## Groups

1. Choose the plus button next to **Groups**.
2. Enter a **New group name**, pick the **Address books** entry it belongs to, and
   choose **Create**.

   ![The New group form with the name Print shop entered, the address book choice and the Create button.](/screenshots/user-features/contacts-group.webp){.mw-crop}

   1. The group name.
   2. The address book the group belongs to.
   3. **Create** makes the group.
   {.mw-callouts}

3. Add members on the **Group membership** page: pick a group, type a name or address
   in "Type a name or address to add a member", and remove members from the same
   list. Changes there are saved as you make them.

## Import contacts

1. Choose **Import…** in the left pane.
2. Choose a file with **Choose File**, or paste text into **Paste vCard or CSV**. The
   format is detected automatically.
3. Choose **Preview**. The **Review** pane lists the contacts found, each with a
   checkbox, so you can leave any of them out. For CSV files a **Map columns** table
   lets you say which column holds which field.
4. Choose the **Address book** the contacts go into, then choose the **Import N
   contacts** button.

![The Import contacts screen with a pasted vCard for Hana Weber, the Preview button, and the Review pane listing the contact that was found.](/screenshots/user-features/contacts-import.webp){.mw-crop}

1. **Import file**.
2. **Paste vCard or CSV**.
3. **Preview**.
{.mw-callouts}

## Merge duplicates

Choose **Find duplicates**. Contacts that share an email address or the same full name
are grouped into sets. Choose a set, pick the **Card to keep**, check the **Merged
preview**, and choose **Merge contacts**. The merged card holds the email addresses and
phone numbers of all the cards in the set. If nothing is found the screen says "No
duplicates found."

![The Merge duplicates screen listing the possible duplicates found among the contacts.](/screenshots/user-features/contacts-dups.webp){.mw-crop}

## Save contacts from a message

Right-click a message, or use the reader's **More actions** menu, to save the people in
it.

![The message context menu with the Save contacts from this message submenu open: Save new contacts, Save sender as contact, Save all recipients and Choose contacts.](/screenshots/user-features/contacts-save-menu.webp){.mw-crop}

- **Save sender as contact** saves the sender.
- **Save contacts from this message** has **Save new contacts**, **Save all
  recipients** and **Choose contacts…**.

**Choose contacts…** opens a dialog with one row per address on the message. A row
shows where the address was found (**From**, **To**, **Cc**…), a **Name** you can edit,
and the **Address book**. Your own addresses are not ticked, and automated senders such
as no-reply addresses are hidden unless you ask to show them. Choose **Save N
contacts** to save the ticked rows.

![The Save contacts from this message dialog with the sender of a newsletter listed, a Name, Email, Address book and Group choice, and a Save 1 contact button.](/screenshots/user-features/contacts-save-dialog.webp){.mw-crop}

1. The checkbox for a row. A ticked row is saved.
2. The **Name** the contact gets.
3. The **Address book** it goes into.
4. **Save 1 contact** saves the ticked rows.
{.mw-callouts}

After a save a notice says "Saved" with the name and address book, with **Undo** and
**Open** buttons.

## Options

The save-from-message behaviour is set under **Save contacts from messages** in
Message display settings.

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Offer Save contacts from this message** ([`contacts.saveFromMessage.enabled`](/admin/settings#contacts.savefrommessage.enabled)) | Adds the item to message menus and the reader More menu | On | You, unless an administrator locks it |
| **Offer Save sender as contact** ([`contacts.saveSender.enabled`](/admin/settings#contacts.savesender.enabled)) | Adds a one-click item to message menus | On | You, unless locked |
| **Ticked first in the dialog** ([`contacts.saveFromMessage.defaultScope`](/admin/settings#contacts.savefrommessage.defaultscope)) | Sender only, sender and Reply-To, or everyone on the message | Sender only | You, unless locked |
| **Skip automated senders** ([`contacts.saveFromMessage.skipAutomated`](/admin/settings#contacts.savefrommessage.skipautomated)) | Leaves out no-reply, notification and bounce addresses unless you tick them | On | You, unless locked |
| **Add a note with the date** ([`contacts.saveFromMessage.addNote`](/admin/settings#contacts.savefrommessage.addnote)) | Adds "Saved from a message on" and the date to the contact | On | You, unless locked |
| **Use Reply-To for list and notification senders** ([`contacts.saveSender.useReplyTo`](/admin/settings#contacts.savesender.usereplyto)) | Saves the Reply-To address instead of From for those senders | Off | You, unless locked |
| **Address book** ([`contacts.saveFromMessage.defaultBook`](/admin/settings#contacts.savefrommessage.defaultbook)) | The address book new contacts go into, or the default | **Default address book** | You, unless locked |

## Troubleshooting

- **Save contacts says "Nobody to save".** Everyone on the message is already in your
  contacts, is you, or looks automated.
- **A contact cannot be saved.** The notice says "The contact could not be saved. Try
  again." Check that an address book accepts new contacts; if none does, the notice
  says "No address book can take new contacts."

## Related

- [Calendar](/user/calendar)
- [Encryption keys](/user/encryption-keys)
- [Composing messages](/user/mailbox/composing)
- [Settings: message display](/user/settings/mail)
