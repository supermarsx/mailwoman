# Mailwoman — in-app password change + zero-access re-wrap strings (source locale: en).
#
# Lazily loaded catalog for the `passwd` module (SPEC §18.3, plan §3 e7). Security-
# explanatory copy (key re-wrap / recovery phrase / what happens to encrypted data) is
# localized FAITHFULLY — do not soften, shorten, or editorialize it. Message ids are
# kebab-case and module-prefixed (`passwd-*`).

# -- Region / heading --------------------------------------------------------
passwd-region-label = Change password
passwd-heading = Change password

# -- Forced change -----------------------------------------------------------
passwd-force-change = Your administrator requires you to change your password before continuing.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Current password
passwd-new-label = New password
passwd-confirm-label = Confirm new password

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = New passwords match.
passwd-match-no = New passwords do not match yet.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = at least { $count } characters
passwd-rule-uppercase = an uppercase letter
passwd-rule-lowercase = a lowercase letter
passwd-rule-digit = a digit
passwd-rule-symbol = a symbol

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = This account is zero-access. Before the change is applied you will be shown a recovery phrase — save it so you can still reach your data if anything goes wrong.

# -- Actions -----------------------------------------------------------------
passwd-continue = Continue
passwd-submit = Change password

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Save your recovery phrase
passwd-recovery-prose = Write this phrase down and keep it somewhere safe. It is shown before the password change so you can recover your data even if the new password is lost. It is not stored on the server.
passwd-recovery-ack-label = I have saved my recovery phrase
passwd-recovery-ack-text = I have saved my recovery phrase somewhere safe.

# -- Done phase --------------------------------------------------------------
passwd-done = Your password has been changed.
passwd-done-reencrypt = Your stored server credentials were re-encrypted under the new password.
passwd-done-rewrap = Your zero-access keys were re-wrapped under the new password.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = enter your current password
passwd-error-mismatch = the new password and its confirmation do not match
passwd-error-policy = the new password needs { $rules }
passwd-error-ack-first = confirm you have saved the recovery phrase first
passwd-error-prepare-recovery = could not prepare the recovery phrase
passwd-error-change = could not change the password
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Your session has ended. Sign in again.
passwd-error-not-configured = This server is not set up to change passwords. Contact your administrator.
passwd-error-contact-admin = The server could not complete the change. Contact your administrator.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Change your password to continue
passwd-forced-signed-in-as = Signed in as { $username }.
passwd-forced-explain = An administrator has required a password change for this account. Mail, calendar and contacts are not available until the change succeeds.
passwd-forced-backend-note = If the change is refused even though your current password is correct, this server may not be able to change your password. Contact your administrator: they can set up password changes or remove the requirement.
passwd-forced-sign-out = Sign out
passwd-forced-still-required = The server accepted the change but still requires a password change for this account. Contact your administrator.
passwd-forced-reload-failed = The password was changed, but your account could not be loaded. Check your connection and try again.
passwd-forced-retry = Try again
