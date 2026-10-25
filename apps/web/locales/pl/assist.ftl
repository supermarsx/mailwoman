# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Asystent AI
assist-heading = Asystent AI
assist-transcript-label = Rozmowa z asystentem AI
assist-proposed-action = Proponowana akcja
assist-outbox-note = Spowoduje to umieszczenie wiadomości w Skrzynce nadawczej. Nic nie zostanie wysłane, dopóki tego tam nie potwierdzisz.
assist-review-outbox = Przejrzyj w Skrzynce nadawczej
assist-review = Przejrzyj
assist-empty = Poproś asystenta AI o podsumowanie, szkic lub wyszukanie czegoś. Może proponować akcje, ale zawsze potwierdzasz je samodzielnie.
assist-error = Asystent AI nie mógł teraz odpowiedzieć.
assist-input-label = Napisz do asystenta AI
assist-input-placeholder = Zapytaj asystenta AI…
assist-thinking = Myślę…
assist-ask = Zapytaj

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Pomoc w pisaniu
assist-composer-toolbar = Narzędzia edytora
assist-tool-grammar = Popraw gramatykę
assist-tool-rewrite = Przeredaguj
assist-tool-tone = Dostosuj ton
assist-tool-translate = Przetłumacz
assist-toolarg-tone = Ton
assist-toolarg-translate = Język
assist-tone-neutral = Neutralny
assist-tone-friendly = Przyjazny
assist-tone-formal = Formalny
assist-tone-concise = Zwięzły
assist-tone-assertive = Stanowczy
assist-busy = { $label }…
assist-composer-empty-error = Najpierw coś napisz.
assist-composer-error = Asystent AI nie mógł teraz tego wykonać.
assist-suggested-edit = Sugerowana zmiana
assist-apply-draft = Zastosuj w wersji roboczej
assist-discard = Odrzuć

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Sugerowane etykiety
assist-apply-auto = Stosuj automatycznie
assist-apply-label = Zastosuj { $label }
assist-remove-label = Usuń { $label }
assist-apply = Zastosuj
assist-undo = Cofnij

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Przytrzymaj, aby dyktować
assist-dictate-stop = Zatrzymaj dyktowanie
assist-dictate-hold-text = 🎙 Przytrzymaj, aby dyktować
assist-dictate-listening = ● Słucham…
assist-dictate-endpoint-note = Dźwięk jest transkrybowany przez punkt końcowy Twojego asystenta AI.
assist-dictate-err = Błąd dyktowania.
assist-transcribe-err = Nie udało się przetranskrybować dźwięku.
assist-mic-err = Mikrofon niedostępny.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Wyszukiwanie semantyczne
assist-semantic-note = Tekst zapytania jest wysyłany do punktu końcowego Twojego asystenta AI w celu uszeregowania wyników według znaczenia.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Co może opuścić to urządzenie
assist-left-1 = zaznaczony tekst wiadomości (temat + treść) dla wybranej funkcji
assist-left-2 = host punktu końcowego, do którego został wysłany
assist-left-3 = nigdy: treść szyfrowana end-to-end (domyślnie wykluczona)
assist-left-4 = nigdy: załączniki (domyślnie wykluczone)
assist-left-5 = nigdy: Twoje dane logowania ani inne konta
assist-disclosure-off = Asystent AI jest wyłączony. Żadna treść wiadomości nie opuszcza tego urządzenia.
assist-disclosure-sentence = Gdy używasz narzędzia asystenta AI, zaznaczony tekst wiadomości jest przekazywany przez serwer pośredniczący do { $host }. { $excl } Wysyłanie nigdy nie jest automatyczne — zawsze potwierdzasz, zanim cokolwiek opuści Skrzynkę nadawczą.
assist-disclosure-excl-one = Nigdy nie wysyła { $a }.
assist-disclosure-excl-two = Nigdy nie wysyła { $a } ani { $b }.
assist-disclosure-admin-allowed = Administrator zezwolił na wysyłanie treści szyfrowanych i załączników.
assist-excluded-e2ee = treści szyfrowanej end-to-end
assist-excluded-attachments = załączników

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = Asystent AI
assist-admin-intro = Asystent AI wysyła tekst wiadomości zaznaczony przez użytkowników do skonfigurowanego tutaj punktu końcowego AI. Nie może wysyłać, usuwać ani odbierać poczty. Treść szyfrowana end-to-end i załączniki są wstrzymywane, chyba że zezwolisz na nie poniżej.
assist-admin-load-unreadable = Nie można odczytać zapisanej konfiguracji asystenta AI: jej klucza API nie da się odszyfrować kluczem tego serwera (MW_SERVER_KEY). Została zapisana z innym kluczem lub została zmieniona. Asystent AI jest wyłączony. Wypełnij formularz, łącznie z kluczem API, i zapisz, aby ją zastąpić.
assist-admin-load-error = Nie udało się wczytać konfiguracji asystenta AI. Zapisywanie jest wyłączone, aby nie nadpisać zapisanej konfiguracji.
assist-admin-status-running = Asystent AI działa. Żądania trafiają do { $host }.
assist-admin-status-running-pending = Asystent AI działa z konfiguracją, z którą uruchomiono serwer. Zapisane zmiany zaczną obowiązywać po ponownym uruchomieniu serwera.
assist-admin-status-off = Asystent AI jest wyłączony. Każde żądanie do asystenta AI jest odrzucane.
assist-admin-status-pending = Asystent AI nie działa. Zapisana konfiguracja zacznie obowiązywać po ponownym uruchomieniu serwera.
assist-admin-status-local-program-not-allowed = Asystent AI nie działa. Skonfigurowany program { $program } nie jest wymieniony w MW_ASSIST_LOCAL_PROGRAMS na serwerze, więc serwer go nie uruchomił.
assist-admin-status-no-endpoint = Asystent AI jest włączony, ale nie działa, ponieważ nie skonfigurowano użytecznego punktu końcowego.
assist-admin-stop = Zatrzymaj asystenta AI teraz
assist-admin-stop-note = Zatrzymanie działa natychmiast dla wszystkich użytkowników i jest zachowywane po ponownym uruchomieniu.
assist-admin-resume = Włącz asystenta AI
assist-admin-resume-note = Jeśli serwer uruchomiono z wyłączonym asystentem AI, włączenie zacznie obowiązywać po ponownym uruchomieniu serwera.
assist-admin-kill-error = Nie udało się zmienić stanu włączenia asystenta AI.
assist-admin-endpoint = Punkt końcowy
assist-admin-endpoint-note = Z tym punktem końcowym łączy się serwer, nigdy przeglądarki. Zmiany zaczną obowiązywać po ponownym uruchomieniu serwera.
assist-admin-kind = Typ punktu końcowego
assist-admin-kind-none = Brak
assist-admin-kind-open-ai-compatible = Zgodny z OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Program lokalny
assist-admin-local-program-enable = Aby asystent AI uruchamiał program na tym serwerze, ustaw MW_ASSIST_LOCAL_PROGRAMS na bezwzględną ścieżkę programu w środowisku, w którym uruchamiany jest serwer (kilka ścieżek rozdziel przecinkami), a następnie uruchom serwer ponownie.
assist-admin-local-program-note = Serwer uruchamia ten program przy każdym żądaniu do asystenta AI i przekazuje mu żądanie na standardowe wejście, łącznie z tekstem wiadomości zaznaczonym przez użytkownika. Odpowiedź odczytuje ze standardowego wyjścia programu. Można wybrać tylko programy wymienione w MW_ASSIST_LOCAL_PROGRAMS na serwerze.
assist-admin-local-program = Program
assist-admin-local-program-unlisted = { $program } (niewymieniony)
assist-admin-local-program-unlisted-note = Ten program nie jest wymieniony w MW_ASSIST_LOCAL_PROGRAMS, więc serwer go nie uruchamia i nie zaakceptuje go przy zapisie. Wybierz wymieniony program lub inny typ punktu końcowego.
assist-admin-local-program-required = Wybierz program do uruchomienia.
assist-admin-local-args = Argumenty, po jednym w wierszu
assist-admin-local-args-note = Każdy wiersz jest przekazywany programowi jako jeden argument. Powłoka nie jest używana.
assist-admin-base-url = Bazowy URL
assist-admin-base-url-required = Wpisz bazowy URL punktu końcowego, zaczynający się od http:// lub https://.
assist-admin-api-key = Klucz API
assist-admin-api-key-stored = Klucz jest zapisany. Serwer nie wyświetli go ponownie.
assist-admin-api-key-replace = Zastąp klucz API
assist-admin-api-key-replace-placeholder = Pozostaw puste, aby zachować zapisany klucz
assist-admin-api-key-remove = Usuń zapisany klucz
assist-admin-api-key-keep = Zachowaj zapisany klucz
assist-admin-api-key-remove-pending = Zapisany klucz zostanie usunięty po zapisaniu.
assist-admin-chat-model = Model czatu
assist-admin-embed-model = Model osadzeń (embedding)
assist-admin-stt-model = Model zamiany mowy na tekst
assist-admin-model = Model
assist-admin-anthropic-version = Wersja API
assist-admin-max-tokens = Maksymalna liczba tokenów odpowiedzi
assist-admin-default-placeholder = Domyślne ustawienie serwera
assist-admin-grants = Funkcje
assist-admin-grants-note = Można używać tylko funkcji zaznaczonych tutaj.
assist-admin-cap-summarize = Podsumowanie
assist-admin-cap-draft = Szkic i przeredagowanie
assist-admin-cap-grammar = Gramatyka
assist-admin-cap-dictation = Dyktowanie
assist-admin-cap-search-semantic = Wyszukiwanie semantyczne
assist-admin-cap-auto-tag = Automatyczne etykiety
assist-admin-cap-recap = Streszczenie
assist-admin-cap-assistant = Czat z asystentem AI
assist-admin-ceilings = Limity danych
assist-admin-ceilings-note = Te limity dotyczą każdego żądania, niezależnie od funkcji.
assist-admin-accounts = Identyfikatory kont, których poczta może być wysyłana, po jednym w wierszu
assist-admin-accounts-note = Jeśli nie wymieniono żadnego konta, żadna treść poczty nie jest wysyłana do punktu końcowego.
assist-admin-folders = Identyfikatory folderów, do których ograniczyć wysyłanie, po jednym w wierszu
assist-admin-folders-note = Jeśli nie wymieniono żadnego folderu, dozwolony jest każdy folder wymienionych kont.
assist-admin-allow-e2ee = Zezwól na wysyłanie treści szyfrowanej end-to-end
assist-admin-allow-attachments = Zezwól na wysyłanie załączników
assist-admin-save = Zapisz konfigurację
assist-admin-saved = Zapisano.
assist-admin-saved-pending = Zapisano. Zmiana zacznie obowiązywać po ponownym uruchomieniu serwera.
assist-admin-save-error = Konfiguracja nie została zapisana.
assist-admin-save-not-allowed = Konfiguracja nie została zapisana, ponieważ serwer nie zezwala na ten program. Można używać tylko programów wymienionych w MW_ASSIST_LOCAL_PROGRAMS.
