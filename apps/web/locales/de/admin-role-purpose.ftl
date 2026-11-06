# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Vollständige Administration aller Funktionen, einschließlich Administratoren, Rollen, Sicherheit, E-Mail-Routing und Dienstkonfiguration. Reservieren Sie diese Rolle für vertrauenswürdige Betreiber, die für die gesamte Bereitstellung verantwortlich sind.
admin-role-purpose-domain_admin = Alltägliche Benutzer- und Domainverwaltung: Benutzer anlegen, aktualisieren, deaktivieren oder löschen; Kontingente, Benutzerfunktionen, Anmelderegeln der Domain, Einstellungen und Branding verwalten; Sitzungen widerrufen und Sperren verwalten. Dazu gehören auch E-Mail-Diagnose sowie Nutzungs- und Datenexporte. Sie umfasst weder die Verwaltung von Administratorrollen noch Änderungen an Upstream-Servern.
admin-role-purpose-helpdesk = Kontosupport und Störungsklärung: Benutzer, Audit-Aktivität und Nutzung einsehen; Benutzerkennzeichen ändern, Konten deaktivieren und Sitzungen widerrufen; Sperren einsehen und Anmeldesimulationen ausführen. Sie umfasst weder das Anlegen oder Löschen von Benutzern noch Kontingentänderungen oder allgemeine Schreibzugriffe auf Einstellungen.
admin-role-purpose-auditor = Prüfung und Berichte: gewährt jede Berechtigung, die auf .read endet, sowie Audit- und Nutzungsexport. Sie umfasst keine Konfigurationsänderungen oder Kontoänderungen. Exportierte Berichte können sensible Verwaltungsinformationen enthalten.
admin-role-purpose-security_officer = Sicherheitsbetrieb: Sicherheits- und MFA-Richtlinien, Anmelderichtlinie und Sperren verwalten; Benutzer deaktivieren, Sitzungen oder API-Schlüssel widerrufen; Audit-Aktivität einsehen und exportieren; Authentifizierungsprotokolle lesen und Diagnosen ausführen. Die SSO-Konfiguration ist lesbar, aber diese Rolle umfasst weder die Bearbeitung von SSO noch die Verwaltung von Administratorrollen.
admin-role-purpose-custom = Eine benutzerdefinierte Rolle mit { $count } ausgewählten Berechtigungen. Ihre Befugnis ergibt sich aus der genauen Berechtigungsliste unten, nicht aus ihrem Namen. Prüfen Sie jede Aktion, bevor Sie sie zuweisen.
admin-role-purpose-scope = Der Domainbereich wird festgelegt, wenn diese Rolle einem Administrator zugewiesen wird. Erlauben-/Verweigern-Überschreibungen auf Kontoebene können den tatsächlichen Zugriff ändern. Eine Beschreibung gewährt keine zusätzlichen Berechtigungen.
admin-role-purpose-editor = Wählen Sie die Aufgaben aus, die dieser Administrator ausführen muss, und wählen Sie dann die entsprechenden Berechtigungen. Lesezugriff, Datenexport und das Ändern von Datensätzen sind getrennte Aktionen. Prüfen Sie die genauen Bezeichner der Berechtigungen vor dem Speichern.
admin-role-purpose-selection = Diese Beschreibungen erläutern die ausgewählten Aktionen. Sie erweitern nicht die Berechtigungen, die an den Server gesendet werden.
admin-role-purpose-read = Informationen in diesem Bereich einsehen.
admin-role-purpose-export = Informationen aus diesem Bereich exportieren.
admin-role-purpose-revoke = Die von dieser Berechtigung genannten Sitzungen oder Zugangsdaten widerrufen.
admin-role-purpose-delete = Von dieser Berechtigung abgedeckte Datensätze löschen.
admin-role-purpose-disable = Die von dieser Berechtigung abgedeckten Konten deaktivieren.
admin-role-purpose-create = Von dieser Berechtigung abgedeckte Datensätze anlegen.
admin-role-purpose-specific = Die von dieser Berechtigung genannte Aktion ausführen; prüfen Sie ihren Bezeichner, bevor Sie sie gewähren.
admin-role-purpose-description = Zweck und Zuständigkeiten
admin-role-purpose-description-help = Erläutern Sie, wer diese benutzerdefinierte Rolle erhalten soll und welche Aufgaben die Person übernimmt (bis zu 1024 Zeichen). Dieser Text ändert die Berechtigungen nicht.
