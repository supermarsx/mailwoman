---
title: Package tracking
screen: true
---

# Package tracking

**Package tracking** keeps a list of parcels you are expecting and shows their delivery
status. It is off until your administrator sets up a tracking provider and you opt in.
Mailwoman does not scan your mailbox for tracking numbers and does not check packages in
the background: you add a package by hand, and Mailwoman asks the provider only when you
ask it to.

Open **Settings**, then **Package tracking**.

![The Package tracking page in Settings, with the settings list on the left and the Package tracking card on the right.](/screenshots/user-features/packages-screen.webp){.mw-shot}

The card with the privacy note, the opt-in checkbox, **Add package** and the package table:

![The Package tracking settings with the privacy note, the message that tracking is not set up by the administrator, the opt-in checkbox, Add package and the empty package table.](/screenshots/user-features/packages-settings.webp){.mw-crop}

## Before you start

- Your administrator must configure the tracking provider (AfterShip Tracking). Until
  then the page says "Package tracking is disabled or has not been configured by your
  administrator." and you cannot opt in.
- You must opt in for your own account. Tick **Enable package tracking for my account**.

## What is sent

Only pressing **Track or refresh** on a package sends anything. It sends the tracking
number and the carrier to the administrator's AfterShip provider. Message contents, your
email addresses and the package label are never sent. Saving a package only stores it in
your Mailwoman account.

## Add a package

1. Choose **Add package**.
2. Enter a **Tracking number**, using letters, digits or hyphens.
3. Enter the **Carrier code**, the exact AfterShip code, such as `ups`, `fedex` or
   `usps`. Some carriers need extra information and cannot be tracked this way.
4. Optionally enter a **Label**, a private name such as a shop or order. It stays in
   Mailwoman.
5. Choose **Save package**. The page says "The package was saved in your account. No
   tracking request was sent."

**Back to packages** leaves without saving; if you changed anything it asks "Discard the
unsaved package changes?"

## Check a package

The package table has the columns **Label**, **Tracking number**, **Status**,
**Estimated delivery** and **Actions**. For each package the actions are edit, **Track or
refresh**, and **Remove**.

Choose **Track or refresh** to ask the provider. The status then shows one of **Not yet
tracked**, **Shipment information received**, **In transit**, **Out for delivery**,
**Delivery attempt failed**, **Delivered**, **Ready for pickup**, **Delivery exception**
or **Tracking expired**. Opening a package shows **Latest tracking events** with the
carrier time, the location, and the event. Provider estimates are not delivery
guarantees, and carrier event times are shown as the carrier supplied them.

A package can be refreshed at most once per refresh interval that your administrator
sets. The page states the interval under the table.

**Remove** deletes the package from your Mailwoman account after you confirm. It does not
erase the records the tracking provider already holds.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Enable package tracking for my account** | Lets you save and track packages | Off | You, once the administrator has configured a provider |
| Refresh interval | Shortest time between two refreshes of one package | Set by the administrator (60 to 86400 seconds) | Administrator |

Your account can save up to 100 packages.

## Troubleshooting

- **"This tracking number and carrier are already saved."** Each tracking number and
  carrier pair can be saved once.
- **"Check the tracking number, carrier code and label."** One of the fields is not
  valid.
- **"This package was checked recently. Wait for the configured refresh interval."** Try
  again later.
- **"The tracking provider could not complete the request. Check the carrier and number,
  then wait before retrying."** The carrier code or number may be wrong, or the provider
  is unavailable. Your administrator can check the provider configuration.

## Related

- [Settings: integrations](/user/settings/integrations)
- [Package tracking (administrators)](/admin/package-tracking)
