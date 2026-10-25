# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Chiudi

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = Non disponibile — { $reason }
admin-ux-unavailable-default = questo server non lo offre ancora.
admin-ux-state-code = Codice di errore: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Apri
admin-ux-list-back = Torna all'elenco

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Nessuna modifica non salvata
admin-ux-form-changes =
    { $count ->
        [one] 1 modifica non salvata
       *[other] { $count } modifiche non salvate
    }
admin-ux-form-discard = Annulla le modifiche
admin-ux-form-saving = Salvataggio…
admin-ux-form-saved =
    { $count ->
        [one] 1 modifica salvata.
       *[other] { $count } modifiche salvate.
    }
admin-ux-form-fix =
    { $count ->
        [one] Non è stato salvato nulla. Correggere il campo segnalato qui sotto.
       *[other] Non è stato salvato nulla. Correggere i { $count } campi segnalati qui sotto.
    }
admin-ux-learn-more = Ulteriori informazioni
admin-ux-learn-more-field = su { $label }
admin-ux-propagation-hot = Si applica entro 30 secondi su ogni processo del server.
admin-ux-propagation-restart = Si applica dopo il riavvio del server.
admin-ux-secret-set = È memorizzato un valore. Digitarne uno nuovo per sostituirlo.
admin-ux-secret-unset = Nessun valore memorizzato.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Inserire un valore.
admin-ux-err-number = Inserire un numero.
admin-ux-err-whole = Inserire un numero intero.
admin-ux-err-min = Inserire { $min } o più.
admin-ux-err-max = Inserire { $max } o meno.
admin-ux-err-max-length = Usare al massimo { $max } caratteri.
admin-ux-err-url = Inserire un indirizzo completo che inizi con https:// o http://.
admin-ux-err-email = Inserire un indirizzo email come name@example.org.
admin-ux-err-pattern = Questo valore non ha il formato previsto.
admin-ux-err-option = Scegliere una delle opzioni elencate.
admin-ux-err-out-of-range = Il server ha rifiutato questo valore perché è fuori dall'intervallo consentito.
admin-ux-err-too-long = Il server ha rifiutato questo valore perché è troppo lungo.
admin-ux-err-invalid = Il server ha rifiutato questo valore. Verificarlo e salvare di nuovo.
admin-ux-err-pinned = Questo valore è impostato nell'ambiente del server e non può essere modificato qui.
admin-ux-err-locked = Questo valore è bloccato a un ambito più ampio e non può essere modificato qui.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Predefinito
admin-ux-source-stored = Memorizzato
admin-ux-source-env-pin = Impostato da { $name }
admin-ux-source-env-fallback = Ripiego da { $name }
admin-ux-source-build = Impostato in compilazione
admin-ux-source-locked = Bloccato da { $scope }
admin-ux-source-read-only = Sola lettura
admin-ux-source-restart = Riavvio necessario
admin-ux-source-env-pin-reason = L'ambiente del server imposta questo valore con { $name }. Modificarlo lì e riavviare il server.
admin-ux-source-build-reason = Questo valore viene fissato durante la compilazione del server.
admin-ux-source-locked-reason = Un blocco nell'ambito { $scope } fissa questo valore. Rimuovere il blocco lì per modificarlo.
admin-ux-source-read-only-reason = La console mostra questo valore ma non può modificarlo. La documentazione indica dove viene impostato.

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = Ultima modifica di { $who } il { $time }
admin-ux-history-changed-by = Ultima modifica di { $who }
admin-ux-history-changed-at = Ultima modifica il { $time }
admin-ux-history-never = Non modificato dalla console
admin-ux-history-link = Cronologia

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = Predefinito: { $value }
admin-ux-revert = Ripristina il predefinito
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Ripristinato il valore predefinito ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } ha modificato questo valore il { $time } mentre era in corso la modifica.
admin-ux-stale-by = { $who } ha modificato questo valore mentre era in corso la modifica.
admin-ux-stale-unknown = Qualcun altro ha modificato questo valore mentre era in corso la modifica.
admin-ux-stale-choice = Ricaricare per vedere la sua versione perdendo le proprie modifiche, oppure mantenere le proprie e salvarle sopra le sue.
admin-ux-stale-reload = Ricarica
admin-ux-stale-keep = Mantieni le mie

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Verifica degli effetti di questa modifica…
admin-ux-impact-heading = Questa modifica riguarda
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] e 1 altro
       *[other] e altri { $count }
    }
admin-ux-impact-none = Nient'altro cambia.
admin-ux-impact-unavailable = Anteprima degli effetti non disponibile — il server non ha indicato cosa riguarda questa modifica.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtro
admin-ux-stream-pause = Pausa
admin-ux-stream-resume = Riprendi
admin-ux-stream-clear = Svuota
admin-ux-stream-announce = Leggi ad alta voce le nuove voci
admin-ux-stream-connecting = Connessione…
admin-ux-stream-live = In diretta
admin-ux-stream-ended = Terminato
admin-ux-stream-lost = Connessione persa
admin-ux-stream-reconnect = Riconnetti
admin-ux-stream-paused =
    { $count ->
        [0] In pausa
        [one] In pausa — 1 nuova voce in attesa
       *[other] In pausa — { $count } nuove voci in attesa
    }
admin-ux-stream-no-match = Nessuna voce corrisponde al filtro.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Campo

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Aspetto
admin-ux-appearance-title = Aspetto della console
admin-ux-appearance-note = Queste scelte modificano solo la console di amministrazione, in questo browser.
admin-ux-appearance-following = La console segue il tema della webmail di questo browser finché non si sceglie qui. Queste scelte modificano solo la console di amministrazione, in questo browser.
admin-ux-appearance-mode = Modalità
admin-ux-appearance-mode-light = Chiaro
admin-ux-appearance-mode-dark = Scuro
admin-ux-appearance-mode-system = Segui il sistema
admin-ux-appearance-theme = Tema
admin-ux-appearance-accent = Colore di accento
admin-ux-appearance-light = Chiaro
admin-ux-appearance-dark = Scuro
admin-ux-appearance-reset = Usa il tema della webmail
