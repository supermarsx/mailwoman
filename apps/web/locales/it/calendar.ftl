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
