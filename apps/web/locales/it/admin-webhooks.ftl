# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhook
admin-webhooks-intro = Un webhook in uscita invia un avviso firmato a un URL quando succede qualcosa in un account. Un webhook in entrata consente a un altro sistema di eseguire un'azione di regola di posta in un account.
admin-webhooks-empty = Nessun webhook per ora.
admin-webhooks-unavailable = Questo server non offre la gestione dei webhook.
admin-webhooks-load-error = Impossibile caricare i webhook.
admin-webhooks-direction-outbound = In uscita
admin-webhooks-direction-inbound = In entrata
admin-webhooks-col-direction = Direzione
admin-webhooks-col-label = Nome
admin-webhooks-col-account = Account
admin-webhooks-col-target = Invia a o consente
admin-webhooks-col-created = Creato
admin-webhooks-no-label = (senza nome)
admin-webhooks-created-by = da { $who }
admin-webhooks-last-used = Ultimo utilizzo { $when }
admin-webhooks-never-used = Non ancora utilizzato
admin-webhooks-delete = Elimina
admin-webhooks-delete-named = Elimina il webhook { $name }
admin-webhooks-delete-title = Eliminare il webhook { $name }?
admin-webhooks-delete-body = Le consegne verso di esso si interrompono, oppure le chiamate firmate con il suo segreto vengono rifiutate. Non si può annullare; un nuovo webhook riceve un nuovo segreto.
admin-webhooks-deleted = Webhook eliminato.
admin-webhooks-add = Aggiungi un webhook
admin-webhooks-add-direction = Direzione
admin-webhooks-add-outbound-help = Mailwoman chiama il suo URL.
admin-webhooks-add-inbound-help = Il suo sistema chiama Mailwoman.
admin-webhooks-add-account = Account
admin-webhooks-add-account-help = L'account i cui eventi vengono inviati, oppure l'unico account su cui può agire una chiamata in entrata.
admin-webhooks-add-label = Nome
admin-webhooks-add-label-help = A cosa serve il webhook, così gli altri lo capiscono. Facoltativo.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = Deve raggiungere un indirizzo Internet pubblico. Gli indirizzi privati, di loopback e link-local vengono rifiutati.
admin-webhooks-add-events = Eventi
admin-webhooks-add-actions = Azioni consentite
admin-webhooks-add-submit = Crea webhook
admin-webhooks-add-saving = Creazione…
admin-webhooks-add-check = Verifica
admin-webhooks-add-check-ok = L'URL supera la verifica. Non è stato salvato nulla.
admin-webhooks-event-message-received = Nuovo messaggio
admin-webhooks-event-message-flags = Messaggio letto, contrassegnato o etichettato
admin-webhooks-event-message-moved = Messaggio spostato
admin-webhooks-event-message-deleted = Messaggio eliminato
admin-webhooks-event-submission-sent = Messaggio inviato
admin-webhooks-event-submission-failed = Impossibile inviare il messaggio
admin-webhooks-event-pim-changed = Calendario, contatti, attività o note modificati
admin-webhooks-action-run-rules = Esegui le regole di posta dell'account su una descrizione di messaggio
admin-webhooks-action-sieve = Applica a un messaggio un'azione di regola
admin-webhooks-secret-title = Segreto per { $name }
admin-webhooks-secret-once = Copia subito questo segreto. Non verrà mostrato di nuovo.
admin-webhooks-secret-outbound = Ogni consegna contiene X-Mailwoman-Signature: sha256= seguito dall'HMAC-SHA256 del corpo della richiesta con questo segreto.
admin-webhooks-secret-inbound = Invia X-Mailwoman-Webhook: { $id }, X-Mailwoman-Timestamp (secondi Unix), X-Mailwoman-Nonce (da 16 a 128 lettere, cifre, - o _, mai riutilizzato) e X-Mailwoman-Signature: sha256= seguito dall'HMAC-SHA256 di timestamp.nonce.body con questo segreto. Le chiamate con più di cinque minuti di scarto, o che riutilizzano un nonce, vengono rifiutate.
admin-webhooks-secret-done = L'ho copiato
admin-webhooks-error-stale = Qualcuno ha modificato i webhook da quando ha aperto questa pagina. L'elenco è stato ricaricato; riprova.
admin-webhooks-error-account = Inserisci l'account.
admin-webhooks-error-url = Inserisci un URL che inizi con https:// o http://.
admin-webhooks-error-events = Scegli almeno un evento.
admin-webhooks-error-actions = Scegli almeno un'azione.
