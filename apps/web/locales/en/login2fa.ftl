# Mailwoman — login-time second-factor strings (source locale: en).
# Shown on the sign-in screen after the password is accepted but a second factor
# must be cleared before a session is issued (SPEC §7.4/§19, S1 login step). The
# challenge itself (TOTP / passkey / recovery UI + its copy) lives in the settings
# catalog — this file only covers the wrapper the login screen adds around it.

# Shown when an account is REQUIRED to use a second factor but has none enrolled
# yet, and the server offers no method this client can set up (for example only
# a passkey, in a browser without passkey support). Otherwise the sign-in screen
# runs the enrolment step below instead (no downgrade to password).
login-2fa-enroll-required = This account requires two-step verification, but no method is set up yet. Sign in on a device where you can finish setup, or contact your administrator.

# Return to the credential form (e.g. to sign in as a different account).
login-2fa-back = Back to sign in

# Forced enrolment at sign-in (t47-e2): the policy requires a second factor and
# the account has none, so the sign-in screen sets one up before the session opens.
# The setup controls reuse the settings catalog's two-factor strings.
login-2fa-enrol-title = Set up two-step verification to continue
login-2fa-enrol-intro = Your administrator requires a second step at sign-in for this account. Set one up now to finish signing in.
login-2fa-enrol-choose = Choose a verification method
login-2fa-enrol-totp-scan = Scan the QR code with your authenticator app, or enter the setup key by hand. Then enter the code the app shows.
login-2fa-enrol-qr-alt = QR code for adding this account to an authenticator app
login-2fa-enrol-manual-key = Setup key
login-2fa-enrol-passkey-help = Your browser will ask you to create a passkey on this device or on a security key.
login-2fa-enrol-code-rejected = That code was not accepted. Enter the current code from your authenticator app, and check that your device's clock is correct.
login-2fa-enrol-expired = Sign-in timed out before setup was finished. Sign in again. You will get a new setup key, so remove the unfinished entry from your authenticator app.
login-2fa-enrol-done = Two-step verification is set up and you are signed in.
login-2fa-enrol-continue = Continue
login-2fa-enrol-other-method = Use a different method
