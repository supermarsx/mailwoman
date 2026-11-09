# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Retencja
retention-intro = Reguły, które przenoszą lub usuwają Twoją pocztę po upływie określonej liczby dni. Reguła może uwzględniać folder, etykietę, nadawcę, rozmiar, stan przeczytania lub obecność załącznika. Wiadomości oflagowane i z etykietą keep są pomijane, dopóki tego nie zmienisz. Usunięte wiadomości trafiają najpierw do Kosza i są trwale usuwane po upływie okresu na cofnięcie.
retention-disabled = Reguły retencji są wyłączone na tym serwerze. Twoje istniejące reguły nie działają.
retention-never = nigdy
retention-error = Nie udało się wczytać lub zapisać ustawień retencji. Spróbuj ponownie.
retention-last-run = Ostatnie uruchomienie: { $when }
retention-next-run = następne uruchomienie za około { $days } dn.
retention-next-run-unknown = następne uruchomienie nie jest jeszcze zaplanowane
retention-enabled = Uruchamiaj moje reguły retencji
retention-trash-first = Najpierw przenoś usuniętą pocztę do Kosza
retention-trash-forced = Ten serwer zawsze najpierw przenosi pocztę do Kosza i trwale ją usuwa po upływie okresu na cofnięcie.
retention-undo = Okres na cofnięcie w dniach
retention-undo-help = Poczta przeniesiona przez regułę do Kosza pozostaje tam tak długo, zanim zostanie trwale usunięta. Od 0 do { $max } dni.
retention-pause = Wstrzymaj wszystkie reguły na 30 dni
retention-paused-until = Wstrzymano do { $when }
retention-resume = Wznów
retention-policy = Co dopuszcza ten serwer
retention-limits-both = Reguła usuwająca pocztę czeka co najmniej { $min } dni i najwyżej { $max } dni.
retention-limits-min = Reguła usuwająca pocztę czeka co najmniej { $min } dni.
retention-max-rules = Możesz mieć do { $count } reguł.
retention-permanent-allowed = Reguła może trwale usuwać pocztę, jeśli wyłączysz krok z Koszem.
retention-permanent-denied = Reguły nie mogą trwale usuwać poczty. Poczta trafia do Kosza i jest trwale usuwana po { $days } dniach.
retention-protected = Foldery, których reguły nigdy nie dotykają: { $folders }.
retention-mandatory = Ustawione przez administratora: poczta w folderze { $folder } starsza niż { $days } dni jest usuwana.
retention-rules = Twoje reguły
retention-add = Nowa reguła
retention-no-rules = Nie masz jeszcze reguł. Użyj szablonu poniżej lub dodaj własną.
retention-rule-paused = wstrzymana
retention-rule-on = Włączona
retention-rule-enabled = Reguła { $name } jest włączona
retention-edit-short = Edytuj
retention-delete = Usuń
retention-new = Nowa reguła
retention-edit = Edytuj regułę
retention-name = Nazwa
retention-action = Co zrobić
retention-action-keep = Zachowaj na zawsze
retention-action-delete = Usuń
retention-action-trash = Przenieś do Kosza
retention-action-archive = Przenieś do Archiwum
retention-action-move = Przenieś do folderu
retention-action-markRead = Oznacz jako przeczytane
retention-days = Po ilu dniach
retention-target = Folder
retention-choose-folder = Wybierz folder
retention-scope = Która poczta
retention-folder = Folder
retention-any-folder = Dowolny folder
retention-subfolders = Uwzględnij podfoldery
retention-labels = Etykiety (oddzielone przecinkami)
retention-senders = Adresy nadawców (oddzielone przecinkami)
retention-domains = Domeny nadawców (oddzielone przecinkami)
retention-min-size = Najmniejszy rozmiar w KB
retention-max-size = Największy rozmiar w KB
retention-read = Stan przeczytania
retention-read-any = Przeczytane lub nieprzeczytane
retention-read-read = Tylko przeczytane
retention-read-unread = Tylko nieprzeczytane
retention-attachment = Załączniki
retention-attachment-any = Z załącznikiem lub bez
retention-attachment-with = Z załącznikiem
retention-attachment-without = Bez załącznika
retention-exclude = Pomiń
retention-exclude-flagged = Wiadomości oflagowane (z gwiazdką)
retention-exclude-unread = Wiadomości nieprzeczytane
retention-exclude-tags = Wiadomości z tymi etykietami (oddzielonymi przecinkami)
retention-exclude-note = Poczta objęta blokadą prawną nigdy nie jest ruszana, niezależnie od tego, co tu ustawiono.
retention-role-inbox = Odebrane
retention-role-sent = Wysłane
retention-role-archive = Archiwum
retention-role-junk = Spam
retention-role-trash = Kosz
retention-role-drafts = Wersje robocze
retention-btn-preview = Podgląd
retention-save = Zapisz
retention-cancel = Anuluj
retention-preview-none = Żadna wiadomość nie zostałaby zmieniona.
retention-preview-count = Zmieniłoby to wiadomości: { $count }.
retention-preview-truncated = Na potrzeby tego podglądu odczytano tylko część skrzynki.
retention-preview-sample = Przykłady
retention-no-subject = (bez tematu)
retention-do-trash = do Kosza
retention-do-move = przeniesiono
retention-do-delete = usunięto
retention-do-markRead = oznaczono jako przeczytane
retention-skipped-legalHold = Twoje konto jest objęte blokadą prawną. Nic nie zostało zmienione.
retention-skipped-restricted = Przetwarzanie Twojego konta jest ograniczone. Nic nie zostało zmienione.
retention-skipped-noRules = Nie masz aktywnych reguł, a administrator nie ustawił żadnych.
retention-error-name = Nadaj regule nazwę.
retention-error-days = Liczba dni musi być liczbą całkowitą od 0 do 3650.
retention-error-min = Reguła usuwająca pocztę musi czekać co najmniej { $days } dni.
retention-error-max = Reguła usuwająca pocztę może czekać najwyżej { $days } dni.
retention-error-target = Wybierz folder, do którego mają być przenoszone wiadomości.
retention-summary-keep = zachowaj na zawsze
retention-summary-delete = po { $days } dniach do Kosza, a następnie trwale usunięte
retention-summary-delete-permanent = po { $days } dniach trwale usunięte
retention-summary-trash = po { $days } dniach do Kosza
retention-summary-archive = po { $days } dniach do Archiwum
retention-summary-move = po { $days } dniach do folderu
retention-summary-markRead = po { $days } dniach oznaczone jako przeczytane
retention-templates = Szablony
retention-templates-help = Szablon dodaje gotową regułę. Sprawdź jej etykiety i liczbę dni, zanim na niej polegasz.
retention-adopt = Dodaj tę regułę
retention-run-heading = Uruchom teraz
retention-preview-all = Podgląd wszystkich reguł
retention-btn-run = Uruchom moje reguły teraz
retention-run-result = Zmieniono wiadomości: { $count }.
retention-history = Ostatnie uruchomienia
retention-history-when = Kiedy
retention-history-matched = Dopasowano
retention-history-changed = Zmieniono
retention-history-errors = Błędy
retention-context-action = Retencja…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Kalendarze

pim-cal-days = Kilka dni po zakończeniu wydarzenia

pim-cal-enabled = Usuń zakończone wydarzenia

pim-cal-help = Usuń wydarzenia z wybranych kalendarzy po określonej liczbie dni po ich zakończeniu. Wydarzenia cykliczne nigdy nie są usuwane.

pim-cal-title = Wydarzenia w kalendarzu

pim-days-range = Wybierz pomiędzy { $min } i { $max } dni.

pim-held-title = Usunięte notatki, które możesz przywrócić

pim-held-until = Przechowywane do { $when }

pim-limits = Od { $min } do { $max } dni.

pim-notes-days = Dni od ostatniej zmiany notatki

pim-notes-enabled = Usuń nietknięte notatki

pim-notes-help = Usuń notatki, których nie zmieniłeś przez określoną liczbę dni. Przypięte notatki zostaną zachowane. Usuniętą notatkę można przywrócić przez { $days } dni.

pim-notes-title = Notatki

pim-off = Wyłączone przez administratora.

pim-preview = Podgląd

pim-restore = Przywróć

pim-result = { $events } zdarzenia i { $notes } notatki.

pim-run = Uciekaj teraz

pim-run-title = Uciekaj teraz

pim-save = Zapisz

pim-skipped = { $count } opcji zostało pominiętych, ponieważ administrator je wyłączył.

pim-untitled = Notatka bez tytułu

retention-action-stripAttachments = Usuń duże załączniki

retention-do-stripAttachments = załączniki usunięte

retention-error-strip-disabled = Usuwanie załączników zostało wyłączone przez administratora.

retention-error-strip-min = Reguła usuwająca załączniki musi poczekać co najmniej { $days } dni.

retention-fd-action = Co dzieje się z pocztą w numerze { $folder }

retention-fd-days = Kilka dni przed przetworzeniem { $folder }

retention-fd-help = Ustaw, jak długo poczta pozostaje w folderze, zanim zostanie przeniesiona lub usunięta. Folder ustawiony na „Dziedzicz” nie ma własnego folderu domyślnego; w dalszym ciągu obowiązują zasady powyższe oraz limity administratora. Podfoldery podążają za folderem.

retention-fd-inherit = Dziedzicz

retention-fd-mandatory = Administrator przechowuje ten folder przez { $days } dni

retention-fd-protected = Chronione przez administratora

retention-fd-save = Zapisz domyślne ustawienia folderu

retention-fd-title = Domyślne ustawienia folderu

retention-rule-strip-off = Wyłączone przez administratora

retention-strip-allowed = Reguła może usuwać załączniki o rozmiarze co najmniej { $kb } KB z poczty starszej niż { $days } dni. Krótka notatka zastępuje każdy załącznik.

retention-strip-denied = Usuwanie załączników zostało wyłączone przez administratora.

retention-strip-done = Usunięto załączniki z wiadomości { $count }, zwalniając { $size }.

retention-strip-inline-kept = Obrazy pokazane w wiadomości zostaną zachowane.

retention-strip-inline-removed = Obrazy widoczne w wiadomości również zostaną usunięte.

retention-strip-limits = Usuwa załączniki o rozmiarze co najmniej { $kb } KB z poczty starszej niż { $days } dni. Oryginał pozostaje w Koszu przez { $undo } dni.

retention-strip-preview = Wiadomości { $count } utraciłyby załączniki, uwalniając { $size }.

retention-strip-preview-estimate = Około { $count } wiadomości spowoduje utratę załączników, uwalniając maksymalnie { $size }.

retention-strip-skipped = { $count } reguły usuwające załączniki zostały pominięte, ponieważ administrator wyłączył tę opcję.

retention-summary-stripAttachments = po { $days } dniach usunięto duże załączniki
