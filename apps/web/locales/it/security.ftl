# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Dettagli di sicurezza del messaggio
security-section-auth = Autenticazione
security-section-received = Percorso di consegna
security-section-signature = Firma
security-section-attachments = Allegati
security-section-warnings = Avvisi
security-section-controls = Controlli sul mittente
security-received-empty = Nessuna catena Received disponibile.
security-attachments-empty = Nessun allegato.
security-hop-unknown = sconosciuto
security-unknown = sconosciuto

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = superato
security-auth-fail = non superato
security-auth-none = assente
security-auth-neutral = neutro
security-auth-temperror = errore temporaneo
security-auth-permerror = errore permanente

security-detail-domain = dominio
security-detail-selector = selettore
security-detail-policy = criterio
security-detail-alignment = allineamento
security-detail-chain-length = lunghezza della catena
security-aligned = allineato
security-not-aligned = non allineato

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Firma verificata
security-sig-unverified-key = Firmato — chiave del firmatario non verificata
security-sig-invalid = La firma non è valida
security-sig-none = Non firmato
security-chain-trusted = Catena attendibile
security-chain-untrusted = Catena non attendibile
security-chain-expired = Catena scaduta
security-chain-unknown = Catena sconosciuta
security-revocation-good = Non revocata
security-revocation-revoked = Chiave revocata
security-revocation-unknown = Revoca sconosciuta
security-key-changed = La chiave del firmatario è cambiata dall'ultima volta
security-fact-signer-key = Chiave del firmatario
security-fact-algorithm = Algoritmo
security-fact-key-created = Chiave creata
security-fact-key-expires = Scadenza della chiave
security-fact-chain = Catena
security-fact-revocation = Revoca
security-fact-key-change = Cambio di chiave

# -- Attachment risk --------------------------------------------------------
security-attach-none = Nessun rischio noto
security-attach-macro = Contiene macro
security-attach-executable = File eseguibile
security-attach-encrypted-archive = Archivio cifrato
security-attach-double-extension = Doppia estensione del file
security-attach-mismatch = tipo non corrispondente (dichiarato { $declared }, rilevato { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = L'indirizzo Reply-To è diverso dal mittente
security-anomaly-envelopeFromDivergence = Il mittente della busta è diverso dall'indirizzo From
security-anomaly-messageIdDomainAnomaly = Il dominio del Message-ID non corrisponde al mittente
security-anomaly-dateSkew = La data di invio sembra sfasata
security-anomaly-punycodeSender = Il mittente usa punycode (possibile dominio simile)

# -- Sender controls --------------------------------------------------------
security-control-block = Blocca mittente
security-control-silence = Silenzia mittente
security-control-ignore-conversation = Ignora conversazione
security-control-report-phishing = Segnala phishing
security-control-report-junk = Segnala posta indesiderata
security-control-done-block = Mittente bloccato
security-control-done-silence = Mittente silenziato
security-control-done-ignore-conversation = Conversazione ignorata
security-control-done-report-phishing = Segnalato come phishing
security-control-done-report-junk = Segnalato come posta indesiderata

security-za-title = Archiviazione zero-access
security-za-on = Attiva
security-za-off = Disattiva
security-za-protects = Zero-access cifra la posta, le note e i dati PIM archiviati, in modo che il server di hosting conservi solo testo cifrato che non può leggere. Protegge il contenuto dei dati archiviati da un operatore curioso o da una violazione dell'host di archiviazione: chiunque legga il database, i file su disco o un backup rubato vede testo cifrato XChaCha20-Poly1305 e non può recuperare il corpo dei messaggi, gli oggetti, gli allegati, il testo delle note o l'indice di ricerca.

security-za-server-sees-title = Cosa vede ancora il server
security-za-sees-1 = Blob di testo cifrato — le righe cifrate stesse.
security-za-sees-2 = ID di riga opachi — gli identificatori usati per archiviare e recuperare le righe.
security-za-sees-3 = Dimensioni — la lunghezza di ogni testo cifrato (dimensione approssimativa di messaggi e allegati).
security-za-sees-4 = Marcature temporali — quando le righe vengono scritte e aggiornate.
security-za-sees-5 = Metadati di instradamento della busta necessari per fare da proxy IMAP/SMTP — il server si connette comunque al provider di posta upstream per conto dell'utente, quindi i metadati di connessione e la busta di instradamento necessaria per inviare e ricevere posta passano attraverso di esso.

security-za-active-server-caveat = Zero-access protegge i dati A RIPOSO da un host curioso o violato. Un server attivo del tutto malevolo che fa da proxy al traffico IMAP/SMTP in tempo reale è un avversario più forte, e zero-access NON protegge da esso: un server del genere si trova sulla connessione attiva con il provider di posta e potrebbe osservare o alterare la posta mentre transita, indipendentemente da come è cifrata la copia archiviata. Scegliere zero-access quando la preoccupazione è "non voglio che chi gestisce (o viola) l'host di archiviazione legga la mia posta archiviata." Non è una difesa contro un operatore che sovverte il percorso della posta in tempo reale.

security-za-no-search-claim = Qui non c'è cifratura ricercabile lato server, e non viene dichiarato nulla del genere. La ricerca avviene interamente sul client, sul contenuto decifrato localmente; il server non detiene mai una forma ricercabile del testo in chiaro.

security-za-recovery-tradeoff = Le chiavi vengono derivate e conservate solo sui dispositivi dell'utente — il server non riceve mai una chiave in chiaro. Se si perdono la passphrase (o la passkey) E tutti i dispositivi associati E la frase di recupero, i dati cifrati non possono essere recuperati. Conservare la frase di recupero offline e trattarla come equivalente al segreto principale dell'account.

security-za-setup-key = Configura la chiave
security-za-key-source = Origine della chiave
security-za-passphrase = Passphrase
security-za-passkey = Passkey (senza password)
security-za-passkey-unavailable = le passkey non sono disponibili in questo browser
security-za-passphrase-label = Passphrase zero-access
security-za-enable = Attiva zero-access
security-za-recovery-title = Frase di recupero
security-za-recovery-heading = Frase di recupero — salvarla offline adesso
security-za-recovery-note = Questa è l'unica copia. Chiunque la possieda può leggere i dati; senza di essa (e senza un dispositivo associato) i dati non possono essere recuperati.
security-za-disable-section = Disattiva zero-access
security-za-disable-heading = Disattivare zero-access
security-za-disable-note = I nuovi dati verranno di nuovo archiviati non cifrati. I dati cifrati esistenti restano leggibili solo finché è ancora possibile derivare la chiave.
security-za-disable-btn = Disattiva zero-access
security-za-err-passphrase-len = usare una passphrase di almeno 8 caratteri
security-za-err-enable = impossibile attivare zero-access
security-za-err-disable = impossibile disattivare zero-access

security-pair-title = Associazione dispositivo
security-pair-heading = Associa un dispositivo
security-pair-intro = Trasferire le chiavi su un altro dispositivo senza inviarle tramite il server — il server inoltra solo una busta sigillata opaca. Confrontare le sei parole su entrambi gli schermi prima di considerare attendibile l'associazione.
security-pair-role = Ruolo nell'associazione
security-pair-new-role = Questo è il nuovo dispositivo
security-pair-existing-role = Associa un altro dispositivo da qui
security-pair-show-qr = Mostra QR di associazione
security-pair-scan = Scansionarlo sul dispositivo esistente
security-pair-qr-label = Codice QR di associazione dispositivo
security-pair-qr-default = Codice QR di associazione
security-pair-copy-code = Oppure copiare questo codice di associazione
security-pair-paste-envelope = Incollare la busta sigillata dall'altro dispositivo
security-pair-envelope-placeholder = busta…
security-pair-envelope-label = Busta sigillata
security-pair-complete = Completa associazione
security-pair-code-from-new = Codice di associazione dal nuovo dispositivo
security-pair-code-placeholder = codice di associazione…
security-pair-code-label = Codice di associazione
security-pair-seal = Sigilla le mie chiavi per quel dispositivo
security-pair-envelope-out = Busta sigillata — incollarla sul nuovo dispositivo
security-pair-compare = Confrontare queste parole su entrambi i dispositivi
security-pair-sas-label = Stringa di autenticazione breve
security-pair-match = Le parole corrispondono
security-pair-abort = Sono diverse — interrompi
security-pair-confirmed = Associazione confermata. Questo canale è autenticato.
security-pair-err-start = impossibile avviare l'associazione
security-pair-err-complete = impossibile completare l'associazione
security-pair-err-unlock = sbloccare prima zero-access per associare un altro dispositivo
security-pair-err-seal = impossibile sigillare la chiave radice
