# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Usuwanie i anonimizacja
compliance-card-erasure-coverage = Zastąp identyfikatory jednej osoby w historii audytu i dziennikach oraz sprawdź, które kopie zapasowe nadal zawierają osoby usunięte.
admin-erasure-title = Usuwanie i anonimizacja
admin-erasure-intro = Anonimizacja osoby zastępuje adres i identyfikatory konta osoby w historii audytu oraz w rotowanych plikach dziennika stałym pseudonimem z kluczem. Łańcuch skrótów audytu jest ponownie zapieczętowany, więc weryfikacja nadal przechodzi. Każde usunięcie i anonimizacja pozostawia też znacznik, więc przywrócenie starszej kopii zapasowej ponownie usuwa te osoby.
admin-erasure-docs = Rejestr tego, co jest usuwane, eksportowane i zachowywane dla każdej tabeli, znajduje się w docs/admin/erasure-coverage.md.
admin-erasure-scrub = Anonimizuj osobę
admin-erasure-scrub-note = Najpierw uruchom podgląd: zlicza wiersze i linie dziennika, które by się zmieniły, i niczego nie zmienia.
admin-erasure-address = Adres
admin-erasure-preview = Podgląd
admin-erasure-run = Anonimizuj osobę
admin-erasure-dry-note = Tylko podgląd. Nic nie zostało zmienione.
admin-erasure-done-note = Zanonimizowano. Osoba występuje teraz jako { $pseudonym }.
admin-erasure-nothing = Żadna historia nie wymienia tej osoby.
admin-erasure-col-table = Tabela lub plik
admin-erasure-col-rows = Wiersze
admin-erasure-confirm-title = Zanonimizować tę osobę w historii?
admin-erasure-confirm-body = Wpisy audytu i linie dziennika wymieniające osobę są przepisywane z użyciem pseudonimu. Nie można tego cofnąć; adresu nie da się odzyskać z pseudonimu bez klucza.
admin-erasure-settings = Ustawienia
admin-erasure-keys = Klucze pseudonimów
admin-erasure-keys-note = Pseudonimy są tworzone bieżącą generacją klucza. Rotacja dodaje generację; starsze pozostają, dopóki nie usunie ich ustawiony wyżej okres przechowywania.
admin-erasure-rotate = Obróć klucz
admin-erasure-keys-none = Nie utworzono jeszcze klucza. Pierwsza anonimizacja go utworzy.
admin-erasure-col-gen = Generacja
admin-erasure-col-created = Utworzono
admin-erasure-col-state = Stan
admin-erasure-key-current = Bieżący
admin-erasure-key-retired = Wycofany, zachowany
admin-erasure-key-destroy = Wycofany, usunięty { $when }
admin-erasure-chain = Łańcuch audytu
admin-erasure-chain-note = Każdy wpis audytu zawiera skrót poprzedniego. Anonimizacja przepisuje wpisy i ponownie pieczętuje łańcuch, zapisując stary i nowy skrót czoła.
admin-erasure-seal = Zapieczętuj nowe wpisy
admin-erasure-chain-ok = Łańcuch przechodzi weryfikację: zapieczętowano wpisów: { $rows }, oczekuje na zapieczętowanie: { $unsealed }.
admin-erasure-chain-bad = Łańcuch nie przechodzi weryfikacji; pierwszy błędny wpis na pozycji { $seq }.
admin-erasure-col-time = Czas
admin-erasure-col-range = Pozycje
admin-erasure-backups = Kopie zapasowe
admin-erasure-backups-note = Kopia zapasowa zapisuje znak wodny znaczników w chwili utworzenia. Bieżący znak wodny to { $watermark }.
admin-erasure-col-label = Etykieta
admin-erasure-col-location = Lokalizacja
admin-erasure-col-watermark = Znak wodny
admin-erasure-col-marker = Usunięte od
admin-erasure-col-actions = Akcje
admin-erasure-backups-none = Nie zarejestrowano kopii zapasowych.
admin-erasure-contains-erased = Zawiera usunięte podmioty ({ $count })
admin-erasure-no-marker = Brak
admin-erasure-forget = Zapomnij
admin-erasure-register = Zarejestruj kopię zapasową
