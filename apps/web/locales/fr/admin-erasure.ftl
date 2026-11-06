# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Effacement et anonymisation
compliance-card-erasure-coverage = Remplacez les identifiants d’une personne dans l’historique d’audit et les journaux, et voyez quelles sauvegardes contiennent encore des personnes effacées.
admin-erasure-title = Effacement et anonymisation
admin-erasure-intro = « Anonymiser la personne » remplace l’adresse et les identifiants de compte d’une personne dans l’historique d’audit et dans les fichiers de journaux archivés par un pseudonyme stable, dérivé d’une clé. La chaîne de hachage d’audit est rescellée afin que la vérification réussisse toujours. Chaque effacement et chaque anonymisation laisse aussi une pierre tombale, de sorte que la restauration d’une sauvegarde plus ancienne efface de nouveau ces personnes.
admin-erasure-docs = Le registre de ce qui est effacé, exporté et conservé pour chaque table se trouve dans docs/admin/erasure-coverage.md.
admin-erasure-scrub = Anonymiser la personne
admin-erasure-scrub-note = Lancez d’abord l’aperçu : il compte les lignes et les lignes de journal qui changeraient, et ne modifie rien.
admin-erasure-address = Adresse
admin-erasure-preview = Aperçu
admin-erasure-run = Anonymiser la personne
admin-erasure-dry-note = Aperçu uniquement. Rien n’a été modifié.
admin-erasure-done-note = Anonymisée. La personne apparaît désormais sous le nom { $pseudonym }.
admin-erasure-nothing = Aucun historique ne mentionne cette personne.
admin-erasure-col-table = Table ou fichier
admin-erasure-col-rows = Lignes
admin-erasure-confirm-title = Anonymiser cette personne dans l’historique ?
admin-erasure-confirm-body = Les entrées d’audit et les lignes de journal qui mentionnent la personne sont réécrites avec le pseudonyme. Cette action est irréversible ; sans la clé, l’adresse ne peut pas être retrouvée à partir du pseudonyme.
admin-erasure-settings = Paramètres
admin-erasure-keys = Clés de pseudonymes
admin-erasure-keys-note = Les pseudonymes sont créés avec la génération de clé actuelle. Une rotation ajoute une génération ; les plus anciennes restent jusqu’à ce que la durée de conservation définie ci-dessus les supprime.
admin-erasure-rotate = Renouveler la clé
admin-erasure-keys-none = Aucune clé n’a encore été créée. La première anonymisation en crée une.
admin-erasure-col-gen = Génération
admin-erasure-col-created = Créée
admin-erasure-col-state = État
admin-erasure-key-current = Actuelle
admin-erasure-key-retired = Retirée, conservée
admin-erasure-key-destroy = Retirée, supprimée le { $when }
admin-erasure-chain = Chaîne d’audit
admin-erasure-chain-note = Chaque entrée d’audit contient le hachage de la précédente. Une anonymisation réécrit des entrées et rescelle la chaîne, puis enregistre l’ancien et le nouveau hachage de tête.
admin-erasure-seal = Sceller les nouvelles entrées
admin-erasure-chain-ok = La chaîne est valide : { $rows } entrées scellées, { $unsealed } en attente de scellement.
admin-erasure-chain-bad = La chaîne n’est pas valide ; première entrée erronée à la position { $seq }.
admin-erasure-col-time = Heure
admin-erasure-col-range = Positions
admin-erasure-backups = Sauvegardes
admin-erasure-backups-note = Une sauvegarde enregistre le repère des pierres tombales au moment de sa création. Le repère actuel est { $watermark }.
admin-erasure-col-label = Libellé
admin-erasure-col-location = Emplacement
admin-erasure-col-watermark = Repère
admin-erasure-col-marker = Effacé depuis
admin-erasure-col-actions = Actions
admin-erasure-backups-none = Aucune sauvegarde n’est enregistrée.
admin-erasure-contains-erased = Contient des personnes effacées ({ $count })
admin-erasure-no-marker = Aucun
admin-erasure-forget = Oublier
admin-erasure-register = Enregistrer une sauvegarde
