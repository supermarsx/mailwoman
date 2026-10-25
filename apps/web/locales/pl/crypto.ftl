# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: crypto. Source: ../en/crypto.ftl

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = Szyfrowanie end-to-end
crypto-banner-e2ee-detail = Każdy odbiorca ma klucz — treść wiadomości jest szyfrowana na tym urządzeniu.
crypto-banner-tls-label = Szyfrowanie transportu (TLS)
crypto-banner-tls-detail = Nie znaleziono kluczy szyfrowania odbiorców; dostarczenie jest chronione tylko w trakcie przesyłania.
crypto-banner-mixed-label = Ochrona mieszana
crypto-banner-mixed-detail = Część odbiorców może otrzymać wiadomość szyfrowaną end-to-end; pozostali tylko z TLS.
crypto-banner-checking-label = Sprawdzanie kluczy odbiorców
crypto-banner-checking = · sprawdzanie…
crypto-banner-tls-only = Tylko TLS dla:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Bezpieczeństwo wiadomości
crypto-encrypt-label = Szyfruj (end-to-end)
crypto-encrypt-hint-default = Zaszyfruj treść na tym urządzeniu przed wysłaniem.
crypto-encrypt-drafted = Wersja robocza zaszyfrowana na tym urządzeniu.
crypto-encrypt-no-key = Brak dostępnego klucza szyfrowania odbiorcy.
crypto-sign-label = Podpisz (potwierdza, że wiadomość jest od Ciebie)
crypto-protect-subject = Szyfruj również temat
crypto-reason-add-recipient = Dodaj odbiorcę, aby sprawdzić klucze szyfrowania.
crypto-reason-tls = Brak dostępnego klucza szyfrowania odbiorcy — wysyłanie przez TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Ostrzeżenia ochrony przed utratą danych
crypto-dlp-block = Wysyłanie zablokowane
crypto-dlp-require = Wymagane szyfrowanie
crypto-dlp-warn = Uwaga
crypto-dlp-notify = Administrator zostanie powiadomiony
crypto-dlp-matched = Dopasowano: { $list }
