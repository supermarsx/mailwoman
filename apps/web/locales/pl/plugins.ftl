# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Ostrzeżenie o niepodpisanej wtyczce interfejsu
plugins-unsigned-title = Działają niepodpisane wtyczki
plugins-unsigned-text = Te wtyczki zostały dopuszczone bez zweryfikowanego podpisu. Działają w piaskownicy, ale ich kod nie został sprawdzony względem zaufanego klucza.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Wtyczka: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Wtyczki silnika działają w piaskownicy WebAssembly. Wtyczka działa dopiero wtedy, gdy jest zarejestrowana, zatwierdzona, ma przyznane co najmniej jedno uprawnienie i jest włączona. Może korzystać tylko z uprawnień przyznanych tutaj.
plugins-admin-unsigned-banner = Wczytano wtyczkę zewnętrzną bez podpisu. Działa, ponieważ administrator na to zezwolił; serwer sprawdza jej plik tylko względem zatwierdzonego skrótu.
plugins-admin-load-error = Nie udało się wczytać wtyczek.
plugins-admin-register-heading = Zarejestruj wtyczkę
plugins-admin-register-component = Komponent
plugins-admin-register-third-party = Komponent zewnętrzny
plugins-admin-register-id = Identyfikator wtyczki
plugins-admin-register-id-hint = Serwer odczytuje komponent z pliku nazwanego tym identyfikatorem w katalogu wtyczek zewnętrznych. Najpierw zatwierdź skrót tego pliku na liście dozwolonych poniżej.
plugins-admin-register-name = Nazwa
plugins-admin-register-version = Wersja
plugins-admin-register-capabilities = Uprawnienia deklarowane przez wtyczkę
plugins-admin-register-hosts = Hosty, z którymi wtyczka może się łączyć
plugins-admin-register-hosts-hint-first-party = Nazwy hostów rozdzielone przecinkami. Pozostaw puste, aby użyć domyślnych hostów komponentu.
plugins-admin-register-hosts-hint-third-party = Nazwy hostów rozdzielone przecinkami. Pozostaw puste, aby nie zezwalać na dostęp do sieci.
plugins-admin-register-submit = Zarejestruj
plugins-admin-trust-first-party = Własna, skrót wbudowany
plugins-admin-trust-third-party = Zewnętrzna, skrót zatwierdzony
plugins-admin-status-loaded = Wczytana
plugins-admin-status-not-loaded = Niewczytana
plugins-admin-status-restart = Wymagane ponowne uruchomienie
plugins-admin-restart-to-load = Aby wczytać tę wtyczkę, wymagane jest ponowne uruchomienie.
plugins-admin-restart-to-apply = Aby zastosować te zmiany, wymagane jest ponowne uruchomienie. Do tego czasu niektóre konta tego mostka nie są obsługiwane.
plugins-admin-reason-not-approved = Niewczytana: administrator nie zatwierdził tej wtyczki.
plugins-admin-reason-disabled = Niewczytana: wtyczka nie jest włączona.
plugins-admin-reason-unsigned-not-allowed = Niewczytana: wtyczka nie ma podpisu i nie zezwolono jej na działanie bez niego.
plugins-admin-reason-no-grant = Niewczytana: nie przyznano żadnego wymaganego uprawnienia.
plugins-admin-reason-component-unavailable = Niewczytana: żaden plik komponentu nie przeszedł sprawdzenia skrótu.
plugins-admin-reason-load-failed = Niewczytana: serwer nie mógł wczytać komponentu. Zobacz dziennik serwera.
plugins-admin-reason-proxy-mode = Niewczytana: ten serwer działa w trybie proxy i sam nie przetwarza poczty.
plugins-admin-reason-no-account-binding = Niewczytana: z tym mostkiem nie powiązano żadnego konta.
plugins-admin-reason-no-host-caller = Niewczytana: ta wersja serwera nie wywołuje punktów zaczepienia udostępnianych przez tę wtyczkę.
plugins-admin-reason-another-classifier-active = Niewczytana: wczytano inny klasyfikator spamu. Jednocześnie może działać tylko jeden.
plugins-admin-running-with = Działa z uprawnieniami: { $capabilities }
plugins-admin-hosts = Hosty: { $hosts }
plugins-admin-hosts-none = Hosty: brak
plugins-admin-grants-heading = Przyznane uprawnienia
plugins-admin-grants-hint = Wtyczka może korzystać tylko z zaznaczonych tutaj uprawnień. Zapisanie zastępuje wcześniejsze przyznanie.
plugins-admin-grant-for = Przyznaj { $capability } wtyczce { $name }
plugins-admin-grants-save = Zapisz uprawnienia
plugins-admin-endpoint = Adres klasyfikatora
plugins-admin-endpoint-for = Adres klasyfikatora dla { $name }
plugins-admin-endpoint-hint = Adres, pod którym wtyczka łączy się ze swoim demonem. Host musi też znajdować się na liście hostów wtyczki. Pozostaw puste, aby użyć wartości domyślnej komponentu.
plugins-admin-endpoint-save = Zapisz adres
plugins-admin-test = Testuj klasyfikator
plugins-admin-test-verdict = Wynik testu: { $verdict }
plugins-admin-uninstall = Odinstaluj
plugins-admin-uninstall-confirm = Potwierdź odinstalowanie
plugins-admin-uninstall-note = Odinstalowanie usuwa rejestrację, przyznane uprawnienia i zapisane ustawienia.
plugins-admin-uninstall-detail = Odinstalowanie usuwa zapisane dane wtyczki dla wszystkich kont, jej przypięcia na liście dozwolonych oraz jej rejestrację i uprawnienia. Plik komponentu na dysku nie jest usuwany.
plugins-admin-revoke-detail = Cofnięcie usuwa zatwierdzenie tego skrótu i wyłącza wtyczkę. Jeśli wtyczka działa, zostanie natychmiast zatrzymana.
plugins-admin-error-already-registered = Ta wtyczka jest już zarejestrowana. Odinstaluj ją przed ponowną rejestracją.
plugins-admin-error-component-unavailable = Żaden plik komponentu tej wtyczki nie pasuje do skrótu wbudowanego w serwer.
plugins-admin-error-digest-not-approved = Nie znaleziono pliku komponentu z zatwierdzonym skrótem dla tego identyfikatora. Najpierw zatwierdź skrót pliku na liście dozwolonych.
plugins-admin-error-not-approved = Zatwierdź wtyczkę przed jej włączeniem.
plugins-admin-error-unsigned-not-allowed = Ta wtyczka nie ma podpisu. Przed włączeniem zezwól na jej działanie bez podpisu.
plugins-admin-error-not-loaded = Wtyczka nie jest wczytana, więc nie można jej przetestować.
plugins-admin-error-classifier-error = Wywołanie klasyfikatora nie powiodło się: { $detail }
plugins-admin-error-other = Serwer odrzucił żądanie: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Wtyczki interfejsu
plugins-admin-ui-title = Wtyczki interfejsu
plugins-admin-ui-intro = Wtyczki interfejsu dodają elementy sterujące do klienta webowego. Każda działa w ramce w piaskownicy i może korzystać tylko z uprawnień przyznanych tutaj. Wtyczka jest udostępniana zalogowanym użytkownikom, gdy zostanie zatwierdzona i włączona.
plugins-admin-ui-empty = Brak zarejestrowanych wtyczek interfejsu.
plugins-admin-ui-load-error = Nie udało się wczytać wtyczek interfejsu.
plugins-admin-ui-register-heading = Zarejestruj wtyczkę interfejsu
plugins-admin-ui-manifest = Manifest (JSON)
plugins-admin-ui-bundle = Plik pakietu
plugins-admin-ui-bundle-hint = Wymagany dla podpisanej wtyczki: podpis jest sprawdzany względem tego pliku.
plugins-admin-ui-allow-unsigned = Zezwól na rejestrację tej wtyczki bez podpisu
plugins-admin-ui-register = Zarejestruj
plugins-admin-ui-manifest-invalid = Manifest nie jest prawidłowym plikiem JSON.
plugins-admin-ui-refused = Serwer odrzucił żądanie: { $detail }
plugins-admin-ui-approve = Zatwierdź i włącz
plugins-admin-ui-extension-points = Punkty rozszerzeń: { $points }
plugins-admin-ui-grants-heading = Przyznaj uprawnienie
plugins-admin-ui-grants-note = Serwer nie informuje, które uprawnienia zostały już przyznane. Ponowne przyznanie uprawnienia zastępuje jego wcześniejsze przyznanie.
plugins-admin-ui-grant = Przyznaj { $capability }
plugins-admin-ui-grant-for = Przyznaj { $capability } wtyczce { $name }
plugins-admin-ui-grant-hosts = Hosty dla { $capability } we wtyczce { $name }
plugins-admin-ui-grant-hosts-hint = Nazwy hostów rozdzielone przecinkami, z których wtyczka może pobierać dane.
plugins-admin-ui-granted = Przyznano { $capability }.
plugins-admin-ui-delete = Usuń
plugins-admin-ui-delete-confirm = Potwierdź usunięcie
