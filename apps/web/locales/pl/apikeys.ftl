# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Klucze API
apikeys-heading = Klucze API
apikeys-intro = Twórz klucze o ograniczonym zakresie dla skryptów i integracji. Każdy klucz jest przechowywany jako skrót, wyświetlany tylko raz i może zostać odwołany osobno. Przyznawaj najmniejszy zakres, który wystarcza.

apikeys-label = Etykieta
apikeys-label-aria = Etykieta klucza
apikeys-label-placeholder = np. skrypt kopii zapasowej

apikeys-create = Utwórz klucz
apikeys-saved = Sekret jest zapisany

apikeys-reveal-warning = Skopiuj ten sekret teraz — jest wyświetlany tylko raz i nie można go później odzyskać.
apikeys-copy = Kopiuj
apikeys-copy-aria = Kopiuj sekret do schowka
apikeys-copied = Skopiowano do schowka

apikeys-error-need-label = nadaj kluczowi etykietę, aby móc go później rozpoznać
apikeys-error-create = nie udało się utworzyć klucza
apikeys-error-revoke = nie udało się odwołać klucza

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Istniejące klucze
apikeys-existing = Istniejące klucze
apikeys-none = Brak kluczy.
apikeys-created = utworzono { $date }
apikeys-last-used = ostatnio użyto { $date }
apikeys-never-used = nigdy nie użyto
apikeys-revoked-at = odwołano { $date }
apikeys-revoke = Odwołaj
apikeys-status-active = Aktywny
apikeys-status-revoked = Odwołany

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Zakres
apikeys-capabilities = Uprawnienia
apikeys-cap-read = Odczyt
apikeys-cap-send = Wysyłanie
apikeys-cap-delete = Usuwanie
apikeys-surface = Obszar
apikeys-surface-mail-label = Poczta
apikeys-surface-pim-label = PIM (kalendarz / zadania / notatki / kontakty)
apikeys-accounts = Konta
apikeys-all-accounts = Wszystkie konta
apikeys-folders = Foldery
apikeys-all-folders = Wszystkie foldery
apikeys-folder-ids = Identyfikatory folderów (rozdzielone przecinkami)
apikeys-folder-subset-aria = Wybrane foldery
apikeys-constraints = Ograniczenia
apikeys-expiry = Wygaśnięcie (RFC 3339, puste = bez wygaśnięcia)
apikeys-expiry-aria = Wygaśnięcie
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Limit żądań (żądań/min, puste = bez limitu)
apikeys-rate-limit-aria = Limit żądań
apikeys-ip-allowlist = Lista dozwolonych IP (CIDR/IP, rozdzielone przecinkami; puste = dowolne)
apikeys-ip-allowlist-aria = Lista dozwolonych IP
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = odczyt
apikeys-verb-send = wysyłanie
apikeys-verb-delete = usuwanie
apikeys-summary-mail = poczta
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = brak operacji
apikeys-summary-nothing = nic
apikeys-summary-none = brak
apikeys-summary-verbs-on = { $verbs } w: { $surfaces }
apikeys-summary-all-accounts = wszystkie konta
apikeys-summary-accounts = konta: { $ids }
apikeys-summary-all-folders = wszystkie foldery
apikeys-summary-folders = foldery: { $ids }
apikeys-summary-expires = wygasa { $date }
apikeys-summary-rate = { $n } żąd./min
apikeys-summary-ips = IP: { $ips }
apikeys-summary-mcp-tools = Narzędzia MCP: { $tools }
apikeys-summary-unattended = Wysyłanie BEZ NADZORU

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Klucze MCP
apikeys-mcp-heading = Klucze MCP
apikeys-mcp-intro = Przyznaj agentowi AI (przez Model Context Protocol) dokładnie te narzędzia, które wybierzesz. Treści wiadomości zwracane agentowi są oznaczane jako niezaufane dane wejściowe. Każde narzędzie jest przyznawane osobno.
apikeys-mcp-label-aria = Etykieta klucza MCP
apikeys-mcp-label-placeholder = np. agent asystenta
apikeys-mcp-create = Utwórz klucz MCP
apikeys-mcp-reveal-warning = Skopiuj ten sekret teraz — jest wyświetlany tylko raz.
apikeys-mcp-error-need-label = nadaj kluczowi MCP etykietę
apikeys-mcp-error-need-tool = przyznaj co najmniej jedno narzędzie
apikeys-mcp-error-create = nie udało się utworzyć klucza MCP

apikeys-tools = Narzędzia
apikeys-tools-group = Narzędzia MCP
apikeys-outbox-suffix = (przez Skrzynkę nadawczą)

apikeys-unattended-aria = Wysyłanie bez nadzoru
apikeys-unattended-allow = Zezwól na wysyłanie bez nadzoru (z pominięciem Skrzynki nadawczej — wymaga kontrasygnaty administratora)
apikeys-unattended-pending = Wysyłanie bez nadzoru pozostaje nieaktywne, dopóki administrator nie kontrasygnuje tego klucza.
apikeys-unattended-disclosure = Domyślnie przyznane wysyłanie trafia do Twojej Skrzynki nadawczej i czeka, aż potwierdzisz je w aplikacji — automatyzacja nie może wysyłać samodzielnie. Wysyłanie bez nadzoru USUWA ten krok z udziałem człowieka, więc ten klucz może wysyłać pocztę bez potwierdzenia. Dodatkowo wymaga to kontrasygnaty klucza przez administratora. Przyznawaj je tylko automatyzacji, której w pełni ufasz; przejęty klucz z wysyłaniem bez nadzoru może wysyłać pocztę w Twoim imieniu bez żadnego pytania.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Wyszukiwanie poczty
apikeys-tool-mail-search-desc = Wyszukiwanie wiadomości. Wyniki są oznaczane jako niezaufane dane wejściowe.
apikeys-tool-mail-read-label = Odczyt poczty
apikeys-tool-mail-read-desc = Odczyt treści wiadomości. Treści są oznaczane jako niezaufane dane wejściowe.
apikeys-tool-folders-list-label = Lista folderów
apikeys-tool-folders-list-desc = Wyświetlanie folderów skrzynki pocztowej.
apikeys-tool-drafts-create-label = Tworzenie wersji roboczych
apikeys-tool-drafts-create-desc = Tworzenie wersji roboczej wiadomości (nigdy nie jest wysyłana automatycznie).
apikeys-tool-mail-send-label = Wysyłanie poczty
apikeys-tool-mail-send-desc = Umieszczenie wiadomości w Skrzynce nadawczej do potwierdzenia w aplikacji.
apikeys-tool-calendar-read-label = Odczyt kalendarza
apikeys-tool-calendar-read-desc = Odczyt wydarzeń z kalendarza.
apikeys-tool-calendar-propose-label = Proponowanie wydarzeń
apikeys-tool-calendar-propose-desc = Proponowanie (bez zatwierdzania) wydarzeń w kalendarzu.
apikeys-tool-tasks-read-label = Odczyt zadań
apikeys-tool-tasks-read-desc = Odczyt zadań.
apikeys-tool-tasks-write-label = Zapis zadań
apikeys-tool-tasks-write-desc = Tworzenie lub aktualizowanie zadań.
apikeys-tool-contacts-read-label = Odczyt kontaktów
apikeys-tool-contacts-read-desc = Odczyt kontaktów.
apikeys-surface-scripts-label = Zarządzaj skryptami Sieve
