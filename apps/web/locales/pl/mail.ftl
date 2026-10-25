# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Ustawienia
mail-compose = Nowa wiadomość
mail-nav-mailboxes = Foldery
mail-nav-attachments = Załączniki
mail-nav-outbox = Skrzynka nadawcza
mail-nav-sharing = Udostępnij folder
mail-nav-apps = Aplikacje
mail-logout = Wyloguj
mail-offline = Offline
mail-module-loading = Ładowanie: { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Szukaj
mail-search-label = Szukaj w poczcie
mail-search-placeholder = Szukaj w poczcie — from:alice subject:invoice larger:1mb
mail-search-clear = Wyczyść

# -- Message list ------------------------------------------------------------
mail-list-label = Wiadomości
mail-loading = Ładowanie wiadomości…
mail-empty = Brak wiadomości
mail-unknown-sender = (nieznany nadawca)
mail-no-subject = (bez tematu)
mail-pinned = Przypięte
mail-has-attachment = Ma załącznik
mail-answered = Odpowiedziano
mail-forwarded = Przekazano dalej
mail-unread = Nieprzeczytana
mail-flagged = Oznaczona flagą
# Announced on a selected row; the row also shows a check mark.
mail-selected = Zaznaczona
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Wiadomość { $pos } z { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } wiadomość
    [few] { $count } wiadomości
    [many] { $count } wiadomości
   *[other] { $count } wiadomości
}
mail-thread-expand = Rozwiń wątek, { $count ->
    [one] { $count } wiadomość
    [few] { $count } wiadomości
    [many] { $count } wiadomości
   *[other] { $count } wiadomości
}
mail-thread-collapse = Zwiń wątek, { $count ->
    [one] { $count } wiadomość
    [few] { $count } wiadomości
    [many] { $count } wiadomości
   *[other] { $count } wiadomości
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Opcje widoku
mail-reading-pane = Okienko odczytu
mail-reading-pane-right = Z prawej
mail-reading-pane-bottom = Na dole
mail-reading-pane-off = Wyłączone

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Zaznaczone wiadomości
mail-bulk-count = Zaznaczono: { $count }
mail-mark-read = Oznacz jako przeczytane
mail-mark-unread = Oznacz jako nieprzeczytane
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Oznacz flagą
mail-unflag-message = Usuń flagę
mail-move-to = Przenieś do folderu
mail-move-to-placeholder = Przenieś do…
mail-select-all = Zaznacz wszystko
mail-clear-selection = Wyczyść zaznaczenie

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = Priorytetowe Odebrane
mail-inbox-filter = Filtr folderu Odebrane
mail-inbox-focused = Priorytetowe
mail-inbox-other = Inne
mail-inbox-turn-off = Wyłącz

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Otwarte karty
mail-subtab-pin = Przypnij { $title }
mail-subtab-unpin = Odepnij { $title }
mail-subtab-tearoff = Otwórz { $title } w nowym oknie
mail-subtab-close = Zamknij { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Przypnij
mail-unpin = Odepnij
mail-snooze = Odłóż
mail-snooze-menu = Odłóż do
mail-snooze-later = Później dzisiaj
mail-snooze-tomorrow = Jutro
mail-snooze-next-week = W przyszłym tygodniu
mail-unsnooze = Anuluj odłożenie
mail-label = Etykieta
mail-labels-menu = Etykiety
mail-flag = Oznacz do wykonania
mail-clear-flag = Usuń oznaczenie do wykonania
mail-archive = Archiwizuj
mail-delete = Usuń
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Więcej działań

# -- Reader ------------------------------------------------------------------
mail-reader-label = Wiadomość
mail-reader-empty = Wybierz wiadomość do przeczytania
mail-back = Wstecz
mail-reader-from = Od: { $addr }
mail-reader-to = Do: { $addr }
mail-reader-cc = DW: { $addr }
mail-reader-actions = Działania na wiadomości
mail-reply = Odpowiedz
mail-reply-all = Odpowiedz wszystkim
mail-forward = Przekaż dalej
mail-reply-failed = Nie udało się przygotować wiadomości. Spróbuj ponownie.
mail-reply-unthreaded = Nie udało się odczytać identyfikatora oryginalnej wiadomości. Ta odpowiedź nie zostanie z nią połączona w wątek.
mail-spam = Spam
mail-export = Eksportuj
mail-sweep-sender = Uprzątnij wiadomości nadawcy
mail-message-body = Treść wiadomości
mail-no-content = Brak treści
mail-sanitizing = Oczyszczanie…
mail-attachments = Załączniki
mail-attachment-unnamed = (bez nazwy)
mail-attachment-open = Załącznik { $name }
mail-attachment-close = Zamknij załącznik
mail-attachment-loading = Ładowanie załącznika…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Zaszyfrowana wiadomość
mail-encrypted-note = 🔒 Ta wiadomość jest zaszyfrowana end-to-end. Odblokuj ją na tym urządzeniu, aby ją przeczytać.
mail-key-passphrase = Hasło klucza
mail-decrypt = Odszyfruj
mail-decrypting = Odszyfrowywanie…
mail-decrypt-no-key = Brak klucza prywatnego, którym można odszyfrować tę wiadomość.
mail-decrypt-failed = Odszyfrowanie nie powiodło się

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = { $date }, { $name } napisał(a):
mail-quote-attribution-undated = { $name } napisał(a):
# The header block above a forwarded original.
mail-forward-heading = ---------- Przekazana wiadomość ----------
mail-forward-from = Od: { $addr }
mail-forward-date = Data: { $date }
mail-forward-subject = Temat: { $subject }
mail-forward-to = Do: { $addr }
mail-forward-cc = DW: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Tworzenie wiadomości
mail-compose-title = Nowa wiadomość
mail-compose-title-reply = Odpowiedź
mail-compose-title-reply-all = Odpowiedź do wszystkich
mail-compose-title-forward = Przekazanie dalej
mail-compose-close = Zamknij
mail-compose-from = Od
mail-compose-from-default = Domyślna
mail-compose-to = Do
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Sugestie kontaktów
mail-compose-cc = DW
mail-compose-bcc = UDW
mail-compose-show-cc-bcc = Dodaj DW lub UDW
mail-compose-bcc-encrypted = Zaszyfrowana wiadomość zawiera klucz każdego odbiorcy, więc odbiorca UDW byłby widoczny dla pozostałych. Usuń odbiorców UDW lub wyłącz szyfrowanie.
mail-compose-plaintext-confirm-title = Wysłać odszyfrowany tekst bez szyfrowania?
mail-compose-plaintext-confirm-body = Ta wiadomość cytuje tekst z zaszyfrowanej wiadomości. Nie jest ustawiona do zaszyfrowania, więc cytowany tekst zostałby wysłany bez ochrony.
mail-compose-plaintext-confirm-send = Wyślij bez szyfrowania
mail-compose-plaintext-confirm-cancel = Wróć
mail-compose-subject = Temat
mail-compose-body = Treść
mail-compose-attachments = Załączniki
mail-compose-remove-attachment = Usuń { $name }
mail-compose-attach-nextcloud = Załącz z Nextcloud
mail-compose-close-nextcloud = Zamknij Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Załącz plik
mail-compose-uploading = Przesyłanie…
mail-compose-upload-unavailable = Przesyłanie plików jest obecnie niedostępne.
mail-compose-upload-failed = Nie udało się przesłać { $name }.
mail-compose-upload-too-large = { $name } ma { $size } MB. Maksymalny rozmiar przesyłanego pliku to { $max } MB.
mail-compose-send-later = Wyślij później
mail-compose-cancel = Anuluj
mail-compose-send = Wyślij
mail-compose-schedule = Zaplanuj
mail-compose-sending = Wysyłanie…
mail-compose-encrypted-subject = Zaszyfrowana wiadomość
mail-compose-dlp-blocked = Wysyłanie jest zablokowane przez regułę ochrony przed utratą danych (zobacz ostrzeżenie powyżej).
mail-compose-send-failed = Wysyłanie nie powiodło się
mail-compose-send-later-past = Wybierz przyszły czas wysłania lub wyczyść pole, aby wysłać teraz.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Nie zalogowano.
mail-send-no-draft-folder = To konto nie ma folderu na wersję roboczą.
mail-send-no-recipient = Dodaj co najmniej jednego odbiorcę.
mail-send-bad-recipient = { $count ->
    [one] To nie jest adres e-mail: { $addresses }
    [few] To nie są adresy e-mail: { $addresses }
    [many] To nie są adresy e-mail: { $addresses }
   *[other] To nie są adresy e-mail: { $addresses }
}
mail-send-later-past = Czas wysłania nie jest w przyszłości. Nic nie zostało wysłane.
mail-send-draft-refused = Serwer odrzucił tę wiadomość: { $reason }. Nic nie zostało wysłane.
mail-send-submission-refused = Serwer odmówił wysłania tej wiadomości: { $reason }. Nic nie zostało wysłane.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Odblokuj klucz podpisujący
mail-compose-sign-unlock-note = Wpisz hasło klucza, aby podpisać tę wiadomość. Klucz pozostaje odblokowany tylko w tym oknie tworzenia wiadomości.
mail-compose-sign-unlock = Odblokuj
mail-compose-sign-unlocking = Odblokowywanie…
mail-compose-sign-no-key = Na tym urządzeniu nie ma klucza podpisującego.
mail-compose-sign-unlock-failed = Nie udało się odblokować klucza podpisującego.
mail-compose-sign-unlock-required = Odblokuj klucz podpisujący, aby wysłać tę podpisaną wiadomość.

# Rich-text editor (W1)
mail-compose-format-plain = Zwykły tekst
mail-compose-format-rich = Tekst sformatowany
mail-compose-rt-toolbar = Formatowanie
mail-compose-rt-bold = Pogrubienie
mail-compose-rt-italic = Kursywa
mail-compose-rt-underline = Podkreślenie
mail-compose-rt-strike = Przekreślenie
mail-compose-rt-bullet = Lista punktowana
mail-compose-rt-ordered = Lista numerowana
mail-compose-rt-quote = Cytat
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Zastosuj

# Signature picker (W12)
mail-compose-signature = Podpis
mail-compose-signature-none = Dodaj podpis…

# Recall (W10)
mail-compose-recall = Wycofaj
mail-compose-recall-action = Wycofaj
mail-compose-recall-holding = Wstrzymana, oczekuje na wysłanie
mail-compose-recall-scheduled = Zaplanowano na { $when }

# Drafts drawer (W9)
mail-compose-drafts = Wersje robocze
mail-compose-drafts-empty = Brak zapisanych wersji roboczych
mail-compose-drafts-delete = Usuń wersję roboczą
mail-compose-drafts-no-recipient = (brak odbiorcy)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Skrzynka nadawcza
mail-outbox-refresh = Odśwież
mail-outbox-empty = Nic nie czeka na wysłanie
mail-outbox-send-now = Wyślij teraz
mail-outbox-cancel = Anuluj
mail-outbox-scheduled = Zaplanowana
mail-outbox-holding = Wkrótce zostanie wysłana
mail-outbox-sent = Wysłana
mail-outbox-canceled = Anulowana
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Wstrzymana
mail-outbox-failed = Niewysłana
mail-outbox-release = Zwolnij
mail-outbox-discard = Odrzuć
mail-outbox-held-note = Wstrzymana. Nie zostanie wysłana, dopóki nie zostanie zwolniona.
mail-outbox-held-note-api-key = Wstrzymana — utworzona przez klucz API { $name }. Nie zostanie wysłana, dopóki nie zostanie zwolniona.
mail-outbox-held-note-oauth = Wstrzymana — utworzona przez połączoną aplikację { $name }. Nie zostanie wysłana, dopóki nie zostanie zwolniona.
mail-outbox-origin-api-key = Utworzona przez klucz API { $name }
mail-outbox-origin-oauth = Utworzona przez połączoną aplikację { $name }
mail-outbox-to = Do: { $recipients }
mail-outbox-no-subject = (bez tematu)
mail-outbox-attempts = Liczba prób: { $count }. Ostatni błąd: { $error }
mail-outbox-error = Błąd: { $error }
mail-outbox-toast-canceled = Wysyłanie anulowane
mail-outbox-toast-discarded = Odrzucono. Wiadomość nie została wysłana.
mail-outbox-toast-sent = Wysłano
mail-outbox-toast-retrying = Jeszcze nie wysłano: { $error }. Nastąpi ponowna próba.
mail-outbox-toast-failed = Nie wysłano: { $error }
mail-outbox-toast-not-released = Nie udało się wysłać: { $error }
mail-outbox-toast-not-canceled = Nie udało się anulować: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Utracono połączenie — ponawianie…
mail-net-back = Ponownie online
mail-queue-sent = { $count ->
    [one] Wysłano { $count } zmianę z kolejki
    [few] Wysłano { $count } zmiany z kolejki
    [many] Wysłano { $count } zmian z kolejki
   *[other] Wysłano { $count } zmiany z kolejki
}
mail-queue-failed = { $count ->
    [one] { $count } zmiana z kolejki nie powiodła się
    [few] { $count } zmiany z kolejki nie powiodły się
    [many] { $count } zmian z kolejki nie powiodło się
   *[other] { $count } zmiany z kolejki nie powiodło się
}
mail-queue-notice-label = Zmiany, których nie udało się zastosować
mail-queue-notice-title = { $count ->
    [one] Nie udało się zastosować { $count } zmiany wprowadzonej offline
    [few] Nie udało się zastosować { $count } zmian wprowadzonych offline
    [many] Nie udało się zastosować { $count } zmian wprowadzonych offline
   *[other] Nie udało się zastosować { $count } zmiany wprowadzonej offline
}
mail-queue-kind-send = Wyślij „{ $subject }”
mail-queue-kind-send-untitled = Wyślij wiadomość bez tematu
mail-queue-kind-move = Przenieś wiadomość
mail-queue-kind-flag = Zmień etykietę lub flagę
mail-queue-kind-draft = Zapisz wersję roboczą
mail-queue-kind-pim = Zmiana w kalendarzu, zadaniu, notatce lub kontakcie
mail-queue-reason = Przyczyna: { $reason }
mail-queue-reason-unknown = Serwer nie podał przyczyny.
mail-queue-retry = Ponów
mail-queue-discard = Odrzuć
mail-queue-retry-item = Ponów: { $what }
mail-queue-discard-item = Odrzuć: { $what }
mail-queue-discarded = Odrzucono. Zmiana nie została zastosowana.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Uprzątnij wiadomości
mail-sweep-title = Uprzątnij: { $sender }
mail-sweep-all = Usuń wszystkie od tego nadawcy
mail-sweep-keep-latest = Zachowaj najnowszą, usuń pozostałe
mail-sweep-older-than = Usuń starsze niż N dni
mail-sweep-block = Usuń wszystkie i zablokuj tego nadawcę
mail-sweep-days = Dni
mail-sweep-count = { $count ->
    [one] { $count } wiadomość zostanie przeniesiona do kosza
    [few] { $count } wiadomości zostaną przeniesione do kosza
    [many] { $count } wiadomości zostanie przeniesionych do kosza
   *[other] { $count } wiadomości zostanie przeniesione do kosza
}
mail-sweep-cancel = Anuluj
mail-sweep-run = Uprzątnij ({ $count })
mail-sweeping = Porządkowanie…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Zamknij
mail-undo-action = Cofnij
mail-undo-send-cancel = Anuluj
mail-undo-failed = Nie udało się cofnąć

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Dodano etykietę
mail-toast-label-removed = Usunięto etykietę
mail-toast-pinned = Przypięto
mail-toast-unpinned = Odpięto
mail-toast-snoozed = Odłożono
mail-toast-follow-up-set = Oznaczono do wykonania
mail-toast-follow-up-cleared = Usunięto oznaczenie do wykonania
mail-toast-archived = Zarchiwizowano
mail-toast-trashed = Przeniesiono do kosza
mail-toast-marked-spam = Oznaczono jako spam
mail-toast-moved-to = Przeniesiono do { $folder }
mail-toast-moved = Przeniesiono do folderu
mail-toast-archived-count = Zarchiwizowane wiadomości: { $count }
mail-toast-trashed-count = Wiadomości przeniesione do kosza: { $count }
mail-toast-marked-spam-count = Wiadomości oznaczone jako spam: { $count }
mail-toast-moved-count = Przeniesione wiadomości: { $count }
mail-toast-moved-to-count = Wiadomości przeniesione do { $folder }: { $count }
mail-toast-marked-read = { $count ->
    [one] Oznaczono jako przeczytane
    [few] Oznaczono jako przeczytane { $count } wiadomości
    [many] Oznaczono jako przeczytane { $count } wiadomości
   *[other] Oznaczono jako przeczytane { $count } wiadomości
}
mail-toast-marked-unread = { $count ->
    [one] Oznaczono jako nieprzeczytane
    [few] Oznaczono jako nieprzeczytane { $count } wiadomości
    [many] Oznaczono jako nieprzeczytane { $count } wiadomości
   *[other] Oznaczono jako nieprzeczytane { $count } wiadomości
}
mail-toast-flagged = { $count ->
    [one] Oznaczono flagą
    [few] Oznaczono flagą { $count } wiadomości
    [many] Oznaczono flagą { $count } wiadomości
   *[other] Oznaczono flagą { $count } wiadomości
}
mail-toast-unflagged = { $count ->
    [one] Usunięto flagę
    [few] Usunięto flagę z { $count } wiadomości
    [many] Usunięto flagę z { $count } wiadomości
   *[other] Usunięto flagę z { $count } wiadomości
}
mail-toast-move-failed = Nie udało się przenieść wiadomości. Nic nie zostało przeniesione.
mail-toast-update-failed = Nie udało się zmienić wiadomości. Nic nie zostało zmienione.
mail-toast-update-refused = { $count ->
    [one] Serwer odrzucił zmianę dla jednej wiadomości
    [few] Serwer odrzucił zmianę dla { $count } wiadomości
    [many] Serwer odrzucił zmianę dla { $count } wiadomości
   *[other] Serwer odrzucił zmianę dla { $count } wiadomości
}
mail-toast-no-archive-folder = Brak folderu Archiwum
mail-toast-no-trash-folder = Brak folderu Kosz
mail-toast-no-spam-folder = Brak folderu Spam
mail-toast-sweep-nothing = Nie ma nic do uprzątnięcia
mail-toast-swept = { $count ->
    [one] Uprzątnięto { $count } wiadomość
    [few] Uprzątnięto { $count } wiadomości
    [many] Uprzątnięto { $count } wiadomości
   *[other] Uprzątnięto { $count } wiadomości
}
mail-toast-load-more-failed = Nie udało się wczytać więcej wiadomości
mail-toast-export-nothing = Nie ma nic do wyeksportowania
mail-toast-exported = Wyeksportowano plik .eml
mail-toast-export-failed = Eksport nie powiódł się
mail-toast-send-queued = W kolejce — zostanie wysłana po powrocie online
mail-toast-send-scheduled = Zaplanowano wysłanie
mail-toast-sent = Wiadomość wysłana
mail-toast-send-canceled = Wysyłanie anulowane

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Wstążka
mail-ribbon-tab-home = Narzędzia główne
mail-ribbon-tab-view = Widok
mail-ribbon-tab-folder = Folder
mail-ribbon-expand = Rozwiń
mail-ribbon-collapse = Zwiń
mail-ribbon-group-new = Nowy
mail-ribbon-compose = Nowa wiadomość
mail-ribbon-group-session = Sesja
mail-ribbon-logout = Wyloguj
mail-ribbon-group-theme = Motyw
mail-ribbon-group-density = Gęstość
mail-ribbon-group-settings = Ustawienia
mail-ribbon-more = Więcej…
mail-ribbon-group-folders = Foldery
mail-density-compact = kompaktowa
mail-density-cozy = wygodna
mail-density-relaxed = swobodna

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Poziom bezpieczeństwa wiadomości
mail-maxsec-full = Pełny
mail-maxsec-full-hint = Oczyszczony HTML z obrazami i stylami.
mail-maxsec-no-media = Bez multimediów
mail-maxsec-no-media-hint = Oczyszczony HTML bez wszystkich obrazów i multimediów.
mail-maxsec-plain-text = Zwykły tekst
mail-maxsec-plain-text-hint = Bez renderowania HTML — wiadomość wyświetlana tylko jako zwykły tekst.
