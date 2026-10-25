# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Waarschuwing voor niet-ondertekende UI-plug-in
plugins-unsigned-title = Er draaien niet-ondertekende plug-ins
plugins-unsigned-text = Deze plug-ins zijn toegelaten zonder geverifieerde handtekening. Ze draaien in een sandbox, maar hun code is niet gecontroleerd met een vertrouwde sleutel.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plug-in: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Engine-plug-ins draaien in een WebAssembly-sandbox. Een plug-in draait pas nadat hij is geregistreerd, goedgekeurd, minstens één mogelijkheid heeft gekregen en is ingeschakeld. Hij kan alleen de mogelijkheden gebruiken die hier zijn toegekend.
plugins-admin-unsigned-banner = Er is een plug-in van derden zonder handtekening geladen. Hij draait omdat een beheerder dat heeft toegestaan; de server controleert zijn bestand alleen aan de hand van de goedgekeurde digest.
plugins-admin-load-error = Kan de plug-ins niet laden.
plugins-admin-register-heading = Plug-in registreren
plugins-admin-register-component = Component
plugins-admin-register-third-party = Component van derden
plugins-admin-register-id = Plug-in-ID
plugins-admin-register-id-hint = De server leest de component uit het bestand met deze ID als naam in de map voor plug-ins van derden. Keur eerst de digest van dat bestand goed in de toegestane lijst hieronder.
plugins-admin-register-name = Naam
plugins-admin-register-version = Versie
plugins-admin-register-capabilities = Mogelijkheden die de plug-in opgeeft
plugins-admin-register-hosts = Hosts waarmee de plug-in contact mag maken
plugins-admin-register-hosts-hint-first-party = Hostnamen, gescheiden door komma's. Laat leeg om de standaardhosts van de component te gebruiken.
plugins-admin-register-hosts-hint-third-party = Hostnamen, gescheiden door komma's. Laat leeg voor geen netwerktoegang.
plugins-admin-register-submit = Registreren
plugins-admin-trust-first-party = Eigen plug-in, digest ingebouwd
plugins-admin-trust-third-party = Van derden, digest goedgekeurd
plugins-admin-status-loaded = Geladen
plugins-admin-status-not-loaded = Niet geladen
plugins-admin-status-restart = Herstart vereist
plugins-admin-restart-to-load = Herstart vereist om deze plug-in te laden.
plugins-admin-restart-to-apply = Herstart vereist om deze wijzigingen toe te passen. Sommige accounts van deze bridge worden tot dan niet bediend.
plugins-admin-reason-not-approved = Niet geladen: een beheerder heeft deze plug-in niet goedgekeurd.
plugins-admin-reason-disabled = Niet geladen: de plug-in is niet ingeschakeld.
plugins-admin-reason-unsigned-not-allowed = Niet geladen: de plug-in heeft geen handtekening en mag niet zonder handtekening draaien.
plugins-admin-reason-no-grant = Niet geladen: geen van de mogelijkheden die hij nodig heeft, is toegekend.
plugins-admin-reason-component-unavailable = Niet geladen: geen enkel componentbestand heeft de digestcontrole doorstaan.
plugins-admin-reason-load-failed = Niet geladen: de server kon de component niet laden. Zie het serverlogboek.
plugins-admin-reason-proxy-mode = Niet geladen: deze server draait in proxymodus en verwerkt zelf geen mail.
plugins-admin-reason-no-account-binding = Niet geladen: er is geen account aan deze bridge gekoppeld.
plugins-admin-reason-no-host-caller = Niet geladen: deze serverversie roept de hooks die deze plug-in biedt niet aan.
plugins-admin-reason-another-classifier-active = Niet geladen: er is al een andere spamclassificatie geladen. Er draait er maar één tegelijk.
plugins-admin-running-with = Draait met: { $capabilities }
plugins-admin-hosts = Hosts: { $hosts }
plugins-admin-hosts-none = Hosts: geen
plugins-admin-grants-heading = Toegekende mogelijkheden
plugins-admin-grants-hint = De plug-in kan alleen de hier aangevinkte mogelijkheden gebruiken. Opslaan vervangt de eerdere toekenning.
plugins-admin-grant-for = { $capability } toekennen aan { $name }
plugins-admin-grants-save = Toekenningen opslaan
plugins-admin-endpoint = Adres van classificatie
plugins-admin-endpoint-for = Adres van classificatie voor { $name }
plugins-admin-endpoint-hint = Waar de plug-in zijn daemon bereikt. De host moet ook in de hostlijst van de plug-in staan. Laat leeg om de standaardwaarde van de component te gebruiken.
plugins-admin-endpoint-save = Adres opslaan
plugins-admin-test = Classificatie testen
plugins-admin-test-verdict = Testresultaat: { $verdict }
plugins-admin-uninstall = Verwijderen
plugins-admin-uninstall-confirm = Verwijderen bevestigen
plugins-admin-uninstall-note = Verwijderen wist de registratie, de toekenningen en de opgeslagen instellingen.
plugins-admin-uninstall-detail = Verwijderen wist de opgeslagen gegevens van de plug-in voor elk account, de vastgezette items in de toegestane lijst, en de registratie en toekenningen. Het componentbestand op schijf wordt niet verwijderd.
plugins-admin-revoke-detail = Intrekken verwijdert de goedkeuring voor deze digest en schakelt de plug-in uit. Als de plug-in draait, wordt hij direct gestopt.
plugins-admin-error-already-registered = Deze plug-in is al geregistreerd. Verwijder hem voordat je hem opnieuw registreert.
plugins-admin-error-component-unavailable = Er is geen componentbestand voor deze plug-in dat overeenkomt met de digest die in de server is ingebouwd.
plugins-admin-error-digest-not-approved = Er is voor deze ID geen componentbestand met een goedgekeurde digest gevonden. Keur eerst de digest van het bestand goed in de toegestane lijst.
plugins-admin-error-not-approved = Keur de plug-in goed voordat je hem inschakelt.
plugins-admin-error-unsigned-not-allowed = Deze plug-in heeft geen handtekening. Sta toe dat hij zonder handtekening draait voordat je hem inschakelt.
plugins-admin-error-not-loaded = De plug-in is niet geladen en kan dus niet worden getest.
plugins-admin-error-classifier-error = De aanroep van de classificatie is mislukt: { $detail }
plugins-admin-error-other = De server heeft het verzoek geweigerd: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI-plug-ins
plugins-admin-ui-title = UI-plug-ins
plugins-admin-ui-intro = UI-plug-ins voegen bedieningselementen toe aan de webclient. Elke plug-in draait in een frame met sandbox en kan alleen de mogelijkheden gebruiken die hier zijn toegekend. Een plug-in wordt aan ingelogde gebruikers aangeboden zodra hij is goedgekeurd en ingeschakeld.
plugins-admin-ui-empty = Er zijn geen UI-plug-ins geregistreerd.
plugins-admin-ui-load-error = Kan de UI-plug-ins niet laden.
plugins-admin-ui-register-heading = UI-plug-in registreren
plugins-admin-ui-manifest = Manifest (JSON)
plugins-admin-ui-bundle = Bundelbestand
plugins-admin-ui-bundle-hint = Vereist voor een ondertekende plug-in: de handtekening wordt met dit bestand gecontroleerd.
plugins-admin-ui-allow-unsigned = Toestaan dat deze plug-in zonder handtekening wordt geregistreerd
plugins-admin-ui-register = Registreren
plugins-admin-ui-manifest-invalid = Het manifest is geen geldige JSON.
plugins-admin-ui-refused = De server heeft het verzoek geweigerd: { $detail }
plugins-admin-ui-approve = Goedkeuren en inschakelen
plugins-admin-ui-extension-points = Uitbreidingspunten: { $points }
plugins-admin-ui-grants-heading = Mogelijkheid toekennen
plugins-admin-ui-grants-note = De server meldt niet welke mogelijkheden al zijn toegekend. Een mogelijkheid opnieuw toekennen vervangt de eerdere toekenning.
plugins-admin-ui-grant = { $capability } toekennen
plugins-admin-ui-grant-for = { $capability } toekennen aan { $name }
plugins-admin-ui-grant-hosts = Hosts voor { $capability } bij { $name }
plugins-admin-ui-grant-hosts-hint = Hostnamen, gescheiden door komma's, waarvan de plug-in gegevens mag ophalen.
plugins-admin-ui-granted = { $capability } toegekend.
plugins-admin-ui-delete = Verwijderen
plugins-admin-ui-delete-confirm = Verwijderen bevestigen
