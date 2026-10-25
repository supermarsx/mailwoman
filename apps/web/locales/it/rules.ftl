# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: rules. Source: ../en/rules.ftl

rules-title = Regole e filtri
rules-intro = Gestisci script di filtro e regole visive. Le funzioni disponibili dipendono dall’account e dalle impostazioni dell’amministratore.
rules-empty = Ancora nessuna regola.
rules-unavailable = Questo server di posta non supporta le regole.
rules-load-failed = Impossibile caricare le regole.
rules-new = Nuova regola
rules-list-label = Regole della posta
rules-enabled = Attiva
rules-delete-named = Elimina la regola { $name }

rules-builder-title = Editor delle regole
rules-builder-hint = Selezionare una regola da modificare o crearne una nuova.
rules-name-label = Nome della regola
rules-match-legend = Quando eseguire
rules-match-all = Tutte le condizioni
rules-match-any = Almeno una condizione
rules-conditions-legend = Condizioni
rules-actions-legend = Azioni
rules-add-condition = Aggiungi condizione
rules-add-action = Aggiungi azione
rules-remove-condition = Rimuovi condizione
rules-remove-action = Rimuovi azione
rules-cond-type-label = Campo
rules-cond-op-label = Operatore
rules-cond-value-label = Valore da confrontare
rules-act-type-label = Azione
rules-act-value-label = Valore dell'azione
rules-save = Salva regola
rules-editing = Modifica di { $name }
rules-save-refused = La regola non è stata salvata: { $reason }

rules-cond-from = Da
rules-cond-to = A
rules-cond-subject = Oggetto
rules-cond-thread-unsupported = Conversazione { $op } { $value }: le condizioni sulle conversazioni non sono supportate, quindi la regola non può essere salvata finché questa condizione non viene rimossa.
rules-op-contains = contiene
rules-op-is = è

rules-act-move = Sposta nella casella
rules-act-tag = Aggiungi tag
rules-act-archive = Archivia
rules-act-suppress = Disattiva notifica
rules-act-stop = Interrompi l'elaborazione

rules-runs-server = Server (Sieve)
rules-runs-server-detail = Caricata sul server di posta e applicata lì.
rules-runs-engine = Motore
rules-runs-engine-detail = Applicato durante la sincronizzazione dei nuovi messaggi; risposte e spostamenti attendono la sincronizzazione successiva.

rules-panes-label = Strumenti delle regole
rules-tab-builder = Editor
rules-tab-raw = Sieve grezzo
rules-tab-dryrun = Prova

rules-raw-label = Sorgente Sieve grezzo
rules-raw-reset = Ripristina il generato
rules-lint-clean = Nessun problema trovato.

rules-dryrun-title = Prova
rules-dryrun-help = Inserire un messaggio di esempio per vedere quali regole verrebbero attivate.
rules-dryrun-results = Risultati della prova
rules-dryrun-none = Nessuna regola attiva da valutare.
rules-dryrun-nomatch = Nessuna corrispondenza
rules-dryrun-stopped = Non raggiunta (una regola precedente ha interrotto l'elaborazione)

rules-scripts-label = Script Sieve
rules-script-active = Attivo
rules-script-new = Nuovo script
rules-script-export = Esporta script
rules-script-import = Importa script
rules-script-too-large = Lo script supera il limite di dimensione.
rules-script-read-only = La modifica diretta degli script è disabilitata.
rules-script-validate = Convalida script
rules-script-deactivate = Disattiva
rules-script-activate = Attiva
rules-script-saved = Script salvato.

rules-vacation-title = Risposta di assenza
rules-vacation-enabled = Attiva risposte automatiche
rules-vacation-from = Inizio (UTC)
rules-vacation-to = Fine (UTC)
rules-vacation-body = Testo della risposta
rules-vacation-saved = Risposta di assenza salvata.
rules-script-template = Modello di script
rules-forward-title = Inoltro
rules-forward-address = Inoltra a
rules-forward-copy = Conserva una copia in questo account
rules-forward-refused = L’amministratore non consente l’inoltro a questo dominio.
