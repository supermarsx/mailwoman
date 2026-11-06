# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Volledig beheer over alle mogelijkheden, inclusief beheerders, rollen, beveiliging, mailroutering en serviceconfiguratie. Reserveer deze rol voor vertrouwde operators die verantwoordelijk zijn voor de hele installatie.
admin-role-purpose-domain_admin = Dagelijks gebruikers- en domeinbeheer: gebruikers aanmaken, bijwerken, uitschakelen of verwijderen; quota, gebruikersmogelijkheden, inlogregels van het domein, instellingen en huisstijl beheren; sessies intrekken en blokkeringen beheren. Omvat ook maildiagnostiek en exports van gebruik en gegevens. Geeft geen beheer van beheerdersrollen of wijzigingen aan upstreamservers.
admin-role-purpose-helpdesk = Accountondersteuning en incidentensortering: gebruikers, auditactiviteit en gebruik inzien; gebruikersvlaggen wijzigen, accounts uitschakelen en sessies intrekken; blokkeringen bekijken en inlogsimulaties uitvoeren. Geeft geen rechten om gebruikers aan te maken of te verwijderen, quota te wijzigen of algemene instellingen te schrijven.
admin-role-purpose-auditor = Inspectie en rapportage: geeft elke mogelijkheid die op .read eindigt, plus audit- en gebruiksexport. Geeft geen configuratiewijzigingen of accountmutaties. Geëxporteerde rapporten kunnen gevoelige beheerinformatie bevatten.
admin-role-purpose-security_officer = Beveiligingsoperaties: beveiligings- en MFA-beleid, inlogbeleid en blokkeringen beheren; gebruikers uitschakelen, sessies of API-sleutels intrekken; auditactiviteit inzien en exporteren; authenticatielogboeken lezen en diagnostiek uitvoeren. De SSO-configuratie is leesbaar, maar deze rol geeft geen rechten om SSO te bewerken of beheerdersrollen te beheren.
admin-role-purpose-custom = Een aangepaste rol met { $count } geselecteerde rechten. Het gezag komt van de exacte lijst met rechten hieronder, niet van de naam. Controleer elke actie voordat u de rol toewijst.
admin-role-purpose-scope = Het domeinbereik wordt toegewezen wanneer deze rol aan een beheerder wordt gegeven. Overschrijvingen per account (toestaan/weigeren) kunnen de effectieve toegang wijzigen. Een beschrijving geeft geen extra rechten.
admin-role-purpose-editor = Kies de taken die deze beheerder moet uitvoeren en selecteer daarna de bijbehorende rechten. Leestoegang, gegevens exporteren en records wijzigen zijn aparte acties. Controleer de exacte identificatoren van de mogelijkheden voordat u opslaat.
admin-role-purpose-selection = Deze beschrijvingen verklaren de geselecteerde acties. Ze breiden de rechten die naar de server worden gestuurd niet uit.
admin-role-purpose-read = Informatie in dit gebied bekijken.
admin-role-purpose-export = Informatie uit dit gebied exporteren.
admin-role-purpose-revoke = De sessies of inloggegevens intrekken die door dit recht worden genoemd.
admin-role-purpose-delete = Records verwijderen die onder dit recht vallen.
admin-role-purpose-disable = De accounts uitschakelen die onder dit recht vallen.
admin-role-purpose-create = Records aanmaken die onder dit recht vallen.
admin-role-purpose-specific = De specifieke actie uitvoeren die door deze mogelijkheid wordt genoemd; controleer de identificator voordat u haar toekent.
admin-role-purpose-description = Doel en verantwoordelijkheden
admin-role-purpose-description-help = Leg uit wie deze aangepaste rol moet krijgen en welke taken diegene afhandelt (maximaal 1024 tekens). Deze tekst wijzigt de rechten niet.
