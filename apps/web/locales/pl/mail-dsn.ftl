# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Raport o stanie dostarczenia
dsn-unverified = Ten raport opisuje próbę dostarczenia podjętą przez nadawcę. Zawarte w nim informacje nie zostały niezależnie zweryfikowane. Dostarczenie nie oznacza, że wiadomość została przeczytana.
dsn-reporting-mta = Serwer raportujący:
dsn-original-recipient = Pierwotny odbiorca:
dsn-remote-mta = Serwer zdalny:
dsn-last-attempt = Ostatnia próba:
dsn-retry-until = Nadawca ponawia próby do:
dsn-action-failed = Zgłoszono niepowodzenie
dsn-action-delayed = Zgłoszono opóźnienie
dsn-action-delivered = Zgłoszono dostarczenie
dsn-action-relayed = Zgłoszono przekazanie
dsn-action-expanded = Zgłoszono rozwinięcie listy

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Skorzystaj z diagnostyki serwera, prosząc o pomoc swojego dostawcę poczty e-mail.

dsn-advice-1 = Sprawdź adresy nadawcy i odbiorcy. W razie potrzeby potwierdź aktualny adres odbiorcy.

dsn-advice-2 = Jeśli problem ze skrzynką pocztową będzie się powtarzał, skontaktuj się z odbiorcą w inny sposób.

dsn-advice-3 = Udostępnij ten raport administratorowi systemu poczty odbierającej, jeśli problem będzie się powtarzał.

dsn-advice-4 = Jeśli problem będzie się powtarzał, poproś administratora poczty o sprawdzenie łączności, DNS i tras przekazywania.

dsn-advice-5 = Udostępnij diagnostykę swojemu dostawcy poczty e-mail, aby mógł sprawdzić zgodność i ograniczenia SMTP.

dsn-advice-6 = Sprawdź format wiadomości, adresy międzynarodowe i załączniki pod kątem wymagań odbiorcy.

dsn-advice-7 = Udostępnij diagnostykę administratorowi poczty, aby określić dokładną regułę zabezpieczeń lub uwierzytelniania.

dsn-advice-success = Sprawdź powyższą zgłoszoną akcję dostarczenia, czy wiadomość została dostarczona, przekazana lub rozszerzona na listę.

dsn-category-0 = Serwer podał nierozpoznany kod szczegółowy lub specyficzny dla dostawcy. Poniższa diagnostyka może wyjaśnić dokładną przyczynę.

dsn-category-1 = To jest kod związany z adresem. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-2 = To jest kod skrzynki pocztowej odbiorcy. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-3 = To jest kod systemu odbierającego pocztę. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-4 = Jest to kod sieci lub routingu. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-5 = To jest kod protokołu pocztowego. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-6 = To jest treść wiadomości lub kod formatu. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane.

dsn-category-7 = Jest to kod zabezpieczający lub kod zasad. Dokładne szczegóły zależą od dostawcy lub nie są rozpoznawane; samo w sobie nie oznacza to, że wiadomość jest spamem.

dsn-class-2 = Sukces: serwer zgłasza pozytywny krok dostarczania. Nie oznacza to, że wiadomość została przeczytana.

dsn-class-4 = Tymczasowa awaria: późniejsza próba może zakończyć się sukcesem. Nadawca może w dalszym ciągu ponowić zgłoszenie opóźnienia; raport o niepowodzeniu oznacza, że ​​próba została zakończona.

dsn-class-5 = Trwała awaria: ponowne wysłanie tej samej wiadomości w niezmienionej postaci prawdopodobnie nie zadziała. Przed ponownym wysłaniem usuń przyczynę.

dsn-meaning-title = Co oznacza ten kod

dsn-next-step = Co robić

dsn-provider-barracuda = Barakuda

dsn-provider-detected = Interpretacja oparta na diagnostyce serwera lub nazwie hosta:

dsn-provider-exchange = Microsoft Exchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmaila/Google Workspace

dsn-provider-help = Dokumentacja dostawcy

dsn-provider-mimecast = Mimecast

dsn-provider-proofpoint = Dowód

dsn-provider-relay = Przekaźnik SMTP

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnostyka serwera

dsn-status-0-0 = Serwer nie podał bardziej konkretnego powodu.

dsn-status-1-0 = Wystąpił problem z adresem e-mail.

dsn-status-1-1 = Skrzynka pocztowa odbiorcy nie istnieje. Potwierdź adres z odbiorcą.

dsn-status-1-10 = Domena odbiorcy wyraźnie deklaruje, że nie przyjmuje żadnych wiadomości e-mail (null MX). Użyj innego adresu.

dsn-status-1-2 = Domena odbiorcy nie może odebrać tej poczty. Sprawdź część po @.

dsn-status-1-3 = Adres odbiorcy ma nieprawidłowy format. Sprawdź, czy nie ma literówek lub dodatkowych spacji.

dsn-status-1-4 = Serwer nie może zidentyfikować unikalnego odbiorcy dla tego adresu.

dsn-status-1-5 = Adres odbiorcy został zaakceptowany jako ważny.

dsn-status-1-6 = Ta skrzynka pocztowa została przeniesiona i nie jest dostępny żaden adres do przekazywania dalej. Zapytaj o nowy adres.

dsn-status-1-7 = Adres nadawcy ma nieprawidłowy format. Sprawdź adres wysyłkowy konta.

dsn-status-1-8 = Domena nadawcy jest nieprawidłowa lub nie może odebrać poczty zwrotnej.

dsn-status-1-9 = Wiadomość została przekazana do systemu, który nie może dostarczać dalszych raportów o doręczeniu.

dsn-status-2-0 = W skrzynce pocztowej odbiorcy występuje nieokreślony problem.

dsn-status-2-1 = Skrzynka pocztowa odbiorcy jest wyłączona lub nie przyjmuje wiadomości. Skontaktuj się z odbiorcą w inny sposób.

dsn-status-2-2 = Skrzynka pocztowa odbiorcy jest pełna. Odbiorca musi zwolnić miejsce lub zwiększyć swój przydział.

dsn-status-2-3 = Wiadomość przekracza limit rozmiaru tego odbiorcy. Zmniejsz liczbę załączników lub użyj linku do udostępniania.

dsn-status-2-4 = Serwer nie mógł rozpoznać członków listy mailingowej. Skontaktuj się z właścicielem listy.

dsn-status-3-0 = System poczty odbiorczej zgłosił nieokreślony problem.

dsn-status-3-1 = Na serwerze odbierającym pocztę zabrakło miejsca. Administrator musi zwolnić miejsce.

dsn-status-3-2 = Serwer odbierający nie przyjmuje poczty, na przykład podczas konserwacji lub przeciążenia.

dsn-status-3-3 = System odbierający nie obsługuje funkcji wymaganej przez tę wiadomość.

dsn-status-3-4 = Wiadomość przekracza limit rozmiaru serwera odbierającego. Zmniejsz liczbę załączników lub użyj linku do udostępniania.

dsn-status-3-5 = Problem z konfiguracją serwera pocztowego uniemożliwia dostarczenie. Poproś administratora poczty o zbadanie problemu.

dsn-status-3-6 = Serwer przyjął wiadomość, ale zmienił żądany priorytet dostarczenia.

dsn-status-4-0 = Zgłoszono problem z siecią lub routingiem poczty bez dalszych szczegółów.

dsn-status-4-1 = Serwer odbierający nie odpowiedział na próbę połączenia.

dsn-status-4-2 = Połączenie zostało nawiązane, ale zostało przerwane przed zakończeniem przesyłania poczty.

dsn-status-4-3 = DNS lub wyszukiwanie katalogu nie powiodło się. Administrator poczty może być zmuszony sprawdzić rozpoznawanie nazw.

dsn-status-4-4 = Serwer nie mógł znaleźć trasy do systemu pocztowego odbiorcy.

dsn-status-4-5 = Przeciążenie ruchu pocztowego uniemożliwiło dostarczenie.

dsn-status-4-6 = Wykryto pętlę przekazywania. Poproś administratora o sprawdzenie reguł przekazywania i routingu.

dsn-status-4-7 = Upłynął termin dostawy. Przed ponownym wysłaniem sprawdź diagnostykę dotyczącą podstawowego problemu.

dsn-status-5-0 = Serwery pocztowe nie mogły dokończyć wymiany protokołów dostarczania.

dsn-status-5-1 = Serwer pocztowy odrzucił nieobsługiwane polecenie lub polecenie wysłane w niewłaściwej kolejności.

dsn-status-5-2 = Serwer pocztowy nie mógł zrozumieć składni polecenia.

dsn-status-5-3 = Wiadomość ma zbyt wielu odbiorców. Wyślij do mniejszych grup.

dsn-status-5-4 = Serwer pocztowy odrzucił parametry polecenia.

dsn-status-5-5 = Serwery używają niekompatybilnych wersji protokołu pocztowego.

dsn-status-5-6 = Odpowiedź uwierzytelniająca była za długa dla serwera. Poproś administratora poczty o sprawdzenie ustawień uwierzytelniania.

dsn-status-6-0 = Serwer zgłosił nieokreślony problem z treścią lub formatem wiadomości.

dsn-status-6-1 = System odbierający nie obsługuje typu treści tej wiadomości.

dsn-status-6-10 = Ten przestarzały kod oznacza, że odpowiedź międzynarodowa (UTF-8) jest wymagana, ale nie jest obsługiwana, tak jak w przypadku kodu x.6.8.

dsn-status-6-2 = Dostawa wymaga konwersji treści, której zabraniają zasady nadawcy lub serwera.

dsn-status-6-3 = Dostawa wymaga konwersji treści, której serwer nie jest w stanie wykonać.

dsn-status-6-4 = Konwersja treści spowodowała utratę niektórych informacji. Sprawdź oryginalną wiadomość i diagnostykę serwera.

dsn-status-6-5 = Serwer uległ awarii podczas konwertowania treści wiadomości.

dsn-status-6-6 = Serwer nie mógł pobrać treści wiadomości z innego systemu.

dsn-status-6-7 = Międzynarodowy adres e-mail nie jest obsługiwany w przypadku tego nadawcy lub odbiorcy. Zapytaj o alternatywny adres.

dsn-status-6-8 = Serwer potrzebuje międzynarodowej odpowiedzi (UTF-8), której system wysyłający nie może przyjąć.

dsn-status-6-9 = Wiadomość zawiera międzynarodowe (UTF-8) nagłówki, których nie można dostarczyć przynajmniej do jednego odbiorcy.

dsn-status-7-0 = Reguła bezpieczeństwa lub zasady dotyczące poczty uniemożliwiają normalne dostarczenie. Sprawdź diagnostykę serwera, aby uzyskać dokładną regułę.

dsn-status-7-1 = Serwer odmówił pozwolenia na dostarczenie, prawdopodobnie z powodu ograniczeń filtrowania lub przekazywania. Poproś administratora poczty o sprawdzenie diagnostyki.

dsn-status-7-10 = Serwer wymaga szyfrowanego połączenia lub silniejszej ochrony przed uwierzytelnieniem.

dsn-status-7-11 = Ta metoda uwierzytelniania wymaga szyfrowanego połączenia SMTP (TLS).

dsn-status-7-12 = Konto wymaga metody uwierzytelniania lub migracji hasła. Skontaktuj się z dostawcą konta.

dsn-status-7-13 = Konto wysyłające jest wyłączone. Skontaktuj się z administratorem, aby przywrócić dostęp.

dsn-status-7-14 = Serwer potrzebuje skonfigurowanej relacji zaufania, aby pobrać wiadomość. Skontaktuj się z jego administratorem.

dsn-status-7-15 = Priorytet wiadomości jest niższy od bieżącego minimum serwera odbierającego.

dsn-status-7-16 = Wiadomość jest za duża dla żądanego priorytetu. Zmniejsz jego rozmiar lub zapytaj administratora.

dsn-status-7-17 = Skrzynka pocztowa odbiorcy zmieniła właściciela od wymaganej daty. Potwierdź odbiorcę przed ponownym wysłaniem.

dsn-status-7-18 = Domena odbiorcy zmieniła właściciela od wymaganej daty. Potwierdź odbiorcę przed ponownym wysłaniem.

dsn-status-7-19 = Serwer nie mógł zweryfikować ciągłej własności adresu odbiorcy. Zapytaj administratora o sprawdzenie ważności odbiorcy.

dsn-status-7-2 = Nadawca nie może publikować postów na tej liście mailingowej. Skontaktuj się z właścicielem listy.

dsn-status-7-20 = Żaden podpis DKIM nie przeszedł weryfikacji. Poproś administratora poczty nadawcy o sprawdzenie podpisu i DNS.

dsn-status-7-21 = Podpis DKIM został przekazany, ale polityka serwera odbierającego go nie zaakceptowała.

dsn-status-7-22 = Żaden zaakceptowany podpis DKIM nie jest zgodny z adresem Od. Poproś administratora poczty o sprawdzenie wyrównania domeny podpisującej.

dsn-status-7-23 = SPF nie powiodło się: serwer wysyłający nie został autoryzowany przez politykę SPF domeny nadawcy.

dsn-status-7-24 = Serwer napotkał błąd podczas sprawdzania SPF. Poproś administratora poczty o sprawdzenie rekordów SPF i DNS.

dsn-status-7-25 = Serwer wysyłający nie przeszedł pomyślnie sprawdzenia odwrotnego DNS przez odbiorcę. Jego administrator musi sprawdzić zapisy DNS adresu IP.

dsn-status-7-26 = Wiele kontroli uwierzytelnienia wiadomości nie powiodło się. Diagnostyka może zidentyfikować problemy z zasadami SPF, DKIM lub DMARC.

dsn-status-7-27 = Domena nadawcy deklaruje, że nie przyjmuje wiadomości zwrotnej (null MX), którą odbiorca odrzuca.

dsn-status-7-28 = System odbiorczy wykrył możliwą powódź obraźliwych wiadomości.

dsn-status-7-29 = Weryfikacja ARC nie powiodła się, więc odbiorca nie mógł zaakceptować zarejestrowanego łańcucha uwierzytelniania wiadomości.

dsn-status-7-3 = Dostarczanie wymaga konwersji formatu zabezpieczeń, której serwer nie może wykonać.

dsn-status-7-30 = Wiadomość wymaga REQUIRETLS, ale następny serwer pocztowy jej nie obsługuje. Poproś administratora o sprawdzenie obsługi bezpiecznej dostawy.

dsn-status-7-4 = System dostarczania nie obsługuje wymaganej funkcji bezpieczeństwa.

dsn-status-7-5 = Serwer nie mógł zweryfikować ani odszyfrować wiadomości, na przykład z powodu braku klucza lub nieprawidłowego klucza.

dsn-status-7-6 = Serwer nie obsługuje algorytmu kryptograficznego wymaganego przez wiadomość.

dsn-status-7-7 = Kontrola integralności wskazuje, że wiadomość została zmieniona lub uszkodzona.

dsn-status-7-8 = Serwer wysyłający odrzucił dane logowania. Sprawdź ustawienia uwierzytelniania konta.

dsn-status-7-9 = Metoda uwierzytelniania jest słabsza niż pozwala na to serwer. Wymagana jest silniejsza metoda.

dsn-vendor-advice-account = Poproś administratora wysyłającego o zbadanie możliwych nadużyć na koncie, zabezpieczenie konta i poproszenie o usunięcie ograniczenia.

dsn-vendor-advice-admin = Udostępnij tę diagnostykę administratorowi poczty w organizacji, aby sprawdzić zgodną konfigurację lub zasady.

dsn-vendor-advice-auth = Sprawdź obsługiwaną metodę uwierzytelniania i autoryzację SMTP konta. Dokonaj wymaganego logowania za pośrednictwem oficjalnej strony konta dostawcy.

dsn-vendor-advice-authentication = Poproś administratora nadawcy o sprawdzenie SPF, DKIM, DMARC i wyrównanie domeny przy użyciu dokładnej diagnostyki.

dsn-vendor-advice-connector = Poproś administratora o sprawdzenie autoryzacji przekazywania, zgodności złącza IP/certyfikatu, zaakceptowanych domen i rzeczywistej trasy poczty.

dsn-vendor-advice-content = Przejrzyj dzienniki diagnostyczne i dzienniki bramy wraz z administratorem odbiorcy, aby zidentyfikować zablokowaną zawartość i obowiązujące zasady.

dsn-vendor-advice-dns = Poproś administratora poczty wysyłającej lub dostawcę IP o poprawienie DNS, odwrócenie DNS i tożsamość serwera SMTP.

dsn-vendor-advice-domain = Poproś administratora Microsoft 365 o zarejestrowanie i zweryfikowanie domeny wysyłającej oraz sprawdzenie limitów wysyłania dzierżawcy.

dsn-vendor-advice-format = Poproś aplikację wysyłającą lub administratora poczty o poprawienie generowanej wiadomości, a następnie utwórz nową wiadomość.

dsn-vendor-advice-rate = Zmniejsz liczbę wysyłanych przesyłek i poczekaj, aż limit lub ograniczenie zostaną usunięte. Sprawdź zgłoszoną akcję, zanim założysz, że ta próba nadal znajduje się w kolejce.

dsn-vendor-advice-recipient-admin = Skontaktuj się z odbiorcą lub właścicielem grupy w inny sposób i poproś administratora o zapoznanie się z odpowiednimi uprawnieniami lub zasadami.

dsn-vendor-advice-reputation = Poproś administratora wysyłającego o sprawdzenie niechcianego ruchu i reputacji, a następnie postępuj zgodnie z oficjalnym procesem sprawdzenia lub usunięcia dostawcy.

dsn-vendor-advice-route = Poproś administratora poczty o sprawdzenie miejsca docelowego, reguł przekazywania, usługi pocztowej i zapory sieciowej/ścieżki sieciowej.

dsn-vendor-advice-tls = Poproś administratorów poczty o naprawę TLS, certyfikatów lub rekordów zabezpieczeń DNS, aby wymagana bezpieczna trasa mogła się powieść.

dsn-vendor-barracuda-reputation = Usługa reputacji firmy Barracuda wyświetla lub blokuje adres wysyłający IP.

dsn-vendor-certificate-chain = Nie można zweryfikować zdalnego certyfikatu TLS względem wymaganego zaufania lub zasad MTA-STS.

dsn-vendor-certificate-expired = Docelowy serwer pocztowy przedstawił wygasły certyfikat TLS.

dsn-vendor-certificate-host = Certyfikat docelowy TLS nie jest zgodny z wymaganą nazwą hosta serwera pocztowego.

dsn-vendor-connection-refused = Serwer docelowy odmówił połączenia sieciowego. Należy sprawdzić usługę pocztową, zaporę sieciową lub trasę sieciową.

dsn-vendor-content-security = Brama odrzuciła treść wiadomości lub załącznik zgodnie ze swoimi regułami filtrowania zabezpieczeń. Sprawdź diagnostykę lub dzienniki dotyczące konkretnego elementu.

dsn-vendor-credentials = Usługa SMTP odrzuciła podane dane logowania.

dsn-vendor-dane = Certyfikat TLS miejsca docelowego lub konfiguracja TLSA nie przeszły weryfikacji DANE.

dsn-vendor-dkim = Odbiorca nie zaakceptował konfiguracji uwierzytelniania lub podpisywania wiadomości DKIM.

dsn-vendor-dmarc = Polityka DMARC odbiorcy nie została spełniona. Sprawdź uwierzytelnianie SPF lub DKIM i zgodność z widoczną domeną From.

dsn-vendor-dnssec = Nie można zweryfikować rekordów DNSSEC domeny docelowej.

dsn-vendor-exchange-auth = Aplikacja lub urządzenie nie zostało poprawnie uwierzytelnione na potrzeby przesłania SMTP Microsoft 365.

dsn-vendor-exchange-connector = Firma Microsoft nie mogła autoryzować przekaźnika za pośrednictwem skonfigurowanego łącznika dzierżawy. Jego certyfikat, wysyłanie IP lub zaakceptowana domena mogą już nie pasować.

dsn-vendor-exchange-domain-blocked = Firma Microsoft ograniczyła tę domenę wysyłającą po wykryciu podejrzenia spamu.

dsn-vendor-exchange-forward-relay = System niższego szczebla odrzucił przesłaną lub przekazaną pocztę. Na uwierzytelnianie mogło mieć wpływ przekazywanie lub brama firmy innej niż Microsoft.

dsn-vendor-exchange-forwarding = Zasady Microsoft 365 blokują automatyczne przekazywanie dalej do odbiorców zewnętrznych. Administrator organizacji musi zapoznać się z polityką przekazywania.

dsn-vendor-exchange-generic = Exchange zgłosiła ogólne odrzucenie dostawy. Diagnostyka identyfikuje faktyczny problem, który może dotyczyć formatowania wiadomości lub zasad dotyczących załączników.

dsn-vendor-exchange-group-allow = Nadawcy nie ma na liście dozwolonych nadawców tej grupy Exchange.

dsn-vendor-exchange-group-internal = Ta grupa dystrybucyjna Exchange odrzuca nadawców spoza swojej organizacji, chyba że jej właściciel lub administrator na to pozwoli.

dsn-vendor-exchange-internal-only = Ten odbiorca Exchange akceptuje uwierzytelnionych nadawców z własnej organizacji; wiadomość ta została potraktowana jako zewnętrzna lub nieuwierzytelniona.

dsn-vendor-exchange-ipv6 = Miejsce docelowe nie akceptuje tej dostawy powyżej IPv6. Administrator wysyłający musi sprawdzić trasę lub skorzystać z zaakceptowanego środka transportu.

dsn-vendor-exchange-journaling = Organizacja próbuje kronikować wiadomości lokalne na platformie Microsoft 365 bez wymaganej konfiguracji archiwum.

dsn-vendor-exchange-onmicrosoft-limit = Organizacja osiągnęła limit odbiorców zewnętrznych dla swojej domeny wysyłającej onmicrosoft.com. Regularne wysyłanie powinno korzystać ze zweryfikowanej domeny niestandardowej.

dsn-vendor-exchange-public-folder = Ten folder publiczny Exchange nie akceptuje tego zewnętrznego lub nieuwierzytelnionego nadawcy.

dsn-vendor-exchange-receive-rate = Ten odbiorca Exchange osiągnął godzinowy limit odbioru dla wszystkich nadawców.

dsn-vendor-exchange-recipient = Exchange nie mógł rozpoznać tego odbiorcy w swoim katalogu. Przed utworzeniem nowej wiadomości sprawdź adres i usuń wszystkie nieaktualne wpisy autouzupełniania.

dsn-vendor-exchange-recipient-access = Exchange odrzuciła dostęp do tej domeny odbiorcy lub docelowej. Sprawdź adres odbiorcy i konfigurację akceptowanej domeny organizacji.

dsn-vendor-exchange-recipient-rate = Exchange odebrał zbyt wiele wiadomości od tego nadawcy do tego odbiorcy w swoim oknie godzinowym.

dsn-vendor-exchange-sender-blocked = Firma Microsoft ograniczyła to konto wysyłające po wykryciu podejrzenia spamu lub nadużycia. Może to wymagać od administratora zabezpieczenia i odblokowania konta.

dsn-vendor-exchange-temporary-restriction = Firma Microsoft tymczasowo ogranicza wysyłanie IP lub ruch na czas oceny podejrzanej aktywności.

dsn-vendor-exchange-tenant-blocked = Firma Microsoft ograniczyła wysyłanie dla tej organizacji, ponieważ jej ruch uznano za podejrzany.

dsn-vendor-exchange-tenant-blocklist = Adres odbiorcy lub domena jest blokowana przez listę dozwolonych/blokowanych dzierżaw organizacji nadawcy. Jeden zablokowany odbiorca może odrzucić całą wiadomość.

dsn-vendor-exchange-tenant-limit = Organizacja Microsoft 365 osiągnęła swój kroczący dzienny limit dla odbiorców zewnętrznych.

dsn-vendor-exchange-unregistered = Firma Microsoft odrzuciła wysyłanie z domeny, która nie jest prawidłowo zarejestrowana i zweryfikowana w dzierżawie.

dsn-vendor-expired = Usługa wysyłania zrezygnowała po upływie okna czasu dostawy. Sprawdź diagnostykę pod kątem błędu połączenia lub routingu, który spowodował opóźnienie.

dsn-vendor-fortimail-relay = FortiMail odrzucił tę dostawę zgodnie ze swoimi zasadami kontroli dostępu lub przekazywania. Administrator bramy musi sprawdzić pasującą regułę odbierania.

dsn-vendor-gmail-app-password = Google wymaga hasła do aplikacji lub obsługiwanej metody autoryzacji konta dla tego loginu SMTP.

dsn-vendor-gmail-auth-method = Firma Google odrzuciła nieobsługiwaną metodę uwierzytelniania, taką jak wycofana metoda XOAUTH.

dsn-vendor-gmail-authentication = Google nie zaakceptował uwierzytelnienia nadawcy. Diagnostyka rozróżnia brakujący SPF/DKIM, SPF awarię twardą i odrzucenie DMARC.

dsn-vendor-gmail-browser-login = Przed zezwoleniem na logowanie SMTP Google wymaga zalogowania się na konto lub sprawdzenia bezpieczeństwa w przeglądarce.

dsn-vendor-gmail-custom-policy = Niestandardowa reguła poczty administratora Google Workspace odrzuciła wiadomość. Nie musi to oznaczać, że filtr spamu Google go odrzucił.

dsn-vendor-gmail-dmarc-record = Google wymaga rekordu DMARC z zadeklarowaną polityką dla tej domeny wysyłającej.

dsn-vendor-gmail-rate = Firma Google zastosowała limit wysyłania lub przekazywania albo wykryła nietypową częstotliwość wiadomości. Diagnostyka identyfikuje dotknięte konto, IP lub domenę.

dsn-vendor-gmail-relay = Google Workspace nie autoryzował wysyłania IP, domeny ani konta w ramach usługi przekaźnika SMTP.

dsn-vendor-gmail-reputation = Filtrowanie Google odrzuciło wiadomość, ponieważ uznało ją za niechcianą lub reputacja domeny wysyłającej IP/domeny jest zbyt niska.

dsn-vendor-gmail-spf-record = Google uważa konfigurację domeny nadawcy SPF za nieprawidłową lub podejrzaną.

dsn-vendor-greylist = Brama tymczasowo odroczyła tę dostawę, korzystając z szarej listy. Prawidłowo skonfigurowany serwer wysyłający zwykle podejmuje próbę później.

dsn-vendor-headers = Usługa odbierająca odrzuciła nieprawidłowe lub brakujące nagłówki wiadomości. Aplikacja wysyłająca musi wygenerować poprawnie sformatowaną pocztę.

dsn-vendor-helo = Serwer wysyłający użył niedopuszczalnej tożsamości HELO/EHLO. Jego powitanie musi poprawnie identyfikować serwer wysyłający.

dsn-vendor-ip-block = Usługa odbierająca odrzuciła wysłanie IP ze względu na swoją reputację lub wpis na liście zablokowanych.

dsn-vendor-mimecast-block = Mimecast odrzucił nadawcę lub domenę zgodnie z zasadami zablokowanego nadawcy lub zasadami administracyjnymi.

dsn-vendor-mimecast-content = Zasady badania treści Mimecast odrzuciły wiadomość.

dsn-vendor-mimecast-spoof = Polityka Mimecast zapobiegająca fałszowaniu odrzuciła tożsamość nadawcy.

dsn-vendor-mta-sts-host = Docelowy serwer pocztowy nie jest dozwolony przez politykę domeny MTA-STS.

dsn-vendor-proofpoint-block = Proofpoint zablokował wysyłanie IP lub domeny po wykryciu spamu lub złośliwej aktywności.

dsn-vendor-proofpoint-throttle = Proofpoint tymczasowo odłożył wysłanie IP, ponieważ jego wzór wysłania został uznany za nietypowy.

dsn-vendor-ptr = Wysyłającemu IP brakuje akceptowalnej odwrotności DNS lub jego nazwa PTR nie jest przekształcana z powrotem na tę samą IP.

dsn-vendor-recipient-ip-block = Organizacja odbiorcy wyraźnie blokuje adres wysyłający IP. Jego administrator kontroluje ten blok lokalny.

dsn-vendor-relay = Serwer pocztowy nie przekaże tej wiadomości do miejsca docelowego. Sprawdź uwierzytelnianie SMTP, zaufane adresy IP wysyłania, łączniki i akceptowane domeny.

dsn-vendor-routing-loop = Wiadomość weszła w pętlę przesyłania lub routingu i przekroczyła dozwoloną liczbę przeskoków.

dsn-vendor-sending-limit = Konto wysyłające przekroczyło limit wiadomości lub odbiorców. Powtarzające się natychmiastowe ponowne wysłanie nie usunie limitu.

dsn-vendor-spf = Odbiorca nie zaakceptował uwierzytelnienia SPF dla domeny wysyłającej IP i domeny nadawcy kopertowego.

dsn-vendor-starttls = Nie można ustanowić wymaganego szyfrowanego połączenia SMTP lub nie było ono obsługiwane przez następny serwer.

dsn-vendor-yahoo-deferred = Yahoo tymczasowo odkładało tę wiadomość, zwykle podczas oceny liczby, reputacji lub skarg odbiorców. Sprawdź diagnostykę pod kątem podanej przyczyny.

dsn-vendor-yahoo-policy = Yahoo odrzuciło wiadomość zgodnie ze swoimi zasadami dotyczącymi poczty. W grę może wchodzić uwierzytelnianie, dopasowanie DMARC lub treść wiadomości; diagnostyka podaje konkretny powód.
