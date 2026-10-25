# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: keys. Source: ../en/keys.ftl

# -- Panel header ------------------------------------------------------------
keys-panel-label = Gestione delle chiavi
keys-title = Chiavi e certificati
keys-subtitle = Chiavi OpenPGP e S/MIME. Le chiavi private restano su questo dispositivo e non raggiungono mai il server.
keys-generate = Genera chiave
keys-import = Importa chiave

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Chiavi personali
keys-empty-own = Ancora nessuna chiave. Generarne o importarne una per iniziare.
keys-contact-keys = Chiavi dei contatti
keys-empty-contact = Nessuna chiave dei contatti. Cercarne una qui sotto.
keys-loading = Caricamento delle chiavi…
keys-select-prompt = Selezionare una chiave per visualizzarla e verificarla.
keys-untitled = Chiave { $kind }

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = non verificata
keys-trust-tofu = tofu
keys-trust-verified = verificata
keys-trust-revoked = revocata

# -- Key detail --------------------------------------------------------------
keys-key-card = Chiave { $name }
keys-fingerprint = Impronta digitale
keys-safe-words = Parole di sicurezza
keys-safe-words-help = Leggerle ad alta voce con il contatto per confermare la chiave tramite un canale separato.
keys-safe-words-list = Parole di sicurezza dell'impronta digitale
keys-scan-label = Scansiona per verificare
keys-qr-label = Codice QR dell'impronta digitale
keys-autocrypt = Autocrypt
keys-autocrypt-on = Pubblicata nelle intestazioni Autocrypt
keys-autocrypt-off = Non pubblicata tramite Autocrypt
keys-trust-label = Attendibilità
keys-trust-level = Livello di attendibilità
keys-associate-label = Associa a un contatto
keys-associate-select = Contatto da associare
keys-choose-contact = Scegli un contatto…
keys-associate-button = Associa
keys-backup-label = Backup
keys-backup-help = Esportare un Autocrypt Setup Message per trasferire questa chiave su un altro dispositivo.
keys-export-backup = Esporta backup
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = Nessuna chiave privata su questo dispositivo di cui fare il backup

# -- Own S/MIME key: certificate, export, request, issued certificate ---------
keys-smime-section = Certificato S/MIME
keys-smime-self-signed = Questo certificato è autofirmato: è stato firmato dalla sua stessa chiave, non da un'autorità di certificazione. I programmi di posta di altre persone non lo considereranno attendibile a meno che non lo accettino manualmente, o finché un'autorità di certificazione non emetta un certificato a partire dal file di richiesta qui sotto.
keys-smime-valid-until = Valido fino al { $date }.
keys-smime-not-in-app = Mailwoman non firma né decifra ancora la posta con una chiave S/MIME. Esportare la chiave per usarla in un altro programma di posta.
keys-smime-passphrase = Passphrase per questa chiave S/MIME
keys-smime-export-help = Il file .p12 contiene il certificato e la chiave privata. La chiave è cifrata con la passphrase sopra, e la stessa passphrase apre il file in un altro programma. Il file .csr non contiene alcuna chiave privata.
keys-smime-export = Esporta come PKCS#12 (.p12)
keys-smime-request = Scarica richiesta di certificato (.csr)
keys-smime-issued-label = Certificato emesso per questa chiave (.pem, .crt, .cer)
keys-smime-issued-file = File del certificato emesso
keys-smime-issued-help = Sostituisce il certificato conservato per questa chiave. Un certificato creato per una chiave diversa viene rifiutato.
keys-smime-attach = Importa certificato emesso
keys-smime-failed = Operazione non riuscita. Controllare la passphrase.
keys-smime-attach-failed = Il certificato non è stato importato. Controllare la passphrase e che il file sia un unico certificato emesso per questa chiave.

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Cerca la chiave di un contatto
keys-lookup-heading = Cerca una chiave
keys-lookup-address = Indirizzo da cercare
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Fonti di ricerca
keys-source = Fonte { $source }
keys-consent-label = Consenso alla ricerca esterna
keys-consent-text = La ricerca di una chiave contatta directory esterne (WKD/VKS). Ho il consenso di questa persona per farlo.
keys-lookup-button = Cerca
keys-looking-up = Ricerca…
keys-lookup-found = Chiavi trovate: { $count } — aggiunte alle chiavi dei contatti.
keys-lookup-none = Nessuna chiave trovata.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Genera una chiave
keys-type = Tipo
keys-key-type = Tipo di chiave
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-generate-smime-note = Crea una chiave RSA a 3072 bit e un certificato per l'indirizzo qui sotto, valido per due anni. Il certificato è autofirmato: i programmi di posta di altre persone non lo considereranno attendibile a meno che non lo accettino manualmente, o finché un'autorità di certificazione non ne emetta uno. Mailwoman non firma né decifra ancora la posta con esso; in seguito è possibile esportarlo per un altro programma di posta.
keys-name = Nome
keys-email = Email
keys-key-passphrase = Passphrase della chiave
keys-passphrase-help = La passphrase protegge la chiave privata su questo dispositivo. Non lascia mai il browser.
keys-cancel = Annulla
keys-generate-submit = Genera
keys-generating = Generazione…
keys-generating-smime = Generazione di una chiave RSA in questo browser. Può richiedere fino a un minuto.
keys-generate-failed = Nessuna chiave è stata generata.
keys-generate-failed-smime = Nessuna chiave è stata generata. Un certificato può contenere solo un indirizzo ASCII semplice (nome@dominio) e un nome di al massimo 64 caratteri.

# -- Import dialog -----------------------------------------------------------
keys-import-title = Importa una chiave
keys-import-type = Tipo di importazione
keys-tab-armored = Armored (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Chiave armored
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Passphrase (se la chiave è privata)
keys-import-passphrase = Passphrase di importazione
keys-pkcs12-file-label = File PKCS#12 (.p12 / .pfx)
keys-pkcs12-file = File PKCS#12
keys-import-password-label = Password di importazione
keys-pkcs12-password = Password PKCS#12
keys-import-preview = Anteprima dell'importazione
keys-preview = Anteprima
keys-preview-type = Tipo: { $kind }
keys-preview-fingerprint-aria = Impronta digitale in anteprima
keys-preview-fingerprint = Impronta digitale: { $fp }
keys-preview-has-private = Include una chiave privata (verrà archiviata su questo dispositivo)
keys-preview-public-only = Solo chiave pubblica
keys-import-submit = Importa
