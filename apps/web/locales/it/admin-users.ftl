# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Utenti. Le intestazioni di colonna ordinano l'elenco.
admin-userlist-search = Cerca utenti
admin-userlist-search-placeholder = Nome, indirizzo, alias o dominio
admin-userlist-count = { $count ->
    [one] { $count } utente
   *[other] { $count } utenti
 }
admin-userlist-count-filtered = { $count ->
    [one] { $count } utente su { $total } corrisponde
   *[other] { $count } utenti su { $total } corrispondono
 }
admin-userlist-range = { $from }–{ $to } di { $total }
admin-userlist-loading = Caricamento degli utenti…
admin-userlist-retry = Riprova
admin-userlist-empty-filtered = Nessun utente corrisponde a questa ricerca e a questi filtri.
admin-userlist-compact = Righe compatte
admin-userlist-columns = Colonne
admin-userlist-columns-legend = Mostra colonne
admin-userlist-actions = Azioni
admin-userlist-paging = Pagine di utenti
admin-userlist-page-size = Righe per pagina
admin-userlist-prev = Precedente
admin-userlist-next = Successiva
admin-userlist-sort-by = Ordina per
admin-userlist-sort-asc = { $column }, crescente
admin-userlist-sort-desc = { $column }, decrescente
admin-userlist-col-user = Utente
admin-userlist-col-domain = Dominio
admin-userlist-col-status = Stato
admin-userlist-col-2fa = Verifica in due passaggi
admin-userlist-col-role = Ruolo console
admin-userlist-col-last-signin = Ultimo accesso
admin-userlist-col-last-seen = Ultima attività
admin-userlist-col-created = Creato il
admin-userlist-col-quota = Quota
admin-userlist-col-profile = Profilo funzionalità
admin-userlist-col-linked = Caselle collegate
admin-userlist-state-active = Attivo
admin-userlist-state-pending = Mai acceduto
admin-userlist-state-blocked = Bloccato
admin-userlist-state-disabled = Disabilitato
admin-userlist-state-deleted = Eliminato
admin-userlist-2fa-passkey = Passkey
admin-userlist-2fa-totp = App di autenticazione
admin-userlist-2fa-none = Nessuna
admin-userlist-signin-never = Mai
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Nessuna attività registrata
admin-userlist-seen-at = Ultima attività { $relative } ({ $exact })
admin-userlist-quota-unlimited = Illimitata
admin-userlist-profile-default = Predefinito
admin-userlist-profile-via-user = Assegnato a questo utente
admin-userlist-profile-via-group = Da un gruppo
admin-userlist-profile-via-domain = Dal dominio
admin-userlist-more = Altre azioni
admin-userlist-more-for = Altre azioni per { $account }
admin-userlist-select = Seleziona { $account }
admin-userlist-select-page = Seleziona tutti gli utenti di questa pagina
admin-userlist-filters = Filtri
admin-userlist-add-filter = Aggiungi filtro
admin-userlist-clear-all = Cancella tutto
admin-userlist-chip-edit = Modifica filtro { $label }
admin-userlist-chip-remove = Rimuovi filtro { $label }
admin-userlist-filter-domain = Dominio
admin-userlist-filter-status = Stato
admin-userlist-filter-tfa = Verifica in due passaggi
admin-userlist-filter-role = Ruolo console
admin-userlist-filter-profile = Profilo funzionalità
admin-userlist-filter-signin = Ultimo accesso
admin-userlist-filter-created = Creato il
admin-userlist-filter-no-choices = Ancora nulla tra cui scegliere.
admin-userlist-filter-remove = Rimuovi filtro
admin-userlist-apply = Applica
admin-userlist-cancel = Annulla
admin-userlist-role-any = Qualsiasi ruolo console
admin-userlist-role-none = Nessun ruolo console
admin-userlist-profile-none = Nessun profilo (predefinito)
admin-userlist-signin-older = { $days ->
    [one] Più di { $days } giorno fa
   *[other] Più di { $days } giorni fa
 }
admin-userlist-signin-within = { $days ->
    [one] Nell'ultimo { $days } giorno
   *[other] Negli ultimi { $days } giorni
 }
admin-userlist-signin-custom = Più di questo numero di giorni fa:
admin-userlist-signin-days = Giorni
admin-userlist-days-invalid = Inserire un numero intero di giorni da 1 a 36500.
admin-userlist-created-from = Da
admin-userlist-created-to = A
admin-userlist-created-between = dal { $from } al { $to }
admin-userlist-created-since = dal { $date }
admin-userlist-created-until = fino al { $date }
admin-userlist-dates-invalid = La data di inizio è successiva alla data di fine.
admin-userlist-views = Viste salvate
admin-userlist-view-save = Salva questa vista…
admin-userlist-view-name = Nome della vista
admin-userlist-view-save-confirm = Salva vista
admin-userlist-view-delete = Elimina la vista “{ $name }”
admin-userlist-bulk = Azioni sugli utenti selezionati
admin-userlist-selected = { $count ->
    [one] { $count } selezionato
   *[other] { $count } selezionati
 }
admin-userlist-clear-selection = Deseleziona tutto
admin-userlist-block = Blocca accesso
admin-userlist-unblock = Sblocca accesso
admin-userlist-sign-out = Disconnetti ovunque
admin-userlist-delete = Elimina accesso locale
admin-userlist-bulk-profile = Assegna profilo funzionalità
admin-userlist-bulk-profile-clear = Rimuovi il profilo proprio dell'utente
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } utente selezionato viene ignorato: l'azione non si applica.
   *[other] { $count } utenti selezionati vengono ignorati: l'azione non si applica.
 }
admin-userlist-confirm-block = { $count ->
    [one] Bloccare l'accesso di { $count } utente?
   *[other] Bloccare l'accesso di { $count } utenti?
 }
admin-userlist-confirm-block-one = Bloccare l'accesso di { $account }?
admin-userlist-confirm-block-body = I nuovi accessi vengono rifiutati e le sessioni Mailwoman in corso terminano. La casella upstream non viene modificata.
admin-userlist-confirm-block-action = Blocca accesso
admin-userlist-confirm-unblock = { $count ->
    [one] Sbloccare l'accesso di { $count } utente?
   *[other] Sbloccare l'accesso di { $count } utenti?
 }
admin-userlist-confirm-unblock-one = Sbloccare l'accesso di { $account }?
admin-userlist-confirm-unblock-body = Il blocco locale dell'accesso viene rimosso. Le altre restrizioni di sicurezza restano in vigore.
admin-userlist-confirm-unblock-action = Sblocca accesso
admin-userlist-confirm-signout = { $count ->
    [one] Disconnettere ovunque { $count } utente?
   *[other] Disconnettere ovunque { $count } utenti?
 }
admin-userlist-confirm-signout-one = Disconnettere { $account } ovunque?
admin-userlist-confirm-signout-body = Tutte le sessioni Mailwoman aperte di questi utenti terminano. Potranno accedere di nuovo.
admin-userlist-confirm-signout-action = Disconnetti
admin-userlist-confirm-delete = { $count ->
    [one] Eliminare l'accesso locale di { $count } utente?
   *[other] Eliminare l'accesso locale di { $count } utenti?
 }
admin-userlist-confirm-delete-one = Eliminare l'accesso locale di { $account }?
admin-userlist-confirm-delete-body = Un record di accesso eliminato viene conservato affinché l'utente non venga registrato di nuovo automaticamente. Le impostazioni di Mailwoman, la cronologia e la casella upstream restano memorizzate. Digitare l'indirizzo per confermare.
admin-userlist-confirm-delete-action = Elimina accesso locale
admin-userlist-confirm-profile = { $count ->
    [one] Modificare il profilo funzionalità di { $count } utente?
   *[other] Modificare il profilo funzionalità di { $count } utenti?
 }
admin-userlist-confirm-profile-one = Modificare il profilo funzionalità di { $account }?
admin-userlist-confirm-profile-body = Il profilo viene assegnato direttamente a ciascun utente e si applica dalla richiesta successiva. Sono supportate le identità di casella native, proxy e SSO.
admin-users-profile-help = Selezionare Predefinito per rimuovere l'assegnazione diretta di questo utente e usare i profili ereditati dai suoi gruppi o dal dominio.
admin-users-profile-ambiguous = Questo utente ha più identità di accesso. Scegliere l'identità desiderata in Profili funzionalità.
admin-users-profile-unavailable = Nessuna identità di casella verificata è disponibile. L'utente deve accedere prima che sia possibile assegnare un profilo.
admin-userlist-confirm-profile-action = Cambia profilo
admin-userlist-bulk-done-block = { $count ->
    [one] Accesso bloccato per { $count } utente.
   *[other] Accesso bloccato per { $count } utenti.
 }
admin-userlist-bulk-done-unblock = { $count ->
    [one] Accesso sbloccato per { $count } utente.
   *[other] Accesso sbloccato per { $count } utenti.
 }
admin-userlist-bulk-done-signout = { $count ->
    [one] { $count } utente disconnesso.
   *[other] { $count } utenti disconnessi.
 }
admin-userlist-bulk-done-delete = { $count ->
    [one] Accesso locale eliminato per { $count } utente.
   *[other] Accesso locale eliminato per { $count } utenti.
 }
admin-userlist-bulk-done-profile = { $count ->
    [one] Profilo funzionalità modificato per { $count } utente.
   *[other] Profilo funzionalità modificato per { $count } utenti.
 }
admin-userlist-bulk-failed = { $done } completati, { $failed } non riusciti: { $reason }

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-user-compliance-due = dovuto { $date }

admin-user-compliance-empty = Nessuna richiesta di dati per questo account.

admin-user-compliance-help = Richieste presentate per gli indirizzi di questo account. Aprine uno in Richieste dati per agire su di esso.

admin-user-compliance-open-all = Apri tutte le richieste di dati

admin-user-compliance-title = Richieste di dati per questo account

admin-user-never = Mai

admin-user-overview-last-signin = Ultimo accesso

admin-user-overview-status = Stato

admin-user-status-active = Attivo

admin-user-status-deleted = Eliminato

admin-user-status-deleting = Eliminazione

admin-user-status-disabled = Bloccato

admin-user-tab-access = Accesso e rimozione

admin-user-tab-compliance = Conformità

admin-user-tab-overview = Panoramica

admin-user-tab-profile = Profilo delle funzionalità

admin-user-tab-security = Sicurezza

admin-user-tab-sessions = Sessioni

admin-user-tabs-label = Sezioni del conto
