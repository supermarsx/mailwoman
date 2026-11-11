# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Visualizzazione dei messaggi
reader-settings-help = Scegli come vengono visualizzati i messaggi. I valori predefiniti dell'amministratore si applicano finché non sceglie una modifica. Il contenuto del messaggio resta isolato dall'applicazione.
reader-colorMode = Colori dei messaggi
reader-colorMode-help = Automatico adatta i colori dei messaggi nei temi scuri. Originale conserva i colori del mittente; adattivo usa il tema attivo; inversione inverte i colori preservando quelli delle immagini.
reader-fontSize = Dimensione del carattere di lettura (px)
reader-fontSize-help = Dimensione base del carattere per il testo dei messaggi, da 12 a 24 pixel. La formattazione del mittente può comunque definire le dimensioni dei titoli.
reader-lineHeight = Interlinea di lettura
reader-lineHeight-help = Spazio tra le righe del testo dei messaggi, da 1,2 a 2,2 volte la dimensione del carattere.
reader-contentWidth = Larghezza di lettura
reader-contentWidth-help = Limita le righe lunghe per una lettura comoda oppure usa l'intero riquadro del messaggio.
reader-loading = Aspetto durante il caricamento
reader-loading-help = Scegli righe segnaposto discrete, un indicatore di testo o nessun indicatore visivo di caricamento. Gli screen reader ricevono comunque lo stato di caricamento.
reader-loadingDelayMs = Ritardo dell'indicatore di caricamento (ms)
reader-loadingDelayMs-help = Evita che un indicatore lampeggi per richieste veloci. Ritarda solo l'indicatore, mai il messaggio.
reader-option-auto = Automatico con il tema
reader-option-original = Colori originali del messaggio
reader-option-adaptive = Adatta al tema
reader-option-invert = Inverti i colori del messaggio
reader-option-comfortable = Confortevole
reader-option-wide = Ampia
reader-option-full = Riquadro intero
reader-option-skeleton = Righe segnaposto
reader-option-indicator = Indicatore di testo
reader-option-none = Nessun indicatore visivo
reader-inherit = Usa il valore predefinito dell'amministratore
reader-actionStyle = Barra delle azioni del messaggio
reader-actionStyle-help = Le icone compatte lasciano più spazio al messaggio. Ogni azione mantiene il nome per la tastiera e un suggerimento esplicativo. Scegli le etichette visibili se preferisci il testo accanto alle icone.
reader-option-icons = Icone compatte
reader-option-labels = Icone ed etichette
reader-zoom-group = Zoom del messaggio
reader-zoom-in = Ingrandisci
reader-zoom-in-help = Ingrandisce il messaggio. Cambia solo la dimensione del messaggio.
reader-zoom-out = Riduci
reader-zoom-out-help = Riduce il messaggio. Cambia solo la dimensione del messaggio.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Zoom del messaggio { $level }%. Ripristina al 100%
reader-zoom-reset-help = Ripristina il messaggio al 100%
reader-zoom-reset-menu = Ripristina zoom ({ $level }%)
reader-zoom = Zoom del messaggio (%)
reader-zoom-help = La dimensione con cui si aprono i messaggi, dal 50% al 200%. Viene ridimensionato solo il corpo del messaggio, mai l'app, e la stampa usa sempre il 100%. Anche i pulsanti di zoom accanto alle azioni del messaggio lo modificano.
reader-attachments-summary = { $count ->
    [one] 1 allegato
   *[other] { $count } allegati
 } · { $size }
reader-attachments-show = Mostra allegati
reader-attachments-hide = Nascondi allegati
reader-attachmentsDensity = Densità degli allegati
reader-attachmentsDensity-help = Compatta mostra piccoli pulsanti dei file; confortevole mostra anteprime più grandi quando è espansa.
reader-option-compact = Compatta
reader-attachmentsCollapsed = Comprimi gli allegati per impostazione predefinita
reader-attachmentsCollapsed-help = Inizia ogni messaggio con un riepilogo compatto degli allegati. Può espandere quel messaggio senza modificare questa impostazione predefinita.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Testo e colori
reader-card-text-help = Dimensione, spaziatura, larghezza e colori del testo dei messaggi.
reader-card-actions = Azioni e allegati
reader-card-actions-help = La barra delle azioni del messaggio e come vengono inizialmente mostrati gli allegati.
reader-card-loading = Caricamento
reader-card-loading-help = Cosa viene mostrato mentre un messaggio si apre.
settings-mail-card-list = Elenco dei messaggi
settings-mail-card-list-help = Azioni sulle righe e cosa fa l'elenco quando si agisce su un messaggio.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Scarica { $name }

reader-attachments-download-all = Scarica tutto

reader-attachments-download-all-blocked = La politica degli allegati blocca tutti i file qui

reader-attachments-download-all-busy = Preparazione del file ZIP

reader-attachments-download-all-label = Scarica tutti gli allegati come file ZIP

reader-attachments-download-all-none = Non ci sono allegati da scaricare

reader-attachments-download-all-warned = { $count ->
        [one] Un file che necessita di conferma è stato tralasciato. Scaricalo da solo.
        [many] { $count } file che necessitano di conferma sono stati esclusi. Scaricateli uno per uno.
       *[other] { $count } file che necessitano di conferma sono stati esclusi. Scaricateli uno per uno.
    }

reader-attachments-download-blocked = { $name } non può essere scaricato: { $reason }

reader-attachments-download-label = Scarica { $name } ({ $size })

reader-attachments-download-off = I download sono disattivati

reader-attachments-inline-group = { $count ->
        [one] { $count } immagine in linea
        [many] { $count } immagini in linea
       *[other] { $count } immagini in linea
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } immagine in linea
        [many] { $count } immagini in linea
       *[other] { $count } immagini in linea
    } · { $size }

reader-attachmentsBarClickToToggle = Fare clic sulla barra per espandere o comprimere

reader-attachmentsBarClickToToggle-help = Facendo clic in un punto qualsiasi della barra degli allegati si attiva/disattiva. Quando è spento, funziona solo il pulsante freccia.

reader-attachmentsBarDownloadAll = Pulsante Scarica tutto

reader-attachmentsBarDownloadAll-help = Mostra un pulsante sulla barra che scarica gli allegati come un unico file ZIP.

reader-attachmentsBarShowDownloadButtons = Pulsanti di download sugli allegati

reader-attachmentsBarShowDownloadButtons-help = Mostra il pulsante di download di ciascun allegato al passaggio del mouse o sullo stato attivo, sempre o mai.

reader-attachmentsBarZipInlineImages = Includi immagini in linea nel ZIP

reader-attachmentsBarZipInlineImages-help = Le immagini che appaiono all'interno del testo del messaggio vengono escluse da Scarica tutto a meno che questa non sia attiva.

reader-card-attachments-bar = Barra degli allegati

reader-card-attachments-bar-help = Come risponde la barra sopra gli allegati di un messaggio e cosa offre. Le linee contrassegnate come bloccate sono impostate dall'amministratore.

reader-option-always = Sempre

reader-option-hover = Al passaggio del mouse o messa a fuoco

reader-option-never = Mai

reader-card-header = Intestazione del messaggio
reader-autoHideHeader = Nascondi automaticamente le intestazioni dei messaggi
reader-autoHideHeader-help = Apri ogni messaggio con i dettagli di mittente, destinatari e metadati compressi. Oggetto e azioni restano visibili. Usa Mostra intestazione per vedere i dettagli. Disattivato per impostazione predefinita.
reader-header-show = Mostra intestazione
reader-header-hide = Nascondi intestazione
