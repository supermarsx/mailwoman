# Mailwoman — login-time second-factor strings (source locale: en).
# Shown on the sign-in screen after the password is accepted but a second factor
# must be cleared before a session is issued (SPEC §7.4/§19, S1 login step). The
# challenge itself (TOTP / passkey / recovery UI + its copy) lives in the settings
# catalog — this file only covers the wrapper the login screen adds around it.

# Shown when an account is REQUIRED to use a second factor but has none enrolled
# yet: it cannot be completed from the sign-in screen (no downgrade to password).
login-2fa-enroll-required = This account requires two-step verification, but no method is set up yet. Sign in on a device where you can finish setup, or contact your administrator.

# Return to the credential form (e.g. to sign in as a different account).
login-2fa-back = Back to sign in
