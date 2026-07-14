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
