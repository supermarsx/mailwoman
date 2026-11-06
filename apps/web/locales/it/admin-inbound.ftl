# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Accesso in entrata
admin-inbound-help = Limita gli IP sorgente attendibili per la console di amministrazione e la webmail separatamente. Entrambi i criteri partono disattivati. Queste regole non modificano i proxy attendibili, gli ambiti API né l'accesso in uscita.
admin-inbound-unscoped = Questo criterio valido per tutta l'installazione richiede un amministratore con accesso di sicurezza senza ambito.
admin-inbound-admin = Console di amministrazione e API
admin-inbound-admin-help = Include l'accesso e questo editor. L'autenticazione esistente dell'amministratore non aggira il criterio di rete.
admin-inbound-webmail = Webmail e API degli account
admin-inbound-webmail-help = Copre posta, DAV, contenuti firmati, SSO, OIDC e percorsi API. Le connessioni già aggiornate mantengono la decisione presa durante l'handshake.
admin-inbound-enabled = Limita questa superficie all'elenco consentito
admin-inbound-cidrs = IP e CIDR di emergenza
admin-inbound-cidrs-help = Un IP o CIDR letterale per riga, fino a 128. Almeno uno è obbligatorio quando è attivo. Includi una sorgente attendibile che può usare per il ripristino.
admin-inbound-hostnames = Nomi host DNS consentiti
admin-inbound-hostnames-help = Un nome DNS ASCII minuscolo per riga. Usa IDNA ASCII per i nomi internazionali. Niente URL, porte, caratteri jolly, IP letterali o punti finali. I nomi concedono solo gli indirizzi attualmente risolti.
admin-inbound-dns = Limiti di risoluzione DNS
admin-inbound-dns-help = Nomi nuovi, scaduti e non riusciti non concedono alcun accesso. Le risposte riuscite rispettano il loro TTL DNS e il limite di cache configurato. Le richieste non attendono mai la risoluzione DNS.
admin-inbound-servers = Server DNS ricorsivi
admin-inbound-servers-help = Lascia vuoto per usare i resolver di sistema, oppure inserisci fino a quattro socket IP:porta, uno per riga (per esempio 10.0.0.53:53 o [fd00::53]:53). I server espliciti sostituiscono i resolver di sistema senza ripiego pubblico. L'attendibilità del resolver è una sua responsabilità; non si dichiara una validazione DNSSEC indipendente.
admin-inbound-dns-timeoutMs = Timeout di ricerca (millisecondi)
admin-inbound-dns-cacheSeconds = Durata massima della cache (secondi)
admin-inbound-dns-maxHosts = Numero massimo di nomi host distinti
admin-inbound-dns-maxAddresses = Numero massimo di indirizzi per nome host
admin-inbound-dns-concurrency = Ricerche simultanee
admin-inbound-bounds = Intervallo consentito: { $min }–{ $max }.
admin-inbound-lockout = Questo criterio può bloccare immediatamente il suo stesso accesso, compreso questo editor. Verifica che gli IP di emergenza coprano la sua sorgente effettiva, come determinata dai proxy attendibili configurati. Non esiste alcun bypass automatico per loopback o per l'amministratore con accesso effettuato. Il recupero da un blocco richiede un ripristino lato installazione; il normale accesso non può aggirarlo.
admin-inbound-confirm-title = Applicare il criterio di accesso in entrata?
admin-inbound-confirm-word = APPLICA
admin-inbound-save = Applica il criterio confermato
admin-inbound-review = Controlla le modifiche
admin-inbound-refresh = Aggiorna il criterio salvato e lo stato DNS
admin-inbound-reset = Ripristina la bozza al criterio caricato
admin-inbound-saved = Il server ha accettato il criterio di accesso in entrata.
admin-inbound-conflict = Il criterio salvato è cambiato mentre lo modificavi. Ripristina la bozza e aggiorna per caricare la revisione corrente prima di modificare di nuovo.
admin-inbound-no-revision = Il server non ha restituito una revisione del criterio. Aggiorna prima di salvare.
admin-inbound-status = Concessioni DNS attuali
admin-inbound-status-help = È un'istantanea dell'ultimo aggiornamento dello stato, non un conto alla rovescia né una prova di connettività. Aggiorna per vedere le concessioni attuali. Un errore può coesistere con una risposta precedente ancora valida fino alla sua scadenza originale.
admin-inbound-refreshing = È in corso un aggiornamento DNS limitato.
admin-inbound-idle = Nessun aggiornamento DNS era in corso all'ultima lettura dello stato.
admin-inbound-no-hosts = Nessun nome host è attivo nei criteri abilitati.
admin-inbound-hostname = Nome host
admin-inbound-state = Concessione di accesso
admin-inbound-addresses = Indirizzi utilizzabili
admin-inbound-validity = Validità all'ultimo aggiornamento
admin-inbound-error = Codice del risultato DNS
admin-inbound-usable = Utilizzabile
admin-inbound-unavailable = Nessuna concessione attuale
admin-inbound-seconds = { $seconds } secondi
admin-inbound-on = Attivo
admin-inbound-off = Disattivato
admin-inbound-invalid-dns = I limiti DNS o il numero di resolver sono fuori dai limiti consentiti.
admin-inbound-rescue-required = Ogni superficie attiva richiede almeno un IP o CIDR di emergenza.
admin-inbound-too-many = Il criterio supera il limite configurato di nomi host o indirizzi letterali.
admin-inbound-invalid-host = Usa nomi DNS ASCII minuscoli senza URL, porte, caratteri jolly, IP letterali o punti finali.
