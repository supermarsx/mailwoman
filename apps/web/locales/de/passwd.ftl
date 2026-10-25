# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Passwort ändern
passwd-heading = Passwort ändern

# -- Forced change -----------------------------------------------------------
passwd-force-change = Ihr Administrator verlangt, dass Sie Ihr Passwort ändern, bevor Sie fortfahren.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Aktuelles Passwort
passwd-new-label = Neues Passwort
passwd-confirm-label = Neues Passwort bestätigen

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Die neuen Passwörter stimmen überein.
passwd-match-no = Die neuen Passwörter stimmen noch nicht überein.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = mindestens { $count } Zeichen
passwd-rule-uppercase = einen Großbuchstaben
passwd-rule-lowercase = einen Kleinbuchstaben
passwd-rule-digit = eine Ziffer
passwd-rule-symbol = ein Sonderzeichen

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = Dieses Konto verwendet Zero-Access. Bevor die Änderung übernommen wird, wird Ihnen eine Wiederherstellungsphrase angezeigt — speichern Sie sie, damit Sie Ihre Daten auch dann noch erreichen, wenn etwas schiefgeht.

# -- Actions -----------------------------------------------------------------
passwd-continue = Weiter
passwd-submit = Passwort ändern

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Wiederherstellungsphrase speichern
passwd-recovery-prose = Schreiben Sie diese Phrase auf und bewahren Sie sie an einem sicheren Ort auf. Sie wird vor der Passwortänderung angezeigt, damit Sie Ihre Daten auch dann wiederherstellen können, wenn das neue Passwort verloren geht. Sie wird nicht auf dem Server gespeichert.
passwd-recovery-ack-label = Ich habe meine Wiederherstellungsphrase gespeichert
passwd-recovery-ack-text = Ich habe meine Wiederherstellungsphrase an einem sicheren Ort gespeichert.

# -- Done phase --------------------------------------------------------------
passwd-done = Ihr Passwort wurde geändert.
passwd-done-reencrypt = Ihre gespeicherten Server-Zugangsdaten wurden mit dem neuen Passwort neu verschlüsselt.
passwd-done-rewrap = Ihre Zero-Access-Schlüssel wurden mit dem neuen Passwort neu verpackt.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = Geben Sie Ihr aktuelles Passwort ein
passwd-error-mismatch = Das neue Passwort und seine Bestätigung stimmen nicht überein
passwd-error-policy = Das neue Passwort benötigt { $rules }
passwd-error-ack-first = Bestätigen Sie zuerst, dass Sie die Wiederherstellungsphrase gespeichert haben
passwd-error-prepare-recovery = Die Wiederherstellungsphrase konnte nicht vorbereitet werden
passwd-error-change = Das Passwort konnte nicht geändert werden
passwd-error-session-ended = Ihre Sitzung ist abgelaufen. Melden Sie sich erneut an.
passwd-error-not-configured = Dieser Server ist nicht für Passwortänderungen eingerichtet. Wenden Sie sich an Ihren Administrator.
passwd-error-contact-admin = Der Server konnte die Änderung nicht abschließen. Wenden Sie sich an Ihren Administrator.

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = Ändern Sie Ihr Passwort, um fortzufahren
passwd-forced-signed-in-as = Angemeldet als { $username }.
passwd-forced-explain = Ein Administrator hat für dieses Konto eine Passwortänderung verlangt. E-Mail, Kalender und Kontakte sind erst verfügbar, wenn die Änderung erfolgreich war.
passwd-forced-backend-note = Wenn die Änderung abgelehnt wird, obwohl Ihr aktuelles Passwort korrekt ist, kann dieser Server Ihr Passwort möglicherweise nicht ändern. Wenden Sie sich an Ihren Administrator: Er kann Passwortänderungen einrichten oder die Anforderung aufheben.
passwd-forced-sign-out = Abmelden
passwd-forced-still-required = Der Server hat die Änderung angenommen, verlangt für dieses Konto aber weiterhin eine Passwortänderung. Wenden Sie sich an Ihren Administrator.
passwd-forced-reload-failed = Das Passwort wurde geändert, aber Ihr Konto konnte nicht geladen werden. Prüfen Sie Ihre Verbindung und versuchen Sie es erneut.
passwd-forced-retry = Erneut versuchen
