# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Cofnij wysłanie
send-delay-help = Po naciśnięciu Wyślij wiadomość czeka w Skrzynce nadawczej przez ten czas. Do tego momentu możesz ją wycofać i dalej edytować. Wiadomość przechowuje serwer, więc zamknięcie przeglądarki nie wyśle jej wcześniej ani nie spowoduje jej utraty.
send-delay-label = Wstrzymuj wysyłane wiadomości przez
send-delay-off = Wyłączone (wysyłaj natychmiast)
send-delay-seconds = { $n } s
send-delay-minutes = { $n ->
    [one] 1 minutę
    [few] { $n } minuty
    [many] { $n } minut
   *[other] { $n } minuty
}
send-delay-custom = Własny…
send-delay-custom-label = Własne opóźnienie w sekundach
send-delay-custom-help = Pełne sekundy, od 0 do { $max }.
send-delay-capped = Administrator ogranicza tę wartość do { $max }.
send-delay-disabled = Administrator wyłączył wycofywanie wysłanych wiadomości. Wiadomości są wysyłane natychmiast.
send-delay-invalid = Wpisz liczbę całkowitą sekund od 0 do { $max }.
send-later-button = Wyślij później
send-later-menu = Opcje wysyłania później
send-later-preset-hour = Za 1 godzinę
send-later-preset-today = Dziś później, 17:00
send-later-preset-tomorrow = Jutro rano, 09:00
send-later-preset-monday = W poniedziałek rano, 09:00
send-later-custom = Wybierz datę i godzinę…
send-later-custom-label = Wyślij o
send-later-zone = Strefa czasowa: { $zone }
send-later-schedule = Zaplanuj wysłanie
send-later-cancel = Anuluj
send-later-err-invalid = Wpisz prawidłową datę i godzinę.
send-later-err-past = Wybierz czas w przyszłości.
send-later-err-horizon = To zbyt odległy termin. Wiadomości można planować z wyprzedzeniem do { $days } dni.
send-later-err-off = Administrator wyłączył zaplanowane wysyłanie.
send-queued = Wiadomość w kolejce — zostanie wysłana za
send-queued-undo = Cofnij
send-queued-now = Wyślij teraz
send-queued-recalled = Wiadomość wycofana. Wróciła do wersji roboczych.
send-queued-too-late = Za późno na wycofanie: wiadomość jest już wysyłana.
send-queued-sent-now = Wysyłanie.
send-queued-scheduled = Zaplanowano na { $when } ({ $zone }).
outbox-scheduled-badge = Zaplanowano
outbox-sends-in = Wysłanie za { $time }
outbox-sends-at = Wysłanie: { $when }
outbox-recall-edit = Wycofaj i edytuj
outbox-recall-too-late = Wiadomość jest już wysyłana; nie można jej wycofać.
outbox-failed-drafts = Nie udało się wysłać; wiadomość zachowano w wersjach roboczych.
outbox-queued-count = { $count ->
    [one] 1 wiadomość czeka na wysłanie
    [few] { $count } wiadomości czekają na wysłanie
    [many] { $count } wiadomości czeka na wysłanie
   *[other] { $count } wiadomości czeka na wysłanie
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Następne wysłanie: { $when }

outbox-chip-title = Wiadomości oczekujące na wysłanie

outbox-chip-view-all = Otwórz skrzynkę nadawczą

outbox-delete = Usuń

outbox-delete-confirm = Usunąć wiadomość?

outbox-delete-failed = Nie udało się usunąć wiadomości. { $error }

outbox-deleted = Wiadomość oczekująca w kolejce została usunięta.

outbox-edit-time = Edytuj czas wysłania

outbox-edit-time-conflict = Zmienił się czas wysłania lub wiadomość została już wysłana. Skrzynka nadawcza została ponownie załadowana.

outbox-edit-time-current = Obecnie { $when }

outbox-edit-time-refused = Nie można zmienić godziny wysłania. { $error }

outbox-edit-time-save = Oszczędzaj czas wysyłania

outbox-failed-toast = Nie udało się wysłać wiadomości. { $error }

outbox-failed-uncertain-toast = Wiadomość mogła zostać dostarczona lub nie. Przed ponownym wysłaniem zaznacz opcję Wysłano.

outbox-open = Otwórz

outbox-reader-actions = Akcje wiadomości w kolejce

outbox-reader-cc = DW: { $recipients }

outbox-reader-label = Wiadomość w kolejce

outbox-reader-loading = Ładowanie wiadomości…

outbox-reader-to = Do: { $recipients }

outbox-reader-unavailable = Nie udało się załadować wiadomości.

outbox-recall-to-drafts = Przypomnijmy sobie wersje robocze

outbox-send-again = Wyślij ponownie

outbox-send-again-failed = Nie udało się ponownie umieścić wiadomości w kolejce. { $error }

outbox-send-again-queued = Wiadomość znajduje się w kolejce do ponownego wysłania.

outbox-sent-copy-folder = Zostanie zapisany za { $folder }

outbox-sent-copy-missing = Zostaną zapisane w Wysłanych (wybrany folder jest niedostępny)

outbox-sent-copy-skip = Żadna kopia nie zostanie zapisana

outbox-status-attempts = { $count ->
        [one] Jak dotąd { $count } próba
        [few] Na razie { $count } próby
        [many] Jak dotąd { $count } prób
       *[other] Na razie { $count } próby
    }

outbox-status-canceled = Anulowano

outbox-status-failed = Nie wysłano

outbox-status-held = Trzymane, dopóki go nie zwolnisz

outbox-status-last-error = Ostatni błąd: { $error }

outbox-status-next-attempt = Następna próba { $when }

outbox-status-queued = W kolejce

outbox-status-retrying = Ponawiam próbę

outbox-status-scheduled = Zaplanowane

outbox-status-sending = Wysyłanie

outbox-status-sends-at-in = Wysyła { $when } (w { $time })

outbox-status-sent = Wysłane

outbox-status-uncertain = Dostawa nieznana
