# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = E-Mail-Adresse
auth-password = Passwort
auth-sign-in = Anmelden
auth-signing-in = Anmeldung läuft…
auth-invalid-credentials = Ungültige Anmeldedaten
auth-refused-note = Wenn E-Mail-Adresse und Passwort korrekt sind, wurde das Konto möglicherweise von einem Administrator deaktiviert. Ein deaktiviertes Konto kann sich erst anmelden, wenn ein Administrator es wieder aktiviert.
auth-unreachable = Der Server ist nicht erreichbar
auth-address-not-configured = Diese Adresse ist auf diesem Server nicht eingerichtet. Wenden Sie sich an Ihren Administrator.

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = Single Sign-on
auth-sso-divider = oder weiter mit
auth-sso-button = Mit { $name } anmelden
auth-sso-error = Die einmalige Anmeldung (SSO) wurde nicht abgeschlossen. Bitte versuchen Sie es erneut oder melden Sie sich mit Ihrem Passwort an.

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = Anwendung autorisieren
auth-consent-title = Zugriff autorisieren
auth-consent-loading = Anfrage wird geladen…
auth-consent-intro = möchte auf Ihr Konto zugreifen.
auth-consent-approved = Vom Administrator genehmigter Client
auth-consent-unapproved = Unbekannter Client — nicht vom Administrator genehmigt
auth-consent-requesting = Angefordert wird
auth-consent-redirects-to = Leitet weiter an
auth-consent-for-resource = Für Ressource
auth-consent-deny = Ablehnen
auth-consent-allow = Zulassen
auth-consent-error = Ihre Entscheidung konnte nicht gespeichert werden

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = Zu viele Anmeldeversuche. Bitte warten Sie, bevor Sie es erneut versuchen.
auth-retry-countdown = Erneut versuchen in { $seconds } s
auth-upstream-protocol = Anmeldung abgebrochen: Der Mailserver hat eine Antwort gesendet, die dieser Server nicht lesen konnte. Versuchen Sie es später erneut oder informieren Sie Ihre Administration.
auth-account-disabled = Dieses Konto wurde von einer Administratorin oder einem Administrator deaktiviert. Eine Anmeldung ist erst nach der erneuten Aktivierung möglich.
auth-signin-not-allowed = Diese Adresse darf sich auf diesem Server nicht anmelden. Wenden Sie sich an Ihre Administration.
auth-signin-not-provisioned = Für diese Adresse existiert noch kein Konto, und neue Konten werden nicht automatisch angelegt. Bitten Sie Ihre Administration, Sie hinzuzufügen.
auth-reference = Referenz: { $id }
auth-domain-hint = Für { $domain } können Sie sich nur mit Ihrem Benutzernamen anmelden.
auth-passkey-sign-in = Mit Passkey anmelden
auth-passkey-failed = Die Anmeldung per Passkey ist fehlgeschlagen. Versuchen Sie es erneut oder verwenden Sie Ihr Passwort.
auth-remembered = Frühere Anmeldungen
auth-remembered-help = Adressen, die auf diesem Gerät erfolgreich verwendet wurden. Beim Auswählen wird nur die Adresse eingetragen.
auth-forget-address = { $address } auf diesem Gerät vergessen
auth-login-timeout = Zeitüberschreitung bei der Anmeldung. Es wurde kein automatischer Wiederholungsversuch unternommen. Prüfen Sie die Verbindung und versuchen Sie es erneut.
auth-verification-title = Anmeldeverifizierung
auth-verification-loading = Anmeldeverifizierung wird geladen …
auth-verification-error = Die Verifizierung konnte nicht abgeschlossen werden. Wiederholen Sie die Verifizierung, bevor Sie sich anmelden.
auth-verification-required = Schließen Sie eine neue Anmeldeverifizierung ab, bevor Sie es erneut versuchen.
auth-identity-title = Identität teilen
auth-identity-sign-in = Melden Sie sich erneut an, um diese Identitätsanfrage fortzusetzen.
auth-identity-sign-out = Von Mailwoman abmelden
auth-identity-expired = Diese Identitätsanfrage ist abgelaufen oder wurde geändert. Kehren Sie zur anfragenden Anwendung zurück und beginnen Sie erneut.
auth-identity-retry = Erneut versuchen
auth-identity-disclosure = Diese Anwendung erhält die unten stehenden Identitätsinformationen. Diese Berechtigung gewährt keinen Zugriff auf Ihre E-Mails.
auth-identity-scope-openid = Eine stabile Identitätskennung
auth-identity-scope-profile = Verfügbare Profilinformationen
auth-identity-scope-email = Ihre angemeldete E-Mail-Adresse, ohne Zusicherung einer unabhängigen Überprüfung
auth-identity-scope-offline_access = Identitätszugriff erneuern, solange diese Mailwoman-Sitzung gültig ist, bis zu 15 Tage ab der Anmeldung
auth-identity-deny = Abbrechen
auth-identity-approve = Identitätszugriff erlauben
auth-identity-grants = Anwendungen, die Ihre Identität verwenden
auth-identity-grants-help = Identitätszugriff für eine Anwendung widerrufen. Separate Berechtigungen für die Mail-API bleiben davon unberührt.
auth-identity-no-grants = Derzeit hat keine Anwendung eine Identitätseinwilligung.
auth-identity-revoke = Identitätszugriff widerrufen
auth-maintenance = Webmail ist wegen Wartungsarbeiten vorübergehend pausiert. Bitte versuchen Sie es später erneut.
