# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Dostęp przychodzący
admin-inbound-help = Ogranicz zaufane adresy IP źródłowe osobno dla konsoli administratora i poczty w przeglądarce. Obie zasady są początkowo wyłączone. Te reguły nie zmieniają zaufanych serwerów proxy, zakresów API ani dostępu wychodzącego.
admin-inbound-unscoped = Ta zasada obejmująca całe wdrożenie wymaga administratora z nieograniczonym dostępem do bezpieczeństwa.
admin-inbound-admin = Konsola administratora i API
admin-inbound-admin-help = Obejmuje logowanie i ten edytor. Istniejące uwierzytelnienie administratora nie omija zasad sieciowych.
admin-inbound-webmail = Poczta w przeglądarce i API kont
admin-inbound-webmail-help = Obejmuje pocztę, DAV, podpisaną zawartość, SSO, OIDC i trasy API. Istniejące połączenia, które zostały już podniesione, zachowują decyzję z uzgadniania.
admin-inbound-enabled = Ogranicz tę powierzchnię do listy dozwolonych
admin-inbound-cidrs = Awaryjne adresy IP i CIDR
admin-inbound-cidrs-help = Jeden dosłowny adres IP lub CIDR w wierszu, do 128. Po włączeniu wymagany jest co najmniej jeden. Dodaj zaufane źródło, którego możesz użyć do odzyskania dostępu.
admin-inbound-hostnames = Dozwolone nazwy hostów DNS
admin-inbound-hostnames-help = Jedna nazwa DNS ASCII małymi literami w wierszu. W przypadku nazw międzynarodowych użyj IDNA ASCII. Bez adresów URL, portów, symboli wieloznacznych, literałów IP i końcowych kropek. Nazwy przyznają dostęp tylko bieżącym rozwiązanym adresom.
admin-inbound-dns = Limity rozwiązywania DNS
admin-inbound-dns-help = Nowe, wygasłe i nieudane nazwy nie przyznają dostępu. Poprawne odpowiedzi respektują swój TTL DNS i skonfigurowany limit pamięci podręcznej. Żądania nigdy nie czekają na rozwiązanie DNS.
admin-inbound-servers = Rekurencyjne serwery DNS
admin-inbound-servers-help = Zostaw puste, aby użyć systemowych resolwerów, lub podaj do czterech gniazd IP:port, po jednym w wierszu (na przykład 10.0.0.53:53 lub [fd00::53]:53). Jawne serwery zastępują resolwery systemowe bez publicznego rozwiązania awaryjnego. Zaufanie do resolwera należy do Ciebie; niezależna walidacja DNSSEC nie jest deklarowana.
admin-inbound-dns-timeoutMs = Limit czasu zapytania (milisekundy)
admin-inbound-dns-cacheSeconds = Maksymalny czas życia pamięci podręcznej (sekundy)
admin-inbound-dns-maxHosts = Maksymalna liczba różnych nazw hostów
admin-inbound-dns-maxAddresses = Maksymalna liczba adresów na nazwę hosta
admin-inbound-dns-concurrency = Równoczesne zapytania
admin-inbound-bounds = Dozwolony zakres: { $min }–{ $max }.
admin-inbound-lockout = Ta zasada może natychmiast zablokować Twój własny dostęp, w tym do tego edytora. Upewnij się, że awaryjne adresy IP obejmują Twoje rzeczywiste źródło określone przez skonfigurowane zaufane serwery proxy. Nie ma automatycznego obejścia dla pętli zwrotnej ani zalogowanego administratora. Odzyskanie dostępu po zablokowaniu wymaga przywrócenia po stronie wdrożenia; zwykłe logowanie nie może go ominąć.
admin-inbound-confirm-title = Zastosować zasady dostępu przychodzącego?
admin-inbound-confirm-word = ZASTOSUJ
admin-inbound-save = Zastosuj potwierdzone zasady
admin-inbound-review = Przejrzyj zmiany
admin-inbound-refresh = Odśwież zapisane zasady i stan DNS
admin-inbound-reset = Przywróć wersję roboczą do wczytanych zasad
admin-inbound-saved = Serwer zaakceptował zasady dostępu przychodzącego.
admin-inbound-conflict = Zapisane zasady zmieniły się podczas edycji. Zresetuj wersję roboczą i odśwież, aby wczytać aktualną wersję, zanim znowu zaczniesz edytować.
admin-inbound-no-revision = Serwer nie zwrócił wersji zasad. Odśwież przed zapisem.
admin-inbound-status = Bieżące przyznania DNS
admin-inbound-status-help = To migawka z ostatniego odświeżenia stanu, a nie odliczanie ani test łączności. Odśwież, aby zobaczyć bieżące przyznania. Błąd może współistnieć z wciąż ważną wcześniejszą odpowiedzią do czasu jej pierwotnego wygaśnięcia.
admin-inbound-refreshing = Trwa ograniczone odświeżanie DNS.
admin-inbound-idle = Podczas ostatniego odczytu stanu nie trwało odświeżanie DNS.
admin-inbound-no-hosts = W włączonych zasadach nie ma aktywnych nazw hostów.
admin-inbound-hostname = Nazwa hosta
admin-inbound-state = Przyznanie dostępu
admin-inbound-addresses = Używalne adresy
admin-inbound-validity = Ważność przy ostatnim odświeżeniu
admin-inbound-error = Kod wyniku DNS
admin-inbound-usable = Używalny
admin-inbound-unavailable = Brak bieżącego przyznania
admin-inbound-seconds = { $seconds } s
admin-inbound-on = Włączone
admin-inbound-off = Wyłączone
admin-inbound-invalid-dns = Limity DNS lub liczba resolwerów wykraczają poza dozwolone granice.
admin-inbound-rescue-required = Każda włączona powierzchnia wymaga co najmniej jednego awaryjnego adresu IP lub CIDR.
admin-inbound-too-many = Zasady przekraczają skonfigurowany limit nazw hostów lub adresów dosłownych.
admin-inbound-invalid-host = Używaj nazw DNS ASCII małymi literami bez adresów URL, portów, symboli wieloznacznych, literałów IP i końcowych kropek.
