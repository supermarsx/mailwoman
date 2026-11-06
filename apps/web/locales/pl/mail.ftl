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
mail-inbox-filter = Filtr folderu Odebrane
mail-inbox-focused = Priorytetowe
mail-inbox-other = Inne

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
mail-encrypted-note = Ta wiadomość jest zaszyfrowana end-to-end. Odblokuj ją na tym urządzeniu, aby ją przeczytać.
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
mail-compose-options = Opcje wysyłania
mail-compose-request-receipt = Poproś o potwierdzenie przeczytania
mail-compose-request-receipt-hint = Program pocztowy odbiorcy może zapytać odbiorcę, wysłać potwierdzenie bez pytania lub zignorować prośbę.
mail-compose-track-opens = Rejestruj otwarcie tej wiadomości
mail-compose-track-opens-hint = Do wiadomości zostaje dodany obraz 1×1 wczytywany z tego serwera. Wiele programów pocztowych blokuje obrazy lub wczytuje je z wyprzedzeniem, więc zarejestrowane otwarcie nie dowodzi, że wiadomość została przeczytana, a brak zapisu nie dowodzi, że nie została. Odbiorca nie jest o tym informowany.
mail-compose-image-type = Nie można umieścić { $name } w tekście. Użyj obrazu PNG, JPEG, GIF lub WebP.
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
mail-send-tracking-failed = Nie można teraz rejestrować otwarć, więc nic nie zostało wysłane. Odznacz „Rejestruj otwarcie tej wiadomości”, aby wysłać bez tego.
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
mail-compose-rt-image = Wstaw obraz

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
mail-toast-pinned = Przypięto wiadomość „{ $target }”
mail-toast-unpinned = Odpięto wiadomość „{ $target }”
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
mail-toast-tracking-not-bound = Wiadomość została wysłana, ale jej otwarcia nie będą rejestrowane.
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
mail-feature-off = Wyłączone w Twoich ustawieniach lub przez administratora
mail-send-disabled = Wysyłanie jest wyłączone w Twoich ustawieniach lub przez administratora. Nadal możesz zapisać wersję roboczą.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Wydarzenia
mail-events-upcoming = Tylko nadchodzące
mail-events-filter = Filtr wydarzeń
mail-loading-more = Wczytywanie starszych wiadomości…
mail-event-state-invitation = Zaproszenie
mail-event-state-updated = Zaktualizowano
mail-event-state-cancelled = Anulowano
mail-event-state-reply = Odpowiedź
mail-event-state-counter = Zaproponowano nowy termin
mail-event-state-event = Wydarzenie
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Pokaż więcej akcji
mail-remote-moved = Ta wiadomość została przeniesiona do folderu { $folder } z innego urządzenia lub aplikacji. Nadal czytasz otwartą kopię.
mail-remote-moved-unknown = Ta wiadomość została przeniesiona z innego urządzenia lub aplikacji. Nadal czytasz otwartą kopię.
mail-remote-deleted = Ta wiadomość została usunięta z innego urządzenia lub aplikacji. Możesz czytać tę kopię do czasu jej zamknięcia.
mail-remote-dismiss = Zamknij
mail-reply-draft = Odpowiedź w toku
mail-reply-draft-open = Otwórz wersję roboczą
mail-draft-load-failed = Nie udało się otworzyć wersji roboczej. Spróbuj ponownie.
mail-reply-images-omitted =
    { $count ->
        [one] Nie udało się dołączyć jednego obrazu do odpowiedzi.
        [few] Nie udało się dołączyć { $count } obrazów do odpowiedzi.
        [many] Nie udało się dołączyć { $count } obrazów do odpowiedzi.
       *[other] Nie udało się dołączyć { $count } obrazu do odpowiedzi.
    }
mail-export-forbidden = Administrator wyłączył eksport wiadomości dla tego konta.
mail-export-open-failed = Nie udało się otworzyć opcji eksportu. Zamknij kartę przestrzeni roboczej i spróbuj ponownie.
mail-headers-title = Surowe nagłówki wiadomości
mail-headers-action-help = Wyświetl oryginalne nagłówki wiadomości, w tym szczegóły dostarczenia i uwierzytelniania.
mail-headers-help = Oryginalne nagłówki wiadomości. Zachowano zawinięte wiersze; tutaj widać tylko nagłówki.
mail-headers-loading = Wczytywanie nagłówków wiadomości…
mail-headers-failed = Nie udało się wczytać nagłówków wiadomości. Sprawdź połączenie i spróbuj ponownie.
mail-headers-copy = Kopiuj nagłówki
mail-headers-copied = Skopiowano nagłówki.
mail-headers-copy-failed = Nie udało się skopiować nagłówków. Zaznacz tekst, aby skopiować go ręcznie.
mail-attachment-failed = Nie udało się pobrać tego załącznika. Spróbuj ponownie, gdy połączenie będzie dostępne.
mail-compose-rt-rule = Linia pozioma
mail-compose-signature-new = Nowe wiadomości
mail-compose-signature-replies = Odpowiedzi
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Rozmiar treści
mail-editor-body-size-help = Rozmiar bieżącej treści w UTF-8, w tym znaczniki HTML i osadzone obrazy. Nagłówki wiadomości i osobne załączniki nie są wliczane.
mail-outbox-load-failed = Nie udało się sprawdzić Skrzynki nadawczej. Spróbuj Odświeżyć po sprawdzeniu połączenia.
mail-outbox-unavailable = Skrzynka nadawcza niedostępna. Użyj Odśwież, aby spróbować ponownie.
mail-maxsec-view = Widok: { $mode }
mail-maxsec-menu = Tryb widoku
mail-toast-tab-pinned = Przypięto kartę „{ $target }”
mail-toast-tab-unpinned = Odpięto kartę „{ $target }”
mail-workspace-message = Wiadomość
mail-opened-announcement = Otwarto: { $subject }
mail-workspace-layout = Układ przestrzeni roboczej
mail-workspace-tabs = Karty
mail-workspace-side-by-side = Obok siebie
mail-workspace-stack = Jedno pod drugim
mail-workspace-refresh = Odśwież folder
mail-workspace-load-error = Nie udało się wczytać tej wiadomości. Zamknij kartę i spróbuj ponownie.
mail-resize-folders = Zmień rozmiar paska folderów
mail-resize-list = Zmień rozmiar listy wiadomości
mail-resize-tiles = Zmień rozmiar paneli przestrzeni roboczej
mail-show-folders = Pokaż pasek folderów
mail-hide-folders = Ukryj pasek folderów
mail-compact-folders = Kompaktowe ikony folderów
mail-toggle-list = Pokaż lub ukryj listę wiadomości
mail-toggle-reader = Pokaż lub ukryj okienko czytania
mail-folder-counts = Nieprzeczytane: { $unread }, wiadomości: { $total }
mail-print = Drukuj / Zapisz jako PDF
mail-go-to-top = Na górę
mail-print-help = Wydrukuj tę wiadomość lub wybierz Zapisz jako PDF w oknie drukowania. Obrazy zewnętrzne pozostają zablokowane.
mail-file-details = Szczegóły pliku i odcisk
mail-file-type = Typ pliku
mail-file-extension = Rozszerzenie
mail-file-detected = Wykryta sygnatura
mail-file-unknown = Brak rozpoznanej sygnatury
mail-file-size = Rozmiar
mail-file-size-bytes = { $count } B
mail-file-hash-unavailable = Odcisk niedostępny (wymaga bezpiecznej przeglądarki i pliku do 64 MiB).
mail-file-fingerprint-help = Odcisk identyfikuje dokładnie te bajty. Nie dowodzi, że plik jest bezpieczny ani kto go utworzył.
mail-file-warning-executable = Ten plik może uruchamiać programy lub zmieniać ustawienia systemu. Pobieraj go tylko, jeśli ufasz jego źródłu.
mail-file-warning-macro = Ten dokument może zawierać makra. Nie włączaj makr, dopóki nie zweryfikujesz nadawcy i celu.
mail-file-warning-disguisedName = Nazwa pliku może ukrywać jego prawdziwe rozszerzenie. Sprawdź szczegóły pliku przed pobraniem.
mail-file-warning-typeMismatch = Wykryta sygnatura pliku nie odpowiada zadeklarowanemu typowi. Automatyczny podgląd został zablokowany.
mail-file-warning-activeDocument = Ten format może zawierać aktywną zawartość. Podgląd jest izolowany; otwarcie pobranej kopii może działać inaczej.
mail-file-preview-blocked = Automatyczny podgląd jest wyłączony dla tego typu pliku lub niezgodności typu.
mail-quote-header-from = Od:
mail-quote-header-sent = Wysłano:
mail-quote-header-to = Do:
mail-quote-header-cc = DW:
mail-quote-header-subject = Temat:
mail-spam-score = Analiza spamu
mail-spam-score-loading = Sprawdzanie nagłówków spamu…
mail-spam-score-unavailable = Ocena spamu niedostępna
mail-spam-score-unverified = Zgłoszone przez nagłówki wiadomości, niezweryfikowane niezależnie. Nagłówki podane przez nadawcę mogą być sfałszowane.
mail-spam-score-positive = Nagłówek zgłasza spam.
mail-spam-score-negative = Nagłówek zgłasza wynik inny niż spam.
mail-spam-score-unknown = Do tej oceny nie podano werdyktu.
mail-external-recipients = Odbiorcy zewnętrzni
mail-external-count = Odbiorcy spoza zaufanych domen: { $count }
mail-external-help = Sprawdź odbiorców w polach Do, DW i UDW przed wysłaniem informacji poufnych. Zaufanie wynika z wybranej tożsamości nadawcy i obowiązujących zasad domen.
mail-external-reviewed = Sprawdzono tych odbiorców zewnętrznych.
mail-external-confirm-required = Przed wysłaniem sprawdź i potwierdź odbiorców zewnętrznych.
mail-quick-reply = Szybka odpowiedź
mail-quick-reply-help = Oferuj kompaktową kartę odpowiedzi z już wypełnionymi odbiorcami. Domyślnie wyłączone. Zwykłe kontrole bezpieczeństwa i opcje wysyłania nadal obowiązują.
mail-quick-reply-expand = Edytuj odbiorców i temat
mail-attachment-policy-loading = Sprawdzanie zasad dotyczących załączników…
mail-attachment-policy-unavailable = Nie udało się sprawdzić zasad dotyczących załączników. Spróbuj ponownie przed otwarciem pliku.
mail-attachment-policy-blocked = Administrator zablokował ten załącznik lub jego pobranie w oryginale.
mail-attachment-policy-warning = Administrator zaleca ostrożność przy tym załączniku. Kontynuuj tylko, jeśli ufasz jego źródłu.
mail-attachment-policy-acknowledge = Rozumiem — kontynuuj
mail-attachment-policy-no-preview = Administrator wyłączył podglądy załączników. Pobieranie oryginałów może nadal być dostępne.
mail-time-sent = Wysłano
mail-time-received = Odebrano
mail-time-unknown = Nie podano
mail-time-comparison = Czas w strefie { $zone }
mail-time-privacy = Strefa porównania pochodzi z Twojego wyboru. Znaczniki czasu wiadomości nie wskazują niczyjej lokalizacji.
mail-time-event-comparison = Porównanie czasu wydarzenia
mail-time-ambiguous = Ta godzina lokalna występuje dwa razy przy zmianie czasu. Pokazano oba możliwe momenty; przed zapisaniem sprawdź zamierzone przesunięcie wydarzenia.
mail-bulk-add-label = Dodaj etykietę
mail-select-messages = Zaznacz wiadomości
mail-bulk-remove-label = Usuń etykietę
mail-mark-spam = Oznacz jako spam
mail-bulk-forward = Przekaż zaznaczone wiadomości jako załączniki
mail-bulk-download = Pobierz zaznaczone wiadomości jako mbox
mail-bulk-forward-subject = Przekazane wiadomości ({ $count })
mail-bulk-transfer-failed = Nie udało się przygotować zaznaczonych wiadomości. Zaznacz do 100 wiadomości o łącznym rozmiarze najwyżej 32 MiB i sprawdź uprawnienia eksportu konta.
mail-toast-labels-added = Dodano etykietę { $label } do wiadomości: { $count }
mail-toast-labels-removed = Usunięto etykietę { $label } z wiadomości: { $count }
mail-action-help-reply = Odpowiedz nadawcy w nowej karcie pisania.
mail-action-help-reply-all = Odpowiedz nadawcy i pozostałym odbiorcom w nowej karcie pisania.
mail-action-help-forward = Sprawdź i przekaż tę wiadomość wybranym odbiorcom.
mail-action-help-pin = Przypnij tę wiadomość dla łatwego dostępu albo usuń jej przypięcie.
mail-action-help-read = Zmień, czy ta wiadomość jest oznaczona jako przeczytana. Zmianę można cofnąć.
mail-action-help-flag = Ustaw lub usuń flagę wiadomości do dalszej obsługi.
mail-action-help-archive = Przenieś tę wiadomość do skonfigurowanego folderu archiwum.
mail-action-help-delete = Przenieś tę wiadomość do Kosza. Przeniesienie można cofnąć.
mail-action-help-spam = Oznacz tę wiadomość jako niechcianą i przenieś ją do skonfigurowanego folderu spamu.
mail-action-help-export = Otwórz stronę eksportu, aby pobrać tę wiadomość w wybranym formacie.
mail-action-help-sweep = Sprawdź akcje dla innych wiadomości od tego nadawcy przed ich zastosowaniem.
mail-empty-trash = Opróżnij Kosz
mail-trash-help = Sprawdź i trwale usuń wiadomości znajdujące się w Koszu. Tej operacji nie można cofnąć.
mail-trash-preparing = Przygotowywanie migawki wiadomości w Koszu…
mail-trash-prepare-failed = Nie udało się przygotować podglądu Kosza. Sprawdź połączenie i otwórz go ponownie. Żadne wiadomości nie zostały usunięte.
mail-trash-confirm = Trwale usunąć z Kosza wiadomości: { $count }? Tej operacji nie można cofnąć. Wiadomości, które nadeszły po tym podglądzie, są pomijane; wiadomości przeniesione z Kosza są pomijane.
mail-trash-limit = Ta operacja obejmuje do { $count } wiadomości. W Koszu pozostają kolejne; sprawdź następną operację, aby je usunąć.
mail-trash-permanent = Usuń trwale
mail-trash-stop = Zatrzymaj po bieżącym żądaniu
mail-trash-progress = Usunięto: { $deleted }; pominięto wiadomości, których nie ma już w Koszu: { $skipped }; w tym podglądzie pozostało: { $remaining }.
mail-trash-stopped = Usuwanie zatrzymane. Potwierdzone usunięcia: { $deleted }; pominięte: { $skipped }; pozostałe: { $remaining }; wynik nieznany: { $uncertain }. Sprawdź Kosz przed rozpoczęciem kolejnej operacji.
mail-links-title = Linki w wiadomości
mail-links-review = Sprawdź linki ({ $count })
mail-links-help = Linki w wiadomości otwierają się dopiero po sprawdzeniu tutaj ich celu. Żaden cel nie jest pobierany automatycznie.
mail-links-inert = Użyj opcji Sprawdź linki nad wiadomością, aby obejrzeć i otworzyć ten cel.
mail-links-limit = Wyświetlono pierwsze 100 różnych obsługiwanych celów. Pozostałe linki w wiadomości są nieaktywne.
mail-links-destination = Cel do otwarcia
mail-links-external = Ten cel pochodzi od nadawcy wiadomości. Jego otwarcie opuszcza Mailwoman; sprawdź adres przed kontynuowaniem.
mail-links-open = Otwórz w nowej karcie
mail-links-compose = Sprawdź nową wiadomość
mail-links-warning-http = Ten adres używa niezaszyfrowanego HTTP.
mail-links-warning-address = Ten cel używa adresu IP zamiast nazwy domeny.
mail-links-warning-international = Ta domena zawiera nazwę międzynarodową. Uważnie sprawdź jej pisownię ASCII.
mail-links-warning-mismatch = Widoczny adres internetowy wskazuje inny host niż rzeczywisty cel.
mail-links-warning-direction = Tekst linku zawiera znaki zmieniające kierunek, które mogą ukrywać jego adres.
mail-outbox-uncertain = Stan dostarczenia nieznany
mail-outbox-uncertain-note = Wysyłanie się rozpoczęło, ale nie udało się potwierdzić dostarczenia. Ta wiadomość nie zostanie wysłana ponownie automatycznie. Sprawdź folder Wysłane lub potwierdź u odbiorcy przed ponownym wysłaniem.
mail-conversation-title = Zobacz konwersację
mail-conversation-help = Przeglądaj dostępne wiadomości konwersacji tego konta ze wszystkich folderów. Otwarcie wiadomości nie zmienia wybranego folderu.
mail-conversation-count = Wyświetlono { $loaded } z { $total } wiadomości konwersacji.
mail-conversation-other-folder = Inny folder
mail-conversation-loading = Wczytywanie konwersacji…
mail-conversation-missing = Niedostępne wiadomości: { $count }.
mail-conversation-error = Nie udało się wczytać konwersacji. Spróbuj ponownie.
mail-conversation-more = Wczytaj więcej wiadomości
mail-conversation-limit = Ten widok jest ograniczony do wiadomości: { $count }.
mail-compose-request-delivery = Żądaj powiadomień o dostarczeniu
mail-compose-request-delivery-hint = Poproś serwery poczty odbiorców o raport o dostarczeniu, niepowodzeniu lub opóźnieniu. Nie jest to prośba o potwierdzenie przeczytania ani jego dowód. Zwracane raporty zawierają nagłówki, a nie pełną treść.
mail-compose-delivery-unavailable = Powiadomienia o dostarczeniu są niedostępne lub wyłączone przez administratora dla tego konta. Wyraźnie zażądane powiadomienie nie zostanie po cichu usunięte.
mail-compose-account-changed = Ta wiadomość należy do innego logowania. Zaloguj się na właściwe konto i ponownie otwórz wersję roboczą.
mail-context-open-message = Otwórz wiadomość
mail-context-open-folder = Otwórz folder
mail-context-download = Pobierz wiadomość
mail-context-refresh = Odśwież folder
mail-context-folder-settings = Ustawienia folderu
mail-context-failed = Nie udało się wykonać akcji. Spróbuj ponownie.
mail-sharing-unavailable = Udostępnianie folderu jest niedostępne albo nie masz już uprawnień do zarządzania nim.
mail-trash-cancelling = Zatrzymywanie po bieżącym żądaniu. Potwierdzone usunięcia: { $deleted }; pominięte: { $skipped }; pozostałe: { $remaining }. Przyjętych już usunięć nie można cofnąć.
mail-trash-already-running = Usuwanie z Kosza już trwa. Użyj jego powiadomienia o postępie, aby zatrzymać kolejne partie.
mail-context-search-sender = Szukaj tego nadawcy
mail-context-search-subject = Szukaj tego samego tematu
mail-drag-too-many = Przeciągaj naraz do { $count } wiadomości. Przed przeniesieniem zaznacz mniejszą grupę.
mail-subtab-reorder-help = Przeciągnij, aby zmienić kolejność kart, albo naciśnij Ctrl+Shift+Strzałka w lewo lub w prawo na aktywnej karcie.
mail-compose-ribbon = Wstążka edytora
mail-compose-ribbon-format = Formatowanie tekstu
mail-compose-ribbon-insert = Wstaw
mail-compose-font = Czcionka
mail-compose-font-size = Rozmiar czcionki
mail-compose-font-inherit = Domyślna
mail-compose-undo = Cofnij
mail-compose-redo = Ponów
mail-compose-undo-tip = Cofnij ({ $shortcut })
mail-compose-redo-tip = Ponów ({ $shortcut })
mail-workspace-limit = Zamknij kartę przed otwarciem kolejnej. Można zachować do 32 kart.
mail-compose-private-storage-unavailable = Lokalne odzyskiwanie wersji roboczej jest niedostępne w tej sesji. Pozostaw ten edytor otwarty, dopóki nie wyślesz lub nie skopiujesz wiadomości.

# -- Machine translation (2026-11): added to match en ---------------------
mail-offline-storage-unavailable = Zmiany offline są niedostępne, dopóki ta sesja nie zweryfikuje uprawnień do magazynu. Połącz się ponownie i zaloguj jeszcze raz.
mail-draft-status-idle = Jeszcze nie zapisano na koncie
mail-draft-status-pending = Niezapisane zmiany
mail-draft-status-saving = Zapisywanie w Wersjach roboczych…
mail-draft-status-saved = Zapisano w Wersjach roboczych konta
mail-draft-status-error = Nie zapisano wersji roboczej. Lokalna kopia odzyskiwania pozostaje dostępna.
mail-draft-status-uncertain = Brak potwierdzenia zapisu. Sprawdź Wersje robocze, zanim utworzysz kolejną kopię; automatyczne zapisywanie jest wstrzymane.
mail-draft-status-conflict = Ta wersja robocza zmieniła się na innym urządzeniu. Otwórz ponownie jej kopię z konta, zanim przejdziesz dalej.
mail-draft-status-unsupported = To konto nie może obecnie potwierdzać zapisów w folderze Wersje robocze.
mail-draft-protected = Zaszyfrowana lub odszyfrowana treść cytowana nie jest zapisywana jako zwykły tekst. Wcześniejsza wersja robocza na koncie pozostaje, dopóki jej wyraźnie nie odrzucisz.
mail-draft-save-now = Zapisz w Wersjach roboczych
mail-draft-save-failed = Nie udało się zapisać tej wersji roboczej na koncie. Sprawdź edytor przed jego zamknięciem.
mail-draft-uncertain = Wersja robocza mogła zostać zapisana. Automatyczne ponowienia są wstrzymane; sprawdź folder Wersje robocze.
mail-draft-conflict = Inne urządzenie zmieniło tę wersję roboczą. Twoje zmiany nie nadpisały jej kopii.
mail-draft-cleanup-pending = Nowa wersja robocza została zapisana, ale nie udało się usunąć jej poprzedniej kopii. Przejrzyj Wersje robocze przed ponownym zapisem.
mail-draft-account-list = Wersje robocze konta
mail-draft-loading = Wczytywanie wersji roboczych konta…
mail-draft-close-recovery = Zamknij i zachowaj tylko lokalną kopię odzyskiwania
mail-draft-discard-account = Odrzuć
mail-draft-discard-pending = Nie udało się potwierdzić usunięcia wersji roboczej. Jej rekord na koncie pozostaje; ponów Odrzuć po sprawdzeniu folderu Wersje robocze.
mail-draft-limit = To konto ma już 100 zapisanych wersji roboczych. Odrzuć niepotrzebną wersję roboczą konta przed zapisaniem nowej.
mail-draft-quota-help = Przechowywanych jest do 100 wersji roboczych konta. Odrzucenie usuwa potwierdzoną kopię na serwerze, zanim zwolni jej miejsce.
mail-draft-discard-confirm = Potwierdź usunięcie wersji roboczej
mail-draft-discard-confirm-detail = Trwale usunąć „{ $subject }” z Wersji roboczych konta? Zamknięcie edytora zachowuje wersję roboczą.
mail-draft-discard-title = Odrzucić tę wersję roboczą?
mail-draft-keep-editing = Edytuj dalej
mail-draft-discard-action = Odrzuć
mail-draft-close-title = Odrzucić niezapisane zmiany?
mail-draft-close-protected = Ta wiadomość zawiera zaszyfrowaną lub odszyfrowaną treść, więc nie jest zapisywana jako wersja robocza. Zamknięcie odrzuca te zmiany. Wcześniejsza wersja robocza konta, jeśli istnieje, pozostaje w Wersjach roboczych.
mail-draft-close-unstored = W tej sesji nie można zapisywać wersji roboczych, więc zamknięcie tej wiadomości ją odrzuca.
mail-workspace-view-mode = Układ widoków
mail-workspace-tabs-help = Pokaż jeden aktywny widok, zachowując pozostałe karty otwarte.
mail-workspace-side-by-side-help = Pokaż dwa wybrane widoki obok siebie. Przeciągnij kartę na dowolny widok, aby go zastąpić.
mail-workspace-stack-help = Pokaż dwa wybrane widoki jeden nad drugim. Przeciągnij kartę na dowolny widok, aby go zastąpić.
mail-workspace-masonry-side-by-side = Mozaika obok siebie
mail-workspace-masonry-side-by-side-help = Ułóż wszystkie otwarte widoki w przesuniętych kolumnach. Małe ekrany pokazują aktywną kartę.
mail-workspace-masonry-stack = Mozaika w pionie
mail-workspace-masonry-stack-help = Ułóż wszystkie otwarte widoki w przesuniętych wierszach. Małe ekrany pokazują aktywną kartę.
mail-workspace-view-first = Pierwszy widok
mail-workspace-view-second = Drugi widok
mail-workspace-view-empty = Przeciągnij tutaj otwartą kartę, aby wybrać ten widok.
mail-popup-blocked = Zezwól Mailwoman na wyskakujące okna, aby odłączyć tę kartę. Karta pozostaje tutaj.
mail-popup-unavailable = Nie udało się wczytać odłączonego okna. Karta i niezapisane zmiany pozostają w głównym oknie. Spróbuj ponownie po odświeżeniu aplikacji.
mail-popup-reattach = Dołącz z powrotem do głównego okna
mail-popup-action-reattached = Wrócono do głównego okna, aby dokończyć tę akcję.
mail-popup-detached = Ta karta jest otwarta w osobnym oknie. Zamknięcie tego okna przywraca ją tutaj.
mail-compose-from-custom = Inny autoryzowany adres…
mail-compose-from-alias = Adres nadawcy tej wiadomości
mail-compose-from-alias-help = Użyj aliasu, z którego możesz wysyłać. Zmienia tylko tę wersję roboczą; autoryzacja jest sprawdzana przed wysłaniem.
mail-compose-from-refused = Ten adres nie jest autoryzowany do wysyłania z tego konta.
mail-compose-from-unavailable = Nie udało się zweryfikować adresu nadawcy. Wiadomość nie została wysłana.
mail-compose-from-invalid = Wpisz jeden prawidłowy adres e-mail nadawcy.
mail-compose-from-toggle = Pokaż lub ukryj nadawcę tej wiadomości
mail-compose-ribbon-security = Zabezpieczenia
mail-compose-ribbon-delivery = Dostarczanie
mail-compose-ribbon-recipients = Odbiorcy
mail-compose-ribbon-message-type = Typ wiadomości
mail-compose-recall-help = Anuluj wiadomości wciąż wstrzymane do cofnięcia lub zaplanowanego dostarczenia. Wiadomości już dostarczonych nie można wycofać.
mail-compose-recall-loading = Sprawdzanie wiadomości w kolejce…
mail-compose-recall-empty = Brak wiadomości w kolejce do anulowania.
mail-compose-recall-failed = Nie udało się potwierdzić stanu kolejki. Zamknij i otwórz ten panel ponownie, aby go odświeżyć przed kolejną akcją.
mail-compose-importance = Ważność
mail-compose-importance-high = Wysoka
mail-compose-importance-normal = Normalna
mail-compose-importance-low = Niska
mail-insert-account-title = Wstaw element konta
mail-insert-account-help = Dołącz eksport kalendarza, wizytówkę kontaktu lub oryginalną wiadomość z tego konta.
mail-insert-account-close = Zamknij wybór elementu konta
mail-insert-account-kind = Typ elementu
mail-insert-account-event = Wydarzenie kalendarza (.ics)
mail-insert-account-calendar = Kalendarz (.ics)
mail-insert-account-contact = Wizytówka kontaktu (.vcf)
mail-insert-account-message = Oryginalna wiadomość (.eml)
mail-insert-account-search = Filtruj wyświetlone elementy
mail-insert-account-selection = Wybierz element
mail-insert-account-scope = Wyświetlane są tylko elementy bieżącego konta. Oryginalne wiadomości podlegają zasadom eksportu serwera.
mail-insert-account-limited = Wyświetlono do 100 elementów. Filtrowanie przeszukuje tylko tę listę.
mail-insert-account-unavailable = Brak obsługiwanych eksportów konta.
mail-insert-account-empty = Brak pasujących elementów.
mail-insert-account-attach = Dołącz wybrany element
mail-insert-account-loading = Wczytywanie elementu konta…
mail-insert-account-error = Nie udało się dołączyć elementu. Sprawdź dostęp i zasady eksportu, a potem spróbuj ponownie. Duże kalendarze mogą wymagać eksportu pojedynczych wydarzeń.
mail-compose-html-source = Źródło HTML
mail-editor-clear-format = Wyczyść formatowanie
mail-compose-format-html = HTML
mail-compose-html-visual = Edytor wizualny
mail-compose-html-help = Edytuj HTML bezpośrednio. Zapisywane i wysyłane jest tylko obsługiwane formatowanie poczty; skrypty, obrazy zewnętrzne i niebezpieczny kod są usuwane.
mail-compose-html-processing = Przygotowywanie bezpiecznego HTML…
mail-compose-html-failed = Nie udało się przygotować HTML. Edytuj go lub spróbuj ponownie przed zapisaniem albo wysłaniem.
mail-compose-check-names = Sprawdź nazwy
mail-compose-check-names-help = Dopasuj dokładne nazwy z wczytanych kontaktów i sprawdź format adresów. Nie weryfikuje to dostarczania do skrzynki.
mail-compose-names-unresolved = Wybierz pełny adres e-mail dla tych nierozpoznanych lub niejednoznacznych nazw: { $names }
mail-compose-names-checked = Sprawdzono format odbiorców; rozpoznane nazwy kontaktów: { $count }. Dostarczanie do skrzynki nie zostało zweryfikowane.
mail-compose-ribbon-table = Tabela
mail-compose-group-clipboard = Schowek
mail-compose-group-paragraph = Akapit
mail-compose-group-editing = Edytowanie
mail-compose-group-view = Widok
mail-compose-dark-background = Ciemne tło
mail-compose-dark-background-help = Pokaż wiadomość na ciemnym tle podczas pisania. Zmienia się tylko Twój ekran: wiadomość jest wysyłana bez koloru tła.
mail-compose-group-assist = Asystent pisania
mail-editor-counts = Statystyki edytora
mail-editor-words = { $count ->
    [one] { $count } słowo
    [few] { $count } słowa
    [many] { $count } słów
   *[other] { $count } słowa
}
mail-editor-paragraphs = { $count ->
    [one] { $count } akapit
    [few] { $count } akapity
    [many] { $count } akapitów
   *[other] { $count } akapitu
}
mail-editor-characters = { $count ->
    [one] { $count } znak
    [few] { $count } znaki
    [many] { $count } znaków
   *[other] { $count } znaku
}
mail-editor-selection-required = Zaznacz odpowiedni tekst lub komórki tabeli, zanim użyjesz tej akcji.
mail-editor-clipboard-unavailable = Dostęp do schowka był niedostępny. Użyj skrótu Kopiuj, Wytnij lub Wklej w przeglądarce.
mail-editor-selection-changed = Zaznaczenie się zmieniło. Zaznacz właściwy tekst i spróbuj ponownie.
mail-editor-clipboard = Schowek i formatowanie
mail-editor-copy = Kopiuj
mail-editor-cut = Wytnij
mail-editor-paste = Wklej
mail-editor-copy-format = Kopiuj formatowanie
mail-editor-apply-format = Zastosuj formatowanie
mail-editor-style = Styl akapitu
mail-editor-text-color = Kolor czcionki
mail-editor-clear-color = Kolor domyślny
mail-editor-highlight = Wyróżnienie
mail-editor-clear-highlight = Usuń wyróżnienie
mail-editor-paragraph = Opcje akapitu
mail-editor-alignment = Wyrównanie
mail-editor-line-height = Wysokość wiersza
mail-editor-space-before = Odstęp przed
mail-editor-space-after = Odstęp po
mail-editor-indent = Wcięcie
mail-editor-shading = Cieniowanie akapitu
mail-editor-clear-shading = Usuń cieniowanie
mail-editor-case = Zmień wielkość liter
mail-editor-sort = Sortuj zaznaczone akapity
mail-editor-sort-alpha = Alfabetycznie
mail-editor-sort-numeric = Numerycznie
mail-editor-table-rows = Wiersze tabeli
mail-editor-table-columns = Kolumny tabeli
mail-editor-table-insert = Wstaw tabelę
mail-editor-table-limits = Do 20 wierszy, 10 kolumn i 200 komórek na tabelę; najwyżej 10 tabel. Tabele zagnieżdżone nie są obsługiwane.
mail-editor-table-row-before = Wiersz powyżej
mail-editor-table-row-after = Wiersz poniżej
mail-editor-table-row-delete = Usuń wiersz
mail-editor-table-column-before = Kolumna przed
mail-editor-table-column-after = Kolumna po
mail-editor-table-column-delete = Usuń kolumnę
mail-editor-table-merge = Scal komórki
mail-editor-table-split = Podziel komórkę
mail-editor-table-header = Przełącz wiersz nagłówka
mail-editor-table-delete = Usuń tabelę
mail-editor-table-shading = Cieniowanie komórki
mail-editor-find-replace = Znajdź i zamień
mail-editor-find = Znajdź tekst
mail-editor-replacement = Zamień na
mail-editor-match-case = Uwzględnij wielkość liter
mail-editor-previous = Poprzednie dopasowanie
mail-editor-next = Następne dopasowanie
mail-editor-replace = Zamień zaznaczone dopasowanie
mail-editor-replace-all = Zamień wszystkie dopasowania
mail-editor-matches = Dopasowania: { $count } (maksymalnie 1000)
mail-editor-select-all = Zaznacz wszystko
mail-editor-clear-selection = Wyczyść zaznaczenie
mail-editor-history = Historia cofania
mail-editor-undo-groups = Cofnij ostatnie grupy zmian: { $count }
mail-editor-zoom = Powiększenie edytora
mail-editor-markers = Znaczniki akapitów
mail-editor-style-normal = Normalny
mail-editor-style-no-spacing = Bez odstępów
mail-editor-style-title = Tytuł
mail-editor-style-subtitle = Podtytuł
mail-editor-style-heading1 = Nagłówek 1
mail-editor-style-heading2 = Nagłówek 2
mail-editor-style-heading3 = Nagłówek 3
mail-editor-style-heading4 = Nagłówek 4
mail-editor-style-heading5 = Nagłówek 5
mail-editor-style-heading6 = Nagłówek 6
mail-editor-align-left = Do lewej
mail-editor-align-center = Do środka
mail-editor-align-right = Do prawej
mail-editor-align-justify = Wyjustuj
mail-editor-case-lower = małe litery
mail-editor-case-upper = WIELKIE LITERY
mail-editor-case-title = Każdy Wyraz Wielką Literą
mail-editor-case-sentence = Wielka litera na początku zdania
mail-editor-action-history = Historia: { $action }
mail-editor-history-empty = Brak dostępnych akcji
mail-editor-redo-groups = Ponów zmiany: { $count }
mail-compose-identities-failed = Nie udało się wczytać tożsamości nadawcy. Zamknij i otwórz ponownie tę wersję roboczą, aby spróbować jeszcze raz.
mail-folders-loading = Wczytywanie folderów…
mail-initial-sync-error = Nie udało się jeszcze zsynchronizować wiadomości.
mail-initial-sync-error-help = Połączenie z serwerem poczty zostało przerwane. Spróbuj ponownie lub poproś administratora o sprawdzenie dzienników połączeń, jeśli problem się powtarza.
mail-links-trusted = Zaufana domena: otwiera się bez sprawdzania
mail-links-trust-domain = Ufaj domenie { $domain }
mail-links-trust-failed = Nie udało się zapisać zaufanej domeny. Spróbuj ponownie.
mail-focus-always-focused = Zawsze przenoś do Priorytetowych
mail-focus-always-other = Zawsze przenoś do Pozostałych
mail-focus-moved-focused = Wiadomości od { $sender } trafiają teraz do Priorytetowych.
mail-focus-moved-other = Wiadomości od { $sender } trafiają teraz do Pozostałych.
mail-focus-save-failed = Nie udało się zapisać wyboru skrzynki priorytetowej dla tego nadawcy. Spróbuj ponownie.
mail-context-search-domain = Szukaj wiadomości z tej domeny

# -- Machine translation (2026-11): added to match en ---------------------
mail-loading-mailbox = Wczytywanie: { $name }…
mail-user-folders = Foldery użytkownika
mail-context-hide-folder = Ukryj na pasku bocznym
mail-folder-fallback = folder
mail-toast-moving = { $count ->
    [one] Przenoszenie jednej wiadomości do folderu { $folder }…
    [few] Przenoszenie { $count } wiadomości do folderu { $folder }…
    [many] Przenoszenie { $count } wiadomości do folderu { $folder }…
   *[other] Przenoszenie { $count } wiadomości do folderu { $folder }…
}
mail-toast-move-queueing = { $count ->
    [one] Kolejkowanie jednej wiadomości do przeniesienia do folderu { $folder }…
    [few] Kolejkowanie { $count } wiadomości do przeniesienia do folderu { $folder }…
    [many] Kolejkowanie { $count } wiadomości do przeniesienia do folderu { $folder }…
   *[other] Kolejkowanie { $count } wiadomości do przeniesienia do folderu { $folder }…
}
mail-toast-move-queued = { $count ->
    [one] W kolejce jedna wiadomość do przeniesienia do folderu { $folder }
    [few] W kolejce { $count } wiadomości do przeniesienia do folderu { $folder }
    [many] W kolejce { $count } wiadomości do przeniesienia do folderu { $folder }
   *[other] W kolejce { $count } wiadomości do przeniesienia do folderu { $folder }
}
mail-toast-move-partial = Przeniesiono { $count } z { $total } wiadomości do folderu { $folder }; nie udało się przenieść: { $failed }
mail-send-copy-folder-unavailable = Wybrany folder kopii wysłanych jest niedostępny. Wybierz inny folder przed wysłaniem.
mail-send-copy-failed = Wiadomość została przyjęta, ale nie udało się zapisać jej kopii w wysłanych. Nie wysyłaj jej ponownie.
mail-compose-attach-public-key = Dołącz mój klucz publiczny lub certyfikat
mail-compose-include-hashes = Dołącz skróty SHA-256 załączonych plików
mail-compose-hashes-pending = Obliczanie skrótów załączników. Wyślij, gdy będzie gotowe.
mail-compose-hashes-failed = Nie udało się zweryfikować bajtów załączników. Wyłącz skróty lub spróbuj ponownie, włączając je jeszcze raz.
mail-compose-crypto-pending = Szyfrowanie jest aktualizowane. Poczekaj przed wysłaniem.
mail-compose-public-key-unavailable = Żaden aktualny klucz publiczny ani certyfikat nie pasuje do adresu nadawcy.
mail-compose-sent-copy = Zapisz wysłaną wiadomość w
mail-compose-sent-copy-default = Domyślny folder konta
folder-create = Utwórz folder
folder-name = Nazwa folderu
folder-parent = Folder nadrzędny
folder-top-level = Najwyższy poziom
folder-created = Folder utworzony.
folder-create-unavailable = Tworzenie folderów jest niedostępne dla tego konta lub folderu nadrzędnego.
folder-create-invalid = Wybierz prawidłową, niepowtarzalną nazwę folderu bez separatorów hierarchii i znaków sterujących.
folder-create-failed = Nie udało się utworzyć folderu. Odśwież i spróbuj ponownie.
folder-cleanup = Wyczyść folder…
folder-cleanup-title = Wyczyść folder { $name }
folder-cleanup-help = Znajdź wiadomości starsze niż wybrany wiek. Sprawdź liczbę przed usunięciem. Podfoldery są pomijane.
folder-cleanup-age = Starsze niż
folder-age-days = Dni
folder-age-weeks = Tygodnie
folder-age-months = Miesiące
folder-age-years = Lata
folder-cleanup-preview = Podgląd czyszczenia
folder-cleanup-confirm = Usuń pasujące wiadomości
folder-cleanup-permanent = Trwale usuń pasujące wiadomości: { $count }. Tej operacji nie można cofnąć.
folder-cleanup-trash = Przenieś pasujące wiadomości do Kosza: { $count }.
folder-cleanup-before = Odebrane przed { $date }
folder-cleanup-done = Wyczyszczono wiadomości: { $count }.
mail-from = Nadawca
mail-subject = Temat
mail-date = Data
mail-cancel = Anuluj
mail-bulk-selected = Zaznaczone konwersacje: { $count }
mail-bulk-select-folder = Zaznacz wszystko w folderze { $folder }
mail-list-order = Kolejność wiadomości
mail-list-order-newest = Najnowsze najpierw
mail-list-order-oldest = Najstarsze najpierw
mail-list-order-sender = Nadawca
mail-list-order-subject = Temat
mail-list-order-error = Nie udało się zapisać kolejności wiadomości.
mail-bulk-selection-load-failed = Nie udało się zaznaczyć całego folderu. Sprawdź połączenie i spróbuj ponownie.
mail-privacy-bar = Prywatność wiadomości
mail-privacy-trackers-allow = Zezwól na śledzenie w tej wiadomości
mail-privacy-trackers-block = Ponownie blokuj śledzenie
mail-privacy-images-loading = Pokazano obrazów: { $loaded }/{ $total } · wczytywanie: { $pending }
mail-privacy-images-failed = Pokazano obrazów: { $loaded }/{ $total } · niepowodzenia: { $failed }
mail-privacy-images-loaded = Pokazano obrazów: { $loaded }/{ $total }
mail-privacy-images-loading-failed = Pokazano obrazów: { $loaded }/{ $total } · wczytywanie: { $pending } · niepowodzenia: { $failed }
mail-reader-folder-title = Folder: { $folder }
mail-reader-move-to = Przenieś do…
mail-action-help-move = Przenieś tę wiadomość do innego folderu.
folder-ops-new-subfolder = Nowy podfolder
folder-ops-mark-read = Oznacz wszystko jako przeczytane
folder-ops-this-folder = Ten folder
folder-ops-with-subfolders = Uwzględnij podfoldery
folder-ops-no-unread = Brak nieprzeczytanych wiadomości.
folder-ops-read-unavailable = Nie możesz zmieniać wiadomości w tym folderze.
folder-ops-more = Więcej
folder-ops-system-folder = Folderu systemowego nie można zmienić w ten sposób.
folder-ops-sort-subfolders = Sortuj podfoldery według
folder-ops-sort-folders = Sortuj foldery według
folder-ops-sort-name = Nazwy
folder-ops-sort-unread = Liczby nieprzeczytanych
folder-ops-sort-total = Łącznej liczby
folder-ops-sort-date = Ostatniej aktywności
folder-ops-sort-custom = Własna kolejność
folder-ops-sort-asc = Rosnąco
folder-ops-sort-desc = Malejąco
folder-ops-delete-with-subfolders = Usuń folder i podfoldery
folder-ops-delete-unavailable = Tego folderu nie można usunąć.
folder-ops-delete-tree-body = Spowoduje to trwałe usunięcie folderów ({ $folders }) i znajdujących się w nich wiadomości ({ $count }). Tej operacji nie można cofnąć.
folder-ops-copy-all-menu = Kopiuj wszystkie elementy do…
folder-ops-move-all-menu = Przenieś wszystkie elementy do…
folder-ops-move-folder-menu = Przenieś folder do…
folder-ops-move-up = Przesuń w górę
folder-ops-move-down = Przesuń w dół
folder-ops-move-unavailable = Nie możesz przenieść tego folderu ani jego elementów.
folder-ops-move-duplicate = W miejscu docelowym istnieje już folder o tej nazwie.
folder-ops-move-folder-title = Przenieś { $name }
folder-ops-move-folder-body = Folder zostanie przeniesiony wraz z podfolderami i wiadomościami. Wybierz folder, do którego trafi, lub najwyższy poziom.
folder-ops-move-here = Przenieś tutaj
folder-ops-moved-folder = Przeniesiono { $name }.
folder-ops-choose-folder = Wybierz folder docelowy.
folder-ops-destination = Folder docelowy
folder-ops-include-subfolders = Uwzględnij wiadomości z podfolderów: { $count }
folder-ops-copy-all-title = Kopiuj wszystkie elementy folderu { $name }
folder-ops-move-all-title = Przenieś wszystkie elementy folderu { $name }
folder-ops-copy-all-body = Dodaje wiadomości ({ $count }) do folderu docelowego. Pozostają one w obecnym miejscu.
folder-ops-move-all-body = Przenosi wiadomości ({ $count }) do folderu docelowego. Potem można to cofnąć.
folder-ops-copy-all = Kopiuj elementy
folder-ops-move-all = Przenieś elementy
folder-ops-undo = Cofnij
folder-ops-stop = Zatrzymaj
folder-ops-marking = Oznaczanie folderu { $name } jako przeczytanego…
folder-ops-marking-progress = Oznaczanie folderu { $name } jako przeczytanego: { $done } z { $total }
folder-ops-marked = Oznaczono jako przeczytane wiadomości w folderze { $name }: { $count }.
folder-ops-copying = Kopiowanie: { $name }…
folder-ops-copying-progress = Kopiowanie: { $name }: { $done } z { $total }
folder-ops-copied = Skopiowano wiadomości z folderu { $name }: { $count }.
folder-ops-moving = Przenoszenie: { $name }…
folder-ops-moving-progress = Przenoszenie: { $name }: { $done } z { $total }
folder-ops-moved = Przeniesiono wiadomości z folderu { $name }: { $count }.
folder-ops-stopped = Zatrzymano operację na { $name } po { $count } z { $total } wiadomości.
folder-ops-partial = W folderze { $name } wykonano operację dla { $count } z { $total } wiadomości; odrzucono: { $refused }.
folder-ops-failed = Nie udało się ukończyć operacji na { $name }. Odśwież i spróbuj ponownie.
folder-ops-error-forbidden = Nie masz uprawnień do tej zmiany.
folder-ops-error-exists = Folder o tej nazwie już tam istnieje.
folder-ops-error-mailboxHasEmail = Folder nadal zawiera wiadomości.
folder-ops-error-notFound = Folder już nie istnieje.
folder-ops-error-failed = Nie udało się wprowadzić zmiany. Odśwież i spróbuj ponownie.
folder-ops-expand = Pokaż podfoldery folderu { $name }
folder-ops-collapse = Ukryj podfoldery folderu { $name }
mail-context-reply = Odpowiedz
mail-context-reply-all = Odpowiedz wszystkim
mail-context-forward = Przekaż dalej
mail-context-quick-print = Szybkie drukowanie
mail-context-copy = Kopiuj
mail-context-copy-to-folder = Kopiuj wiadomość do folderu…
mail-context-copy-sender = Kopiuj adres nadawcy
mail-context-copy-subject = Kopiuj temat
mail-context-copy-link = Kopiuj link do wiadomości
mail-context-copied = Skopiowano.
mail-context-copy-failed = Nie udało się skopiować do schowka.
mail-context-print-failed = Nie udało się wydrukować wiadomości.
mail-reply-images-title = Obrazy z oryginalnej wiadomości
mail-reply-images-ask = Dołączyć do tej odpowiedzi obrazy z oryginalnej wiadomości?
mail-reply-images-include = Dołącz obrazy
mail-reply-images-not-now = Nie teraz
mail-reply-images-trust = Zawsze ufaj domenie { $domain }
mail-reply-images-loading = Dodawanie obrazów…
mail-reply-images-cancelled = Wczytywanie anulowane. Obrazy nie zostały dodane.
mail-reply-images-failed = Nie udało się dodać obrazów.
mail-reply-images-retry = Wczytaj ponownie
mail-image-cancel = Anuluj
mail-compose-from-why-match = Wysyłanie z tożsamości { $identity }, ponieważ wiadomość wysłano na adres { $address }.
mail-compose-from-why-match-plus = Wysyłanie z tożsamości { $identity }, ponieważ wiadomość wysłano na adres { $address }, będący jego adresem z plusem.
mail-compose-from-why-ambiguous = Adres { $address } pasuje do kilku tożsamości. Wysyłanie z tożsamości { $identity }; wybierz inną:
mail-compose-from-why-default = Wysyłanie z tożsamości domyślnej { $identity }, ponieważ żadna nie pasuje do oryginalnych odbiorców.
mail-compose-from-why-forced-default = Wysyłanie z tożsamości domyślnej { $identity }, zgodnie z Twoimi ustawieniami.
mail-compose-from-why-keep = Wysyłanie z tożsamości { $identity }.
mail-compose-from-change = Zmień
reader-attachments-reason-encrypted = Ta wiadomość jest zaszyfrowana. Jej załączniki pojawią się po odblokowaniu.
reader-attachments-reason-list-missing = Nie udało się wczytać listy załączników tej wiadomości, więc pasek załączników jest ukryty.
reader-attachments-reason-inline-only = Załączniki tej wiadomości ({ $count }) to obrazy wbudowane, widoczne w samej wiadomości.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] Jeden załącznik tej wiadomości nie ma zawartości do pobrania, więc pasek załączników jest ukryty.
    [few] { $count } załączniki tej wiadomości nie mają zawartości do pobrania, więc pasek załączników jest ukryty.
    [many] { $count } załączników tej wiadomości nie ma zawartości do pobrania, więc pasek załączników jest ukryty.
   *[other] { $count } załącznika tej wiadomości nie ma zawartości do pobrania, więc pasek załączników jest ukryty.
  }
reader-attachments-reason-policy-blocked = Zasady dotyczące załączników blokują pobieranie { $count ->
    [one] tego załącznika.
    [few] tych załączników.
    [many] tych załączników.
   *[other] tych załączników.
  }
reader-attachments-reason-policy-unavailable = Nie udało się sprawdzić zasad dotyczących załączników, więc nie można jeszcze ich otwierać.
mail-drafts-popup-count = { $count ->
    [one] 1 wersja robocza
    [few] { $count } wersje robocze
    [many] { $count } wersji roboczych
   *[other] { $count } wersji roboczej
  }
mail-drafts-popup-search = Szukaj według tematu lub odbiorcy
mail-drafts-popup-tab-local = Lokalne wersje robocze
mail-drafts-popup-local-hint = Lokalne wersje robocze są przechowywane tylko w tej przeglądarce.
mail-drafts-popup-badge-account = Konto
mail-drafts-popup-badge-local = Lokalna
mail-drafts-popup-current = Otwarta teraz
mail-drafts-popup-select = Zaznacz
mail-drafts-popup-select-all = Zaznacz wszystko
mail-drafts-popup-clear = Wyczyść
mail-drafts-popup-done = Gotowe
mail-drafts-popup-selected = Zaznaczono: { $count }
mail-drafts-popup-select-row = Zaznacz: { $subject }
mail-drafts-popup-open = Otwórz
mail-drafts-popup-discard = Odrzuć
mail-drafts-popup-discard-selected = Odrzuć zaznaczone
mail-drafts-popup-more = Więcej akcji dla: { $subject }
mail-drafts-popup-retry = Ponów
mail-drafts-popup-load-failed = Nie udało się wczytać wersji roboczych konta.
mail-drafts-popup-empty-account = Brak wersji roboczych konta
mail-drafts-popup-no-match = Brak wersji roboczych pasujących do „{ $query }”
mail-drafts-popup-discarding = Odrzucanie…
mail-drafts-popup-row-failed = Nie udało się odrzucić. Jej kopia na koncie pozostaje.

# -- Machine translation (2026-11): added to match en ---------------------
mail-drafts-popup-confirm-title = { $count ->
    [one] Odrzucić tę wersję roboczą?
    [few] Odrzucić { $count } wersje robocze?
    [many] Odrzucić { $count } wersji roboczych?
   *[other] Odrzucić { $count } wersji roboczej?
  }
mail-drafts-popup-confirm-account = Wersje robocze konta są najpierw usuwane z folderu Wersje robocze na koncie, a ich miejsce jest zwalniane dopiero po potwierdzeniu usunięcia.
mail-drafts-popup-confirm-local = Lokalne wersje robocze są usuwane z tej przeglądarki.
mail-drafts-popup-announce-done = { $count ->
    [one] Odrzucono 1 wersję roboczą
    [few] Odrzucono { $count } wersje robocze
    [many] Odrzucono { $count } wersji roboczych
   *[other] Odrzucono { $count } wersji roboczej
  }
mail-drafts-popup-announce-failed = Odrzucono: { $done }, nie udało się odrzucić: { $failed }
mail-check-title = Sprawdź przed wysłaniem
mail-check-intro = Te elementy mogą wymagać ponownego przejrzenia:
mail-check-invalid = Niektóre adresy odbiorców wyglądają na nieprawidłowe: { $list }
mail-check-subject = Temat jest pusty.
mail-check-attachment = Wiadomość wspomina o załączniku, ale nic nie jest załączone.
mail-check-external = Odbiorcy spoza zaufanych domen: { $list }
mail-check-send-anyway = Wyślij mimo to
mail-check-review = Sprawdź
mail-check-remember = Nie pytaj ponownie o te elementy
mail-check-denied-invalid = Wysyłanie jest zablokowane przez Twoje ustawienia. Popraw nieprawidłowe adresy odbiorców: { $list }
mail-check-denied-subject = Wysyłanie jest zablokowane przez Twoje ustawienia. Dodaj temat.
mail-check-denied-attachment = Wysyłanie jest zablokowane przez Twoje ustawienia. Dołącz wspomniany plik lub zmień treść wiadomości.
mail-check-denied-external = Wysyłanie jest zablokowane przez Twoje ustawienia. Spoza zaufanych domen: { $list }
mail-alerts-label = Problemy z wiadomością
mail-alert-invalid = Nieprawidłowy adres: { $address }
mail-alert-subject = Brak tematu
mail-recipient-chip-remove = Usuń: { $address }
mail-recipient-chips = Odbiorcy w polu { $field }
mail-recipient-invalid-title = To nie jest prawidłowy adres e-mail
mail-recipient-chip-name = { $name }, { $address }. Enter: szczegóły kontaktu, Backspace lub F2: edycja, Delete: usunięcie
mail-recipient-chip-name-invalid = { $address }, nieprawidłowy adres. Enter: edycja, Delete: usunięcie
mail-recipient-added = Dodano: { $name }
mail-recipient-removed = Usunięto: { $name }
mail-recipient-removed-many = { $count ->
    [one] Usunięto { $count } odbiorcę
    [few] Usunięto { $count } odbiorców
    [many] Usunięto { $count } odbiorców
   *[other] Usunięto { $count } odbiorcy
}
mail-recipient-editing = Edytowanie: { $name }
mail-recipient-all-selected = { $count ->
    [one] Zaznaczono { $count } odbiorcę
    [few] Zaznaczono { $count } odbiorców
    [many] Zaznaczono { $count } odbiorców
   *[other] Zaznaczono { $count } odbiorcy
}
mail-external-line = Spoza zaufanych domen: { $count } ({ $domains })
mail-external-dismiss = Zamknij
mail-status-label = Stan edytora
mail-status-more = Kolejne powiadomienia: { $count }
mail-status-more-label = Więcej powiadomień
mail-status-no-recipients = Brak odbiorców
mail-status-no-attachment = Brak załącznika
mail-status-external = Zewnętrzni: { $count }
mail-status-external-reviewed = Odbiorcy zewnętrzni sprawdzeni
mail-status-external-review = Oznacz jako sprawdzone
mail-status-from-match = Od: { $identity }
mail-status-from-match-plus = Od: { $identity }
mail-status-from-ambiguous = Od: { $identity } (pasuje kilka)
mail-status-from-default = Od: tożsamość domyślna
mail-status-from-forced-default = Od: tożsamość domyślna
mail-status-from-keep = Od: { $identity }
mail-status-held = W kolejce: { $count }
mail-status-storage = Brak lokalnego odzyskiwania
mail-status-protected = Nie zapisano jako zwykły tekst
mail-status-saving = Zapisywanie…
mail-status-saved = Zapisano { $time }
mail-status-saved-full = Zapisano w Wersjach roboczych konta · { $time }
mail-status-not-saved = Wersja robocza niezapisana
mail-status-retry = Ponów
mail-status-conflict = Zmieniono na innym urządzeniu
mail-status-uncertain = Zapis niepotwierdzony
mail-status-close-recovery = Zamknij, zachowaj odzyskiwanie
mail-status-unsupported = Zapis wersji roboczej niepotwierdzony
mail-compose-header = Nagłówek wiadomości
mail-header-label = Nagłówek wiadomości
mail-header-collapse = Zwiń nagłówek wiadomości
mail-header-expand = Rozwiń nagłówek wiadomości: { $summary }
mail-header-from-chip = Od { $address }. Zmień nadawcę
mail-compose-group-timing = Czas wysłania
mail-compose-group-priority = Priorytet
mail-compose-group-receipts = Potwierdzenia i śledzenie
mail-compose-group-keys-files = Klucze i pliki
mail-compose-group-sent-copy = Kopia wysłanych
mail-editor-style-quote = Cytat
mail-editor-style-code = Kod
mail-compose-ribbon-table-layout = Układ tabeli
mail-compose-ribbon-picture = Obraz
mail-editor-table-group-rows = Wiersze
mail-editor-table-group-columns = Kolumny
mail-editor-table-group-cells = Komórki
mail-editor-table-row-height = Wysokość wiersza
mail-editor-table-column-width = Szerokość kolumny
mail-editor-table-size-auto = Automatycznie
mail-editor-table-padding = Wypełnienie komórki
mail-editor-table-padding-compact = Kompaktowe
mail-editor-table-padding-normal = Normalne
mail-editor-table-padding-roomy = Przestronne
mail-editor-table-align-left = Wyrównaj komórkę do lewej
mail-editor-table-align-center = Wyśrodkuj komórkę
mail-editor-table-align-right = Wyrównaj komórkę do prawej
mail-picture-group-size = Rozmiar
mail-picture-group-arrange = Rozmieszczenie
mail-picture-group-text = Tekst
mail-picture-group-style = Styl
mail-picture-group-edit = Edycja
mail-picture-width = Szerokość
mail-picture-height = Wysokość
mail-picture-lock = Zachowaj proporcje
mail-picture-reset-size = Oryginalny rozmiar
mail-picture-align-inline = W linii z tekstem
mail-picture-align-left = Do lewej
mail-picture-align-center = Do środka
mail-picture-align-right = Do prawej
mail-picture-alt = Tekst alternatywny
mail-picture-title = Tytuł
mail-picture-border = Obramowanie
mail-picture-radius = Zaokrąglenie rogów
mail-picture-none = Brak
mail-picture-round = Okrągłe
mail-picture-rotate-ccw = Obróć w lewo
mail-picture-rotate-cw = Obróć w prawo
mail-picture-flip-h = Odbij w poziomie
mail-picture-flip-v = Odbij w pionie
mail-picture-crop = Przytnij
mail-picture-crop-left = Lewa (%)
mail-picture-crop-top = Górna (%)
mail-picture-crop-right = Prawa (%)
mail-picture-crop-bottom = Dolna (%)
mail-picture-crop-apply = Przytnij
mail-picture-replace = Zamień
mail-picture-remove = Usuń
mail-picture-failed-no-image = Najpierw zaznacz obraz.
mail-picture-failed-no-source = Ten obraz nie ma lokalnej kopii do edycji. Zamień go.
mail-picture-failed-failed = Nie udało się zmienić obrazu.
mail-context-download-attachments = Pobierz załączniki
mail-context-download-named = Pobierz { $name }
mail-context-download-all = Pobierz wszystko jako ZIP
mail-context-loading-attachments = Wczytywanie załączników…
mail-context-attachments-failed = Nie udało się wczytać załączników.
mail-context-searching = Szukanie wiadomości…
mail-context-go-reply = Przejdź do odpowiedzi
mail-context-go-forward = Przejdź do przekazania
mail-context-reply-not-found = Nie znaleziono odpowiedzi
mail-context-forward-not-found = Nie znaleziono przekazania
mail-no-recipients = (brak odbiorców)
mail-attachment-downloaded = Pobrano: { $name }
mail-attachment-download-failed = Pobieranie nie powiodło się.
mail-attachments-zip-done = Pobrano załączniki jako ZIP: { $count }.
mail-attachments-zip-capped = Archiwum ZIP zawiera pierwsze załączniki ({ $count }); limit to { $max } plików lub { $size }.
mail-focus-not-found = Nie znaleziono wiadomości w folderze { $folder }
mail-focus-go-to = Przejdź do: { $folder }
mail-nav-views-queues = Widoki i kolejki
mail-badge-unread = { $unread ->
    [one] { $unread } nieprzeczytana wiadomość
    [few] { $unread } nieprzeczytane wiadomości
    [many] { $unread } nieprzeczytanych wiadomości
   *[other] { $unread } nieprzeczytanej wiadomości
  }
mail-badge-total = { $total ->
    [one] { $total } wiadomość w tym folderze
    [few] { $total } wiadomości w tym folderze
    [many] { $total } wiadomości w tym folderze
   *[other] { $total } wiadomości w tym folderze
  }
mail-badge-both = { $unread ->
    [one] { $unread } nieprzeczytana wiadomość
    [few] { $unread } nieprzeczytane wiadomości
    [many] { $unread } nieprzeczytanych wiadomości
   *[other] { $unread } nieprzeczytanej wiadomości
  } z { $total ->
    [one] { $total } w tym folderze
    [few] { $total } w tym folderze
    [many] { $total } w tym folderze
   *[other] { $total } w tym folderze
  }
mail-badge-drafts = { $total ->
    [one] Wersje robocze: { $total } wiadomość
    [few] Wersje robocze: { $total } wiadomości
    [many] Wersje robocze: { $total } wiadomości
   *[other] Wersje robocze: { $total } wiadomości
  }
mail-badge-compact-hint = Foldery inne niż Odebrane używają mniejszego znacznika. Zmienisz to w Ustawieniach, w sekcji Foldery i pasek boczny.
mail-badge-outbox = { $count ->
    [one] Skrzynka nadawcza: { $count } wiadomość czeka na wysłanie
    [few] Skrzynka nadawcza: { $count } wiadomości czekają na wysłanie
    [many] Skrzynka nadawcza: { $count } wiadomości czeka na wysłanie
   *[other] Skrzynka nadawcza: { $count } wiadomości czeka na wysłanie
  }
mail-color-menu = Kolory wiadomości
mail-color-button = Kolory wiadomości: { $mode }
mail-color-use-default = Użyj mojego domyślnego ({ $mode })
mail-color-make-default = Ustaw jako mój domyślny
mail-color-locked = Zablokowane przez administratora
mail-color-confirm-title = Zmienić domyślne kolory wiadomości?
mail-color-confirm-body = Każda wiadomość będzie używać trybu „{ $mode }”, chyba że zmienisz go dla jednej wiadomości.
mail-color-confirm = Ustaw jako domyślny
mail-color-default-failed = Nie udało się zapisać domyślnych kolorów wiadomości.
