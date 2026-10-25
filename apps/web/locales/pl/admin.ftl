# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman — administracja
admin-nav = Sekcje administracyjne
admin-sign-out = Wyloguj się
admin-nav-domains = Domeny
admin-nav-users = Użytkownicy
admin-nav-integrations = Integracje
admin-nav-observability = Monitorowanie
admin-nav-plugins = Wtyczki
admin-nav-assist = Asystent AI
admin-nav-sso = Logowanie jednokrotne
admin-nav-servermeta = Metadane serwera
admin-nav-rethread = Konserwacja
admin-nav-2fa = Wymagaj uwierzytelniania dwuskładnikowego

# Shared admin actions / states
admin-delete = Usuń
admin-revoke = Odwołaj
admin-remove = Usuń
admin-saved = Zapisano.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Logowanie administratora
admin-login-note = Ten panel działa w osobnej sesji administratora. Sesja kończy się po 30 minutach bezczynności, a najpóźniej 12 godzin po zalogowaniu.
admin-login-session-ended = Sesja administratora wygasła. Zaloguj się ponownie.
admin-login-username = Nazwa użytkownika administratora
admin-login-password = Hasło
admin-login-sign-in = Zaloguj się
admin-login-signing-in = Logowanie…
admin-login-invalid = Nieprawidłowe dane logowania administratora
admin-login-unreachable = Nie można połączyć się z serwerem

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domeny
admin-domains-load-error = Nie udało się wczytać domen
admin-domains-save-error = Nie udało się zapisać domeny
admin-domains-delete-error = Nie udało się usunąć domeny
admin-domains-add = Dodaj domenę
admin-domains-name = Nazwa domeny
admin-domains-name-placeholder = example.com
admin-domains-note = Domena jest rejestrowana według nazwy. Zarejestrowane nazwy są proponowane na ekranie Wymagaj uwierzytelniania dwuskładnikowego, gdy reguła dotyczy jednej domeny. Zarejestrowanie domeny nie kieruje ani nie filtruje poczty.
admin-domains-save = Zapisz domenę
admin-domains-empty = Brak domen.
admin-domains-delete-for = Usuń { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Użytkownicy
admin-users-load-error = Nie udało się wczytać użytkowników
admin-users-provision-error = Nie udało się utworzyć użytkownika
admin-users-flag-error = Nie udało się zaktualizować flagi
admin-users-revoke-error = Nie udało się odwołać sesji
admin-users-provision = Utwórz użytkownika
admin-users-username = Nazwa użytkownika
admin-users-domain = Domena
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Limit miejsca w bajtach (0 = bez limitu)
admin-users-quota-msgs = Limit liczby wiadomości (0 = bez limitu)
admin-users-empty = Brak użytkowników.
admin-users-col-account = Konto
admin-users-col-quota = Limit miejsca (bajty/wiadomości)
admin-users-col-zeroaccess = Zero-access
admin-users-col-flags = Flagi
admin-users-col-sessions = Sesje
admin-users-zeroaccess-for = Zero-access dla { $account }
admin-users-zeroaccess-help = Tylko informacja. Przechowywanie zero-access włącza i wyłącza użytkownik we własnych ustawieniach, własnym kluczem. Zaznaczenie tego pola nie szyfruje zapisanej poczty konta, a jego odznaczenie jej nie odszyfrowuje.
admin-users-disable-for = Wyłącz { $account }
admin-users-disabled = wyłączone
admin-users-disabled-help = Blokuje logowanie i kończy wszystkie otwarte sesje tego konta. Jego klucze API i tokeny są odrzucane. Nie wyłącza to skrzynki pocztowej na serwerze pocztowym: inne programy pocztowe nadal mają do niej dostęp, dopóki nie wyłączysz jej również tam.
admin-users-force-change-for = Wymuś zmianę hasła dla { $account }
admin-users-force-change = wymuś zmianę
admin-users-force-change-help = Użytkownik jest zatrzymywany na ekranie zmiany hasła — przy następnym logowaniu i w już otwartych sesjach — i nie ma dostępu do poczty, dopóki zmiana się nie powiedzie. Wymaga to działającego mechanizmu zmiany hasła (MW_PASSWD_BACKEND). Bez niego zmiana nie może się powieść, a użytkownik pozostaje na tym ekranie, dopóki nie odznaczysz tego pola.
admin-users-revoke-for = Odwołaj sesje dla { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integracje
admin-integrations-load-error = Nie udało się wczytać integracji
admin-integrations-revoke-error = Nie udało się odwołać klucza
admin-integrations-ldap = Katalog LDAP / GAL
admin-integrations-nextcloud = Mostek Nextcloud
admin-integrations-active = Aktywne
admin-integrations-configured = Skonfigurowane
admin-integrations-not-configured = Nieskonfigurowane
admin-integrations-unknown = Stan nieznany
admin-integrations-config-note = „Skonfigurowane” oznacza, że to wdrożenie ma ustawienia dla integracji: włączony wpis katalogu LDAP w bazie danych lub trzy zmienne środowiskowe MW_NEXTCLOUD_*. Oba są odczytywane przy uruchomieniu serwera. Nie oznacza to, że nawiązano połączenie z usługą zdalną ani że jest ona osiągalna.
admin-integrations-webhooks = Webhooki
admin-integrations-webhooks-empty = Brak zarejestrowanych webhooków.
admin-integrations-keys = Klucze API i MCP
admin-integrations-keys-empty = Nie wydano żadnych kluczy.
admin-integrations-col-prefix = Prefiks
admin-integrations-col-account = Konto
admin-integrations-col-scopes = Zakresy
admin-integrations-col-status = Stan
admin-integrations-status-revoked = odwołany
admin-integrations-status-active = aktywny
admin-integrations-revoke-key = Odwołaj klucz { $prefix }
admin-integrations-col-unattended = Wysyłanie bez nadzoru
admin-integrations-unattended-not-requested = Nie zażądano
admin-integrations-unattended-requested = Zażądano, niezatwierdzone
admin-integrations-unattended-approved = Zatwierdzone
admin-integrations-unattended-approve = Zatwierdź
admin-integrations-unattended-withdraw = Wycofaj
admin-integrations-unattended-approve-key = Zatwierdź wysyłanie bez nadzoru dla klucza { $prefix }
admin-integrations-unattended-withdraw-key = Wycofaj zatwierdzenie wysyłania bez nadzoru dla klucza { $prefix }
admin-integrations-unattended-approve-title = Zatwierdzić wysyłanie bez nadzoru dla tego klucza?
admin-integrations-unattended-withdraw-title = Wycofać zatwierdzenie dla tego klucza?
admin-integrations-unattended-owner = Właściciel: { $account }
admin-integrations-unattended-key = Klucz: { $prefix }
admin-integrations-unattended-scope = Uprawnienia: { $permissions }. Narzędzia MCP: { $tools }.
admin-integrations-unattended-scope-unreadable = Nie udało się odczytać zakresu tego klucza.
admin-integrations-scope-read = odczyt
admin-integrations-scope-send = wysyłanie
admin-integrations-scope-delete = usuwanie
admin-integrations-scope-mail = poczta
admin-integrations-scope-pim = kalendarz, zadania, notatki i kontakty
admin-integrations-scope-none = brak
admin-integrations-unattended-approve-meaning = Wiadomości wysyłane tym kluczem przez MCP są przesyłane bez zatwierdzenia przez człowieka.
admin-integrations-unattended-withdraw-meaning = Wysyłanie tym kluczem przez MCP jest odrzucane, dopóki klucz nie zostanie ponownie zatwierdzony. Wiadomość nie jest przesyłana ani umieszczana w Skrzynce nadawczej właściciela.
admin-integrations-unattended-approve-effect = Zatwierdzenie działa natychmiast, od następnego wysłania tym kluczem. Ponowne uruchomienie nie jest potrzebne.
admin-integrations-unattended-withdraw-effect = Wycofanie działa natychmiast: od następnego wysłania klucz przestaje wysyłać bez nadzoru. Ponowne uruchomienie nie jest potrzebne.
admin-integrations-unattended-cancel = Anuluj
admin-integrations-unattended-saving = Zapisywanie…
admin-integrations-unattended-approved-done = Zatwierdzono wysyłanie bez nadzoru dla klucza { $prefix }. Obowiązuje od teraz.
admin-integrations-unattended-withdrawn-done = Wycofano zatwierdzenie dla klucza { $prefix }. Ten klucz nie wysyła już bez nadzoru.
admin-integrations-unattended-error-gone = Klucz { $prefix } już nie istnieje lub został odwołany. Niczego nie zmieniono.
admin-integrations-unattended-error-conflict = Klucz { $prefix } jest odwołany lub jego właściciel nie zażądał wysyłania bez nadzoru. Niczego nie zatwierdzono.
admin-integrations-unattended-error = Nie udało się zmienić zatwierdzenia dla klucza { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Monitorowanie
admin-obs-load-error = Nie udało się wczytać danych monitorowania
admin-obs-export-error = Nie udało się wyeksportować dziennika audytu
admin-obs-ban-add-error = Nie udało się dodać blokady
admin-obs-unban-error = Nie udało się usunąć blokady
admin-obs-telemetry-note = Rejestrowania i telemetrii nie ustawia się tutaj. Filtr dziennika (MW_LOG), kolektor OTLP (MW_OTLP_ENDPOINT) i punkt końcowy metryk (MW_METRICS_TOKEN) są odczytywane ze środowiska serwera przy jego uruchomieniu.
admin-obs-audit = Dziennik audytu
admin-obs-export = Eksportuj JSONL
admin-obs-audit-empty = Brak wpisów audytu.
admin-obs-col-time = Czas
admin-obs-col-actor = Wykonawca
admin-obs-col-action = Działanie
admin-obs-col-target = Obiekt
admin-obs-bans = Monitor logowań / lista blokad
admin-obs-bans-note = Ta lista jest tylko rejestrem. Mailwoman nie odrzuca połączeń z adresów na niej wymienionych, a wpis nie ma wpływu na logowanie. Każde nieudane logowanie administratora zapisuje w dzienniku serwera wiersz, na który może reagować jail fail2ban, a pięć niepowodzeń z jednego adresu dodaje ten adres tutaj. Blokowanie wykonuje ten jail lub Twoja zapora.
admin-obs-ban-add = Dodaj blokadę
admin-obs-ban-ip = Adres IP
admin-obs-ban-reason = Powód
admin-obs-ban-btn = Zablokuj IP
admin-obs-bans-empty = Brak aktywnych blokad.
admin-obs-unban-for = Odblokuj { $ip }
admin-obs-unban-btn = Odblokuj

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Wtyczki
admin-plugins-intro = Wtyczki silnika działają w piaskownicy WebAssembly z kontrolą uprawnień. Zatwierdź wtyczkę, zanim będzie można ją włączyć, i przyznawaj tylko te uprawnienia, których potrzebuje.
admin-plugins-empty = Brak zarejestrowanych wtyczek.
admin-plugins-signed = Podpisana
admin-plugins-unsigned = Niepodpisana
admin-plugins-approved = Zatwierdzona
admin-plugins-enabled = Włączona
admin-plugins-approve = Zatwierdź
admin-plugins-enable = Włącz
admin-plugins-disable = Wyłącz
admin-plugins-net = sieć: { $hosts }
admin-plugins-limits = limity: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = limity: { $memory } MiB · { $deadline } ms · paliwo: { $fuel }
admin-plugins-allow-unsigned-for = Zezwól na niepodpisaną wtyczkę { $name }
admin-plugins-allow-unsigned = Zezwól na uruchamianie tej niepodpisanej wtyczki
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Lista dozwolonych wtyczek zewnętrznych
admin-allowlist-intro = Komponent zewnętrzny (niepochodzący od twórców Mailwoman) jest wczytywany dopiero wtedy, gdy administrator zatwierdzi jego dokładny skrót SHA-256. Poniższy skrót jest obliczany przez serwer z bajtów komponentu na dysku; zatwierdzenie przypina dokładnie te bajty. Komponenty własne są przypięte w kompilacji i nie są tutaj zarządzane.
admin-allowlist-load-error = Nie udało się wczytać listy dozwolonych wtyczek.
admin-allowlist-present-heading = Komponenty na dysku
admin-allowlist-present-empty = Brak komponentów zewnętrznych. Umieść komponent w katalogu wtyczek zewnętrznych, aby pojawił się tutaj.
admin-allowlist-digest-label = Obliczony SHA-256
admin-allowlist-status-approved = Zatwierdzony
admin-allowlist-status-pending = Niezatwierdzony
admin-allowlist-status-firstparty = Własny
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Dopuszczony przez przypięcie skrótu. Ten komponent nie ma podpisu; zatwierdzenie oznacza zaufanie dokładnie tym bajtom, których skrót jest wyświetlany, co jest oczekiwanym stanem dla komponentu zatwierdzonego w ten sposób.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = Uprawnień wysokiego ryzyka ({ $caps }) nie można przyznać wtyczce zewnętrznej. Serwer odrzuca je niezależnie od zatwierdzenia; są dostępne tylko dla komponentów własnych.
admin-allowlist-firstparty-note = Ten identyfikator odpowiada komponentowi własnemu. Przypięcie komponentu własnego ma zawsze pierwszeństwo, więc tego identyfikatora nie można zatwierdzić jako zewnętrznego.
admin-allowlist-approve = Zatwierdź skrót
admin-allowlist-approve-for = Zatwierdź skrót dla { $id }
admin-allowlist-revoke = Odwołaj
admin-allowlist-revoke-for = Odwołaj przypięcie dla { $id }
admin-allowlist-uninstall = Odinstaluj
admin-allowlist-uninstall-for = Odinstaluj { $id }
admin-allowlist-pins-heading = Zatwierdzone i odwołane przypięcia
admin-allowlist-pins-empty = Brak zarejestrowanych przypięć.
admin-allowlist-pin-approved-by = Zatwierdzone przez { $by } dnia { $at }
admin-allowlist-pin-revoked = Odwołane
admin-allowlist-cancel = Anuluj

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Zatwierdzić wczytywanie tego komponentu?
admin-allowlist-approve-detail = Zatwierdzenie przypina dokładnie te bajty, których SHA-256 jest pokazany poniżej. Po zatwierdzeniu dla tego identyfikatora będą wczytywane tylko bajty zgodne z tym skrótem; każda zmiana komponentu daje inny skrót i nie zostanie wczytana, dopóki nie zostanie ponownie zatwierdzona. Samo zatwierdzenie nie przyznaje żadnych uprawnień.
admin-allowlist-approve-confirm = Zatwierdź skrót

# Revoke confirmation.
admin-allowlist-revoke-title = Odwołać to przypięcie?
admin-allowlist-revoke-detail = Odwołanie usuwa zatwierdzenie tego skrótu i wyłącza wtyczkę. Działa przy następnym wczytaniu; już działająca instancja nie jest do tego czasu zatrzymywana.
admin-allowlist-revoke-confirm = Odwołaj przypięcie

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Odinstalować tę wtyczkę?
admin-allowlist-uninstall-detail = Odinstalowanie usuwa zapisane dane klucz/wartość wtyczki dla wszystkich kont, usuwa jej przypięcia z listy dozwolonych i ją wyłącza. Plik komponentu na dysku nie jest usuwany; można go później ponownie zatwierdzić.
admin-allowlist-uninstall-confirm = Odinstaluj wtyczkę

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Logowanie jednokrotne
admin-sso-intro = Skonfiguruj mechanizmy logowania OIDC i SAML 2.0. Włączone mechanizmy pojawiają się jako przyciski „Zaloguj się przez…” na ekranie logowania do skrzynki, dla całego wdrożenia lub dla jednej domeny.
admin-sso-add = Dodaj mechanizm logowania
admin-sso-edit = Edytuj
admin-sso-create = Dodaj mechanizm
admin-sso-update = Zapisz zmiany
admin-sso-cancel = Anuluj
admin-sso-empty = Nie skonfigurowano mechanizmów SSO.
admin-sso-load-error = Nie udało się wczytać mechanizmów SSO.
admin-sso-save-error = Nie udało się zapisać mechanizmu.
admin-sso-delete-error = Nie udało się usunąć mechanizmu.

# Common fields
admin-sso-id = Identyfikator mechanizmu
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Nazwa wyświetlana
admin-sso-display-name-placeholder = Zaloguj się przez Acme SSO
admin-sso-kind = Protokół
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Zakres
admin-sso-scope-deployment = Całe wdrożenie
admin-sso-scope-domain = Jedna domena
admin-sso-domain = Domena
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Włączony
admin-sso-first-login = Zasady pierwszego logowania
admin-sso-policy-allowlist = Lista dozwolonych (odrzucaj nieznanych użytkowników)
admin-sso-policy-autocreate = Twórz konta automatycznie przy pierwszym logowaniu

# OIDC fields
admin-sso-issuer = URL wystawcy
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Identyfikator klienta
admin-sso-client-secret = Sekret klienta
admin-sso-secret-unchanged = Pozostaw puste, aby zachować zapisany sekret
admin-sso-redirect = URL przekierowania
admin-sso-scopes = Zakresy
admin-sso-metadata = Metadane SP

# SAML fields
admin-sso-sp-entity-id = Identyfikator jednostki SP
admin-sso-acs-url = URL ACS
admin-sso-idp-metadata-url = URL metadanych IdP
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = XML metadanych IdP
admin-sso-idp-metadata-xml-placeholder = Wklej XML metadanych IdP lub użyj powyższego adresu URL
admin-sso-idp-sso-url = URL SSO IdP
admin-sso-idp-slo-url = URL wylogowania IdP (SLO)
admin-sso-idp-certs = Certyfikaty podpisujące IdP (PEM)
admin-sso-idp-certs-placeholder = Jeden certyfikat PEM na blok, bloki oddzielone pustym wierszem
admin-sso-nameid-format = Format NameID
admin-sso-want-signed = Wymagaj podpisanych asercji
admin-sso-want-encrypted = Wymagaj zaszyfrowanych asercji

# Claim map
admin-sso-claims = Mapowanie atrybutów (claims)
admin-sso-claim-email = Atrybut adresu e-mail
admin-sso-claim-username = Atrybut nazwy użytkownika
admin-sso-claim-display = Atrybut nazwy wyświetlanej
admin-sso-claim-groups = Atrybut grup

# List row
admin-sso-badge-enabled = Włączony
admin-sso-badge-disabled = Wyłączony
admin-sso-enable = Włącz
admin-sso-disable = Wyłącz
admin-sso-delete = Usuń
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Włącz { $name }
admin-sso-disable-for = Wyłącz { $name }
admin-sso-delete-for = Usuń { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Metadane serwera
admin-servermeta-intro = Przeglądaj i edytuj adnotacje serwera RFC 5464 dla utworzonego konta. Zmiany są zapisywane bezpośrednio na serwerze pocztowym, który decyduje, czy konto może je ustawiać.
admin-servermeta-account = Konto
admin-servermeta-select-option = Wybierz konto…
admin-servermeta-select-prompt = Wybierz konto, aby przeglądać i edytować jego adnotacje serwera.
admin-servermeta-load-error = Nie udało się wczytać listy kont.
admin-servermeta-no-accounts = Brak utworzonych kont.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Ponowne grupowanie wątków
admin-rethread-intro = Ponownie uruchamia grupowanie wiadomości w wątki (JWZ) dla zapisanej poczty utworzonego konta i na nowo przypisuje klucze wątków. Jest to jednorazowa czynność konserwacyjna, a nie coś, co działa automatycznie.
admin-rethread-account = Konto
admin-rethread-select-option = Wybierz konto…
admin-rethread-no-accounts = Brak utworzonych kont.
admin-rethread-load-error = Nie udało się wczytać listy kont.
admin-rethread-run = Pogrupuj wątki ponownie
admin-rethread-confirm-title = Ponownie pogrupować wątki tej skrzynki?
admin-rethread-confirm-warning = Ponowne grupowanie zmienia przypisanie wiadomości do wątków dla tego konta. Istniejące wątki mogą się połączyć lub rozdzielić, a użytkownicy mogą zobaczyć inaczej pogrupowane rozmowy.
admin-rethread-confirm-detail = Jest to jednorazowa czynność konserwacyjna. Zostanie wykonana raz, teraz; można ją bezpiecznie powtórzyć i nie usuwa żadnej poczty.
admin-rethread-confirm = Pogrupuj wątki ponownie
admin-rethread-running = Grupowanie wątków…
admin-rethread-cancel = Anuluj
admin-rethread-summary = Wiadomości: { $messages }, pogrupowane w wątki: { $threads }, konta: { $accounts }; wiadomości przeniesione do innego wątku: { $reassigned }.
admin-rethread-error = Ponowne grupowanie nie powiodło się. Jeśli serwer odrzucił żądanie, grupowanie wątków nie zostało zmienione; sprawdź dzienniki serwera i spróbuj ponownie.

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Indeks wyszukiwania
admin-searchindex-intro = Wyszukiwanie korzysta z indeksu zbudowanego z zapisanej poczty. Serwer odbudowuje go przy uruchomieniu, jeśli nie odpowiada zapisanej poczcie. Poczta kont zero-access nie jest indeksowana, więc wyszukiwanie niczego dla tych kont nie znajduje.
admin-searchindex-count = Zaindeksowane wiadomości: { $documents } z { $messages } zapisanych.
admin-searchindex-on-disk = Indeks jest przechowywany na dysku.
admin-searchindex-in-memory = Indeks jest przechowywany w pamięci i odbudowywany przy każdym uruchomieniu serwera.
admin-searchindex-progress = Trwa odbudowa: przetworzono { $done } z { $total } wiadomości.
admin-searchindex-run = Odbuduj indeks wyszukiwania
admin-searchindex-running = Odbudowywanie…
admin-searchindex-summary = Odczytane zapisane wiadomości: { $messages }, konta: { $accounts }. Zaindeksowano: { $indexed }, usunięto z indeksu: { $removed }, nie udało się odczytać: { $failed }. Niezaindeksowane konta zero-access: { $zeroAccess }.
admin-searchindex-busy = Odbudowa już trwa. Poczekaj, aż się zakończy.
admin-searchindex-error = Odbudowa nie powiodła się. Indeks zachowuje dotychczasową zawartość; sprawdź dzienniki serwera i spróbuj ponownie.
admin-searchindex-status-error = Nie udało się odczytać stanu indeksu wyszukiwania.
admin-searchindex-unavailable = Ten serwer działa w trybie proxy i nie ma indeksu wyszukiwania.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = Wymagaj uwierzytelniania dwuskładnikowego
admin-2fa-intro = Wymagaj drugiego składnika (klucza dostępu lub aplikacji uwierzytelniającej) przy logowaniu. Każdy użytkownik może samodzielnie dodać składnik; wymaganie go tutaj zmusza konta objęte zakresem do dodania go przy następnym logowaniu.
admin-2fa-load-error = Nie udało się wczytać zasad uwierzytelniania dwuskładnikowego.
admin-2fa-save-error = Nie udało się zapisać zasad uwierzytelniania dwuskładnikowego.
admin-2fa-global = Wymagaj uwierzytelniania dwuskładnikowego w całym wdrożeniu
admin-2fa-global-label = Wymagaj uwierzytelniania dwuskładnikowego w całym wdrożeniu
admin-2fa-global-note = Gdy włączone, każde konto musi mieć drugi składnik. Reguła dla domeny poniżej może też wymagać go w jednej domenie, bez wymagania go wszędzie.
admin-2fa-domains-heading = Wymagania dla domen
admin-2fa-domains-empty = Nie ustawiono wymagań dla domen.
admin-2fa-col-domain = Domena
admin-2fa-col-require = Wymagane
admin-2fa-require-for = Wymagaj uwierzytelniania dwuskładnikowego dla { $domain }
admin-2fa-add-domain = Domena
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Wymagaj uwierzytelniania dwuskładnikowego dla tej domeny
admin-2fa-add-require-label = Wymagaj uwierzytelniania dwuskładnikowego dla tej domeny
admin-2fa-add-save = Dodaj regułę domeny

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Ruch wychodzący
admin-egress-heading = Trasy proxy dla ruchu wychodzącego
admin-egress-intro = Kieruj wychodzące pobrania serwera przez proxy HTTP CONNECT lub SOCKS5. Mailwoman nadal stosuje własne zasady dotyczące adresów — proxy nigdy nie jest proszone o rozwiązanie nazwy.
admin-egress-empty = Nie skonfigurowano tras. Wychodzące pobrania odbywają się bezpośrednio.
admin-egress-none-active = Żadna trasa nie jest używana. Wychodzące pobrania odbywają się bezpośrednio. Zapisana trasa przenosi ruch dopiero po wybraniu „Użyj tej trasy”.
admin-egress-in-use = Wychodzące pobrania używają trasy „{ $id }”.
admin-egress-deactivate = Przestań używać proxy
admin-egress-activate = Użyj tej trasy
admin-egress-activate-for = Użyj trasy { $id }
admin-egress-activate-error = Nie udało się przełączyć na trasę
admin-egress-deactivate-error = Nie udało się przestać używać trasy
admin-egress-col-state = Stan
admin-egress-state-active = W użyciu
admin-egress-state-inactive = Nieużywana
admin-egress-test-scope = Można przetestować tylko używaną trasę. Test pobiera jeden stały URL przez trasę i informuje, jak daleko dotarł; niczego nie zmienia.
admin-egress-probe-url = Pobrano: { $url }
admin-egress-load-error = Nie udało się wczytać tras ruchu wychodzącego
admin-egress-save-error = Nie udało się zapisać trasy
admin-egress-delete-error = Nie udało się usunąć trasy
admin-egress-invalid = Wymagane są identyfikator, host i port większy od zera.
admin-egress-username-required = Hasło wymaga nazwy użytkownika; bez niej trasa uwierzytelniałaby się jako nikt.
admin-egress-col-id = Identyfikator
admin-egress-col-route = Trasa
admin-egress-col-username = Nazwa użytkownika
admin-egress-col-credentials = Hasło
admin-egress-col-plaintext = Tekst jawny
admin-egress-col-actions = Akcje
admin-egress-cred-set = Ustawione
admin-egress-cred-none = Brak
admin-egress-plaintext-on = Dozwolony
admin-egress-plaintext-off = Odrzucany
admin-egress-add-heading = Dodaj trasę
admin-egress-edit-heading = Edytuj trasę
admin-egress-id = Identyfikator
admin-egress-scheme = Schemat
admin-egress-host = Host
admin-egress-port = Port
admin-egress-username = Nazwa użytkownika
admin-egress-password = Hasło
admin-egress-password-unchanged = Pozostaw puste, aby zachować zapisane hasło
admin-egress-allow-plaintext = Zezwól na źródła http w postaci jawnej przez tę trasę
admin-egress-allow-plaintext-note = Domyślnie wyłączone. To źródło https sprawia, że proxy nie może odczytać przenoszonego ruchu; zezwolenie na http oznacza, że proxy go widzi.
admin-egress-test = Testuj
admin-egress-testing = Testowanie…
admin-egress-test-failed = Nie udało się przeprowadzić testu, więc nic nie wiadomo o tej trasie.
admin-egress-outcome-connected = Połączono
admin-egress-outcome-auth-rejected = Uwierzytelnianie odrzucone
admin-egress-outcome-refused-by-policy = Odrzucone przez zasady
admin-egress-outcome-dns-failed = Nie udało się rozwiązać nazwy
admin-egress-outcome-unreachable = Nieosiągalne
admin-egress-outcome-origin-tls-failed = Błąd TLS źródła
admin-egress-outcome-route-invalid = Trasa jest nieprawidłowa
admin-egress-outcome-unknown = Nierozpoznany wynik
admin-egress-stage-dns = zatrzymano na rozwiązywaniu nazwy
admin-egress-stage-connect = zatrzymano podczas łączenia z proxy
admin-egress-stage-tunnel = zatrzymano podczas otwierania tunelu
admin-egress-stage-origin = zatrzymano na źródle
admin-egress-stage-unknown = zatrzymano na nierozpoznanym etapie
admin-egress-proxied-yes = Przeszło przez proxy
admin-egress-proxied-no = Nie przeszło przez proxy
