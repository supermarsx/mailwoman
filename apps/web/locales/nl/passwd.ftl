# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Wachtwoord wijzigen
passwd-heading = Wachtwoord wijzigen

# -- Forced change -----------------------------------------------------------
passwd-force-change = Je beheerder vereist dat je je wachtwoord wijzigt voordat je verdergaat.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Huidig wachtwoord
passwd-new-label = Nieuw wachtwoord
passwd-confirm-label = Nieuw wachtwoord bevestigen

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = De nieuwe wachtwoorden komen overeen.
passwd-match-no = De nieuwe wachtwoorden komen nog niet overeen.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = minstens { $count } tekens
passwd-rule-uppercase = een hoofdletter
passwd-rule-lowercase = een kleine letter
passwd-rule-digit = een cijfer
passwd-rule-symbol = een symbool

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Dit account gebruikt zero-access-versleuteling. Voordat de wijziging wordt doorgevoerd, krijg je een herstelzin te zien — bewaar die, zodat je nog bij je gegevens kunt als er iets misgaat.

# -- Actions -----------------------------------------------------------------
passwd-continue = Doorgaan
passwd-submit = Wachtwoord wijzigen

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Bewaar je herstelzin
passwd-recovery-prose = Schrijf deze zin op en bewaar hem op een veilige plek. Hij wordt vóór de wachtwoordwijziging getoond, zodat je je gegevens kunt herstellen, ook als het nieuwe wachtwoord kwijtraakt. Hij wordt niet op de server opgeslagen.
passwd-recovery-ack-label = Ik heb mijn herstelzin bewaard
passwd-recovery-ack-text = Ik heb mijn herstelzin op een veilige plek bewaard.

# -- Done phase --------------------------------------------------------------
passwd-done = Je wachtwoord is gewijzigd.
passwd-done-reencrypt = Je opgeslagen serveraanmeldgegevens zijn opnieuw versleuteld met het nieuwe wachtwoord.
passwd-done-rewrap = Je zero-access-sleutels zijn opnieuw ingepakt met het nieuwe wachtwoord.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = voer je huidige wachtwoord in
passwd-error-mismatch = het nieuwe wachtwoord en de bevestiging komen niet overeen
passwd-error-policy = het nieuwe wachtwoord moet bevatten: { $rules }
passwd-error-ack-first = bevestig eerst dat je de herstelzin hebt bewaard
passwd-error-prepare-recovery = kan de herstelzin niet voorbereiden
passwd-error-change = kan het wachtwoord niet wijzigen
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Je sessie is beëindigd. Log opnieuw in.
passwd-error-not-configured = Deze server is niet ingesteld om wachtwoorden te wijzigen. Neem contact op met je beheerder.
passwd-error-contact-admin = De server kon de wijziging niet voltooien. Neem contact op met je beheerder.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Wijzig je wachtwoord om door te gaan
passwd-forced-signed-in-as = Ingelogd als { $username }.
passwd-forced-explain = Een beheerder heeft voor dit account een wachtwoordwijziging verplicht gesteld. Mail, agenda en contacten zijn niet beschikbaar totdat de wijziging is gelukt.
passwd-forced-backend-note = Als de wijziging wordt geweigerd terwijl je huidige wachtwoord juist is, kan deze server je wachtwoord mogelijk niet wijzigen. Neem contact op met je beheerder: die kan wachtwoordwijzigingen instellen of de verplichting opheffen.
passwd-forced-sign-out = Uitloggen
passwd-forced-still-required = De server heeft de wijziging geaccepteerd, maar vereist nog steeds een wachtwoordwijziging voor dit account. Neem contact op met je beheerder.
passwd-forced-reload-failed = Het wachtwoord is gewijzigd, maar je account kon niet worden geladen. Controleer je verbinding en probeer het opnieuw.
passwd-forced-retry = Opnieuw proberen
