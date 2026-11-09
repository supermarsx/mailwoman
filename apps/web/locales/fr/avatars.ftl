# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: fr · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Personnes et photos de contact
avatars-mode = Photos des expéditeurs et des contacts
avatars-off = Masquer les photos
avatars-initials = Générer les initiales localement
avatars-contacts = Utiliser les photos de contact locales, avec les initiales en secours
avatars-help = Les initiales sont générées sur cet appareil. Les photos de contact n’utilisent que les photos PNG ou JPEG intégrées déjà enregistrées dans vos propres carnets d’adresses, jusqu’à 512 × 512 pixels et 128 Kio. Les URL de photos distantes et les services d’avatars externes ne sont jamais sollicités. Cette préférence est enregistrée avec votre compte et peut être imposée par votre administrateur.
avatars-default = Rétablir la préférence héritée
avatars-sender-icons = Afficher les icônes de domaine d’expéditeur intégrées
avatars-sender-icons-help = Les domaines reconnus de réseaux sociaux, de développement, d’IA, de commerce et d’administrations utilisent des icônes intégrées à cette application. Vos propres photos de contact sont prioritaires. Les expéditeurs inconnus reçoivent des initiales générées localement parmi 192 couleurs. Les icônes sont des indications de domaine, pas une preuve de l’authenticité d’un message. Aucun service de logos externe n’est contacté.
avatars-icons-default = Rétablir la préférence d’icônes héritée
avatars-brand-avatars = Afficher les logos de marque à côté des expéditeurs vérifiés
avatars-brand-avatars-help = Dans le lecteur, une entreprise reconnue affiche son logo à côté de l’expéditeur uniquement si le message a réussi DKIM et un contrôle DMARC aligné pour le propre domaine de cette entreprise. Sinon, l’avatar normal est utilisé. Les logos sont intégrés à cette application ; rien n’est demandé à l’expéditeur.
avatars-brand-avatars-default = Rétablir la préférence de logos de marque héritée
avatars-domain-icon = Icône du domaine { $name } · l’identité de l’expéditeur n’est pas vérifiée par cette icône
avatars-favorites-first = Afficher d’abord les contacts favoris dans les suggestions de destinataires
avatars-favorites-first-help = Les contacts marqués d’une étoile apparaissent avant les autres résultats lorsque vous saisissez un destinataire. Seul l’ordre change ; aucune suggestion n’est masquée.
avatars-favorites-first-default = Rétablir la préférence de favoris héritée

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Authentification requise pour les logos de marque

avatars-require-auth-always = Toujours (nécessite un chèque réussi)

avatars-require-auth-default = Restaurer les préférences d'authentification héritées

avatars-require-auth-help = Choisissez quand le logo d'une entreprise reconnue peut remplacer les initiales à côté d'un expéditeur. Jamais : affichez-le chaque fois que le domaine de l'expéditeur correspond. Uniquement si une vérification a échoué (par défaut) : l'afficher sauf si le message a échoué DKIM ou DMARC. Toujours : affichez-le uniquement lorsque le message a passé DKIM et une vérification alignée DMARC pour le propre domaine de cette entreprise, afin que le courrier non encore vérifié affiche les initiales. La correspondance de domaine inclut des sous-domaines tels que mail.example.com. Cette préférence est enregistrée avec votre compte et peut être détenue par votre administrateur.

avatars-require-auth-known-failure = Seulement si une vérification a échoué

avatars-require-auth-never = Jamais (correspondance sur le domaine uniquement)

avatars-unverified-sender = Expéditeur non vérifié · aucune icône de marque ne s'affiche car ce message n'a pas passé le contrôle d'authentification
