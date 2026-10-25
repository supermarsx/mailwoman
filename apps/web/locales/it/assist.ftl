# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Assistente IA
assist-heading = Assistente IA
assist-transcript-label = Conversazione con l'assistente IA
assist-proposed-action = Azione proposta
assist-outbox-note = Questo metterebbe un messaggio nella Posta in uscita. Non viene inviato nulla finché non lo si conferma lì.
assist-review-outbox = Controlla in Posta in uscita
assist-review = Controlla
assist-empty = Chiedere all'assistente IA di riassumere, scrivere bozze o trovare elementi. Può proporre azioni, ma la conferma spetta sempre all'utente.
assist-error = L'assistente IA non è riuscito a rispondere in questo momento.
assist-input-label = Scrivi all'assistente IA
assist-input-placeholder = Chiedi all'assistente IA…
assist-thinking = Elaborazione…
assist-ask = Chiedi

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Aiuto alla scrittura
assist-composer-toolbar = Strumenti di composizione
assist-tool-grammar = Correggi grammatica
assist-tool-rewrite = Riscrivi
assist-tool-tone = Adatta il tono
assist-tool-translate = Traduci
assist-toolarg-tone = Tono
assist-toolarg-translate = Lingua
assist-tone-neutral = Neutro
assist-tone-friendly = Cordiale
assist-tone-formal = Formale
assist-tone-concise = Conciso
assist-tone-assertive = Deciso
assist-busy = { $label }…
assist-composer-empty-error = Scrivere prima qualcosa.
assist-composer-error = L'assistente IA non è riuscito a completare l'operazione in questo momento.
assist-suggested-edit = Modifica suggerita
assist-apply-draft = Applica alla bozza
assist-discard = Scarta

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Etichette suggerite
assist-apply-auto = Applica automaticamente
assist-apply-label = Applica { $label }
assist-remove-label = Rimuovi { $label }
assist-apply = Applica
assist-undo = Annulla

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Tieni premuto per dettare
assist-dictate-stop = Interrompi dettatura
assist-dictate-hold-text = Tieni premuto per dettare
assist-dictate-listening = ● In ascolto…
assist-dictate-endpoint-note = L'audio viene trascritto dall'endpoint dell'assistente IA.
assist-dictate-err = Errore di dettatura.
assist-transcribe-err = Impossibile trascrivere l'audio.
assist-mic-err = Microfono non disponibile.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Ricerca semantica
assist-semantic-note = Il testo della ricerca viene inviato all'endpoint dell'assistente IA per ordinare i risultati per significato.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Cosa può uscire da questo dispositivo
assist-left-1 = il testo del messaggio selezionato (oggetto + corpo) per la funzione scelta
assist-left-2 = l'host dell'endpoint a cui è stato inviato
assist-left-3 = mai: contenuto cifrato end-to-end (escluso per impostazione predefinita)
assist-left-4 = mai: allegati (esclusi per impostazione predefinita)
assist-left-5 = mai: le credenziali o altri account
assist-disclosure-off = L'assistente IA è disattivato. Nessun contenuto dei messaggi esce da questo dispositivo.
assist-disclosure-sentence = Quando si usa uno strumento dell'assistente IA, il testo del messaggio selezionato viene inoltrato a { $host }. { $excl } L'invio non è mai automatico: la conferma spetta sempre all'utente prima che qualcosa lasci la Posta in uscita.
assist-disclosure-excl-one = Non invia mai { $a }.
assist-disclosure-excl-two = Non invia mai { $a } né { $b }.
assist-disclosure-admin-allowed = L'amministratore ha consentito l'invio di contenuto cifrato e allegati.
assist-excluded-e2ee = contenuto cifrato end-to-end
assist-excluded-attachments = allegati

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = Assistente IA
assist-admin-intro = L'assistente IA invia il testo dei messaggi selezionato dagli utenti all'endpoint IA configurato qui. Non può inviare, eliminare o accettare posta. Il contenuto cifrato end-to-end e gli allegati vengono trattenuti a meno che non vengano consentiti qui sotto.
assist-admin-load-unreadable = La configurazione memorizzata dell'assistente IA non può essere letta: la sua chiave API non si apre con la chiave di questo server (MW_SERVER_KEY). È stata salvata con un'altra chiave o è stata modificata. L'assistente IA è disattivato. Compilare il modulo, inclusa la chiave API, e salvare per sostituirla.
assist-admin-load-error = Impossibile caricare la configurazione dell'assistente IA. Il salvataggio è disattivato per non sovrascrivere la configurazione memorizzata.
assist-admin-status-running = L'assistente IA è in esecuzione. Le richieste vanno a { $host }.
assist-admin-status-running-pending = L'assistente IA è in esecuzione con la configurazione con cui è stato avviato il server. Le modifiche salvate hanno effetto al riavvio del server.
assist-admin-status-off = L'assistente IA è disattivato. Ogni richiesta all'assistente IA viene rifiutata.
assist-admin-status-pending = L'assistente IA non è in esecuzione. La configurazione salvata ha effetto al riavvio del server.
assist-admin-status-local-program-not-allowed = L'assistente IA non è in esecuzione. Il programma configurato { $program } non è elencato in MW_ASSIST_LOCAL_PROGRAMS sul server, quindi il server non lo ha avviato.
assist-admin-status-no-endpoint = L'assistente IA è attivo ma non in esecuzione perché non è configurato alcun endpoint utilizzabile.
assist-admin-stop = Ferma subito l'assistente IA
assist-admin-stop-note = L'arresto ha effetto immediato per tutti gli utenti e viene mantenuto dopo i riavvii.
assist-admin-resume = Attiva l'assistente IA
assist-admin-resume-note = Se il server è stato avviato con l'assistente IA disattivato, l'attivazione ha effetto al riavvio del server.
assist-admin-kill-error = Impossibile modificare l'attivazione dell'assistente IA.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = È il server a contattare questo endpoint, mai i browser. Le modifiche hanno effetto al riavvio del server.
assist-admin-kind = Tipo di endpoint
assist-admin-kind-none = Nessuno
assist-admin-kind-open-ai-compatible = Compatibile con OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Programma locale
assist-admin-local-program-enable = Perché l'assistente IA esegua un programma su questo server, impostare MW_ASSIST_LOCAL_PROGRAMS sul percorso assoluto del programma nell'ambiente in cui si avvia il server (separare più percorsi con virgole), quindi riavviare il server.
assist-admin-local-program-note = Il server avvia questo programma per ogni richiesta all'assistente IA e gli passa la richiesta sullo standard input, compreso il testo del messaggio selezionato dall'utente. Legge la risposta dallo standard output del programma. Si possono scegliere solo i programmi elencati in MW_ASSIST_LOCAL_PROGRAMS sul server.
assist-admin-local-program = Programma
assist-admin-local-program-unlisted = { $program } (non elencato)
assist-admin-local-program-unlisted-note = Questo programma non è elencato in MW_ASSIST_LOCAL_PROGRAMS, quindi il server non lo esegue e non lo accetterà al salvataggio. Scegliere un programma elencato o un altro tipo di endpoint.
assist-admin-local-program-required = Scegliere il programma da eseguire.
assist-admin-local-args = Argomenti, uno per riga
assist-admin-local-args-note = Ogni riga viene passata al programma come un argomento. Non viene usata alcuna shell.
assist-admin-base-url = URL di base
assist-admin-base-url-required = Inserire l'URL di base dell'endpoint, che inizi con http:// o https://.
assist-admin-api-key = Chiave API
assist-admin-api-key-stored = È memorizzata una chiave. Il server non la mostra di nuovo.
assist-admin-api-key-replace = Sostituisci la chiave API
assist-admin-api-key-replace-placeholder = Lasciare vuoto per mantenere la chiave memorizzata
assist-admin-api-key-remove = Rimuovi la chiave memorizzata
assist-admin-api-key-keep = Mantieni la chiave memorizzata
assist-admin-api-key-remove-pending = La chiave memorizzata verrà rimossa al salvataggio.
assist-admin-chat-model = Modello di chat
assist-admin-embed-model = Modello di embedding
assist-admin-stt-model = Modello di trascrizione vocale
assist-admin-model = Modello
assist-admin-anthropic-version = Versione API
assist-admin-max-tokens = Token massimi per risposta
assist-admin-default-placeholder = Predefinito del server
assist-admin-grants = Funzioni
assist-admin-grants-note = Si possono usare solo le funzioni selezionate qui.
assist-admin-cap-summarize = Riassunto
assist-admin-cap-draft = Bozza e riscrittura
assist-admin-cap-grammar = Grammatica
assist-admin-cap-dictation = Dettatura
assist-admin-cap-search-semantic = Ricerca semantica
assist-admin-cap-auto-tag = Etichettatura automatica
assist-admin-cap-recap = Riepilogo
assist-admin-cap-assistant = Chat con l'assistente IA
assist-admin-ceilings = Limiti sui dati
assist-admin-ceilings-note = Questi limiti si applicano a ogni richiesta, qualunque sia la funzione.
assist-admin-accounts = Id degli account la cui posta può essere inviata, uno per riga
assist-admin-accounts-note = Se non è elencato alcun account, nessun contenuto di posta viene inviato all'endpoint.
assist-admin-folders = Id delle cartelle a cui limitare l'invio, uno per riga
assist-admin-folders-note = Se non è elencata alcuna cartella, sono consentite tutte le cartelle degli account elencati.
assist-admin-allow-e2ee = Consenti l'invio di contenuto cifrato end-to-end
assist-admin-allow-attachments = Consenti l'invio di allegati
assist-admin-save = Salva configurazione
assist-admin-saved = Salvato.
assist-admin-saved-pending = Salvato. La modifica ha effetto al riavvio del server.
assist-admin-save-error = La configurazione non è stata salvata.
assist-admin-save-not-allowed = La configurazione non è stata salvata perché il server non consente questo programma. Si possono usare solo i programmi elencati in MW_ASSIST_LOCAL_PROGRAMS.
