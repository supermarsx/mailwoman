# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = Contenu distant

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = Impossible de mettre à jour les autorisations des images distantes. { $reason }
remote-banner-blocked = Images distantes bloquées
remote-banner-blocked-policy = Images distantes bloquées par vos paramètres de confidentialité
remote-banner-shown = Images affichées
remote-banner-proxied = Images chargées via le serveur
remote-banner-direct = Images chargées depuis les serveurs de l’expéditeur
remote-banner-trackers = { $count ->
    [one] { $count } traceur supprimé
   *[other] { $count } traceurs supprimés
}
remote-banner-trackers-limited = Au moins { $count } traceurs supprimés
remote-banner-show = Afficher les images
remote-banner-show-help = Charger les images de ce message via le serveur, afin que l’expéditeur ne voie pas votre adresse IP
remote-banner-more = Autres options d’images
remote-banner-always-sender = Toujours afficher les images de { $sender }
remote-banner-always-domain = Toujours afficher les images de { $domain }
remote-banner-always-all = Toujours afficher les images dans tous les e-mails
remote-banner-settings = Paramètres de confidentialité
remote-banner-undo = Annuler
remote-banner-undo-help-single = Bloquer à nouveau les images de ce message
remote-banner-undo-help-per-sender = Ne plus afficher automatiquement les images de { $sender }
remote-banner-undo-help-per-domain = Ne plus afficher automatiquement les images de { $domain }
remote-banner-undo-help-all = Ne plus afficher automatiquement les images dans tous les e-mails
remote-banner-blocked-help = Les images hébergées sur d’autres serveurs peuvent indiquer à l’expéditeur quand et où vous avez ouvert ce message. Elles restent bloquées jusqu’à ce que vous choisissiez de les afficher.
remote-banner-help-block = Vos paramètres de confidentialité bloquent toutes les images distantes. Modifiez-les dans les paramètres de confidentialité.
remote-banner-help-ask = Les images sont chargées via ce serveur, qui masque votre adresse IP à l’expéditeur.
remote-banner-help-proxy = Les images sont chargées via ce serveur, qui masque votre adresse IP à l’expéditeur.
remote-banner-help-direct = Les images sont chargées directement depuis les serveurs de l’expéditeur, qui peuvent voir votre adresse IP.
remote-banner-hosts = Hôtes bloqués : { $hosts }
remote-banner-trackers-help = Les images de suivi sont supprimées avant l’affichage du message. Les images que le serveur ne reconnaît comme traceurs qu’après les avoir téléchargées sont comptées pendant une durée limitée.
remote-images-toast-loading = Chargement des images…
remote-images-toast-done = Images chargées
remote-images-toast-failed = { $count ->
    [one] { $count } image n’a pas pu être chargée
   *[other] { $count } images n’ont pas pu être chargées
}
remote-images-toast-retry = Réessayer
remote-images-toast-progress = Chargement des images… { $loaded }/{ $total }
remote-images-toast-retrying-progress = Chargement des images… { $loaded }/{ $total } · Nouvelle tentative { $attempt }/{ $max }
remote-images-toast-cancel = Annuler
remote-images-toast-cancelled = Chargement des images annulé
remote-images-toast-load-again = Recharger
remote-banner-hosts-manage = Sources d’images…
image-hosts-title = Images par source
image-hosts-help = Choisissez ce qu’il advient des images de chaque source. Ne jamais afficher l’emporte sur Toujours afficher et s’applique à tous les messages.
image-hosts-none = Ce message ne contient aucune image distante.
image-hosts-for = Images de { $host }
image-hosts-allow = Toujours afficher
image-hosts-ask = Demander
image-hosts-block = Ne jamais afficher
image-hosts-failed = Le réglage de la source d’images n’a pas pu être enregistré.
image-tip-blocked = Image bloquée. Choisissez Afficher les images pour la charger.
image-tip-loading = Chargement de l’image…
image-tip-error = Image indisponible
