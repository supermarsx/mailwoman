# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: appearance. Source: ../en/appearance.ftl

## Theme mode (fixed / system / schedule)

appearance-mode = Quando cambiare
appearance-mode-fixed = Un solo tema
appearance-mode-system = Segui il sistema
appearance-mode-schedule = In base all'ora

appearance-mode-hint-fixed = Il tema scelto resta attivo finché non lo si cambia.
appearance-mode-hint-system = Cambia in base all'impostazione chiaro/scuro del sistema operativo.
appearance-mode-hint-schedule = Passa al tema scuro nelle ore impostate qui sotto.

## Gallery

appearance-gallery = Tema
appearance-gallery-hint = Scegliendo un tema qui si passa a un tema fisso.
appearance-active = In uso
appearance-light = Chiaro
appearance-dark = Scuro

## The light/dark pair used by the two automatic modes

appearance-pair = Temi da alternare
appearance-pair-light = Tema chiaro
appearance-pair-dark = Tema scuro

## Schedule window

appearance-schedule = Ore con tema scuro
appearance-schedule-start = Dalle
appearance-schedule-end = Alle

## Server sync

appearance-sync = Salvataggio
appearance-sync-idle = Non avviato.
appearance-sync-loading = Verifica dell'account…
appearance-sync-synced = Salvato nell'account. Gli altri dispositivi lo applicano all'avvio.
appearance-sync-local-only = Salvato su questo dispositivo. Accedere per salvare queste impostazioni nell'account.
appearance-sync-error = Impossibile raggiungere il server. Salvato su questo dispositivo; verrà inviato di nuovo alla prossima modifica.
appearance-sync-reset = Dimentica l'aspetto salvato per il mio account
appearance-sync-reset-done = Rimosso dall'account. Questo dispositivo mantiene ciò che mostra ora.
appearance-sync-deployment = Il tema predefinito di questo server è { $theme }.

## Theme names and descriptions (theme/tokens.ts THEME_META)

appearance-theme-dark = Scuro
appearance-theme-dark-desc = Pagina quasi nera, testo bianco sporco, azioni blu, avvisi rossi, indicatori gialli.
appearance-theme-light = Chiaro
appearance-theme-light-desc = Pagina bianco sporco, testo nero, azioni blu, avvisi rossi, indicatori gialli.
appearance-theme-bauhaus-black = Nero
appearance-theme-bauhaus-black-desc = Pagina nera, testo e controlli bianchi, azioni blu, avvisi rossi, indicatori gialli.
appearance-theme-paper = Carta
appearance-theme-paper-desc = Pagina bianco caldo, inchiostro nero, blu tenue, rosso e ocra.
appearance-theme-blueprint = Cianografia
appearance-theme-blueprint-desc = Pagina blu scuro, testo e linee bianchi, azioni ciano, indicatori gialli.
appearance-theme-terminal = Terminale
appearance-theme-terminal-desc = Pagina quasi nera, testo verde, indicatori ambra. Solo i colori; il font non cambia.
appearance-theme-primary = Blocchi primari
appearance-theme-primary-desc = Pagina bianca, testo nero, blocchi piatti blu, rossi e gialli.
appearance-theme-slate-light = Ardesia chiaro
appearance-theme-slate-light-desc = Superfici grigio-blu fredde su una pagina tinta, accento blu marino tenue.
appearance-theme-slate-dark = Ardesia scuro
appearance-theme-slate-dark-desc = Tema scuro grigio-blu freddo con un accento blu tenue.
appearance-theme-ocean-light = Oceano chiaro
appearance-theme-ocean-light-desc = Tema chiaro blu-verde con un accento verde acqua scuro.
appearance-theme-ocean-dark = Oceano scuro
appearance-theme-ocean-dark-desc = Tema scuro blu-verde con un accento ciano.
appearance-theme-plum-light = Prugna chiaro
appearance-theme-plum-light-desc = Tema chiaro viola tenue con un accento violetto.
appearance-theme-plum-dark = Prugna scuro
appearance-theme-plum-dark-desc = Tema scuro viola tenue con un accento violetto.
appearance-theme-grove-light = Bosco chiaro
appearance-theme-grove-light-desc = Toni caldi di carta con venatura del legno e un accento verde muschio.
appearance-theme-grove-dark = Bosco scuro
appearance-theme-grove-dark-desc = Toni caldi di noce con venatura del legno e un accento verde muschio.
appearance-theme-amoled = AMOLED
appearance-theme-amoled-desc = Tema scuro nero puro; pixel spenti sugli schermi OLED.
appearance-theme-hc-light = Contrasto elevato (chiaro)
appearance-theme-hc-light-desc = Nero su bianco con bordi neri e nessuna superficie tinta.
appearance-theme-hc-dark = Contrasto elevato (scuro)
appearance-theme-hc-dark-desc = Bianco su nero con bordi bianchi e nessuna superficie tinta.

## Gallery groups (theme/registry.ts)

appearance-family-core = Predefinito
appearance-family-bauhaus = Bauhaus
appearance-family-slate = Ardesia
appearance-family-amoled = AMOLED
appearance-family-contrast = Contrasto elevato

## Accents (theme/accentMath.ts)

appearance-accent-default = Predefinito del tema
appearance-accent-red = Rosso
appearance-accent-orange = Arancione
appearance-accent-yellow = Giallo
appearance-accent-green = Verde
appearance-accent-teal = Verde acqua
appearance-accent-blue = Blu
appearance-accent-indigo = Indaco
appearance-accent-violet = Violetto
appearance-accent-magenta = Magenta
appearance-accent-graphite = Grafite

# -- Machine translation (2026-11): added to match en ---------------------
appearance-family-naturals = Toni naturali
appearance-family-specialty = Temi speciali
appearance-custom-title = Personalizzazione dell'aspetto
appearance-custom-description = Salvata solo per questa area di lavoro con accesso effettuato. I criteri dell'amministratore controllano quali personalizzazioni possono essere eseguite.
appearance-custom-loading = Caricamento della personalizzazione…
appearance-custom-error = La personalizzazione dell'aspetto non è disponibile.
appearance-custom-css-on = CSS personalizzato attivo
appearance-custom-css-off = CSS personalizzato disattivato dall'amministratore
appearance-custom-js-on = Script di aspetto attivi
appearance-custom-js-off = Script di aspetto disattivati dall'amministratore
appearance-custom-locked = I blocchi dell'aspetto impostati dall'amministratore sospendono il CSS personalizzato. Gli script non possono modificare i campi bloccati.
appearance-custom-css-locked = I blocchi dell'aspetto impostati dall'amministratore sospendono questo CSS. La configurazione salvata resta disponibile qui sotto.
appearance-custom-js-locked = Questo script non può modificare i campi dell'aspetto bloccati da un amministratore.
appearance-custom-css = CSS limitato
appearance-custom-css-help = Fino a 16 KiB. Solo proprietà visive e selettori semplici; niente URL, import, attributi, posizionamento o !important. I selettori restano all'interno della sua area di lavoro.
appearance-custom-js = Script di aspetto isolato
appearance-custom-js-help = Fino a 16 KiB e 500 ms. Chiama appearance.set con un oggetto contenente accent '#3366cc' o density 'compact'. Campi supportati: accent, density, font e mode (light o dark). context fornisce i valori booleani dark e reducedMotion. Le modifiche sono temporanee; non sono disponibili posta, dati dell'account, rete né il DOM dell'applicazione.
appearance-custom-save = Salva personalizzazione
appearance-custom-reset = Reimposta personalizzazione
appearance-custom-reload = Scarta la bozza e ricarica la personalizzazione salvata
appearance-custom-discard = Scartare questa bozza di personalizzazione non salvata e ricaricare?
appearance-custom-saved = Personalizzazione salvata.
appearance-custom-safe = Ripristino: aggiungi appearanceSafe=1 alla query della pagina per aprire questa area di lavoro senza CSS o script personalizzati.
appearance-custom-accent = Colore di accento personalizzato
appearance-custom-accent-reset = Usa l'accento con nome o quello del tema
appearance-text-mode-label = Stile del testo
appearance-text-mode-normal = Testo normale
appearance-text-mode-leet = Leetspeak (1337)
appearance-text-mode-help = Usa il leetspeak per le etichette dell'interfaccia e la lettura dei messaggi. La modifica e i messaggi inviati mantengono il testo originale.
appearance-text-mode-example = Ciao, Mailwoman!
appearance-preference-appearance-fontSize = Dimensione del carattere dell'interfaccia
appearance-preference-appearance-fontWeight = Spessore del carattere dell'interfaccia
appearance-preference-appearance-fontColor = Colore del carattere dell'interfaccia
appearance-preference-appearance-lowContrast = Modalità a basso contrasto
appearance-preference-mail-messageListStyle = Stile dell'elenco dei messaggi
appearance-preference-mail-messageListPreview = Mostra anteprime del corpo dei messaggi
appearance-preference-mail-afterDelete = Dopo l'eliminazione di un messaggio
appearance-preference-reader-attachmentsAtEnd = Mostra gli allegati alla fine dei messaggi
appearance-preference-ui-notifyNewVersion = Avvisami quando è disponibile una nuova versione di Mailwoman
appearance-preference-option-300 = Leggero
appearance-preference-option-400 = Normale
appearance-preference-option-500 = Medio
appearance-preference-option-600 = Semigrassetto
appearance-preference-option-cards = Schede
appearance-preference-option-table = Righe in stile Gmail
appearance-preference-option-next = Apri il messaggio successivo
appearance-preference-option-previous = Apri il messaggio precedente
appearance-preference-option-close = Chiudi il messaggio
appearance-preference-inherit = Usa il valore predefinito dell'amministratore o del tema
appearance-preference-mail-messageListOrder = Ordine dell'elenco dei messaggi
appearance-preference-option-newest = Prima i più recenti
appearance-preference-option-oldest = Prima i meno recenti
appearance-preference-option-sender = Mittente
appearance-preference-option-subject = Oggetto
appearance-theme-cyberpunk = Cyberpunk
appearance-theme-cyberpunk-desc = Superfici color antracite con azioni giallo elettrico ed evidenziazioni ciano.
appearance-theme-retrowave = Retrowave
appearance-theme-retrowave-desc = Superfici viola intenso con azioni magenta ed evidenziazioni al neon ciano.
appearance-theme-fight-club = Fight Club
appearance-theme-fight-club-desc = Superfici verde oliva consumate, testo color carta calda e azioni rosa sapone.
appearance-accent-gold = Oro
appearance-accent-brown = Marrone
appearance-accent-beige = Beige
appearance-accent-taupe = Talpa
appearance-accent-silver = Argento
appearance-accent-stone = Pietra
appearance-theme-slate-mist = Slate Mist
appearance-theme-slate-mist-desc = Tema chiaro grigio-azzurro di tono medio con accento verde acqua.
appearance-theme-slate-midnight = Slate Midnight
appearance-theme-slate-midnight-desc = Tema scuro grigio-azzurro quasi nero con accento azzurro ghiaccio.
appearance-theme-nineteen-eighty-four = 1984
appearance-theme-nineteen-eighty-four-desc = Cemento grigio ministeriale, testo color carta sbiadita, rosso Grande Fratello; messaggi in carattere a spaziatura fissa.
appearance-theme-pop-art = Pop Art
appearance-theme-pop-art-desc = Pagina bianca, contorni neri, rosso, blu e giallo primari, punti Ben-Day nella barra laterale.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
appearance-preference-group-option-importance = Importanza

appearance-preference-group-option-none = Nessun raggruppamento

appearance-preference-group-option-received = Data di ricezione

appearance-preference-group-option-sender = Mittente

appearance-preference-group-option-sent = Data di invio

appearance-preference-group-option-size = Dimensione del messaggio

appearance-preference-group-option-subject = Oggetto

appearance-preference-mail-list-pageSize = Messaggi caricati per pagina (da 20 a 200)

appearance-preference-mail-messageListGroup = Raggruppamento dei messaggi

appearance-preference-message-group-help = Mantieni insieme i messaggi correlati sotto intestazioni di gruppo. Scegli Nessun raggruppamento per un elenco continuo. L'ordine dei messaggi si applica all'interno di ciascun gruppo.

appearance-preference-message-order-help = Scegli l'ordine dei messaggi all'interno di ciascun gruppo o nell'intero elenco quando il raggruppamento è disattivato.

appearance-preference-option-importance = Importanza, alta per prima

appearance-preference-option-importance-asc = Importanza, prima bassa

appearance-preference-option-received = Data di ricezione

appearance-preference-option-sender-desc = Mittente, dalla Z alla A

appearance-preference-option-sent = Data di invio

appearance-preference-option-sent-newest = Inviato, prima il più recente

appearance-preference-option-sent-oldest = Inviato, prima il più vecchio

appearance-preference-option-size = Dimensioni, prima la più grande

appearance-preference-option-size-asc = Dimensioni, prima la più piccola

appearance-preference-option-subject-desc = Soggetto, dalla Z alla A
