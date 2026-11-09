# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Personen und Kontaktbilder
avatars-mode = Absender- und Kontaktbilder
avatars-off = Bilder ausblenden
avatars-initials = Initialen lokal erzeugen
avatars-contacts = Lokale Kontaktbilder verwenden, mit Initialen als Ersatz
avatars-help = Initialen werden auf diesem Gerät erzeugt. Kontaktbilder verwenden nur eingebettete PNG- oder JPEG-Fotos, die bereits in Ihren eigenen Adressbüchern gespeichert sind, bis 512 × 512 Pixel und 128 KiB. Bild-URLs aus dem Internet und externe Avatar-Dienste werden nie abgefragt. Diese Einstellung wird mit Ihrem Konto gespeichert und kann von Ihrer Administration festgelegt werden.
avatars-default = Geerbte Einstellung wiederherstellen
avatars-sender-icons = Mitgelieferte Absender-Domain-Symbole anzeigen
avatars-sender-icons-help = Erkannte Domains aus Social Media, Entwicklung, KI, Handel und Behörden verwenden Symbole, die in dieser App enthalten sind. Ihre eigenen Kontaktbilder haben Vorrang. Unbekannte Absender erhalten lokal erzeugte Initialen in 192 Farben. Symbole sind Domain-Hinweise, kein Beweis für die Echtheit einer Nachricht. Es wird kein externer Logo-Dienst kontaktiert.
avatars-icons-default = Geerbte Symbol-Einstellung wiederherstellen
avatars-brand-avatars = Markenlogos neben verifizierten Absendern anzeigen
avatars-brand-avatars-help = In der Leseansicht zeigt ein erkanntes Unternehmen sein Logo neben dem Absender nur dann, wenn die Nachricht DKIM und eine ausgerichtete DMARC-Prüfung für die eigene Domain dieses Unternehmens bestanden hat. Andernfalls wird der normale Avatar verwendet. Die Logos sind in dieser App enthalten; vom Absender wird nichts abgerufen.
avatars-brand-avatars-default = Geerbte Einstellung für Markenlogos wiederherstellen
avatars-domain-icon = Domain-Symbol für { $name } · Die Absenderidentität wird durch dieses Symbol nicht verifiziert
avatars-favorites-first = Favoriten in Empfängervorschlägen zuerst anzeigen
avatars-favorites-first-help = Kontakte mit Stern stehen bei der Eingabe eines Empfängers vor den übrigen Treffern. Nur die Reihenfolge ändert sich; kein Vorschlag wird ausgeblendet.
avatars-favorites-first-default = Geerbte Favoriten-Einstellung wiederherstellen

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Authentifizierung für Markenlogos erforderlich

avatars-require-auth-always = Immer (erfordert einen bestandenen Scheck)

avatars-require-auth-default = Geerbte Authentifizierungseinstellung wiederherstellen

avatars-require-auth-help = Wählen Sie aus, wann das Logo eines anerkannten Unternehmens die Initialen neben einem Absender ersetzen darf. Niemals: Zeigen Sie es immer dann an, wenn die Domäne des Absenders übereinstimmt. Nur wenn eine Prüfung fehlgeschlagen ist (Standard): Zeigt sie an, es sei denn, die Meldung DKIM oder DMARC ist fehlgeschlagen. Immer: nur anzeigen, wenn die Nachricht DKIM und eine ausgerichtete DMARC-Prüfung für die eigene Domäne des Unternehmens bestanden hat, sodass noch nicht überprüfte E-Mails Initialen anzeigen. Die Domänenübereinstimmung umfasst Subdomänen wie mail.example.com. Diese Einstellung wird in Ihrem Konto gespeichert und kann von Ihrem Administrator verwaltet werden.

avatars-require-auth-known-failure = Nur wenn eine Prüfung fehlgeschlagen ist

avatars-require-auth-never = Nie (nur Übereinstimmung mit der Domäne)

avatars-unverified-sender = Nicht bestätigter Absender · Es wird kein Markensymbol angezeigt, da diese Nachricht die Authentifizierungsprüfung nicht bestanden hat
