---
screen: true
---
# Administrator authenticator and passkeys

Each administrator sets up their own second factor for signing in to the console:
an authenticator app (a six-digit code that changes every 30 seconds), one or more
passkeys, or both. This is separate from the mailbox: it changes only your own
console account, never a mailbox user's sign-in. Open **My preferences** >
**Personal security** (`#/preferences/security`).

![My preferences open on the Personal security tab, with Administrator passkeys and the Authenticator app section.](/screenshots/admin-access/authenticator-page.webp){.mw-shot}

The tab has three parts:

![The Personal security tab of My preferences.](/screenshots/admin-access/authenticator-personal-security.webp){.mw-crop}

1. **Add administrator passkey** registers a passkey on this device.
2. **Manage authenticator and recovery codes** opens the authenticator app page.
3. **Change password** changes your console password.
{.mw-callouts}

If your console identity has no account record of its own, the tab shows "Your sign-in
credentials are managed by the server configuration. Personal password and
authenticator changes are unavailable for this identity." instead of these controls.

## Set up an authenticator app

1. Open **My preferences** > **Personal security** and select **Manage
   authenticator and recovery codes**.
2. Select **Set up authenticator**.
3. In **Confirm it is you**, enter your console password (and a code or passkey if
   you already have a factor) and select **Confirm and continue**. The confirmation
   lasts five minutes in this session.

   ![The Confirm it is you dialog asking for the administrator's password.](/screenshots/admin-access/authenticator-confirm.webp){.mw-crop}

4. Scan the QR code with your authenticator app, or type the **Manual setup key**
   into it. The QR code is drawn in your browser.
5. Enter the app's current code in **Authenticator code** and select **Confirm
   authenticator**.

   ![Authenticator setup with the QR code and manual key hidden.](/screenshots/admin-access/authenticator-setup.webp){.mw-crop}

   1. The QR code.
   2. **Manual setup key**: the same secret as text. Keep it private.
   3. **Authenticator code**: the six-digit code from the app.
   4. **Confirm authenticator** finishes the setup.
   {.mw-callouts}

6. If this is your first second factor, the page shows **Save your recovery codes**:
   ten codes, shown only once. Store them somewhere safe, then select **I have saved
   my recovery codes**.

The setup must be finished within ten minutes, in the same console session that
started it. After five wrong codes it is cancelled; select **Set up authenticator**
again. Leaving the page while the key or the recovery codes are shown asks before
discarding them.

From then on, console sign-in asks for the **Authentication code** after the
password.

## Add a passkey

On **Personal security**, enter a **Passkey name** (optional) and select **Add
administrator passkey**. The browser asks you to create the passkey; the passkey
must verify you (PIN or biometric). If this is your first second factor, recovery
codes are shown. A passkey can then be used to sign in to the console and to confirm
sensitive changes. Each passkey is listed with **Supports backup and sync** or
**Bound to this authenticator**, and a **Remove passkey** button. The last usable
credential of the last **Super admin** cannot be removed.

## Recovery codes

Recovery codes are shared by all your console second factors. Each works once, with
your password, in place of a code or passkey, at sign-in or in **Confirm it is you**.
The authenticator page shows **Unused recovery codes**. **Replace recovery codes**
(after a confirmation) shows a new set and makes all earlier unused codes invalid.

## Remove the authenticator

Select **Remove authenticator** on the authenticator page and confirm. If you have
lost the authenticator, confirm with your password and a recovery code, or with a
passkey. To set up a different authenticator, remove the current one first.

## Related

- [Security centre](./security-center.md)
- [Admin console security](/security/admin-panel): sign-in, sessions and
  confirmation of sensitive actions in detail
- [Deployment checks](./deployment-security.md): shows administrators without a
  second factor
