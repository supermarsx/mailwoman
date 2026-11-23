---
title: Encryption keys
screen: true
---

# Encryption keys

**Keys** (the screen title is **Keys & certificates**) holds the keys you use for
end-to-end encryption and the keys you have found for other people. In the web app,
OpenPGP keys are used to encrypt and sign messages you write and to decrypt messages you
receive; the cryptography runs in your browser. Open the screen from **Keys** in the
sidebar.

S/MIME keys can be created, imported and exported on this screen, but the web app does
not sign, encrypt or decrypt mail with them. See [S/MIME certificates](#s-mime-certificates).

![The Keys and certificates screen with the Generate key and Import key buttons, the Your keys and Contact keys lists, and the Look up a key form.](/screenshots/user-features/keys-screen.webp){.mw-shot}

## Where your private key is stored

::: warning
A copy of each private key, locked with its key passphrase, is stored on the server so
you can use the key on other devices. The server cannot use the key without the
passphrase, so the strength of the passphrase is what protects that copy.
:::

The passphrase also locks the key in this browser. Choose a long passphrase that you do
not use anywhere else. The passphrase itself never leaves your browser.

## Generate a key

![The top of the Keys and certificates screen with the note about where private keys are stored and markers on Generate key and Import key.](/screenshots/user-features/keys-actions.webp){.mw-crop}

1. **Generate key** opens the key form.
2. **Import key** imports a key you already have.
{.mw-callouts}

1. Choose **Generate key**.
2. Leave **Type** on **OpenPGP**. Enter a **Name** and an **Email** for the key, and a
   **Key passphrase**.
3. Choose **Generate**.

![The Generate a key dialog with the type OpenPGP, the name Alex Rivera, the address alex@example.com and a passphrase entered, and a note that the locked key is also stored on the server.](/screenshots/user-features/keys-generate.webp){.mw-crop}

1. **Type**: **OpenPGP** or **S/MIME**.
2. **Name**.
3. **Email**.
4. **Key passphrase**.
5. **Generate** creates the key. If no key is made, the dialog says "No key was
   generated."
{.mw-callouts}

The new key appears under **Your keys** with its fingerprint and a trust label.

## Check a key

Select a key to see its details.

![The details of an OpenPGP key: its fingerprint, safe words, a QR code, the Autocrypt state, the trust level, a contact to associate it with, and the backup export button.](/screenshots/user-features/keys-card.webp){.mw-crop}

1. **Fingerprint** is the key's identifier. To confirm a contact's key, compare it with
   the fingerprint they give you by another channel.
2. **Safe words** are the fingerprint as words. Read them aloud with the contact to
   confirm the key out-of-band.
3. **Scan to verify** shows the fingerprint as a QR code (it follows the Autocrypt
   line).
4. **Trust level** is one of **unverified**, **tofu**, **verified** or **revoked**.
5. **Associate** links the key to a contact you pick under **Associate with a contact**.
6. **Export backup** makes an Autocrypt Setup Message to move the key to another device.
{.mw-callouts}

The details also say whether the key is **Advertised in Autocrypt headers** or **Not
advertised via Autocrypt**.

## Find someone's key

Under **Look up a key**, type the person's address and choose the sources: **WKD**
and **VKS** are ticked at first; **AUTOCRYPT** and **HARVESTED** are the others. A lookup
first checks the keys already stored for the address. If none is found and **WKD** is
ticked, the server asks the address's domain for the key over WKD. VKS, Autocrypt and
Harvested contact no outside service; they only check stored keys. Because a WKD lookup
contacts the address's domain, you must tick the consent box, which ends "I have this
person's consent to do so." Then choose **Look up**. Keys found are added under **Contact
keys**; if none is found, the screen says "No key found."

![The Look up a key form with an address typed in, the four lookup sources WKD, VKS, Autocrypt and Harvested, the consent checkbox and the Look up button.](/screenshots/user-features/keys-lookup.webp){.mw-crop}

1. The address to look up.
2. The lookup sources.
3. The consent checkbox for external directories.
4. **Look up** runs the search.
{.mw-callouts}

## Import a key

Choose **Import key**. Choose **Armored (PGP)** to paste a key that starts with
`-----BEGIN PGP`, adding its passphrase if it is a private key, or **PKCS#12
(S/MIME)** to pick a `.p12` or `.pfx` file and enter its password. **Preview** shows
the key type and fingerprint, and whether it includes a private key or is a public key
only. Choose **Import** to keep it. A private key is stored on this device and, locked
with its passphrase, on the server.

## Encrypt and sign a message

In the message editor, open the **Security** tab of the ribbon.

![The Security tab of the message editor ribbon with the Encrypt (end-to-end) switch greyed out and the Sign switch available.](/screenshots/user-features/keys-compose-security.webp){.mw-crop}

1. **Encrypt (end-to-end)** encrypts the message body on your device before it is sent.
   It is available when every recipient has a key. Until you add a recipient it stays
   off with "Add a recipient to check for encryption keys."
2. **Sign (verify it's from you)** signs the message with your key. If the key is
   locked, you are asked for its passphrase.
{.mw-callouts}

A status line shows what the message will get: **End-to-end encrypted** when every
recipient has a key, **Transport encryption (TLS)** when no recipient key was found, or
**Mixed protection** when only some recipients have keys. With encryption on you can
also choose **Also encrypt the subject line**.

An encrypted message is not saved to Drafts as plain text. Closing it discards your
changes, and an earlier draft stays in Drafts.

If an administrator rule requires encryption for a message, the composer shows
**Encryption required** and sending is blocked until you turn encryption on.

## Read an encrypted message

An encrypted OpenPGP message shows a key passphrase field and a **Decrypt** button in
the reader. Enter your key passphrase and choose **Decrypt**. If no private key can open the message, the
reader says "No private key is available to decrypt this message." If it fails it says
"Decryption failed".

## S/MIME certificates

Choosing **S/MIME** as the type makes an RSA 3072-bit key and a certificate for the
address, valid for two years. The certificate is self-signed: other people's mail
programs will not trust it unless they accept it by hand, or until a certificate
authority issues one. Key generation can take up to a minute.

The key's details offer **Export as PKCS#12 (.p12)**, **Download certificate request
(.csr)** and **Import issued certificate** (a `.pem`, `.crt` or `.cer` file issued for
this key). The `.p12` file is encrypted with the passphrase you enter on that screen.
Mailwoman does not sign or decrypt mail with an S/MIME key; export the key to use it in
another mail program.

## Troubleshooting

- **Encrypt (end-to-end) is greyed out.** No recipient key was found yet. Add the
  recipients and wait for "Checking recipient keys", or look a key up under
  **Look up a key**.
- **"That did not work. Check the passphrase."** The passphrase for an S/MIME export is
  wrong.
- **"A certificate can only carry a plain ASCII address (name@domain), and a name of at
  most 64 characters."** The S/MIME key was not generated; change the name or address.

## Related

- [Contacts](/user/contacts)
- [Composing messages](/user/mailbox/composing)
- [Settings: account](/user/settings/account)
- [Security model: encryption](/security/crypto)
