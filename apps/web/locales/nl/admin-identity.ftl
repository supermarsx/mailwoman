# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Identiteitsprovider
admin-identity-description = Laat goedgekeurde toepassingen inloggen met Mailwoman. Dit staat los van externe inlogproviders en toegang tot de mail-API. Authorization code met PKCE is vereist; andere grants zijn niet beschikbaar.
admin-identity-configure = Issuer configureren
admin-identity-new = Identiteitsclient toevoegen
admin-identity-edit = Client bewerken
admin-identity-enabled = Ingeschakeld
admin-identity-disabled = Uitgeschakeld
admin-identity-all = Alle
admin-identity-discovery = Discovery-document openen
admin-identity-search = Clients zoeken
admin-identity-status = Status
admin-identity-clients = Identiteitsclients
admin-identity-client = Clientidentiteit
admin-identity-client-id = Client-id
admin-identity-name = Naam
admin-identity-method = Clientauthenticatie
admin-identity-public = Publieke client (PKCE, geen secret)
admin-identity-pkce = Elke client moet PKCE S256 gebruiken. Secrets van vertrouwelijke clients worden hier gegenereerd en eenmalig getoond.
admin-identity-rotate-secret = Het bestaande clientsecret vervangen
admin-identity-redirects = Geregistreerde bestemmingen
admin-identity-redirect-help = Eén exacte HTTPS-URI per regel; numeriek loopback-HTTP is toegestaan voor lokale toepassingen. Jokertekens en fragmenten worden geweigerd.
admin-identity-callbacks = Callback-URI's voor autorisatie
admin-identity-logout-uris = Redirect-URI's na uitloggen
admin-identity-scopes = Toegestane identiteitsscopes
admin-identity-actions = Acties
admin-identity-delete = Client verwijderen
admin-identity-delete-confirm = Identiteitstoegang voor { $name } verwijderen? Bestaande identiteitsgrants werken niet meer en de registratie van de client, die alleen voor identiteit bestond, wordt verwijderd.
admin-identity-empty = Geen overeenkomende identiteitsclients.
admin-identity-cancel = Annuleren
admin-identity-save = Opslaan
admin-identity-done = Klaar
admin-identity-secret = Gegenereerd clientsecret
admin-identity-secret-once = Kopieer dit secret nu naar de vertrouwende toepassing. Het kan niet opnieuw worden getoond. Zet een vertrouwelijk secret nooit in browsercode.
admin-identity-issuer = Publieke HTTPS-issuer-URL
admin-identity-issuer-help = Gebruik de exacte HTTPS-oorsprong van deze installatie en het pad /oidc, inclusief een eventueel installatieprefix. Als u de issuer of de configuratie wijzigt, worden bestaande identiteitsgrants ingetrokken.
admin-identity-rotate-key = Ondertekeningssleutel roteren
admin-identity-key-help = Roteren vereist verse verificatie van de beheerder. Eerdere publieke sleutels blijven 15 minuten beschikbaar zodat onlangs uitgegeven tokens hun korte looptijd kunnen afmaken.
