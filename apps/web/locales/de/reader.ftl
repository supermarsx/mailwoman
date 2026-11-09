# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Nachrichtenanzeige
reader-settings-help = Legen Sie fest, wie Nachrichten angezeigt werden. Standardwerte der Administration gelten, bis Sie sie überschreiben. Der Nachrichteninhalt bleibt von der Anwendung isoliert.
reader-colorMode = Nachrichtenfarben
reader-colorMode-help = „Automatisch“ passt die Nachrichtenfarben in dunklen Designs an. „Original“ behält die Farben des Absenders bei; „Anpassen“ verwendet das aktive Design; „Invertieren“ kehrt die Farben um und lässt die Bildfarben unverändert.
reader-fontSize = Schriftgröße beim Lesen (px)
reader-fontSize-help = Grundschriftgröße für den Nachrichtentext, von 12 bis 24 Pixel. Die Formatierung des Absenders kann weiterhin Überschriftengrößen festlegen.
reader-lineHeight = Zeilenhöhe beim Lesen
reader-lineHeight-help = Abstand zwischen den Zeilen des Nachrichtentexts, das 1,2- bis 2,2-Fache der Schriftgröße.
reader-contentWidth = Lesebreite
reader-contentWidth-help = Lange Zeilen für komfortables Lesen begrenzen oder den gesamten Nachrichtenbereich nutzen.
reader-loading = Darstellung beim Laden
reader-loading-help = Wählen Sie ruhige Platzhalterzeilen, eine Textanzeige oder keine visuelle Ladeanzeige. Screenreader erhalten weiterhin den Ladestatus.
reader-loadingDelayMs = Verzögerung der Ladeanzeige (ms)
reader-loadingDelayMs-help = Verhindert, dass bei schnellen Anfragen eine Anzeige aufblitzt. Verzögert wird nur die Anzeige, nie die Nachricht selbst.
reader-option-auto = Automatisch mit Design
reader-option-original = Originalfarben der Nachricht
reader-option-adaptive = An Design anpassen
reader-option-invert = Nachrichtenfarben invertieren
reader-option-comfortable = Komfortabel
reader-option-wide = Breit
reader-option-full = Gesamter Bereich
reader-option-skeleton = Platzhalterzeilen
reader-option-indicator = Textanzeige
reader-option-none = Keine visuelle Anzeige
reader-inherit = Standard der Administration verwenden
reader-actionStyle = Aktionsleiste der Nachricht
reader-actionStyle-help = Kompakte Symbole lassen mehr Platz für die Nachricht. Jede Aktion behält ihren Tastaturnamen und einen erklärenden Tooltip. Wählen Sie sichtbare Beschriftungen, wenn Sie Text neben den Symbolen bevorzugen.
reader-option-icons = Kompakte Symbole
reader-option-labels = Symbole und Beschriftungen
reader-zoom-group = Nachrichtenzoom
reader-zoom-in = Vergrößern
reader-zoom-in-help = Die Nachricht größer darstellen. Nur die Nachricht ändert ihre Größe.
reader-zoom-out = Verkleinern
reader-zoom-out-help = Die Nachricht kleiner darstellen. Nur die Nachricht ändert ihre Größe.
reader-zoom-level = { $level } %
reader-zoom-reset-label = Nachrichtenzoom { $level } %. Auf 100 % zurücksetzen
reader-zoom-reset-help = Die Nachricht auf 100 % zurücksetzen
reader-zoom-reset-menu = Zoom zurücksetzen ({ $level } %)
reader-zoom = Nachrichtenzoom (%)
reader-zoom-help = Die Größe, mit der Nachrichten geöffnet werden, von 50 % bis 200 %. Nur der Nachrichtentext wird skaliert, nie die App, und beim Drucken gilt immer 100 %. Die Zoom-Schaltflächen neben den Nachrichtenaktionen ändern den Wert ebenfalls.
reader-attachments-summary = { $count ->
    [one] 1 Anhang
   *[other] { $count } Anhänge
    } · { $size }
reader-attachments-show = Anhänge anzeigen
reader-attachments-hide = Anhänge ausblenden
reader-attachmentsDensity = Dichte der Anhänge
reader-attachmentsDensity-help = „Kompakt“ zeigt kleine Dateischaltflächen; „Komfortabel“ zeigt im aufgeklappten Zustand größere Vorschauen.
reader-option-compact = Kompakt
reader-attachmentsCollapsed = Anhänge standardmäßig einklappen
reader-attachmentsCollapsed-help = Jede Nachricht beginnt mit einer kompakten Anhangsübersicht. Sie können die jeweilige Nachricht aufklappen, ohne diesen Standard zu ändern.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Text und Farben
reader-card-text-help = Größe, Abstände, Breite und Farben des Nachrichtentexts.
reader-card-actions = Aktionen und Anhänge
reader-card-actions-help = Die Aktionsleiste der Nachricht und das Verhalten von Anhängen beim Öffnen.
reader-card-loading = Laden
reader-card-loading-help = Was Sie sehen, während eine Nachricht geöffnet wird.
settings-mail-card-list = Nachrichtenliste
settings-mail-card-list-help = Zeilenaktionen und was die Liste tut, wenn Sie eine Nachricht bearbeiten.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Laden Sie { $name } herunter

reader-attachments-download-all = Alles herunterladen

reader-attachments-download-all-blocked = Die Anhangsrichtlinie blockiert hier jede Datei

reader-attachments-download-all-busy = Vorbereiten der ZIP-Datei

reader-attachments-download-all-label = Laden Sie alle Anhänge als ZIP-Datei herunter

reader-attachments-download-all-none = Es sind keine Anhänge zum Herunterladen vorhanden

reader-attachments-download-all-warned = { $count ->
        [one] Eine Datei, die einer Bestätigung bedarf, wurde weggelassen. Laden Sie es einzeln herunter.
       *[other] { $count } Dateien, die eine Bestätigung benötigen, wurden ausgelassen. Laden Sie sie einzeln herunter.
    }

reader-attachments-download-blocked = { $name } kann nicht heruntergeladen werden: { $reason }

reader-attachments-download-label = { $name } ({ $size }) herunterladen

reader-attachments-download-off = Downloads sind deaktiviert

reader-attachments-inline-group = { $count ->
        [one] { $count } Inline-Bild
       *[other] { $count } Inline-Bilder
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } Inline-Bild
       *[other] { $count } Inline-Bilder
    } · { $size }

reader-attachmentsBarClickToToggle = Klicken Sie auf die Leiste, um sie zu erweitern oder zu reduzieren

reader-attachmentsBarClickToToggle-help = Durch Klicken auf eine beliebige Stelle in der Anhangsleiste wird die Funktion umgeschaltet. Im ausgeschalteten Zustand funktioniert nur die Pfeiltaste.

reader-attachmentsBarDownloadAll = Klicken Sie auf die Schaltfläche „Alle herunterladen“.

reader-attachmentsBarDownloadAll-help = Zeigt in der Leiste eine Schaltfläche an, die die Anhänge als eine ZIP-Datei herunterlädt.

reader-attachmentsBarShowDownloadButtons = Download-Schaltflächen in Anhängen

reader-attachmentsBarShowDownloadButtons-help = Zeigen Sie die Download-Schaltfläche jedes Anhangs an, wenn Sie den Mauszeiger darüber bewegen oder den Fokus fokussieren, immer oder nie.

reader-attachmentsBarZipInlineImages = Fügen Sie Inline-Bilder in das ZIP ein.

reader-attachmentsBarZipInlineImages-help = Bilder, die im Nachrichtentext erscheinen, werden von „Alle herunterladen“ ausgeschlossen, es sei denn, diese Option ist aktiviert.

reader-card-attachments-bar = Anhängeleiste

reader-card-attachments-bar-help = Wie die Leiste über den Anhängen einer Nachricht reagiert und was sie bietet. Als gesperrt markierte Leitungen werden von Ihrem Administrator eingerichtet.

reader-option-always = Immer

reader-option-hover = Beim Schweben oder Fokussieren

reader-option-never = Niemals
