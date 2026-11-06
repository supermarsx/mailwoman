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
