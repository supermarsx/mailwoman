# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = AI-assistent
assist-heading = AI-assistent
assist-transcript-label = Gesprek met de AI-assistent
assist-proposed-action = Voorgestelde actie
assist-outbox-note = Hiermee wordt een bericht in je Outbox geplaatst. Er wordt niets verzonden totdat je het daar bevestigt.
assist-review-outbox = Bekijken in Outbox
assist-review = Bekijken
assist-empty = Vraag de AI-assistent om samen te vatten, op te stellen of dingen te zoeken. De assistent kan acties voorstellen, maar je bevestigt ze altijd zelf.
assist-error = De AI-assistent kon nu niet reageren.
assist-input-label = Bericht aan de AI-assistent
assist-input-placeholder = Vraag het de AI-assistent…
assist-thinking = Bezig met nadenken…
assist-ask = Vragen

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Schrijfhulp
assist-composer-toolbar = Schrijftools
assist-tool-grammar = Grammatica verbeteren
assist-tool-rewrite = Herschrijven
assist-tool-tone = Toon aanpassen
assist-tool-translate = Vertalen
assist-toolarg-tone = Toon
assist-toolarg-translate = Taal
assist-tone-neutral = Neutraal
assist-tone-friendly = Vriendelijk
assist-tone-formal = Formeel
assist-tone-concise = Beknopt
assist-tone-assertive = Stellig
assist-busy = { $label }…
assist-composer-empty-error = Schrijf eerst iets.
assist-composer-error = De AI-assistent kon dat nu niet voltooien.
assist-suggested-edit = Voorgestelde wijziging
assist-apply-draft = Toepassen op concept
assist-discard = Negeren

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Voorgestelde labels
assist-apply-auto = Automatisch toepassen
assist-apply-label = { $label } toepassen
assist-remove-label = { $label } verwijderen
assist-apply = Toepassen
assist-undo = Ongedaan maken

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Ingedrukt houden om te dicteren
assist-dictate-stop = Dicteren stoppen
assist-dictate-hold-text = Ingedrukt houden om te dicteren
assist-dictate-listening = ● Luisteren…
assist-dictate-endpoint-note = Audio wordt omgezet naar tekst door het endpoint van je AI-assistent.
assist-dictate-err = Fout bij dicteren.
assist-transcribe-err = De audio kon niet worden omgezet naar tekst.
assist-mic-err = Microfoon niet beschikbaar.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Semantisch zoeken
assist-semantic-note = De zoektekst wordt naar het endpoint van je AI-assistent gestuurd om op betekenis te rangschikken.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Wat dit apparaat kan verlaten
assist-left-1 = de geselecteerde berichttekst (onderwerp + inhoud) voor de gekozen functie
assist-left-2 = de endpointhost waarnaar die is verzonden
assist-left-3 = nooit: end-to-end versleutelde inhoud (standaard uitgesloten)
assist-left-4 = nooit: bijlagen (standaard uitgesloten)
assist-left-5 = nooit: je inloggegevens of andere accounts
assist-disclosure-off = De AI-assistent staat uit. Er verlaat geen berichtinhoud dit apparaat.
assist-disclosure-sentence = Wanneer je een tool van de AI-assistent gebruikt, wordt de geselecteerde berichttekst via de server doorgestuurd naar { $host }. { $excl } Verzenden gebeurt nooit automatisch — je bevestigt altijd voordat er iets je Outbox verlaat.
assist-disclosure-excl-one = Er wordt nooit { $a } verzonden.
assist-disclosure-excl-two = Er wordt nooit { $a } of { $b } verzonden.
assist-disclosure-admin-allowed = Je beheerder heeft toegestaan dat versleutelde inhoud en bijlagen worden verzonden.
assist-excluded-e2ee = end-to-end versleutelde inhoud
assist-excluded-attachments = bijlagen

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = AI-assistent
assist-admin-intro = De AI-assistent stuurt berichttekst die gebruikers selecteren naar het hier ingestelde AI-endpoint. De assistent kan geen mail verzenden, verwijderen of accepteren. End-to-end versleutelde inhoud en bijlagen worden achtergehouden, tenzij je ze hieronder toestaat.
assist-admin-load-unreadable = De opgeslagen configuratie van de AI-assistent kan niet worden gelezen: de API-sleutel ervan opent niet met de sleutel van deze server (MW_SERVER_KEY). Ze is onder een andere sleutel opgeslagen of is gewijzigd. De AI-assistent staat uit. Vul het formulier in, inclusief de API-sleutel, en sla op om haar te vervangen.
assist-admin-load-error = De configuratie van de AI-assistent kon niet worden geladen. Opslaan is uitgeschakeld zodat de opgeslagen configuratie niet wordt overschreven.
assist-admin-status-running = De AI-assistent draait. Verzoeken gaan naar { $host }.
assist-admin-status-running-pending = De AI-assistent draait met de configuratie waarmee de server is gestart. Opgeslagen wijzigingen worden actief wanneer de server opnieuw start.
assist-admin-status-off = De AI-assistent staat uit. Elk verzoek aan de AI-assistent wordt geweigerd.
assist-admin-status-pending = De AI-assistent draait niet. De opgeslagen configuratie wordt actief wanneer de server opnieuw start.
assist-admin-status-local-program-not-allowed = De AI-assistent draait niet. Het ingestelde programma { $program } staat niet in MW_ASSIST_LOCAL_PROGRAMS op de server, dus de server heeft het niet gestart.
assist-admin-status-no-endpoint = De AI-assistent staat aan maar draait niet, omdat er geen bruikbaar endpoint is ingesteld.
assist-admin-stop = De AI-assistent nu stoppen
assist-admin-stop-note = Stoppen geldt direct voor elke gebruiker en blijft behouden na een herstart.
assist-admin-resume = De AI-assistent inschakelen
assist-admin-resume-note = Als de server is gestart met de AI-assistent uitgeschakeld, wordt inschakelen actief wanneer de server opnieuw start.
assist-admin-kill-error = Kan niet wijzigen of de AI-assistent aan staat.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = De server maakt contact met dit endpoint; browsers nooit. Wijzigingen worden actief wanneer de server opnieuw start.
assist-admin-kind = Type endpoint
assist-admin-kind-none = Geen
assist-admin-kind-open-ai-compatible = OpenAI-compatibel
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Lokaal programma
assist-admin-local-program-enable = Wil je dat de AI-assistent een programma op deze server uitvoert, stel dan MW_ASSIST_LOCAL_PROGRAMS in op het absolute pad van het programma in de omgeving waar de server wordt gestart (scheid meerdere paden met komma's) en start de server opnieuw.
assist-admin-local-program-note = De server start dit programma voor elk verzoek aan de AI-assistent en geeft het verzoek door via standaardinvoer, inclusief de berichttekst die de gebruiker heeft geselecteerd. Het antwoord wordt gelezen uit de standaarduitvoer van het programma. Alleen programma's die in MW_ASSIST_LOCAL_PROGRAMS op de server staan, kunnen worden gekozen.
assist-admin-local-program = Programma
assist-admin-local-program-unlisted = { $program } (niet vermeld)
assist-admin-local-program-unlisted-note = Dit programma staat niet in MW_ASSIST_LOCAL_PROGRAMS, dus de server voert het niet uit en accepteert het niet bij het opslaan. Kies een vermeld programma of een ander type endpoint.
assist-admin-local-program-required = Kies het programma dat moet worden uitgevoerd.
assist-admin-local-args = Argumenten, één per regel
assist-admin-local-args-note = Elke regel wordt als één argument aan het programma doorgegeven. Er komt geen shell aan te pas.
assist-admin-base-url = Basis-URL
assist-admin-base-url-required = Voer de basis-URL van het endpoint in, beginnend met http:// of https://.
assist-admin-api-key = API-sleutel
assist-admin-api-key-stored = Er is een sleutel opgeslagen. De server toont hem niet opnieuw.
assist-admin-api-key-replace = De API-sleutel vervangen
assist-admin-api-key-replace-placeholder = Leeg laten om de opgeslagen sleutel te behouden
assist-admin-api-key-remove = De opgeslagen sleutel verwijderen
assist-admin-api-key-keep = De opgeslagen sleutel behouden
assist-admin-api-key-remove-pending = De opgeslagen sleutel wordt verwijderd wanneer je opslaat.
assist-admin-chat-model = Chatmodel
assist-admin-embed-model = Embeddingmodel
assist-admin-stt-model = Spraak-naar-tekstmodel
assist-admin-model = Model
assist-admin-anthropic-version = API-versie
assist-admin-max-tokens = Maximum aantal tokens per antwoord
assist-admin-default-placeholder = Standaard van de server
assist-admin-grants = Functies
assist-admin-grants-note = Alleen de hier aangevinkte functies kunnen worden gebruikt.
assist-admin-cap-summarize = Samenvatten
assist-admin-cap-draft = Opstellen en herschrijven
assist-admin-cap-grammar = Grammatica
assist-admin-cap-dictation = Dicteren
assist-admin-cap-search-semantic = Semantisch zoeken
assist-admin-cap-auto-tag = Automatisch labelen
assist-admin-cap-recap = Overzicht
assist-admin-cap-assistant = Chat met de AI-assistent
assist-admin-ceilings = Gegevenslimieten
assist-admin-ceilings-note = Deze limieten gelden voor elk verzoek, ongeacht de functie.
assist-admin-accounts = Account-id's waarvan mail mag worden verzonden, één per regel
assist-admin-accounts-note = Als er geen account is vermeld, wordt er geen mailinhoud naar het endpoint gestuurd.
assist-admin-folders = Map-id's waartoe het verzenden wordt beperkt, één per regel
assist-admin-folders-note = Als er geen map is vermeld, zijn alle mappen van de vermelde accounts toegestaan.
assist-admin-allow-e2ee = Verzenden van end-to-end versleutelde inhoud toestaan
assist-admin-allow-attachments = Verzenden van bijlagen toestaan
assist-admin-save = Configuratie opslaan
assist-admin-saved = Opgeslagen.
assist-admin-saved-pending = Opgeslagen. De wijziging wordt actief wanneer de server opnieuw start.
assist-admin-save-error = De configuratie is niet opgeslagen.
assist-admin-save-not-allowed = De configuratie is niet opgeslagen omdat de server dit programma niet toestaat. Alleen programma's die in MW_ASSIST_LOCAL_PROGRAMS staan, kunnen worden gebruikt.
