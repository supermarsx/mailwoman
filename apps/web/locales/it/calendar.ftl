# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Calendario
calendar-prev = Precedente
calendar-next = Successivo
calendar-today = Oggi
calendar-views = Viste del calendario
calendar-new-event = Nuovo evento
calendar-import = Importa
calendar-export = Esporta
calendar-import-file = Importa file del calendario

# -- View names --------------------------------------------------------------
calendar-view-day = Giorno
calendar-view-3day = 3 giorni
calendar-view-work-week = Settimana lavorativa
calendar-view-week = Settimana
calendar-view-month = Mese
calendar-view-tri-month = Trimestre
calendar-view-schedule = Programma
calendar-view-agenda = Agenda
calendar-view-year = Anno

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Calendari
calendar-toggle = Mostra/nascondi { $name }
calendar-color-for = Colore per { $name }
calendar-add-calendar = Aggiungi calendario
calendar-new-calendar-name = Nuovo calendario
calendar-holidays-heading = Festività
calendar-subscribe-holidays = Iscriviti alle festività
calendar-add-region = Aggiungi una regione…
calendar-synced = Calendario sincronizzato

# -- Grid / views ------------------------------------------------------------
calendar-all-day = tutto il giorno
calendar-all-day-full = Tutto il giorno
calendar-more = altri { $count }
calendar-conflict = conflitto
calendar-no-events-30 = Nessun evento nei prossimi 30 giorni.
calendar-events-count = { $count ->
    [zero] Nessun evento
    [one] { $count } evento
   *[other] { $count } eventi
}
calendar-month-grid = Mese di { $month }
calendar-time-grid = Griglia oraria
calendar-cell = { $date }, { $events }
calendar-announce = { $date }, { $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = Tutto il giorno: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = Nuovo evento
calendar-editor-edit = Modifica evento
calendar-invitation = Invito
calendar-invited = Invito ricevuto ({ $status }).
calendar-accept = Accetta
calendar-tentative = Provvisorio
calendar-decline = Rifiuta
calendar-counter = Controproposta…
calendar-propose-start = Proponi nuovo inizio
calendar-send-counter = Invia controproposta
calendar-overlaps = Questo evento si sovrappone a { $count ->
    [one] { $count } altro evento
   *[other] { $count } altri eventi
}.
calendar-field-title = Titolo
calendar-untitled = (senza titolo)
calendar-field-calendar = Calendario
calendar-field-all-day = Tutto il giorno
calendar-field-start = Inizio
calendar-field-duration = Durata (min)
calendar-field-location = Luogo
calendar-field-shows-as = Mostra come
calendar-shows-busy = Occupato
calendar-shows-free = Libero
calendar-field-status = Stato
calendar-status-confirmed = Confermato
calendar-status-tentative = Provvisorio
calendar-status-cancelled = Annullato
calendar-repeats = Ripetizione
calendar-frequency = Frequenza
calendar-freq-daily = Ogni giorno
calendar-freq-weekly = Ogni settimana
calendar-freq-monthly = Ogni mese
calendar-freq-yearly = Ogni anno
calendar-every = ogni
calendar-interval = Intervallo
calendar-ends = Termina
calendar-end-mode = Modalità di fine
calendar-end-never = Mai
calendar-end-count = Dopo N volte
calendar-end-until = In data
calendar-occurrences = Occorrenze
calendar-until-date = Fino alla data
calendar-reminders = Promemoria
calendar-reminder-before = { $min } min prima
calendar-remove-reminder = Rimuovi promemoria { $min }
calendar-add-reminder = Aggiungi promemoria
calendar-reminder-none = + promemoria
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 ora
calendar-reminder-1440 = 1 giorno
calendar-attendees = Partecipanti
calendar-add-attendee = Aggiungi partecipante
calendar-attendee-invalid = Non è un indirizzo email. Inserire un solo indirizzo, ad esempio name@example.com.
calendar-save-failed = L'evento non è stato salvato. Il server lo ha rifiutato.
calendar-remove-attendee = Rimuovi { $email }
calendar-notes = Note

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Ruolo di { $email }
calendar-attendee-cutype-for = Tipo di { $email }
calendar-role-chair = Presidente
calendar-role-required = Obbligatorio
calendar-role-optional = Facoltativo
calendar-role-non-participant = Non partecipante
calendar-cutype-individual = Persona
calendar-cutype-group = Gruppo
calendar-cutype-resource = Risorsa
calendar-cutype-room = Sala
calendar-partstat-needs-action = Nessuna risposta
calendar-partstat-accepted = Accettato
calendar-partstat-declined = Rifiutato
calendar-partstat-tentative = Provvisorio

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = Dalle { $start } alle { $end }: { $title }
calendar-schedule-free = { $gap } liberi
calendar-gap-hm = { $h } h { $m } min
calendar-gap-h = { $h } h
calendar-gap-m = { $m } min

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Risolvi { $count ->
    [one] { $count } conflitto
   *[other] { $count } conflitti
}
calendar-resolver-title = Risolvi conflitti
calendar-resolver-none = Nessun conflitto da risolvere.
calendar-resolver-pick = Conflitto
calendar-resolver-pair-n = { $n } di { $total }: { $a } contro { $b }
calendar-resolver-time = Ora
calendar-resolver-overlap = Sovrapposizione { $start } – { $end }
calendar-resolver-reschedule = Sposta l'evento successivo
calendar-resolver-shorten = Accorcia l'evento precedente
calendar-resolver-tentative = Segna il successivo come provvisorio
calendar-resolver-double-book = Doppia prenotazione (segna libero)
calendar-resolver-keep = Mantieni entrambi
calendar-events-count-people = { $count ->
    [zero] Nessun partecipante
    [one] { $count } partecipante
   *[other] { $count } partecipanti
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = Calendari
calendar-fb-own = Propri
calendar-fb-unavailable = Impossibile caricare il tempo occupato.
calendar-fb-cell = { $principal } alle { $hour }: { $status }
calendar-fb-busy = Occupato
calendar-fb-tentative = Provvisorio
calendar-fb-free = Libero

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Aggiunta rapida evento
calendar-quick-add-placeholder = Aggiunta rapida, es. Pranzo venerdì 13:00
calendar-quick-add-btn = Aggiungi
calendar-quick-add-do = Aggiunta rapida

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categorie
calendar-add-category = Aggiungi categoria
calendar-category-placeholder = Aggiungi una categoria
calendar-remove-category = Rimuovi categoria { $name }
calendar-filter-heading = Filtro
calendar-filter-category = Filtra per categoria
calendar-filter-category-placeholder = Categoria da mostrare

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Allegati
calendar-attachment-title = Nome dell'allegato
calendar-attachment-title-placeholder = Nome (facoltativo)
calendar-attachment-uri = Link dell'allegato
calendar-attachment-add = Aggiungi allegato
calendar-remove-attachment = Rimuovi allegato { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Iscriviti tramite URL
calendar-subscribe-url = URL del calendario
calendar-subscribe-add = Iscriviti
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = Impossibile scaricare il calendario a quell'indirizzo, quindi non è stato aggiunto.
calendar-subscribe-hol-many-locations = Questo file di festività elenca più di un luogo. Scaricarlo, poi usare Importa e scegliere i luoghi desiderati.
calendar-subscribe-refresh = Aggiorna { $name }
calendar-subscribe-refresh-btn = Aggiorna
calendar-subscribe-refresh-failed = Impossibile scaricare il feed del calendario. I suoi eventi sono rimasti invariati.
calendar-import-failed = Impossibile leggere il file come calendario. Non è stato importato nulla.
calendar-import-done = { $count ->
    [one] 1 evento importato.
   *[other] { $count } eventi importati.
}
calendar-import-locations-title = Scegliere i luoghi da importare
calendar-import-locations-intro = Questo file contiene festività per { $count } luoghi. Vengono importati solo i luoghi selezionati.
calendar-import-locations-filter = Filtra luoghi
calendar-import-locations-list = Luoghi nel file
calendar-import-locations-none = Nessun luogo corrisponde al filtro.
calendar-import-locations-picked = { $count ->
    [one] 1 luogo selezionato.
   *[other] { $count } luoghi selezionati.
}
calendar-import-locations-import = Importa selezionati
calendar-export-failed = Impossibile esportare il calendario.
calendar-quick-add-failed = Impossibile aggiungere l'evento.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Condividi
calendar-share-for = Condividi { $name }
calendar-share-title = Condividi { $name }
calendar-share-intro = Le persone con cui si condivide possono vedere questo calendario. Concedere l'accesso in lettura e scrittura per consentire loro di aggiungere e modificare eventi.
calendar-share-people = Condiviso con
calendar-share-empty = Questo calendario non è condiviso con nessuno.
calendar-share-add = Aggiungi una persona
calendar-share-new-access = Accesso per la nuova persona
calendar-share-access-for = Accesso per { $principal }
calendar-share-remove = Rimuovi { $principal }
calendar-share-read = Sola lettura
calendar-share-readwrite = Lettura e scrittura

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = Selettore di data
calendar-event-range = { $title }, da { $start } a { $end }
calendar-busy-free = Appare come libero
calendar-event-with-status = { $event } ({ $status })
calendar-more-label = Altri { $count } { $count ->
    [one] evento
   *[other] eventi
  } il { $date }
calendar-fb-caption-people = Orari occupati nel giorno del conflitto, ora per ora, per te e per ogni persona invitata.
calendar-fb-unknown = Impossibile recuperare la disponibilità di { $principal }.
calendar-fb-loading = Caricamento della disponibilità…
calendar-import-itip-message = Questo file è una risposta o un annullamento inviato per email. Apri il messaggio a cui è allegato per applicarlo. Non è stato importato nulla.
calendar-import-skipped = { $count ->
    [one] { $count } evento era già nel calendario ed è stato lasciato com'è.
   *[other] { $count } eventi erano già nel calendario e sono stati lasciati come sono.
 }
calendar-invite-title = Invito del calendario
calendar-invite-sender = Da { $sender }
calendar-invite-add = Aggiungi al calendario
calendar-invite-update = Applica aggiornamento evento
calendar-invite-cancel = Applica annullamento
calendar-invite-reply = Registra risposta del partecipante
calendar-invite-new = Questo invito può essere aggiunto al tuo calendario.
calendar-invite-newer = L'organizzatore ha inviato un evento aggiornato.
calendar-invite-current = Il tuo calendario ha già questa versione.
calendar-invite-stale = Questo messaggio è più vecchio dell'evento nel tuo calendario.
calendar-invite-cancelled = L'organizzatore ha annullato questo evento.
calendar-invite-response = Un partecipante ha inviato una risposta.
calendar-invite-not-organizer = Il mittente non è l'organizzatore memorizzato. Non verrà modificato nulla.
calendar-invite-not-attendee = Il mittente non è un partecipante corrispondente. Non verrà modificato nulla.
calendar-invite-unknown = Questo messaggio si riferisce a un evento che non è nel tuo calendario.
calendar-invite-unsupported = Questa proposta non può essere applicata automaticamente.
calendar-invite-no-reply = Aggiorna il tuo calendario senza inviare una risposta.
calendar-invite-applied = Il tuo calendario è stato aggiornato.
calendar-invite-unchanged = L'evento non è stato modificato. Controlla lo stato attuale dell'invito.
calendar-invite-apply-failed = Impossibile applicare l'invito. Riprova.
calendar-invite-load-failed = Impossibile controllare l'invito.
calendar-invite-retry = Riprova
calendar-field-time-zone = Fuso orario dell'evento
calendar-field-time-zone-help = Usa un nome IANA come Europe/Lisbon. L'evento mantiene questo fuso quando viene visualizzato altrove.
calendar-time-invalid = Inserisci un fuso orario e un'ora locale validi. Alcune ore locali non esistono quando gli orologi vanno avanti.
calendar-mini-prev = Mese precedente
calendar-mini-next = Mese successivo
calendar-card-region-event = Evento del calendario
calendar-card-kind-request = Invito a riunione
calendar-card-kind-update = Invito aggiornato
calendar-card-kind-cancel = Riunione annullata
calendar-card-kind-reply = Risposta alla riunione
calendar-card-kind-counter = Nuovo orario proposto
calendar-card-kind-publish = { $count ->
    [one] Evento del calendario
   *[other] { $count } eventi del calendario
 }
calendar-card-when = Quando
calendar-card-where = Dove
calendar-card-organizer = Organizzatore
calendar-card-attendees = Partecipanti
calendar-card-all-day = { $when } (tutto il giorno)
calendar-card-organizer-time = Orario dell'organizzatore: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Ogni giorno
   *[other] Ogni { $interval } giorni
 }
calendar-card-repeat-weekly = { $interval ->
    [one] Ogni settimana
   *[other] Ogni { $interval } settimane
 }
calendar-card-repeat-weekly-on = { $interval ->
    [one] Ogni settimana: { $days }
   *[other] Ogni { $interval } settimane: { $days }
 }
calendar-card-repeat-monthly = { $interval ->
    [one] Ogni mese
   *[other] Ogni { $interval } mesi
 }
calendar-card-repeat-monthly-day = { $interval ->
    [one] Ogni mese, il giorno { $day }
   *[other] Ogni { $interval } mesi, il giorno { $day }
 }
calendar-card-repeat-yearly = { $interval ->
    [one] Ogni anno
   *[other] Ogni { $interval } anni
 }
calendar-card-repeat-until = { $rule } fino al { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] una volta
   *[other] { $count } volte
 }
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 data saltata
   *[other] { $count } date saltate
 })
calendar-card-repeat-custom = Si ripete secondo una pianificazione personalizzata
calendar-card-join = Partecipa con { $provider }
calendar-card-organizer-is = Organizzato da { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 partecipante
   *[other] { $count } partecipanti
 }
calendar-card-people-all = Mostra tutti ({ $count })
calendar-card-people-fewer = Mostra meno
calendar-card-rsvp-accepted = Accettato
calendar-card-rsvp-tentative = Provvisorio
calendar-card-rsvp-declined = Rifiutato
calendar-card-rsvp-none = Nessuna risposta
calendar-card-in-calendar = Nel tuo calendario
calendar-card-cancelled = Annullato
calendar-card-open = Apri nel calendario
calendar-card-agenda = Il tuo calendario del { $day }
calendar-card-agenda-title = La tua giornata
calendar-card-agenda-this-event = Questo evento
calendar-card-agenda-unavailable = Impossibile caricare il tuo calendario per questo giorno.
calendar-card-conflicts = { $count ->
    [one] In conflitto con { $titles }
   *[other] In conflitto con { $count } eventi: { $titles }
 }
calendar-card-no-conflicts = Nessun conflitto nel tuo calendario
calendar-card-invited-as = Sei stato invitato come { $address }.
calendar-card-not-invited = Questo invito non elenca nessuno dei tuoi indirizzi. Puoi aggiungerlo al calendario, ma non puoi rispondere.
calendar-card-not-invited-short = Questo invito non elenca nessuno dei tuoi indirizzi.
calendar-card-out-of-date = Questo invito non è aggiornato. Nel tuo calendario c'è già una versione più recente.
calendar-card-replied = { $who } { $answer ->
    [accepted] ha accettato.
    [tentative] ha accettato in via provvisoria.
    [declined] ha rifiutato.
   *[other] non ha ancora risposto.
 }
calendar-card-counter = { $who } ha proposto un nuovo orario. Le proposte non vengono applicate automaticamente.
calendar-card-todos-skipped = { $count ->
    [one] Il file contiene anche 1 attività, che non viene aggiunta.
   *[other] Il file contiene anche { $count } attività, che non vengono aggiunte.
 }
calendar-card-respond = Rispondi all'invito
calendar-card-accept = Accetta
calendar-card-tentative = Provvisorio
calendar-card-decline = Rifiuta
calendar-card-propose = Proponi nuovo orario
calendar-card-propose-start = Nuovo inizio
calendar-card-propose-send = Invia proposta
calendar-card-propose-invalid = Inserisci un nuovo orario di inizio valido.
calendar-card-send-response = Invia una risposta a { $organizer }
calendar-card-comment = Commento (facoltativo)
calendar-card-calendar = Calendario
calendar-card-events = Eventi in questo file
calendar-card-add-one = Aggiungi
calendar-card-add-all = Aggiungi tutti ({ $count })
calendar-card-added = { $count ->
    [one] Aggiunto al tuo calendario.
   *[other] { $count } eventi aggiunti al tuo calendario.
 }
calendar-card-remove = Rimuovi dal calendario
calendar-card-removed = Rimosso dal tuo calendario.
calendar-card-record-reply = Aggiorna stato del partecipante
calendar-card-answered-sent = { $answer }. La tua risposta è stata inviata a { $organizer }.
calendar-card-answered-not-sent = { $answer }. Il tuo calendario è stato aggiornato, ma non è stato possibile inviare la risposta.
calendar-card-answered-silently = { $answer }. Nessuna risposta è stata inviata.
calendar-card-proposal-sent = La tua proposta è stata inviata a { $organizer }.
calendar-card-not-sent = Impossibile inviare il messaggio.
calendar-card-more = { $count ->
    [one] altro 1
   *[other] altri { $count }
 }
