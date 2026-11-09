# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Conservazione
retention-intro = Regole che spostano o eliminano la sua posta dopo un certo numero di giorni. Una regola può considerare una cartella, un'etichetta, un mittente, una dimensione, se il messaggio è letto o se ha un allegato. La posta contrassegnata e quella con l'etichetta keep non viene toccata, a meno che non venga modificata questa impostazione. La posta eliminata va prima nel Cestino e viene rimossa definitivamente dopo il periodo di annullamento.
retention-disabled = Le regole di conservazione sono disattivate su questo server. Le sue regole esistenti non vengono eseguite.
retention-never = mai
retention-error = Impossibile caricare o salvare le impostazioni di conservazione. Riprova.
retention-last-run = Ultima esecuzione: { $when }
retention-next-run = prossima esecuzione tra circa { $days } giorno/i
retention-next-run-unknown = prossima esecuzione non ancora pianificata
retention-enabled = Esegui le mie regole di conservazione
retention-trash-first = Sposta prima la posta eliminata nel Cestino
retention-trash-forced = Questo server sposta sempre la posta prima nel Cestino e la rimuove definitivamente dopo il periodo di annullamento.
retention-undo = Periodo di annullamento in giorni
retention-undo-help = La posta che una regola ha spostato nel Cestino vi resta per questo tempo prima di essere rimossa definitivamente. Da 0 a { $max } giorni.
retention-pause = Sospendi tutte le regole per 30 giorni
retention-paused-until = Sospese fino al { $when }
retention-resume = Riprendi
retention-policy = Cosa consente questo server
retention-limits-both = Una regola che elimina la posta attende almeno { $min } giorni e al massimo { $max } giorni.
retention-limits-min = Una regola che elimina la posta attende almeno { $min } giorni.
retention-max-rules = Può tenere fino a { $count } regole.
retention-permanent-allowed = Una regola può eliminare la posta definitivamente se disattivi il passaggio nel Cestino.
retention-permanent-denied = Le regole non possono eliminare la posta definitivamente. La posta va nel Cestino e viene rimossa dopo { $days } giorni.
retention-protected = Cartelle che le regole non toccano mai: { $folders }.
retention-mandatory = Impostato dall'amministratore: la posta in { $folder } più vecchia di { $days } giorni viene eliminata.
retention-rules = Le sue regole
retention-add = Nuova regola
retention-no-rules = Non ha ancora nessuna regola. Adotta un modello qui sotto o aggiungine una.
retention-rule-paused = sospesa
retention-rule-on = Attiva
retention-rule-enabled = La regola { $name } è attiva
retention-edit-short = Modifica
retention-delete = Elimina
retention-new = Nuova regola
retention-edit = Modifica regola
retention-name = Nome
retention-action = Cosa fare
retention-action-keep = Conserva per sempre
retention-action-delete = Elimina
retention-action-trash = Sposta nel Cestino
retention-action-archive = Sposta in Archivio
retention-action-move = Sposta in una cartella
retention-action-markRead = Segna come letto
retention-days = Dopo giorni
retention-target = Cartella
retention-choose-folder = Scegli una cartella
retention-scope = Quale posta
retention-folder = Cartella
retention-any-folder = Qualsiasi cartella
retention-subfolders = Includi le sottocartelle
retention-labels = Etichette (separate da virgole)
retention-senders = Indirizzi dei mittenti (separati da virgole)
retention-domains = Domini dei mittenti (separati da virgole)
retention-min-size = Dimensione minima in KB
retention-max-size = Dimensione massima in KB
retention-read = Stato di lettura
retention-read-any = Letti o non letti
retention-read-read = Solo letti
retention-read-unread = Solo non letti
retention-attachment = Allegati
retention-attachment-any = Con o senza
retention-attachment-with = Con allegato
retention-attachment-without = Senza allegato
retention-exclude = Non toccare
retention-exclude-flagged = Posta contrassegnata (con stella)
retention-exclude-unread = Posta non letta
retention-exclude-tags = Posta con queste etichette (separate da virgole)
retention-exclude-note = La posta soggetta a conservazione legale non viene mai toccata, qualunque cosa sia impostata qui.
retention-role-inbox = In arrivo
retention-role-sent = Inviata
retention-role-archive = Archivio
retention-role-junk = Posta indesiderata
retention-role-trash = Cestino
retention-role-drafts = Bozze
retention-btn-preview = Anteprima
retention-save = Salva
retention-cancel = Annulla
retention-preview-none = Nessun messaggio verrebbe modificato.
retention-preview-count = Verrebbero modificati { $count } messaggio/i.
retention-preview-truncated = Per questa anteprima è stata letta solo una parte della casella.
retention-preview-sample = Esempi
retention-no-subject = (nessun oggetto)
retention-do-trash = nel Cestino
retention-do-move = spostati
retention-do-delete = eliminati
retention-do-markRead = segnati come letti
retention-skipped-legalHold = Il suo account è soggetto a conservazione legale. Non viene modificato nulla.
retention-skipped-restricted = Il trattamento del suo account è limitato. Non viene modificato nulla.
retention-skipped-noRules = Non ha regole attive e l'amministratore non ne ha impostate.
retention-error-name = Assegna un nome alla regola.
retention-error-days = I giorni devono essere un numero intero da 0 a 3650.
retention-error-min = Una regola che elimina la posta deve attendere almeno { $days } giorni.
retention-error-max = Una regola che elimina la posta può attendere al massimo { $days } giorni.
retention-error-target = Scegli la cartella in cui spostare la posta.
retention-summary-keep = conserva per sempre
retention-summary-delete = dopo { $days } giorni, nel Cestino, poi rimossa definitivamente
retention-summary-delete-permanent = dopo { $days } giorni, eliminata definitivamente
retention-summary-trash = dopo { $days } giorni, nel Cestino
retention-summary-archive = dopo { $days } giorni, in Archivio
retention-summary-move = dopo { $days } giorni, in una cartella
retention-summary-markRead = dopo { $days } giorni, segnata come letta
retention-templates = Modelli
retention-templates-help = Un modello aggiunge una regola già pronta. Controlla le sue etichette e i giorni prima di farvi affidamento.
retention-adopt = Aggiungi questa regola
retention-run-heading = Esegui ora
retention-preview-all = Anteprima di tutte le regole
retention-btn-run = Esegui ora le mie regole
retention-run-result = { $count } messaggio/i modificato/i.
retention-history = Esecuzioni recenti
retention-history-when = Quando
retention-history-matched = Corrispondenti
retention-history-changed = Modificati
retention-history-errors = Errori
retention-context-action = Conservazione…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Calendari

pim-cal-days = Giorni dopo la fine di un evento

pim-cal-enabled = Elimina gli eventi terminati

pim-cal-help = Elimina gli eventi nei calendari che scegli un determinato numero di giorni dopo la loro fine. Gli eventi ricorrenti non vengono mai eliminati.

pim-cal-title = Eventi del calendario

pim-days-range = Scegli tra { $min } e { $max } giorni.

pim-held-title = Note rimosse che puoi ripristinare

pim-held-until = Conservato fino al { $when }

pim-limits = Tra { $min } e { $max } giorni.

pim-notes-days = Giorni trascorsi dall'ultima modifica di una nota

pim-notes-enabled = Rimuovi le note intatte

pim-notes-help = Rimuovi le note che non hai modificato per un determinato numero di giorni. Le note appuntate vengono mantenute. Una nota rimossa può essere ripristinata per { $days } giorni.

pim-notes-title = Note

pim-off = Disattivato dall'amministratore.

pim-preview = Anteprima

pim-restore = Ripristina

pim-result = { $events } evento/i e { $notes } nota/e.

pim-run = Corri adesso

pim-run-title = Corri adesso

pim-save = Salva

pim-skipped = { $count } scelte sono state ignorate perché l'amministratore le ha disattivate.

pim-untitled = Nota senza titolo

retention-action-stripAttachments = Rimuovere gli allegati di grandi dimensioni

retention-do-stripAttachments = allegati rimossi

retention-error-strip-disabled = La rimozione degli allegati è stata disabilitata dall'amministratore.

retention-error-strip-min = Una regola che rimuove gli allegati deve attendere almeno { $days } giorni.

retention-fd-action = Cosa succede alla posta in { $folder }

retention-fd-days = Giorni prima dell'elaborazione di { $folder }

retention-fd-help = Imposta per quanto tempo la posta rimane in una cartella prima che venga spostata o eliminata. Una cartella impostata su "Eredita" non ha una propria impostazione predefinita; valgono comunque le regole di cui sopra ed i limiti dell'amministratore. Le sottocartelle seguono la rispettiva cartella.

retention-fd-inherit = Eredita

retention-fd-mandatory = L'amministratore conserva questa cartella per { $days } giorni

retention-fd-protected = Protetto dall'amministratore

retention-fd-save = Salva le impostazioni predefinite della cartella

retention-fd-title = Impostazioni predefinite della cartella

retention-rule-strip-off = Disattivato dall'amministratore

retention-strip-allowed = Una regola può rimuovere allegati di almeno { $kb } KB dalla posta vecchia di almeno { $days } giorni. Una breve nota sostituisce ogni allegato.

retention-strip-denied = La rimozione degli allegati è stata disattivata dall'amministratore.

retention-strip-done = Allegati rimossi dai messaggi { $count }, liberando { $size }.

retention-strip-inline-kept = Le immagini mostrate all'interno del messaggio vengono conservate.

retention-strip-inline-removed = Verranno rimosse anche le immagini mostrate all'interno del messaggio.

retention-strip-limits = Rimuove gli allegati di almeno { $kb } KB dai messaggi di posta vecchi di almeno { $days } giorni. L'originale rimane nel Cestino per { $undo } giorni.

retention-strip-preview = I messaggi { $count } perderebbero gli allegati, liberando { $size }.

retention-strip-preview-estimate = Circa { $count } messaggi perderebbero gli allegati, liberando fino a { $size }.

retention-strip-skipped = { $count } regole che rimuovono gli allegati sono state ignorate perché l'amministratore ha disattivato questa opzione.

retention-summary-stripAttachments = dopo { $days } giorni, allegati di grandi dimensioni rimossi
