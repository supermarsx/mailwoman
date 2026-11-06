# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooki
admin-webhooks-intro = Webhook wychodzący wysyła podpisane powiadomienie na adres URL, gdy coś zdarzy się na jednym koncie. Webhook przychodzący pozwala innemu systemowi uruchomić akcję reguły poczty na jednym koncie.
admin-webhooks-empty = Brak webhooków.
admin-webhooks-unavailable = Ten serwer nie oferuje zarządzania webhookami.
admin-webhooks-load-error = Nie udało się wczytać webhooków.
admin-webhooks-direction-outbound = Wychodzący
admin-webhooks-direction-inbound = Przychodzący
admin-webhooks-col-direction = Kierunek
admin-webhooks-col-label = Nazwa
admin-webhooks-col-account = Konto
admin-webhooks-col-target = Wysyła do lub zezwala
admin-webhooks-col-created = Utworzono
admin-webhooks-no-label = (bez nazwy)
admin-webhooks-created-by = przez { $who }
admin-webhooks-last-used = Ostatnio użyty { $when }
admin-webhooks-never-used = Jeszcze nieużywany
admin-webhooks-delete = Usuń
admin-webhooks-delete-named = Usuń webhook { $name }
admin-webhooks-delete-title = Usunąć webhook { $name }?
admin-webhooks-delete-body = Dostarczanie do niego zostanie zatrzymane, a wywołania podpisane jego sekretem będą odrzucane. Tej operacji nie można cofnąć; nowy webhook otrzyma nowy sekret.
admin-webhooks-deleted = Webhook usunięty.
admin-webhooks-add = Dodaj webhook
admin-webhooks-add-direction = Kierunek
admin-webhooks-add-outbound-help = Mailwoman wywołuje Twój adres URL.
admin-webhooks-add-inbound-help = Twój system wywołuje Mailwoman.
admin-webhooks-add-account = Konto
admin-webhooks-add-account-help = Konto, którego zdarzenia są wysyłane, lub jedyne konto, na którym może działać wywołanie przychodzące.
admin-webhooks-add-label = Nazwa
admin-webhooks-add-label-help = Do czego służy webhook, aby inni mogli go rozpoznać. Opcjonalne.
admin-webhooks-add-url = Adres URL
admin-webhooks-add-url-help = Musi wskazywać publiczny adres internetowy. Adresy prywatne, pętli zwrotnej i link-local są odrzucane.
admin-webhooks-add-events = Zdarzenia
admin-webhooks-add-actions = Dozwolone akcje
admin-webhooks-add-submit = Utwórz webhook
admin-webhooks-add-saving = Tworzenie…
admin-webhooks-add-check = Sprawdź
admin-webhooks-add-check-ok = Adres URL przechodzi sprawdzenie. Nic nie zapisano.
admin-webhooks-event-message-received = Nowa wiadomość
admin-webhooks-event-message-flags = Wiadomość przeczytana, oflagowana lub oznaczona etykietą
admin-webhooks-event-message-moved = Wiadomość przeniesiona
admin-webhooks-event-message-deleted = Wiadomość usunięta
admin-webhooks-event-submission-sent = Wiadomość wysłana
admin-webhooks-event-submission-failed = Nie udało się wysłać wiadomości
admin-webhooks-event-pim-changed = Zmieniono kalendarz, kontakty, zadania lub notatki
admin-webhooks-action-run-rules = Uruchom reguły poczty konta dla opisu wiadomości
admin-webhooks-action-sieve = Zastosuj jedną akcję reguły do wiadomości
admin-webhooks-secret-title = Sekret dla { $name }
admin-webhooks-secret-once = Skopiuj ten sekret teraz. Nie zostanie pokazany ponownie.
admin-webhooks-secret-outbound = Każde dostarczenie zawiera X-Mailwoman-Signature: sha256= a po nim HMAC-SHA256 treści żądania z tym sekretem.
admin-webhooks-secret-inbound = Wyślij X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (sekundy Unix), X-Mailwoman-Nonce (od 16 do 128 liter, cyfr, - lub _, nigdy nie używany ponownie) oraz X-Mailwoman-Signature: sha256= a po nim HMAC-SHA256 z timestamp.nonce.body z tym sekretem. Wywołania z czasem odbiegającym o ponad pięć minut lub z ponownie użytym nonce są odrzucane.
admin-webhooks-secret-done = Skopiowano
admin-webhooks-error-stale = Ktoś zmienił webhooki od otwarcia tej strony. Lista została wczytana ponownie; spróbuj jeszcze raz.
admin-webhooks-error-account = Podaj konto.
admin-webhooks-error-url = Podaj adres URL zaczynający się od https:// lub http://.
admin-webhooks-error-events = Wybierz co najmniej jedno zdarzenie.
admin-webhooks-error-actions = Wybierz co najmniej jedną akcję.
