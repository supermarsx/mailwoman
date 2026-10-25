# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = Zmiana hasła
passwd-heading = Zmiana hasła

# -- Forced change -----------------------------------------------------------
passwd-force-change = Administrator wymaga zmiany hasła przed kontynuowaniem.

# -- Fields ------------------------------------------------------------------
passwd-current-label = Obecne hasło
passwd-new-label = Nowe hasło
passwd-confirm-label = Potwierdź nowe hasło

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = Nowe hasła są zgodne.
passwd-match-no = Nowe hasła nie są jeszcze zgodne.

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count ->
    [one] co najmniej { $count } znak
    [few] co najmniej { $count } znaki
    [many] co najmniej { $count } znaków
   *[other] co najmniej { $count } znaku
}
passwd-rule-uppercase = wielką literę
passwd-rule-lowercase = małą literę
passwd-rule-digit = cyfrę
passwd-rule-symbol = symbol

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = To konto działa w trybie zero-access. Przed zastosowaniem zmiany zostanie wyświetlona fraza odzyskiwania — zapisz ją, aby zachować dostęp do danych, jeśli coś pójdzie nie tak.

# -- Actions -----------------------------------------------------------------
passwd-continue = Kontynuuj
passwd-submit = Zmień hasło

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = Zapisz frazę odzyskiwania
passwd-recovery-prose = Zapisz tę frazę i przechowuj ją w bezpiecznym miejscu. Jest wyświetlana przed zmianą hasła, aby można było odzyskać dane nawet w przypadku utraty nowego hasła. Nie jest przechowywana na serwerze.
passwd-recovery-ack-label = Zapisałem(-am) frazę odzyskiwania
passwd-recovery-ack-text = Zapisałem(-am) frazę odzyskiwania w bezpiecznym miejscu.

# -- Done phase --------------------------------------------------------------
passwd-done = Hasło zostało zmienione.
passwd-done-reencrypt = Zapisane dane logowania do serwera zostały ponownie zaszyfrowane nowym hasłem.
passwd-done-rewrap = Klucze zero-access zostały ponownie opakowane (re-wrap) nowym hasłem.

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = wpisz obecne hasło
passwd-error-mismatch = nowe hasło i jego potwierdzenie nie są zgodne
passwd-error-policy = nowe hasło musi zawierać: { $rules }
passwd-error-ack-first = najpierw potwierdź, że fraza odzyskiwania została zapisana
passwd-error-prepare-recovery = nie udało się przygotować frazy odzyskiwania
passwd-error-change = nie udało się zmienić hasła
# Shown under the server's own message when a change is refused. Each line states
# what the status means; none of them claims the password was or was not changed.
passwd-error-session-ended = Sesja wygasła. Zaloguj się ponownie.
passwd-error-not-configured = Ten serwer nie jest skonfigurowany do zmiany haseł. Skontaktuj się z administratorem.
passwd-error-contact-admin = Serwer nie mógł dokończyć zmiany. Skontaktuj się z administratorem.

# -- Forced change screen ----------------------------------------------------
# Shown instead of the mailbox while an administrator's "force password change"
# flag is set on the account. The server refuses mail requests until the change
# succeeds, so this copy must say exactly that — including the case where the
# server has no way to change the password.
passwd-forced-title = Zmień hasło, aby kontynuować
passwd-forced-signed-in-as = Zalogowano jako { $username }.
passwd-forced-explain = Administrator wymusił zmianę hasła dla tego konta. Poczta, kalendarz i kontakty są niedostępne, dopóki zmiana się nie powiedzie.
passwd-forced-backend-note = Jeśli zmiana zostanie odrzucona, mimo że obecne hasło jest poprawne, ten serwer może nie mieć możliwości zmiany hasła. Skontaktuj się z administratorem: może skonfigurować zmianę haseł lub usunąć ten wymóg.
passwd-forced-sign-out = Wyloguj
passwd-forced-still-required = Serwer przyjął zmianę, ale nadal wymaga zmiany hasła dla tego konta. Skontaktuj się z administratorem.
passwd-forced-reload-failed = Hasło zostało zmienione, ale nie udało się wczytać konta. Sprawdź połączenie i spróbuj ponownie.
passwd-forced-retry = Spróbuj ponownie
