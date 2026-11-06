# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Cancellazione e pseudonimizzazione
compliance-card-erasure-coverage = Sostituisci gli identificatori di una persona nella cronologia di audit e nei log, e vedi quali backup contengono ancora persone cancellate.
admin-erasure-title = Cancellazione e pseudonimizzazione
admin-erasure-intro = Pseudonimizza persona sostituisce l'indirizzo e gli ID account di una persona nella cronologia di audit e nei file di log ruotati con uno pseudonimo stabile con chiave. La catena di hash dell'audit viene risigillata così la verifica continua a riuscire. Ogni cancellazione e pseudonimizzazione lascia anche una tombstone, così il ripristino di un backup più vecchio cancella di nuovo quelle persone.
admin-erasure-docs = Il registro di cosa viene cancellato, esportato e conservato per ogni tabella si trova in docs/admin/erasure-coverage.md.
admin-erasure-scrub = Pseudonimizza persona
admin-erasure-scrub-note = Esegui prima l'anteprima: conta le righe e le righe di log che cambierebbero e non modifica nulla.
admin-erasure-address = Indirizzo
admin-erasure-preview = Anteprima
admin-erasure-run = Pseudonimizza persona
admin-erasure-dry-note = Solo anteprima. Non è stato modificato nulla.
admin-erasure-done-note = Pseudonimizzata. La persona ora compare come { $pseudonym }.
admin-erasure-nothing = Nessuna cronologia nomina questa persona.
admin-erasure-col-table = Tabella o file
admin-erasure-col-rows = Righe
admin-erasure-confirm-title = Pseudonimizzare questa persona nella cronologia?
admin-erasure-confirm-body = Le voci di audit e le righe di log che nominano la persona vengono riscritte con lo pseudonimo. Non si può annullare; l'indirizzo non può essere recuperato dallo pseudonimo senza la chiave.
admin-erasure-settings = Impostazioni
admin-erasure-keys = Chiavi degli pseudonimi
admin-erasure-keys-note = Gli pseudonimi sono creati con la generazione di chiave corrente. La rotazione aggiunge una generazione; quelle più vecchie restano finché la conservazione impostata sopra non le rimuove.
admin-erasure-rotate = Ruota la chiave
admin-erasure-keys-none = Nessuna chiave è stata ancora creata. La prima pseudonimizzazione ne crea una.
admin-erasure-col-gen = Generazione
admin-erasure-col-created = Creata
admin-erasure-col-state = Stato
admin-erasure-key-current = Corrente
admin-erasure-key-retired = Ritirata, conservata
admin-erasure-key-destroy = Ritirata, eliminata il { $when }
admin-erasure-chain = Catena di audit
admin-erasure-chain-note = Ogni voce di audit contiene l'hash della precedente. Una pseudonimizzazione riscrive le voci, risigilla la catena e registra l'hash di testa vecchio e nuovo.
admin-erasure-seal = Sigilla le nuove voci
admin-erasure-chain-ok = La catena risulta valida: { $rows } voci sigillate, { $unsealed } in attesa di essere sigillate.
admin-erasure-chain-bad = La catena non risulta valida; prima voce errata alla posizione { $seq }.
admin-erasure-col-time = Ora
admin-erasure-col-range = Posizioni
admin-erasure-backups = Backup
admin-erasure-backups-note = Un backup registra la soglia delle tombstone quando viene creato. La soglia attuale è { $watermark }.
admin-erasure-col-label = Etichetta
admin-erasure-col-location = Posizione
admin-erasure-col-watermark = Soglia
admin-erasure-col-marker = Cancellati dal
admin-erasure-col-actions = Azioni
admin-erasure-backups-none = Nessun backup registrato.
admin-erasure-contains-erased = Contiene persone cancellate ({ $count })
admin-erasure-no-marker = Nessuno
admin-erasure-forget = Dimentica
admin-erasure-register = Registra backup
