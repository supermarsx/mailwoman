# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenuto remoto

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Impossibile aggiornare i permessi per le immagini remote. { $reason }
remote-banner-blocked = Immagini remote bloccate
remote-banner-blocked-policy = Immagini remote bloccate dalle sue impostazioni di privacy
remote-banner-shown = Immagini mostrate
remote-banner-proxied = Immagini caricate tramite il server
remote-banner-direct = Immagini caricate dai server del mittente
remote-banner-trackers = { $count ->
    [one] { $count } tracker rimosso
   *[other] { $count } tracker rimossi
}
remote-banner-trackers-limited = Almeno { $count } tracker rimossi
remote-banner-show = Mostra immagini
remote-banner-show-help = Carica le immagini di questo messaggio tramite il server, così il mittente non vede il suo indirizzo IP
remote-banner-more = Altre opzioni per le immagini
remote-banner-always-sender = Mostra sempre le immagini di { $sender }
remote-banner-always-domain = Mostra sempre le immagini di { $domain }
remote-banner-always-all = Mostra sempre le immagini in tutta la posta
remote-banner-settings = Impostazioni di privacy
remote-banner-undo = Annulla
remote-banner-undo-help-single = Blocca di nuovo le immagini di questo messaggio
remote-banner-undo-help-per-sender = Smetti di mostrare automaticamente le immagini di { $sender }
remote-banner-undo-help-per-domain = Smetti di mostrare automaticamente le immagini di { $domain }
remote-banner-undo-help-all = Smetti di mostrare automaticamente le immagini in tutta la posta
remote-banner-blocked-help = Le immagini memorizzate su altri server possono dire al mittente quando e dove ha aperto questo messaggio. Restano bloccate finché non sceglie di mostrarle.
remote-banner-help-block = Le sue impostazioni di privacy bloccano tutte le immagini remote. Modificalo nelle impostazioni di privacy.
remote-banner-help-ask = Le immagini vengono caricate tramite questo server, che nasconde il suo indirizzo IP al mittente.
remote-banner-help-proxy = Le immagini vengono caricate tramite questo server, che nasconde il suo indirizzo IP al mittente.
remote-banner-help-direct = Le immagini vengono caricate direttamente dai server del mittente, che possono vedere il suo indirizzo IP.
remote-banner-hosts = Host bloccati: { $hosts }
remote-banner-trackers-help = Le immagini di tracciamento vengono rimosse prima che il messaggio venga mostrato. Le immagini che il server riconosce come tracker solo dopo averle scaricate vengono conteggiate per un tempo limitato.
remote-images-toast-loading = Caricamento delle immagini…
remote-images-toast-done = Immagini caricate
remote-images-toast-failed = { $count ->
    [one] Impossibile caricare { $count } immagine
   *[other] Impossibile caricare { $count } immagini
}
remote-images-toast-retry = Riprova
remote-images-toast-progress = Caricamento delle immagini… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Caricamento delle immagini… { $loaded }/{ $total } · Tentativo { $attempt }/{ $max }
remote-images-toast-cancel = Annulla
remote-images-toast-cancelled = Caricamento delle immagini annullato
remote-images-toast-load-again = Carica di nuovo
remote-banner-hosts-manage = Origini delle immagini…
image-hosts-title = Immagini per origine
image-hosts-help = Scegli cosa succede alle immagini di ogni origine. Non mostrare mai prevale su Mostra sempre e si applica a ogni messaggio.
image-hosts-none = Questo messaggio non contiene immagini remote.
image-hosts-for = Immagini da { $host }
image-hosts-allow = Mostra sempre
image-hosts-ask = Chiedi
image-hosts-block = Non mostrare mai
image-hosts-failed = Impossibile salvare l'impostazione dell'origine delle immagini.
image-tip-blocked = Immagine bloccata. Scegli Mostra immagini per caricarla.
image-tip-loading = Caricamento dell'immagine…
image-tip-error = Immagine non disponibile
