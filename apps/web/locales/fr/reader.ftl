# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Affichage des messages
reader-settings-help = Choisissez l’affichage des messages. Les valeurs par défaut de l’administrateur s’appliquent jusqu’à ce que vous en choisissiez d’autres. Le contenu des messages reste isolé de l’application.
reader-colorMode = Couleurs des messages
reader-colorMode-help = Automatique adapte les couleurs des messages dans les thèmes sombres. Original conserve les couleurs de l’expéditeur ; adaptatif suit le thème actif ; inversion inverse les couleurs tout en préservant celles des images.
reader-fontSize = Taille de police de lecture (px)
reader-fontSize-help = Taille de police de base du texte des messages, de 12 à 24 pixels. La mise en forme de l’expéditeur peut toujours définir la taille des titres.
reader-lineHeight = Interligne de lecture
reader-lineHeight-help = Espace entre les lignes du texte des messages, de 1,2 à 2,2 fois la taille de police.
reader-contentWidth = Largeur de lecture
reader-contentWidth-help = Limiter les longues lignes pour un confort de lecture ou utiliser tout le volet du message.
reader-loading = Apparence du chargement
reader-loading-help = Choisissez des lignes d’espace réservé discrètes, un indicateur textuel ou aucun indicateur visuel. Les lecteurs d’écran reçoivent toujours l’état du chargement.
reader-loadingDelayMs = Délai de l’indicateur de chargement (ms)
reader-loadingDelayMs-help = Évite de faire clignoter un indicateur pour les requêtes rapides. Seul l’indicateur est retardé, jamais le message lui-même.
reader-option-auto = Automatique avec le thème
reader-option-original = Couleurs d’origine du message
reader-option-adaptive = Adapter au thème
reader-option-invert = Inverser les couleurs du message
reader-option-comfortable = Confortable
reader-option-wide = Large
reader-option-full = Volet entier
reader-option-skeleton = Lignes d’espace réservé
reader-option-indicator = Indicateur textuel
reader-option-none = Aucun indicateur visuel
reader-inherit = Utiliser la valeur par défaut de l’administrateur
reader-actionStyle = Ruban d’actions du message
reader-actionStyle-help = Les icônes compactes laissent plus de place au message. Chaque action conserve son nom pour le clavier et une info-bulle explicative. Choisissez des libellés visibles si vous préférez du texte à côté des icônes.
reader-option-icons = Icônes compactes
reader-option-labels = Icônes et libellés
reader-zoom-group = Zoom du message
reader-zoom-in = Agrandir
reader-zoom-in-help = Agrandir le message. Seul le message change de taille.
reader-zoom-out = Réduire
reader-zoom-out-help = Réduire le message. Seul le message change de taille.
reader-zoom-level = { $level } %
reader-zoom-reset-label = Zoom du message { $level } %. Rétablir 100 %
reader-zoom-reset-help = Rétablir le message à 100 %
reader-zoom-reset-menu = Réinitialiser le zoom ({ $level } %)
reader-zoom = Zoom du message (%)
reader-zoom-help = La taille à laquelle les messages s’ouvrent, de 50 % à 200 %. Seul le corps du message est redimensionné, jamais l’application, et l’impression utilise toujours 100 %. Les boutons de zoom à côté des actions du message le modifient aussi.
reader-attachments-summary = { $count ->
    [one] 1 pièce jointe
   *[other] { $count } pièces jointes
    } · { $size }
reader-attachments-show = Afficher les pièces jointes
reader-attachments-hide = Masquer les pièces jointes
reader-attachmentsDensity = Densité des pièces jointes
reader-attachmentsDensity-help = Compact affiche de petits boutons de fichier ; confortable affiche de plus grands aperçus lorsque la section est développée.
reader-option-compact = Compact
reader-attachmentsCollapsed = Réduire les pièces jointes par défaut
reader-attachmentsCollapsed-help = Chaque message commence par un résumé compact des pièces jointes. Vous pouvez développer ce message sans modifier cette valeur par défaut.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Texte et couleurs
reader-card-text-help = Taille, espacement, largeur et couleurs du texte des messages.
reader-card-actions = Actions et pièces jointes
reader-card-actions-help = Le ruban d’actions du message et le démarrage des pièces jointes.
reader-card-loading = Chargement
reader-card-loading-help = Ce que vous voyez pendant l’ouverture d’un message.
settings-mail-card-list = Liste des messages
settings-mail-card-list-help = Actions de ligne et comportement de la liste lorsque vous agissez sur un message.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Télécharger { $name }

reader-attachments-download-all = Téléchargez tout

reader-attachments-download-all-blocked = La politique de pièce jointe bloque tous les fichiers ici

reader-attachments-download-all-busy = Préparation du fichier ZIP

reader-attachments-download-all-label = Téléchargez toutes les pièces jointes sous forme de fichier ZIP

reader-attachments-download-all-none = Il n'y a aucune pièce jointe à télécharger

reader-attachments-download-all-warned = { $count ->
        [0] { $count } fichiers nécessitant une confirmation ont été laissés de côté. Téléchargez-les un par un.
        [one] Un fichier nécessitant une confirmation a été laissé de côté. Téléchargez-le seul.
        [many] { $count } fichiers nécessitant une confirmation ont été laissés de côté. Téléchargez-les un par un.
       *[other] { $count } fichiers nécessitant une confirmation ont été laissés de côté. Téléchargez-les un par un.
    }

reader-attachments-download-blocked = { $name } ne peut pas être téléchargé : { $reason }

reader-attachments-download-label = Télécharger { $name } ({ $size })

reader-attachments-download-off = Les téléchargements sont désactivés

reader-attachments-inline-group = { $count ->
        [0] { $count } images en ligne
        [one] { $count } image en ligne
        [many] { $count } images en ligne
       *[other] { $count } images en ligne
    }

reader-attachments-inline-summary = { $count ->
        [0] { $count } images en ligne
        [one] { $count } image en ligne
        [many] { $count } images en ligne
       *[other] { $count } images en ligne
    } · { $size }

reader-attachmentsBarClickToToggle = Cliquez sur la barre pour agrandir ou réduire

reader-attachmentsBarClickToToggle-help = Cliquer n’importe où sur la barre des pièces jointes la fait basculer. Lorsqu'il est désactivé, seul le bouton fléché le fait.

reader-attachmentsBarDownloadAll = Bouton Télécharger tout

reader-attachmentsBarDownloadAll-help = Afficher un bouton sur la barre qui télécharge les pièces jointes sous la forme d'un seul fichier ZIP.

reader-attachmentsBarShowDownloadButtons = Boutons de téléchargement sur les pièces jointes

reader-attachmentsBarShowDownloadButtons-help = Affichez le bouton de téléchargement de chaque pièce jointe en survol ou en focus, toujours ou jamais.

reader-attachmentsBarZipInlineImages = Inclure des images en ligne dans le ZIP

reader-attachmentsBarZipInlineImages-help = Les images qui apparaissent dans le texte du message sont exclues de l'option Télécharger tout, sauf si cette option est activée.

reader-card-attachments-bar = Barre des pièces jointes

reader-card-attachments-bar-help = Comment la barre au-dessus des pièces jointes d'un message réagit et ce qu'elle propose. Les lignes marquées verrouillées sont définies par votre administrateur.

reader-option-always = Toujours

reader-option-hover = En survol ou en mise au point

reader-option-never = Jamais

reader-card-header = En-tête du message
reader-autoHideHeader = Masquer automatiquement les en-têtes des messages
reader-autoHideHeader-help = Ouvrir chaque message avec les détails de l’expéditeur, des destinataires et les métadonnées repliés. L’objet et les actions restent visibles. Utilisez Afficher l’en-tête pour voir les détails. Désactivé par défaut.
reader-header-show = Afficher l’en-tête
reader-header-hide = Masquer l’en-tête
