---
screen: true
---

# Sign in

The sign-in screen is the first thing you see when you open Mailwoman. You sign in with the address and password of your mail account; Mailwoman checks them with the mail server your administrator configured. This page describes the sign-in screen, the second-factor step and what each error message means.

![The sign-in screen with empty Email or username and Password fields and the Sign in button.](/screenshots/deploy/signin-form.webp){.mw-shot}

## Sign in with your password

1. In **Email or username**, type your mail address.
   If your administrator turned on the domain hint, the screen shows "You can sign in with just your username for *domain*", and you can type only the part before the `@`.
2. In **Password**, type the password of your mail account.
3. Select **Sign in**. The button reads **Signing in…** while the server checks your details.

![The sign-in form with an address in Email or username, a hidden password, and markers 1 to 3 on the address field, the password field and the Sign in button.](/screenshots/deploy/signin-filled.webp){.mw-crop}

1. **Email or username**
2. **Password**
3. **Sign in**
{.mw-callouts}

You do not enter a server address: the server decides which mail server an address signs in to. The first time you sign in, Mailwoman creates your account on this server, unless your administrator only lets in addresses they added in advance or has closed new accounts.

When the sign-in succeeds, the mailbox opens. See [Using Mailwoman](../user/index.md) for a tour.

### Other ways to sign in

These buttons appear only when your administrator has set them up, and only in a browser (the desktop and Android apps sign in with a password):

- **Sign in with a passkey**: shown when the server allows passkey sign-in and your browser supports passkeys. It signs you in with a passkey you registered earlier in your security settings.
- **Sign in with** *provider*: one button per single sign-on provider, below "or continue with". It takes you to your organisation's sign-in page and back.

![The sign-in form with a Sign in with Harbour Lane Studio button below or continue with.](/screenshots/deploy/signin-sso.webp){.mw-crop}

Some servers also show a **Sign-in verification** check above the **Sign in** button. Complete it before you select **Sign in**; it is reset after each attempt.

## Previous sign-ins

On a device where you signed in before, a small arrow at the right of **Email or username** opens **Previous sign-ins**: the addresses that signed in successfully on this device. Choosing one fills in the address only; you still type the password.

![The sign-in form with the Previous sign-ins list open below the address field, listing two addresses, each with a delete icon to forget it.](/screenshots/deploy/signin-remembered.webp){.mw-crop}

- To remove an address, select the delete icon beside it (**Forget** *address* **on this device**).
- The list is kept in this browser only, for up to 10 addresses per server. Passwords are never stored in it.
- Your administrator can turn the list off ([`login.rememberSignins`](../admin/settings.md#login.rememberSignins)); the browser then deletes the stored addresses for this server the next time the sign-in screen opens.

## Confirm your second factor

If your account uses two-step verification, the password alone does not sign you in. After **Sign in**, the screen changes to **Two-factor authentication**: "Confirm your second factor to finish signing in."

1. Choose the method you set up:
   - **Authenticator code**: type the current code from your authenticator app.
   - **Recovery code**: type one of the recovery codes you saved when you set up two-step verification.
   - **Use a passkey**: confirm with your passkey or security key.
2. Select **Verify**.

![The Two-factor authentication step of signing in, with the Authenticator code tab selected, marker 1 on the code field and marker 2 on the Verify button.](/screenshots/deploy/signin-2fa.webp){.mw-crop}

1. The code field, here for an **Authenticator code**.
2. **Verify**
{.mw-callouts}

If the code is wrong, the screen says "That did not verify. Try again." To start over, for example to use another account, select **Back to sign in**.

## Set up two-step verification

If your administrator requires two-step verification and your account has no method set up yet, the screen after **Sign in** is **Set up two-step verification to continue**. You set up a method there and then finish signing in.

<!-- SCREENSHOT-PENDING deploy/signin-2fa-enrol -->

1. Under **Choose a verification method**, select **Set up an authenticator app**. In a browser that supports passkeys, and if your administrator allows them, you can choose **Add a passkey** instead and follow your browser's prompts.
2. Scan the QR code with your authenticator app, or type the **Setup key** into the app by hand.
3. Type the code the app shows into **Authenticator code** and select **Confirm**.
4. The screen says "Two-step verification is set up and you are signed in." and shows your **Recovery codes**. Save them somewhere safe: they are shown once, and each code works one time.
5. Tick **I have saved these codes** and select **Continue**. The mailbox opens.

From then on, signing in asks for a code as described in [Confirm your second factor](#confirm-your-second-factor).

You have 5 minutes to finish the setup. If it takes longer, the screen goes back to the sign-in form with "Sign-in timed out before setup was finished. …". Sign in again: you get a new setup key, so remove the unfinished entry from your authenticator app before you add the new one.

If the server offers no method this device can set up (for example only passkeys, in a browser without passkey support), the screen says "This account requires two-step verification, but no method is set up yet. …". Sign in on a device where you can finish the setup, or contact your administrator.

## If sign-in fails

| Message | What it means | What to do |
| --- | --- | --- |
| Invalid credentials | The mail server did not accept the address and password. If your administrator chose not to explain refusals, a disabled or refused account gets this message too. | Check the address and password. If they are right, ask your administrator. |
| This address is not set up on this server. Contact your administrator. | No mail server is configured for the address's domain. | Ask your administrator to configure a mail server for it. |
| No account exists for this address yet, and new accounts are not created automatically. Ask your administrator to add you. | This server only lets in addresses an administrator added in advance. | Ask your administrator to add you. |
| This address may not sign in on this server. Contact your administrator. | A sign-in policy refuses this address. | Ask your administrator. |
| This account has been disabled by an administrator. … | An administrator disabled the account. | Ask your administrator to enable it again. |
| Too many sign-in attempts. Please wait before trying again. | Too many attempts in a short time. The button shows **Try again in** *n*s. | Wait for the countdown to end. |
| Complete a new sign-in verification before trying again. | The server now asks for the sign-in verification check. | Complete the check, then select **Sign in** again. |
| That code was not accepted. Enter the current code from your authenticator app, and check that your device's clock is correct. | While setting up two-step verification, the code did not match: it was mistyped, expired, or your device's clock is off. | Type the code the app shows now. If it keeps failing, correct the device's time setting. |
| Sign-in timed out before setup was finished. … | Setting up two-step verification took longer than 5 minutes. | Sign in again, and remove the unfinished entry from your authenticator app before adding the new key. |
| Sign-in timed out. … | The server did not answer in time. Nothing was retried automatically. | Check your connection and try again. |
| Sign-in stopped: the mail server sent a reply this server could not read. … | The mail server answered in a way Mailwoman could not use. This is not a password problem. | Try again later, or tell your administrator. |
| Webmail is temporarily paused for maintenance. … | An administrator paused sign-in for maintenance. The screen may show their own message instead. | Try again later. |
| Could not reach the server | The browser could not reach the Mailwoman server. | Check your connection. |

When a message is shown, the screen may also show **Reference:** followed by an identifier. Give it to your administrator: it lets them find your attempt in the server's logs.

## Related

- [Quick install](./install-quick.md), for administrators setting up a server
- [Using Mailwoman](../user/index.md)
- [Passkeys](../security/passkeys.md)
- [Single sign-on](../admin/sso.md)
