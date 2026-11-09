# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Rapporto sullo stato di consegna
dsn-unverified = Questo rapporto descrive il tentativo di consegna del mittente. Quanto dichiara non è verificato in modo indipendente. La consegna non significa che il messaggio sia stato letto.
dsn-reporting-mta = Server segnalante:
dsn-original-recipient = Destinatario originale:
dsn-remote-mta = Server remoto:
dsn-last-attempt = Ultimo tentativo:
dsn-retry-until = Il mittente prevede di riprovare fino a:
dsn-action-failed = Errore segnalato
dsn-action-delayed = Ritardo segnalato
dsn-action-delivered = Consegna segnalata
dsn-action-relayed = Inoltro segnalato
dsn-action-expanded = Espansione della lista segnalata

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Utilizza la diagnostica del server quando chiedi aiuto al tuo provider di posta elettronica.

dsn-advice-1 = Controlla sia l'indirizzo del mittente che quello del destinatario. Se necessario, conferma l'indirizzo attuale del destinatario.

dsn-advice-2 = Contatta il destinatario in un altro modo se il problema della casella di posta persiste.

dsn-advice-3 = Condividi questo rapporto con l'amministratore del sistema di posta ricevente se il problema persiste.

dsn-advice-4 = Chiedi al tuo amministratore di posta di verificare la connettività, DNS e i percorsi di inoltro se il problema persiste.

dsn-advice-5 = Condividi la diagnostica con il tuo provider di posta elettronica in modo che possano verificare la compatibilità e i limiti di SMTP.

dsn-advice-6 = Controlla il formato del messaggio, gli indirizzi internazionali e gli allegati rispetto ai requisiti del destinatario.

dsn-advice-7 = Condividi la diagnostica con il tuo amministratore di posta per identificare l'esatta regola di sicurezza o autenticazione.

dsn-advice-success = Controlla l'azione di consegna segnalata sopra per verificare se il messaggio è stato recapitato, inoltrato o espanso in un elenco.

dsn-category-0 = Il server ha fornito un codice dettaglio non riconosciuto o specifico del provider. La diagnostica riportata di seguito può spiegare la causa esatta.

dsn-category-1 = Questo è un codice relativo all'indirizzo. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-2 = Questo è un codice della casella di posta del destinatario. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-3 = Questo è un codice del sistema di ricezione della posta. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-4 = Si tratta di un codice di rete o di routing. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-5 = Questo è un codice di protocollo di posta. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-6 = Questo è un codice del contenuto o del formato del messaggio. Il dettaglio preciso è specifico del fornitore o non riconosciuto.

dsn-category-7 = Questo è un codice di sicurezza o di policy. Il dettaglio preciso è specifico del fornitore o non riconosciuto; di per sé non stabilisce che il messaggio sia spam.

dsn-class-2 = Successo: il server segnala un passaggio di consegna positivo. Ciò non prova che il messaggio sia stato letto.

dsn-class-4 = Fallimento temporaneo: un tentativo successivo potrebbe avere successo. Il mittente può comunque ritentare una segnalazione di ritardo; una segnalazione di fallimento indica che il tentativo è terminato.

dsn-class-5 = Fallimento permanente: è improbabile che il reinvio dello stesso messaggio senza modifiche funzioni. Correggere la causa prima di inviare nuovamente.

dsn-meaning-title = Cosa significa questo codice

dsn-next-step = Cosa fare

dsn-provider-barracuda = Barracuda

dsn-provider-detected = Interpretazione basata sulla diagnostica del server o sul nome host:

dsn-provider-exchange = MicrosoftExchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail/Google Workspace

dsn-provider-help = Documentazione del fornitore

dsn-provider-mimecast = Mimecast

dsn-provider-proofpoint = Punto di prova

dsn-provider-relay = Relè SMTP

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnostica del server

dsn-status-0-0 = Il server non ha fornito una ragione più specifica.

dsn-status-1-0 = Si è verificato un problema con un indirizzo email.

dsn-status-1-1 = La casella di posta del destinatario non esiste. Conferma l'indirizzo con il destinatario.

dsn-status-1-10 = Il dominio destinatario dichiara esplicitamente di non accettare email (null MX). Utilizza un altro indirizzo.

dsn-status-1-2 = Il dominio destinatario non può ricevere questa posta. Controllare la parte dopo @.

dsn-status-1-3 = L'indirizzo del destinatario ha un formato non valido. Controlla eventuali errori di battitura o spazi aggiuntivi.

dsn-status-1-4 = Il server non può identificare un destinatario univoco per questo indirizzo.

dsn-status-1-5 = L'indirizzo del destinatario è stato accettato come valido.

dsn-status-1-6 = Questa casella di posta è stata spostata e non è disponibile alcun indirizzo di inoltro. Richiedi il nuovo indirizzo.

dsn-status-1-7 = L'indirizzo del mittente ha un formato non valido. Controlla l'indirizzo di invio dell'account.

dsn-status-1-8 = Il dominio del mittente non è valido o non può ricevere posta di ritorno.

dsn-status-1-9 = Il messaggio è stato consegnato a un sistema che non è in grado di fornire ulteriori rapporti di consegna.

dsn-status-2-0 = La casella di posta del destinatario presenta un problema non specificato.

dsn-status-2-1 = La casella di posta del destinatario è disabilitata o non accetta messaggi. Contatta il destinatario in un altro modo.

dsn-status-2-2 = La casella di posta del destinatario è piena. Il destinatario deve liberare spazio o aumentare la propria quota.

dsn-status-2-3 = Il messaggio supera il limite di dimensione del destinatario. Riduci gli allegati o utilizza un collegamento di condivisione.

dsn-status-2-4 = Il server non è riuscito a risolvere i membri di una mailing list. Contatta il proprietario della lista.

dsn-status-3-0 = Il sistema di posta ricevente ha segnalato un problema non specificato.

dsn-status-3-1 = Il server di posta ricevente ha esaurito lo spazio di archiviazione. Il suo amministratore deve liberare spazio.

dsn-status-3-2 = Il server ricevente non accetta la posta, ad esempio durante la manutenzione o un sovraccarico.

dsn-status-3-3 = Il sistema ricevente non supporta una funzionalità richiesta da questo messaggio.

dsn-status-3-4 = Il messaggio supera il limite di dimensione del server ricevente. Riduci gli allegati o utilizza un collegamento di condivisione.

dsn-status-3-5 = Un problema di configurazione del server di posta impedisce la consegna. Chiedi all'amministratore della posta di indagare.

dsn-status-3-6 = Il server ha accettato il messaggio ma ha modificato la priorità di consegna richiesta.

dsn-status-4-0 = È stato segnalato un problema di rete o di instradamento della posta senza ulteriori dettagli.

dsn-status-4-1 = Il server ricevente non ha risposto al tentativo di connessione.

dsn-status-4-2 = La connessione è stata stabilita ma interrotta prima del completamento del trasferimento della posta.

dsn-status-4-3 = Una ricerca DNS o nella directory non è riuscita. Potrebbe essere necessario che un amministratore di posta controlli la risoluzione dei nomi.

dsn-status-4-4 = Il server non è riuscito a trovare un percorso verso il sistema di posta del destinatario.

dsn-status-4-5 = La congestione del traffico postale ha impedito la consegna.

dsn-status-4-6 = È stato rilevato un loop di inoltro. Chiedi all'amministratore di verificare le regole di inoltro e routing.

dsn-status-4-7 = Il termine di consegna è scaduto. Controllare la diagnostica per individuare il problema di fondo prima di inviare nuovamente.

dsn-status-5-0 = I server di posta non sono riusciti a completare lo scambio del protocollo di consegna.

dsn-status-5-1 = Un server di posta ha rifiutato un comando non supportato o un comando inviato nell'ordine sbagliato.

dsn-status-5-2 = Un server di posta non è riuscito a comprendere la sintassi di un comando.

dsn-status-5-3 = Il messaggio ha troppi destinatari. Invia a gruppi più piccoli.

dsn-status-5-4 = Un server di posta ha rifiutato i parametri di un comando.

dsn-status-5-5 = I server utilizzano versioni incompatibili del protocollo di posta.

dsn-status-5-6 = La risposta di autenticazione è stata troppo lunga per il server. Chiedi all'amministratore della posta di verificare le impostazioni di autenticazione.

dsn-status-6-0 = Il server ha segnalato un problema non specificato con il contenuto o il formato del messaggio.

dsn-status-6-1 = Il sistema ricevente non supporta il tipo di contenuto di questo messaggio.

dsn-status-6-10 = Questo codice obsoleto significa che è richiesta una risposta internazionale (UTF-8) ma non supportata, come con il codice x.6.8.

dsn-status-6-2 = La consegna richiede una conversione del contenuto vietata dalla policy del mittente o del server.

dsn-status-6-3 = La consegna richiede una conversione del contenuto che il server non può eseguire.

dsn-status-6-4 = Una conversione del contenuto ha perso alcune informazioni. Controllare il messaggio originale e la diagnostica del server.

dsn-status-6-5 = Si è verificato un errore nel server durante la conversione del contenuto del messaggio.

dsn-status-6-6 = Il server non è riuscito a recuperare il contenuto del messaggio da un altro sistema.

dsn-status-6-7 = Un indirizzo email internazionale non è supportato per questo mittente o destinatario. Richiedi un indirizzo alternativo.

dsn-status-6-8 = Il server necessita di una risposta internazionale (UTF-8) che il sistema mittente non può accettare.

dsn-status-6-9 = Il messaggio contiene intestazioni internazionali (UTF-8) che non possono essere consegnate ad almeno un destinatario.

dsn-status-7-0 = Una regola di sicurezza o un criterio di posta ha impedito la normale consegna. Controllare la diagnostica del server per la regola esatta.

dsn-status-7-1 = Il server ha rifiutato l'autorizzazione alla consegna, probabilmente a causa di restrizioni di filtraggio o di inoltro. Chiedi all'amministratore della posta di controllare la diagnostica.

dsn-status-7-10 = Il server richiede una connessione crittografata o una protezione più forte prima dell'autenticazione.

dsn-status-7-11 = Questo metodo di autenticazione richiede una connessione crittografata SMTP (TLS).

dsn-status-7-12 = L'account necessita di un metodo di autenticazione o di migrazione della password. Contatta il fornitore dell'account.

dsn-status-7-13 = L'account di invio è disabilitato. Contatta il suo amministratore per ripristinare l'accesso.

dsn-status-7-14 = Il server necessita di una relazione di trust configurata per recuperare il messaggio. Contatta il suo amministratore.

dsn-status-7-15 = La priorità del messaggio è inferiore al minimo corrente del server ricevente.

dsn-status-7-16 = Il messaggio è troppo grande per la priorità richiesta. Riduci le sue dimensioni o chiedi all'amministratore.

dsn-status-7-17 = La casella di posta del destinatario ha cambiato proprietà rispetto alla data richiesta. Conferma il destinatario prima di inviare nuovamente.

dsn-status-7-18 = Il dominio destinatario ha cambiato proprietà rispetto alla data richiesta. Confermare la destinazione prima di inviare nuovamente.

dsn-status-7-19 = Il server non è riuscito a verificare la proprietà continua dell'indirizzo del destinatario. Chiedi all'amministratore informazioni sul controllo di validità del destinatario.

dsn-status-7-2 = Al mittente non è consentito postare su questa mailing list. Contatta il proprietario della lista.

dsn-status-7-20 = Nessuna firma DKIM ha superato la verifica. Chiedi all'amministratore della posta del mittente di verificare la firma e DNS.

dsn-status-7-21 = È stata passata una firma DKIM, ma la politica del server ricevente non l'ha accettata.

dsn-status-7-22 = Nessuna firma accettata DKIM corrispondeva all'indirizzo del mittente. Chiedi all'amministratore della posta di verificare l'allineamento del dominio di firma.

dsn-status-7-23 = SPF fallito: il server di invio non era autorizzato dalla policy SPF del dominio del mittente.

dsn-status-7-24 = Il server ha riscontrato un errore durante il controllo di SPF. Chiedi all'amministratore della posta di controllare i record SPF e DNS.

dsn-status-7-25 = Il server mittente non ha superato il controllo inverso DNS del destinatario. Il suo amministratore deve controllare i record DNS dell'indirizzo IP.

dsn-status-7-26 = Diversi controlli di autenticazione dei messaggi non sono riusciti. La diagnostica può identificare problemi relativi ai criteri SPF, DKIM o DMARC.

dsn-status-7-27 = Il dominio del mittente dichiara di non accettare email di ritorno (null MX), che il destinatario rifiuta.

dsn-status-7-28 = Il sistema ricevente ha rilevato una possibile marea di messaggi offensivi.

dsn-status-7-29 = La verifica ARC non è riuscita, quindi il destinatario non ha potuto accettare la catena di autenticazione registrata del messaggio.

dsn-status-7-3 = La consegna richiede una conversione del formato di sicurezza che il server non può eseguire.

dsn-status-7-30 = Il messaggio richiede REQUIRETLS, ma il server di posta successivo non lo supporta. Chiedi all'amministratore di verificare il supporto per la consegna sicura.

dsn-status-7-4 = Il sistema di consegna non supporta una funzionalità di sicurezza richiesta.

dsn-status-7-5 = Il server non è riuscito a convalidare o decrittografare il messaggio, ad esempio perché una chiave mancava o non era valida.

dsn-status-7-6 = Il server non supporta un algoritmo crittografico richiesto dal messaggio.

dsn-status-7-7 = Un controllo di integrità indica che il messaggio è stato modificato o danneggiato.

dsn-status-7-8 = Il server mittente ha rifiutato le credenziali di accesso. Controlla le impostazioni di autenticazione dell'account.

dsn-status-7-9 = Il metodo di autenticazione è più debole di quanto consentito dal server. È necessario un metodo più forte.

dsn-vendor-advice-account = Chiedi all'amministratore mittente di indagare su possibili abusi dell'account, proteggere l'account e richiedere la rimozione della restrizione.

dsn-vendor-advice-admin = Condividi questa diagnostica con l'amministratore di posta dell'organizzazione per esaminare la configurazione o il criterio corrispondente.

dsn-vendor-advice-auth = Controlla il metodo di autenticazione e l'autorizzazione SMTP supportati dall'account. Completa qualsiasi accesso richiesto tramite la pagina dell'account ufficiale del provider.

dsn-vendor-advice-authentication = Chiedi all'amministratore del mittente di verificare SPF, DKIM, DMARC e l'allineamento del dominio utilizzando la diagnostica esatta.

dsn-vendor-advice-connector = Chiedi all'amministratore di verificare l'autorizzazione all'inoltro, la corrispondenza del connettore IP/certificato, i domini accettati e l'effettivo percorso della posta.

dsn-vendor-advice-content = Esaminare i registri diagnostici e del gateway con l'amministratore del destinatario per identificare il contenuto bloccato e la policy applicabile.

dsn-vendor-advice-dns = Chiedi all'amministratore della posta mittente o al provider IP di correggere DNS, invertire DNS e l'identità del server SMTP.

dsn-vendor-advice-domain = Chiedi all'amministratore di Microsoft 365 di registrare e verificare il dominio di invio ed esaminare i limiti di invio del tenant.

dsn-vendor-advice-format = Chiedere all'applicazione mittente o all'amministratore della posta di correggere la generazione del messaggio, quindi creare un nuovo messaggio.

dsn-vendor-advice-rate = Riduci il volume di invio e attendi che la quota o la restrizione venga cancellata. Controlla l'azione segnalata prima di presumere che questo tentativo sia ancora in coda.

dsn-vendor-advice-recipient-admin = Contatta il destinatario o il proprietario del gruppo in un altro modo e chiedi al suo amministratore di rivedere l'autorizzazione o la policy pertinente.

dsn-vendor-advice-reputation = Chiedi all'amministratore mittente di indagare sul traffico e sulla reputazione indesiderati, quindi segui la procedura ufficiale di revisione o cancellazione dall'elenco del fornitore.

dsn-vendor-advice-route = Chiedi all'amministratore della posta di controllare la destinazione, le regole di inoltro, il servizio di posta e il percorso del firewall/rete.

dsn-vendor-advice-tls = Chiedi agli amministratori di posta di riparare TLS, certificati o record di sicurezza DNS in modo che il percorso sicuro richiesto possa avere successo.

dsn-vendor-barracuda-reputation = Il servizio di reputazione di Barracuda elenca o blocca l'indirizzo di invio IP.

dsn-vendor-certificate-chain = Non è stato possibile convalidare il certificato TLS remoto rispetto all'attendibilità richiesta o alla policy MTA-STS.

dsn-vendor-certificate-expired = Il server di posta di destinazione ha presentato un certificato TLS scaduto.

dsn-vendor-certificate-host = Il certificato di destinazione TLS non corrisponde al nome host del server di posta richiesto.

dsn-vendor-connection-refused = Il server di destinazione ha rifiutato la connessione di rete. È necessario verificare il servizio di posta, il firewall o il percorso di rete.

dsn-vendor-content-security = Il gateway ha rifiutato il contenuto del messaggio o un allegato in base alle regole di filtro di sicurezza. Consultare la diagnostica o i registri per l'elemento specifico.

dsn-vendor-credentials = Il servizio SMTP ha rifiutato le credenziali di accesso fornite.

dsn-vendor-dane = Il certificato TLS o la configurazione TLSA della destinazione non hanno soddisfatto la convalida DANE.

dsn-vendor-dkim = Il destinatario non ha accettato l'autenticazione DKIM o la configurazione della firma del messaggio.

dsn-vendor-dmarc = La politica DMARC del destinatario non è stata soddisfatta. Verificare l'autenticazione SPF o DKIM e l'allineamento con il dominio Da visibile.

dsn-vendor-dnssec = Impossibile convalidare i record DNSSEC del dominio di destinazione.

dsn-vendor-exchange-auth = L'applicazione o il dispositivo non è stato autenticato correttamente per l'invio a Microsoft 365 SMTP.

dsn-vendor-exchange-connector = Microsoft non ha potuto autorizzare l'inoltro tramite il connettore tenant configurato. Il relativo certificato, l'invio di IP o il dominio accettato potrebbero non corrispondere più.

dsn-vendor-exchange-domain-blocked = Microsoft ha limitato questo dominio di invio dopo aver rilevato una sospetta attività di spam.

dsn-vendor-exchange-forward-relay = Un sistema a valle ha rifiutato la posta inoltrata o inoltrata. L'autenticazione potrebbe essere stata influenzata dall'inoltro o da un gateway non Microsoft.

dsn-vendor-exchange-forwarding = I criteri di Microsoft 365 bloccano l'inoltro automatico a destinatari esterni. L'amministratore dell'organizzazione deve rivedere la politica di inoltro.

dsn-vendor-exchange-generic = Exchange ha segnalato un rifiuto generale di consegna. La diagnostica identifica il problema effettivo, che potrebbe riguardare la formattazione del messaggio o i criteri per gli allegati.

dsn-vendor-exchange-group-allow = Il mittente non è presente nell'elenco dei mittenti consentiti di questo gruppo Exchange.

dsn-vendor-exchange-group-internal = Questo gruppo di distribuzione di Exchange rifiuta i mittenti esterni alla propria organizzazione a meno che il proprietario o l'amministratore non lo consentano.

dsn-vendor-exchange-internal-only = Questo destinatario di Exchange accetta mittenti autenticati dalla propria organizzazione; questo messaggio è stato considerato esterno o non autenticato.

dsn-vendor-exchange-ipv6 = La destinazione non accetta questa consegna oltre IPv6. L'amministratore mittente deve verificare il percorso o utilizzare un trasporto accettato.

dsn-vendor-exchange-journaling = L'organizzazione sta tentando di inserire nel journal i messaggi locali in Microsoft 365 senza la configurazione di archivio richiesta.

dsn-vendor-exchange-onmicrosoft-limit = L'organizzazione ha raggiunto il limite di destinatari esterni per il suo dominio di invio onmicrosoft.com. L'invio regolare deve utilizzare un dominio personalizzato verificato.

dsn-vendor-exchange-public-folder = Questa cartella pubblica di Exchange non accetta questo mittente esterno o non autenticato.

dsn-vendor-exchange-receive-rate = Questo destinatario di Exchange ha raggiunto il limite di ricezione oraria per tutti i mittenti.

dsn-vendor-exchange-recipient = Exchange non è riuscito a risolvere questo destinatario nella sua directory. Controlla l'indirizzo e rimuovi eventuali voci di completamento automatico obsolete prima di creare un nuovo messaggio.

dsn-vendor-exchange-recipient-access = Exchange ha rifiutato l'accesso a questo destinatario o dominio di destinazione. Controlla l'indirizzo del destinatario e la configurazione del dominio accettato dell'organizzazione.

dsn-vendor-exchange-recipient-rate = Exchange ha ricevuto troppi messaggi da questo mittente a questo destinatario nella sua finestra oraria.

dsn-vendor-exchange-sender-blocked = Microsoft ha limitato questo account di invio dopo aver rilevato sospetti messaggi di spam o abusi. Ciò potrebbe richiedere a un amministratore di proteggere e sbloccare l'account.

dsn-vendor-exchange-temporary-restriction = Microsoft sta temporaneamente limitando l'invio di IP o il traffico durante la valutazione di attività sospette.

dsn-vendor-exchange-tenant-blocked = Microsoft ha limitato l'invio per questa organizzazione perché il suo traffico era considerato sospetto.

dsn-vendor-exchange-tenant-blocklist = L'indirizzo o il dominio di un destinatario viene bloccato dall'elenco tenant consentiti/bloccati dell'organizzazione mittente. Una destinazione bloccata può rifiutare l'intero messaggio.

dsn-vendor-exchange-tenant-limit = L'organizzazione Microsoft 365 ha raggiunto il limite giornaliero progressivo per i destinatari esterni.

dsn-vendor-exchange-unregistered = Microsoft ha rifiutato l'invio da un dominio non correttamente registrato e verificato nel tenant.

dsn-vendor-expired = Il servizio di invio si è interrotto allo scadere del tempo di consegna. Controllare la diagnostica per individuare l'errore di connessione o di routing che ha causato il ritardo.

dsn-vendor-fortimail-relay = FortiMail ha negato questa consegna in base alla sua politica di controllo degli accessi o di inoltro. L'amministratore del gateway deve verificare la regola di ricezione corrispondente.

dsn-vendor-gmail-app-password = Google richiede una password per l'app o un metodo di autorizzazione dell'account supportato per questo accesso SMTP.

dsn-vendor-gmail-auth-method = Google ha rifiutato un metodo di autenticazione non supportato, come il metodo XOAUTH ritirato.

dsn-vendor-gmail-authentication = Google non ha accettato l'autenticazione del mittente. La diagnostica distingue i caratteri mancanti SPF/DKIM, SPF errore grave e DMARC rifiuto.

dsn-vendor-gmail-browser-login = Google richiede l'accesso all'account o un controllo di sicurezza nel browser prima di consentire questo accesso SMTP.

dsn-vendor-gmail-custom-policy = La regola di posta personalizzata di un amministratore di Google Workspace ha rifiutato il messaggio. Ciò non significa necessariamente che il filtro antispam di Google lo abbia rifiutato.

dsn-vendor-gmail-dmarc-record = Google richiede un record DMARC con una policy dichiarata per questo dominio di invio.

dsn-vendor-gmail-rate = Google ha applicato un limite di invio o di inoltro oppure ha rilevato una frequenza di messaggi insolita. La diagnostica identifica l'account interessato, IP, o il dominio.

dsn-vendor-gmail-relay = Google Workspace non ha autorizzato l'invio di IP, dominio o account per il suo servizio di inoltro SMTP.

dsn-vendor-gmail-reputation = Il filtraggio di Google ha rifiutato il messaggio perché considerava la posta non richiesta o la reputazione di invio IP/dominio troppo bassa.

dsn-vendor-gmail-spf-record = Google considera la configurazione del dominio SPF del mittente non valida o sospetta.

dsn-vendor-greylist = Il gateway ha temporaneamente rinviato questa consegna utilizzando il greylisting. Un server di invio configurato correttamente normalmente riprova in seguito.

dsn-vendor-headers = Il servizio ricevente ha rifiutato intestazioni dei messaggi non valide o mancanti. L'applicazione di invio deve generare posta formattata correttamente.

dsn-vendor-helo = Il server mittente ha utilizzato un'identità HELO/EHLO non accettabile. Il suo saluto deve identificare correttamente il server mittente.

dsn-vendor-ip-block = Il servizio ricevente ha rifiutato l'invio IP a causa della sua reputazione o di una voce nella blocklist.

dsn-vendor-mimecast-block = Mimecast ha rifiutato il mittente o il dominio in base a un criterio amministrativo o di blocco del mittente.

dsn-vendor-mimecast-content = Una politica di esame del contenuto Mimecast ha rifiutato il messaggio.

dsn-vendor-mimecast-spoof = La politica anti-spoofing di Mimecast ha rifiutato l'identità del mittente.

dsn-vendor-mta-sts-host = Il server di posta di destinazione non è consentito dalla politica MTA-STS del dominio.

dsn-vendor-proofpoint-block = Proofpoint ha bloccato l'invio IP o il dominio dopo aver rilevato spam o attività dannose.

dsn-vendor-proofpoint-throttle = Proofpoint ha temporaneamente rinviato l'invio di IP perché il suo modello di invio era considerato insolito.

dsn-vendor-ptr = L'invio di IP manca di un inverso accettabile DNS, oppure il suo nome PTR non si risolve nello stesso IP.

dsn-vendor-recipient-ip-block = L'organizzazione destinataria blocca esplicitamente l'indirizzo di invio IP. Il suo amministratore controlla questo blocco locale.

dsn-vendor-relay = Il server di posta non inoltrerà questo messaggio alla destinazione. Controlla l'autenticazione SMTP, gli IP di invio attendibili, i connettori e i domini accettati.

dsn-vendor-routing-loop = Il messaggio è entrato in un ciclo di inoltro o routing e ha superato il numero consentito di hop.

dsn-vendor-sending-limit = L'account mittente ha superato la quota di messaggi o destinatari. I ripetuti invii immediati non rimuoveranno il limite.

dsn-vendor-spf = Il destinatario non ha accettato l'autenticazione SPF per l'invio IP e il dominio del mittente della busta.

dsn-vendor-starttls = Non è stato possibile stabilire la connessione crittografata SMTP richiesta o non è stata supportata dal server successivo.

dsn-vendor-yahoo-deferred = Yahoo ha temporaneamente rinviato questo messaggio, di solito mentre valutava il volume, la reputazione o i reclami dei destinatari. Controllare la diagnostica per la causa indicata.

dsn-vendor-yahoo-policy = Yahoo ha rifiutato il messaggio in base alla sua politica di posta. Potrebbero essere coinvolti l'autenticazione, l'allineamento DMARC o il contenuto del messaggio; la diagnostica fornisce il motivo specifico.
