# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: rules. Source: ../en/rules.ftl

rules-title = Reguły i filtry
rules-intro = Zarządzaj skryptami filtrowania i regułami wizualnymi. Dostępne funkcje zależą od konta i ustawień administratora.
rules-empty = Brak reguł.
rules-unavailable = Ten serwer pocztowy nie obsługuje reguł poczty.
rules-load-failed = Nie udało się wczytać reguł.
rules-new = Nowa reguła
rules-list-label = Reguły poczty
rules-enabled = Włączona
rules-delete-named = Usuń regułę { $name }

# Builder
rules-builder-title = Kreator reguł
rules-builder-hint = Wybierz regułę do edycji lub utwórz nową.
rules-name-label = Nazwa reguły
rules-match-legend = Kiedy uruchamiać
rules-match-all = Spełnione wszystkie warunki
rules-match-any = Spełniony dowolny warunek
rules-conditions-legend = Warunki
rules-actions-legend = Działania
rules-add-condition = Dodaj warunek
rules-add-action = Dodaj działanie
rules-remove-condition = Usuń warunek
rules-remove-action = Usuń działanie
rules-cond-type-label = Pole
rules-cond-op-label = Operator
rules-cond-value-label = Wartość do dopasowania
rules-act-type-label = Działanie
rules-act-value-label = Wartość działania
rules-save = Zapisz regułę
rules-editing = Edycja: { $name }
rules-save-refused = Reguła nie została zapisana: { $reason }

# Condition fields and operators
rules-cond-from = Od
rules-cond-to = Do
rules-cond-subject = Temat
rules-cond-thread-unsupported = Wątek { $op } { $value }: warunki dotyczące wątków nie są obsługiwane, więc reguły nie można zapisać, dopóki ten warunek nie zostanie usunięty.
rules-op-contains = zawiera
rules-op-is = jest równe

# Actions
rules-act-move = Przenieś do folderu
rules-act-tag = Dodaj tag
rules-act-archive = Archiwizuj
rules-act-suppress = Wycisz powiadomienie
rules-act-stop = Zakończ przetwarzanie

# Where it runs
rules-runs-server = Serwer (Sieve)
rules-runs-server-detail = Przesyłana na serwer pocztowy i tam stosowana.
rules-runs-engine = Silnik
rules-runs-engine-detail = Stosowane podczas synchronizacji nowych wiadomości; odpowiedzi i przenoszenie czekają na następną synchronizację.

# Panes
rules-panes-label = Narzędzia reguł
rules-tab-builder = Kreator
rules-tab-raw = Surowy Sieve
rules-tab-dryrun = Próba

# Raw editor
rules-raw-label = Surowy kod Sieve
rules-raw-reset = Przywróć wygenerowany
rules-lint-clean = Nie znaleziono problemów.

# Dry run
rules-dryrun-title = Próba
rules-dryrun-help = Wpisz przykładową wiadomość, aby sprawdzić, które reguły zostałyby uruchomione.
rules-dryrun-results = Wyniki próby
rules-dryrun-none = Brak włączonych reguł do sprawdzenia.
rules-dryrun-nomatch = Brak dopasowania
rules-dryrun-stopped = Nie osiągnięto (wcześniejsza reguła zakończyła przetwarzanie)

rules-scripts-label = Skrypty Sieve
rules-script-active = Aktywny
rules-script-new = Nowy skrypt
rules-script-export = Eksportuj skrypt
rules-script-import = Importuj skrypt
rules-script-too-large = Skrypt przekracza limit rozmiaru.
rules-script-read-only = Administrator wyłączył bezpośrednią edycję skryptów.
rules-script-validate = Sprawdź skrypt
rules-script-deactivate = Dezaktywuj
rules-script-activate = Aktywuj
rules-script-saved = Skrypt zapisany.

rules-vacation-title = Odpowiedź urlopowa
rules-vacation-enabled = Włącz odpowiedzi automatyczne
rules-vacation-from = Początek (UTC)
rules-vacation-to = Koniec (UTC)
rules-vacation-body = Treść odpowiedzi
rules-vacation-saved = Zapisano odpowiedź urlopową.
rules-script-template = Szablon skryptu
rules-forward-title = Przekazywanie
rules-forward-address = Przekazuj do
rules-forward-copy = Zachowaj kopię na tym koncie
rules-forward-refused = Administrator nie zezwala na przekazywanie do tej domeny.
