# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Zustellungsbericht
dsn-unverified = Dieser Bericht beschreibt den Zustellversuch des Absenders. Seine Angaben sind nicht unabhängig überprüft. Eine Zustellung bedeutet nicht, dass die Nachricht gelesen wurde.
dsn-reporting-mta = Meldender Server:
dsn-original-recipient = Ursprünglicher Empfänger:
dsn-remote-mta = Gegenserver:
dsn-last-attempt = Letzter Versuch:
dsn-retry-until = Der Absender versucht es erneut bis:
dsn-action-failed = Fehlschlag gemeldet
dsn-action-delayed = Verzögerung gemeldet
dsn-action-delivered = Zustellung gemeldet
dsn-action-relayed = Weiterleitung gemeldet
dsn-action-expanded = Listenerweiterung gemeldet

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Verwenden Sie die Serverdiagnose, wenn Sie Ihren E-Mail-Anbieter um Hilfe bitten.

dsn-advice-1 = Überprüfen Sie sowohl die Absender- als auch die Empfängeradresse. Bestätigen Sie ggf. die aktuelle Adresse des Empfängers.

dsn-advice-2 = Kontaktieren Sie den Empfänger auf andere Weise, wenn das Postfachproblem weiterhin besteht.

dsn-advice-3 = Geben Sie diesen Bericht an den Administrator des empfangenden E-Mail-Systems weiter, wenn das Problem weiterhin besteht.

dsn-advice-4 = Bitten Sie Ihren E-Mail-Administrator, die Konnektivität, DNS und Weiterleitungsrouten zu überprüfen, wenn das Problem weiterhin besteht.

dsn-advice-5 = Geben Sie die Diagnose an Ihren E-Mail-Anbieter weiter, damit dieser die Kompatibilität und Einschränkungen von SMTP prüfen kann.

dsn-advice-6 = Überprüfen Sie das Nachrichtenformat, die internationalen Adressen und Anhänge anhand der Anforderungen des Empfängers.

dsn-advice-7 = Geben Sie die Diagnose an Ihren E-Mail-Administrator weiter, um die genaue Sicherheits- oder Authentifizierungsregel zu ermitteln.

dsn-advice-success = Überprüfen Sie anhand der oben gemeldeten Zustellungsaktion, ob die Nachricht zugestellt, weitergeleitet oder zu einer Liste erweitert wurde.

dsn-category-0 = Der Server hat einen unbekannten oder anbieterspezifischen Detailcode bereitgestellt. Die folgende Diagnose kann die genaue Ursache erklären.

dsn-category-1 = Hierbei handelt es sich um einen adressbezogenen Code. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-2 = Dies ist ein Empfänger-Mailbox-Code. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-3 = Dies ist ein Systemcode für den Empfang von E-Mails. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-4 = Dies ist ein Netzwerk- oder Routingcode. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-5 = Dies ist ein Mail-Protokollcode. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-6 = Dies ist ein Nachrichteninhalts- oder Formatcode. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt.

dsn-category-7 = Dies ist ein Sicherheits- oder Richtliniencode. Die genauen Einzelheiten sind anbieterspezifisch oder nicht bekannt; Es stellt nicht automatisch fest, dass es sich bei der Nachricht um Spam handelt.

dsn-class-2 = Erfolg: Der Server meldet einen positiven Lieferschritt. Dies beweist nicht, dass die Nachricht gelesen wurde.

dsn-class-4 = Vorübergehender Fehlschlag: Ein späterer Versuch könnte erfolgreich sein. Eine Verzögerungsmeldung kann vom Absender dennoch erneut versucht werden; Ein Fehlerbericht bedeutet, dass der Versuch beendet ist.

dsn-class-5 = Dauerhafter Fehler: Es ist unwahrscheinlich, dass das erneute Senden derselben Nachricht unverändert funktioniert. Beheben Sie die Ursache, bevor Sie es erneut senden.

dsn-meaning-title = Was dieser Code bedeutet

dsn-next-step = Was zu tun ist

dsn-provider-barracuda = Barrakuda

dsn-provider-detected = Interpretation basierend auf der Serverdiagnose oder dem Hostnamen:

dsn-provider-exchange = Microsoft Exchange / Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail / Google Workspace

dsn-provider-help = Dokumentation des Anbieters

dsn-provider-mimecast = Mimecast

dsn-provider-proofpoint = Beweispunkt

dsn-provider-relay = SMTP Relais

dsn-provider-yahoo = Yahoo / AOL

dsn-server-diagnostic = Serverdiagnose

dsn-status-0-0 = Einen genaueren Grund nannte der Server nicht.

dsn-status-1-0 = Es liegt ein Problem mit einer E-Mail-Adresse vor.

dsn-status-1-1 = Das Empfängerpostfach existiert nicht. Bestätigen Sie die Adresse mit dem Empfänger.

dsn-status-1-10 = Die Empfängerdomäne erklärt ausdrücklich, dass sie keine E-Mails akzeptiert (null MX). Verwenden Sie eine andere Adresse.

dsn-status-1-2 = Die Empfängerdomäne kann diese E-Mail nicht empfangen. Überprüfen Sie den Teil nach @.

dsn-status-1-3 = Die Empfängeradresse hat ein ungültiges Format. Suchen Sie nach Tippfehlern oder zusätzlichen Leerzeichen.

dsn-status-1-4 = Der Server kann keinen eindeutigen Empfänger für diese Adresse identifizieren.

dsn-status-1-5 = Die Empfängeradresse wurde als gültig akzeptiert.

dsn-status-1-6 = Dieses Postfach wurde verschoben und es ist keine Weiterleitungsadresse verfügbar. Fragen Sie nach der neuen Adresse.

dsn-status-1-7 = Die Absenderadresse hat ein ungültiges Format. Überprüfen Sie die Absenderadresse des Kontos.

dsn-status-1-8 = Die Absenderdomäne ist ungültig oder kann keine Rückmails empfangen.

dsn-status-1-9 = Die Nachricht wurde an ein System übergeben, das keine weiteren Zustellberichte bereitstellen kann.

dsn-status-2-0 = Im Postfach des Empfängers liegt ein nicht näher bezeichnetes Problem vor.

dsn-status-2-1 = Das Empfängerpostfach ist deaktiviert oder akzeptiert keine Nachrichten. Kontaktieren Sie den Empfänger auf andere Weise.

dsn-status-2-2 = Das Postfach des Empfängers ist voll. Der Empfänger muss Speicherplatz freigeben oder sein Kontingent erhöhen.

dsn-status-2-3 = Die Nachricht überschreitet die Größenbeschränkung dieses Empfängers. Reduzieren Sie Anhänge oder verwenden Sie einen Freigabelink.

dsn-status-2-4 = Der Server konnte die Mitglieder einer Mailingliste nicht auflösen. Kontaktieren Sie den Listeneigentümer.

dsn-status-3-0 = Das empfangende E-Mail-System hat ein nicht näher bezeichnetes Problem gemeldet.

dsn-status-3-1 = Der Speicherplatz des empfangenden Mailservers ist erschöpft. Der Administrator muss Speicherplatz freigeben.

dsn-status-3-2 = Der empfangende Server akzeptiert keine E-Mails, beispielsweise während Wartungsarbeiten oder Überlastung.

dsn-status-3-3 = Das empfangende System unterstützt eine für diese Nachricht erforderliche Funktion nicht.

dsn-status-3-4 = Die Nachricht überschreitet die Größenbeschränkung des empfangenden Servers. Reduzieren Sie Anhänge oder verwenden Sie einen Freigabelink.

dsn-status-3-5 = Ein Problem mit der Konfiguration des Mailservers verhindert die Zustellung. Bitten Sie den E-Mail-Administrator, dies zu untersuchen.

dsn-status-3-6 = Der Server hat die Nachricht akzeptiert, aber die angeforderte Zustellungspriorität geändert.

dsn-status-4-0 = Es wurde ohne weitere Einzelheiten ein Netzwerk- oder E-Mail-Routing-Problem gemeldet.

dsn-status-4-1 = Der empfangende Server hat den Verbindungsversuch nicht beantwortet.

dsn-status-4-2 = Die Verbindung wurde hergestellt, aber unterbrochen, bevor die E-Mail-Übertragung abgeschlossen war.

dsn-status-4-3 = Eine DNS- oder Verzeichnissuche ist fehlgeschlagen. Möglicherweise muss ein E-Mail-Administrator die Namensauflösung überprüfen.

dsn-status-4-4 = Der Server konnte keine Route zum Mailsystem des Empfängers finden.

dsn-status-4-5 = Überlastung des E-Mail-Verkehrs verhinderte die Zustellung.

dsn-status-4-6 = Es wurde eine Weiterleitungsschleife erkannt. Bitten Sie den Administrator, die Weiterleitungs- und Routing-Regeln zu überprüfen.

dsn-status-4-7 = Die Lieferzeit ist abgelaufen. Überprüfen Sie die Diagnose auf das zugrunde liegende Problem, bevor Sie es erneut senden.

dsn-status-5-0 = Die Mailserver konnten den Austausch ihres Zustellungsprotokolls nicht abschließen.

dsn-status-5-1 = Ein Mailserver hat einen nicht unterstützten Befehl oder einen in der falschen Reihenfolge gesendeten Befehl abgelehnt.

dsn-status-5-2 = Ein Mailserver konnte die Syntax eines Befehls nicht verstehen.

dsn-status-5-3 = Die Nachricht hat zu viele Empfänger. An kleinere Gruppen senden.

dsn-status-5-4 = Ein Mailserver hat die Parameter eines Befehls abgelehnt.

dsn-status-5-5 = Die Server verwenden inkompatible Versionen des Mail-Protokolls.

dsn-status-5-6 = Die Authentifizierungsantwort war für den Server zu lang. Bitten Sie den E-Mail-Administrator, die Authentifizierungseinstellungen zu überprüfen.

dsn-status-6-0 = Der Server hat ein nicht näher bezeichnetes Problem mit dem Nachrichteninhalt oder -format gemeldet.

dsn-status-6-1 = Das empfangende System unterstützt den Inhaltstyp dieser Nachricht nicht.

dsn-status-6-10 = Dieser veraltete Code bedeutet, dass eine internationale Antwort (UTF-8) erforderlich ist, aber nicht unterstützt wird, wie beim Code x.6.8.

dsn-status-6-2 = Die Zustellung erfordert eine Inhaltskonvertierung, die die Absender- oder Serverrichtlinie verbietet.

dsn-status-6-3 = Für die Bereitstellung ist eine Inhaltskonvertierung erforderlich, die der Server nicht durchführen kann.

dsn-status-6-4 = Bei einer Inhaltskonvertierung gingen einige Informationen verloren. Überprüfen Sie die Originalnachricht und die Serverdiagnose.

dsn-status-6-5 = Der Server ist beim Konvertieren des Nachrichteninhalts ausgefallen.

dsn-status-6-6 = Der Server konnte den Nachrichteninhalt nicht von einem anderen System abrufen.

dsn-status-6-7 = Eine internationale E-Mail-Adresse wird für diesen Absender oder Empfänger nicht unterstützt. Fragen Sie nach einer alternativen Adresse.

dsn-status-6-8 = Der Server benötigt eine internationale Antwort (UTF-8), die das sendende System nicht akzeptieren kann.

dsn-status-6-9 = Die Nachricht enthält internationale Header (UTF-8), die nicht an mindestens einen Empfänger zugestellt werden können.

dsn-status-7-0 = Eine Sicherheitsregel oder E-Mail-Richtlinie verhinderte eine normale Zustellung. Überprüfen Sie die Serverdiagnose auf die genaue Regel.

dsn-status-7-1 = Der Server hat die Zustellung verweigert, möglicherweise aufgrund von Filter- oder Weiterleitungseinschränkungen. Bitten Sie den E-Mail-Administrator, die Diagnose zu überprüfen.

dsn-status-7-10 = Der Server benötigt vor der Authentifizierung eine verschlüsselte Verbindung oder einen stärkeren Schutz.

dsn-status-7-11 = Diese Authentifizierungsmethode erfordert eine verschlüsselte SMTP-Verbindung (TLS).

dsn-status-7-12 = Das Konto benötigt eine Authentifizierungsmethode oder eine Passwortmigration. Kontaktieren Sie den Kontoanbieter.

dsn-status-7-13 = Das sendende Konto ist deaktiviert. Wenden Sie sich an den Administrator, um den Zugriff wiederherzustellen.

dsn-status-7-14 = Der Server benötigt eine konfigurierte Vertrauensbeziehung, um die Nachricht abzurufen. Kontaktieren Sie den Administrator.

dsn-status-7-15 = Die Nachrichtenpriorität liegt unter dem aktuellen Minimum des empfangenden Servers.

dsn-status-7-16 = Die Nachricht ist zu groß für die angeforderte Priorität. Reduzieren Sie die Größe oder fragen Sie den Administrator.

dsn-status-7-17 = Das Empfängerpostfach hat seit dem erforderlichen Datum den Besitzer geändert. Bestätigen Sie den Empfänger, bevor Sie erneut senden.

dsn-status-7-18 = Die Empfängerdomäne hat seit dem erforderlichen Datum den Besitzer geändert. Bestätigen Sie das Ziel, bevor Sie es erneut senden.

dsn-status-7-19 = Der Server konnte den dauerhaften Besitz der Empfängeradresse nicht überprüfen. Fragen Sie den Administrator nach der Empfängergültigkeitsprüfung.

dsn-status-7-2 = Dem Absender ist es nicht gestattet, auf dieser Mailingliste zu posten. Kontaktieren Sie den Listeneigentümer.

dsn-status-7-20 = Keine DKIM-Signatur hat die Überprüfung bestanden. Bitten Sie den E-Mail-Administrator des Absenders, die Signatur und DNS zu überprüfen.

dsn-status-7-21 = Eine DKIM-Signatur wurde übergeben, die Richtlinie des empfangenden Servers hat sie jedoch nicht akzeptiert.

dsn-status-7-22 = Keine akzeptierte DKIM-Signatur stimmte mit der Absenderadresse überein. Bitten Sie den E-Mail-Administrator, die Ausrichtung der Signaturdomäne zu überprüfen.

dsn-status-7-23 = SPF fehlgeschlagen: Der sendende Server wurde nicht durch die SPF-Richtlinie der Absenderdomäne autorisiert.

dsn-status-7-24 = Beim Überprüfen von SPF ist auf dem Server ein Fehler aufgetreten. Bitten Sie den E-Mail-Administrator, die Datensätze SPF und DNS zu überprüfen.

dsn-status-7-25 = Der sendende Server hat die Reverse-DNS-Prüfung des Empfängers nicht bestanden. Der Administrator muss die Datensätze DNS der Adresse IP überprüfen.

dsn-status-7-26 = Mehrere Überprüfungen der Nachrichtenauthentifizierung sind fehlgeschlagen. Die Diagnose kann SPF-, DKIM- oder DMARC-Richtlinienprobleme identifizieren.

dsn-status-7-27 = Die Absenderdomäne erklärt, dass sie keine Antwort-E-Mail akzeptiert (null MX), die der Empfänger ablehnt.

dsn-status-7-28 = Das empfangende System hat eine mögliche Flut missbräuchlicher Nachrichten erkannt.

dsn-status-7-29 = Die ARC-Verifizierung ist fehlgeschlagen, sodass der Empfänger die aufgezeichnete Authentifizierungskette der Nachricht nicht akzeptieren konnte.

dsn-status-7-3 = Für die Zustellung ist eine Konvertierung des Sicherheitsformats erforderlich, die der Server nicht durchführen kann.

dsn-status-7-30 = Die Nachricht erfordert REQUIRETLS, aber der nächste Mailserver unterstützt es nicht. Bitten Sie den Administrator, die Unterstützung für sichere Zustellung zu überprüfen.

dsn-status-7-4 = Das Liefersystem unterstützt eine erforderliche Sicherheitsfunktion nicht.

dsn-status-7-5 = Der Server konnte die Nachricht nicht validieren oder entschlüsseln, weil beispielsweise ein Schlüssel fehlte oder ungültig war.

dsn-status-7-6 = Der Server unterstützt keinen für die Nachricht erforderlichen kryptografischen Algorithmus.

dsn-status-7-7 = Eine Integritätsprüfung zeigt an, dass die Nachricht geändert oder beschädigt wurde.

dsn-status-7-8 = Der sendende Server hat die Anmeldeinformationen abgelehnt. Überprüfen Sie die Authentifizierungseinstellungen des Kontos.

dsn-status-7-9 = Die Authentifizierungsmethode ist schwächer als der Server zulässt. Eine stärkere Methode ist erforderlich.

dsn-vendor-advice-account = Bitten Sie den sendenden Administrator, einen möglichen Kontomissbrauch zu untersuchen, das Konto zu sichern und die Aufhebung der Einschränkung zu beantragen.

dsn-vendor-advice-admin = Geben Sie diese Diagnose an den E-Mail-Administrator der Organisation weiter, um die entsprechende Konfiguration oder Richtlinie zu überprüfen.

dsn-vendor-advice-auth = Überprüfen Sie die unterstützte SMTP-Authentifizierungsmethode und Autorisierung des Kontos. Führen Sie alle erforderlichen Anmeldungen über die offizielle Kontoseite des Anbieters durch.

dsn-vendor-advice-authentication = Bitten Sie den Administrator des Absenders, SPF, DKIM, DMARC und die Domänenausrichtung mithilfe der genauen Diagnose zu überprüfen.

dsn-vendor-advice-connector = Bitten Sie den Administrator, die Relay-Autorisierung, den Connector IP/Zertifikatsübereinstimmung, akzeptierte Domänen und die tatsächliche E-Mail-Route zu überprüfen.

dsn-vendor-advice-content = Überprüfen Sie gemeinsam mit dem Administrator des Empfängers die Diagnose- und Gateway-Protokolle, um den blockierten Inhalt und die geltende Richtlinie zu ermitteln.

dsn-vendor-advice-dns = Bitten Sie den sendenden E-Mail-Administrator oder IP-Anbieter, DNS zu korrigieren, DNS umzukehren und die Serveridentität SMTP zu korrigieren.

dsn-vendor-advice-domain = Bitten Sie den Microsoft 365-Administrator, die sendende Domäne zu registrieren und zu überprüfen und die Sendebeschränkungen des Mandanten zu überprüfen.

dsn-vendor-advice-format = Bitten Sie die sendende Anwendung oder den E-Mail-Administrator, die Nachrichtengenerierung zu korrigieren und erstellen Sie dann eine neue Nachricht.

dsn-vendor-advice-rate = Reduzieren Sie das Sendevolumen und warten Sie, bis das Kontingent oder die Einschränkung gelöscht ist. Überprüfen Sie die gemeldete Aktion, bevor Sie davon ausgehen, dass sich dieser Versuch noch in der Warteschlange befindet.

dsn-vendor-advice-recipient-admin = Kontaktieren Sie den Empfänger oder Gruppeneigentümer auf andere Weise und bitten Sie seinen Administrator, die entsprechende Berechtigung oder Richtlinie zu überprüfen.

dsn-vendor-advice-reputation = Bitten Sie den sendenden Administrator, unerwünschten Datenverkehr und unerwünschten Ruf zu untersuchen, und folgen Sie dann dem offiziellen Überprüfungs- oder Delisting-Prozess des Anbieters.

dsn-vendor-advice-route = Bitten Sie den E-Mail-Administrator, das Ziel, die Weiterleitungsregeln, den E-Mail-Dienst und den Firewall-/Netzwerkpfad zu überprüfen.

dsn-vendor-advice-tls = Bitten Sie die E-Mail-Administratoren, TLS, Zertifikate oder DNS-Sicherheitseinträge zu reparieren, damit die erforderliche sichere Route erfolgreich ausgeführt werden kann.

dsn-vendor-barracuda-reputation = Der Reputationsdienst von Barracuda listet oder blockiert die sendende IP-Adresse.

dsn-vendor-certificate-chain = Das Remote-Zertifikat TLS konnte nicht anhand der erforderlichen Vertrauensstellung oder der Richtlinie MTA-STS validiert werden.

dsn-vendor-certificate-expired = Der Ziel-Mailserver hat ein abgelaufenes TLS-Zertifikat vorgelegt.

dsn-vendor-certificate-host = Das Zielzertifikat TLS stimmt nicht mit dem erforderlichen Hostnamen des Mailservers überein.

dsn-vendor-connection-refused = Der Zielserver hat die Netzwerkverbindung abgelehnt. Der E-Mail-Dienst, die Firewall oder die Netzwerkroute müssen überprüft werden.

dsn-vendor-content-security = Das Gateway hat den Nachrichteninhalt oder einen Anhang gemäß seinen Sicherheitsfilterregeln abgelehnt. Konsultieren Sie die Diagnose oder Protokolle für das jeweilige Element.

dsn-vendor-credentials = Der SMTP-Dienst hat die angegebenen Anmeldeinformationen abgelehnt.

dsn-vendor-dane = Das TLS-Zertifikat oder die TLSA-Konfiguration des Ziels hat die DANE-Validierung nicht bestanden.

dsn-vendor-dkim = Der Empfänger hat die DKIM-Authentifizierungs- oder Signaturkonfiguration der Nachricht nicht akzeptiert.

dsn-vendor-dmarc = Die DMARC-Richtlinie des Empfängers wurde nicht erfüllt. Überprüfen Sie die SPF- oder DKIM-Authentifizierung und Ausrichtung mit der sichtbaren Von-Domäne.

dsn-vendor-dnssec = Die DNSSEC-Einträge der Zieldomäne konnten nicht validiert werden.

dsn-vendor-exchange-auth = Die Anwendung oder das Gerät wurde für die Microsoft 365 SMTP-Übermittlung nicht ordnungsgemäß authentifiziert.

dsn-vendor-exchange-connector = Microsoft konnte das Relay nicht über den konfigurierten Mandanten-Connector autorisieren. Sein Zertifikat, der sendende IP oder die akzeptierte Domäne stimmen möglicherweise nicht mehr überein.

dsn-vendor-exchange-domain-blocked = Microsoft hat diese Sendedomäne eingeschränkt, nachdem eine vermutete Spam-Aktivität festgestellt wurde.

dsn-vendor-exchange-forward-relay = Ein nachgeschaltetes System hat weitergeleitete oder weitergeleitete E-Mails abgelehnt. Die Authentifizierung wurde möglicherweise durch die Weiterleitung oder ein Nicht-Microsoft-Gateway beeinträchtigt.

dsn-vendor-exchange-forwarding = Die Microsoft 365-Richtlinie blockiert die automatische Weiterleitung an externe Empfänger. Der Administrator der Organisation muss die Weiterleitungsrichtlinie überprüfen.

dsn-vendor-exchange-generic = Exchange hat eine allgemeine Ablehnung der Zustellung gemeldet. Die Diagnose identifiziert das eigentliche Problem, bei dem es sich möglicherweise um die Nachrichtenformatierung oder die Anhangsrichtlinie handelt.

dsn-vendor-exchange-group-allow = Der Absender ist nicht in der Liste der zulässigen Absender dieser Exchange-Gruppe enthalten.

dsn-vendor-exchange-group-internal = Diese Exchange-Verteilergruppe lehnt Absender außerhalb ihrer Organisation ab, es sei denn, ihr Besitzer oder Administrator erlaubt dies.

dsn-vendor-exchange-internal-only = Dieser Exchange-Empfänger akzeptiert authentifizierte Absender aus seiner eigenen Organisation; Diese Nachricht wurde als extern oder nicht authentifiziert behandelt.

dsn-vendor-exchange-ipv6 = Der Bestimmungsort akzeptiert diese Lieferung nicht über IPv6. Der sendende Administrator muss die Route überprüfen oder ein akzeptiertes Transportmittel verwenden.

dsn-vendor-exchange-journaling = Die Organisation versucht, lokale Nachrichten ohne die erforderliche Archivkonfiguration in Microsoft 365 zu protokollieren.

dsn-vendor-exchange-onmicrosoft-limit = Die Organisation hat das externe Empfängerlimit für ihre onmicrosoft.com-Sendedomäne erreicht. Für den regulären Versand sollte eine verifizierte benutzerdefinierte Domäne verwendet werden.

dsn-vendor-exchange-public-folder = Dieser öffentliche Exchange-Ordner akzeptiert diesen externen oder nicht authentifizierten Absender nicht.

dsn-vendor-exchange-receive-rate = Dieser Exchange-Empfänger hat das stündliche Empfangslimit aller Absender erreicht.

dsn-vendor-exchange-recipient = Exchange konnte diesen Empfänger in seinem Verzeichnis nicht auflösen. Überprüfen Sie die Adresse und entfernen Sie alle veralteten Autovervollständigungseinträge, bevor Sie eine neue Nachricht erstellen.

dsn-vendor-exchange-recipient-access = Exchange hat den Zugriff auf diese Empfänger- oder Zieldomäne abgelehnt. Überprüfen Sie die Empfängeradresse und die akzeptierte Domänenkonfiguration der Organisation.

dsn-vendor-exchange-recipient-rate = Exchange hat innerhalb seines stündlichen Zeitfensters zu viele Nachrichten von diesem Absender an diesen Empfänger erhalten.

dsn-vendor-exchange-sender-blocked = Microsoft hat dieses Sendekonto eingeschränkt, nachdem ein Verdacht auf Spam oder Missbrauch festgestellt wurde. Dies erfordert möglicherweise, dass ein Administrator das Konto sichert und entsperrt.

dsn-vendor-exchange-temporary-restriction = Microsoft schränkt das Senden von IP oder diesen Datenverkehr vorübergehend ein, während verdächtige Aktivitäten ausgewertet werden.

dsn-vendor-exchange-tenant-blocked = Microsoft hat den Versand für diese Organisation eingeschränkt, da ihr Datenverkehr als verdächtig eingestuft wurde.

dsn-vendor-exchange-tenant-blocklist = Eine Empfängeradresse oder -domäne wird durch die Mandanten-Zulassungs-/Blockierungsliste der Absenderorganisation blockiert. Ein blockiertes Ziel kann die gesamte Nachricht ablehnen.

dsn-vendor-exchange-tenant-limit = Die Microsoft 365-Organisation hat ihr fortlaufendes Tageslimit für externe Empfänger erreicht.

dsn-vendor-exchange-unregistered = Microsoft hat das Senden von einer Domäne abgelehnt, die im Mandanten nicht ordnungsgemäß registriert und verifiziert ist.

dsn-vendor-expired = Der Versanddienst hat nach Ablauf seines Lieferzeitfensters aufgegeben. Überprüfen Sie die Diagnose auf den Verbindungs- oder Routingfehler, der die Verzögerung verursacht hat.

dsn-vendor-fortimail-relay = FortiMail hat diese Zustellung aufgrund seiner Zugriffskontroll- oder Relay-Richtlinie abgelehnt. Der Gateway-Administrator muss die passende Empfangsregel überprüfen.

dsn-vendor-gmail-app-password = Google benötigt für diese SMTP-Anmeldung ein App-Passwort oder eine unterstützte Kontoautorisierungsmethode.

dsn-vendor-gmail-auth-method = Google hat eine nicht unterstützte Authentifizierungsmethode abgelehnt, beispielsweise die eingestellte Methode XOAUTH.

dsn-vendor-gmail-authentication = Google hat die Authentifizierung des Absenders nicht akzeptiert. Die Diagnose unterscheidet zwischen fehlendem SPF/DKIM, SPF hartem Fehler und DMARC Ablehnung.

dsn-vendor-gmail-browser-login = Google erfordert eine Kontoanmeldung oder eine Sicherheitsüberprüfung im Browser, bevor diese SMTP-Anmeldung zugelassen wird.

dsn-vendor-gmail-custom-policy = Die benutzerdefinierte E-Mail-Regel eines Google Workspace-Administrators hat die Nachricht abgelehnt. Dies bedeutet nicht unbedingt, dass der Spamfilter von Google die E-Mail abgelehnt hat.

dsn-vendor-gmail-dmarc-record = Google benötigt einen DMARC-Eintrag mit einer deklarierten Richtlinie für diese sendende Domain.

dsn-vendor-gmail-rate = Google hat ein Sende- oder Weiterleitungslimit angewendet oder eine ungewöhnliche Nachrichtenrate festgestellt. Die Diagnose identifiziert das betroffene Konto, IP oder die betroffene Domäne.

dsn-vendor-gmail-relay = Google Workspace hat den sendenden IP, die Domäne oder das Konto für seinen Relay-Dienst SMTP nicht autorisiert.

dsn-vendor-gmail-reputation = Die Filterung von Google lehnte die Nachricht ab, weil sie die E-Mail als unaufgefordert einstufte oder die Reputation des Absenders IP/Domain zu niedrig war.

dsn-vendor-gmail-spf-record = Google hält die SPF-Konfiguration der Absenderdomäne für ungültig oder verdächtig.

dsn-vendor-greylist = Das Gateway hat diese Zustellung mithilfe von Greylisting vorübergehend verzögert. Ein korrekt konfigurierter Sendeserver versucht es normalerweise später noch einmal.

dsn-vendor-headers = Der empfangende Dienst hat ungültige oder fehlende Nachrichtenkopfzeilen abgelehnt. Die sendende Anwendung muss korrekt formatierte E-Mails generieren.

dsn-vendor-helo = Der sendende Server hat eine inakzeptable HELO/EHLO-Identität verwendet. Seine Begrüßung muss den sendenden Server korrekt identifizieren.

dsn-vendor-ip-block = Der empfangende Dienst lehnte den sendenden IP aufgrund seiner Reputation oder eines Sperrlisteneintrags ab.

dsn-vendor-mimecast-block = Mimecast hat den Absender oder die Domäne aufgrund einer Blockierungs-Absender- oder Verwaltungsrichtlinie abgelehnt.

dsn-vendor-mimecast-content = Eine Inhaltsprüfungsrichtlinie von Mimecast hat die Nachricht abgelehnt.

dsn-vendor-mimecast-spoof = Die Anti-Spoofing-Richtlinie von Mimecast lehnte die Absenderidentität ab.

dsn-vendor-mta-sts-host = Der Ziel-Mailserver ist gemäß der MTA-STS-Richtlinie der Domäne nicht zulässig.

dsn-vendor-proofpoint-block = Proofpoint hat den sendenden IP oder die Domäne blockiert, nachdem Spam oder böswillige Aktivitäten erkannt wurden.

dsn-vendor-proofpoint-throttle = Proofpoint hat diesen Versand IP vorübergehend zurückgestellt, da sein Sendemuster als ungewöhnlich angesehen wurde.

dsn-vendor-ptr = Dem sendenden IP fehlt die akzeptable Umkehrung DNS, oder sein PTR-Name lässt sich nicht wieder in denselben IP auflösen.

dsn-vendor-recipient-ip-block = Die Empfängerorganisation blockiert explizit die sendende IP-Adresse. Sein Administrator kontrolliert diesen lokalen Block.

dsn-vendor-relay = Der Mailserver leitet diese Nachricht nicht an das Ziel weiter. Überprüfen Sie die SMTP-Authentifizierung, vertrauenswürdige Sende-IPs, Connectors und akzeptierte Domänen.

dsn-vendor-routing-loop = Die Nachricht ist in eine Weiterleitungs- oder Routing-Schleife geraten und hat die zulässige Anzahl an Hops überschritten.

dsn-vendor-sending-limit = Das sendende Konto hat ein Nachrichten- oder Empfängerkontingent überschritten. Durch wiederholtes sofortiges erneutes Versenden wird das Limit nicht aufgehoben.

dsn-vendor-spf = Der Empfänger hat die SPF-Authentifizierung für die sendende Domäne IP und den Umschlagsender nicht akzeptiert.

dsn-vendor-starttls = Die erforderliche verschlüsselte SMTP-Verbindung konnte nicht hergestellt werden oder wurde vom nächsten Server nicht unterstützt.

dsn-vendor-yahoo-deferred = Yahoo hat diese Nachricht vorübergehend zurückgestellt, häufig um die Menge, den Ruf oder Beschwerden von Empfängern zu prüfen. Überprüfen Sie die Diagnose auf die angegebene Ursache.

dsn-vendor-yahoo-policy = Yahoo lehnte die Nachricht gemäß seinen E-Mail-Richtlinien ab. Authentifizierung, DMARC-Ausrichtung oder Nachrichteninhalt können beteiligt sein; Die Diagnose gibt den konkreten Grund an.
