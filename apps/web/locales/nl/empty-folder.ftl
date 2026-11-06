# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Map legen…
empty-folder-title = “{ $name }” legen?
empty-folder-body-trash = { $count ->
    [one] Het bericht in { $name } wordt naar de prullenbak verplaatst, waar u het kunt herstellen.
   *[other] Alle { $count } berichten in { $name } worden naar de prullenbak verplaatst, waar u ze kunt herstellen.
 }
empty-folder-body-destroy-trash = { $count ->
    [one] Het bericht in de prullenbak wordt definitief verwijderd. Dit kan niet ongedaan worden gemaakt.
   *[other] Alle { $count } berichten in de prullenbak worden definitief verwijderd. Dit kan niet ongedaan worden gemaakt.
 }
empty-folder-body-destroy-junk = { $count ->
    [one] Het bericht in { $name } wordt definitief verwijderd. Dit kan niet ongedaan worden gemaakt.
   *[other] Alle { $count } berichten in { $name } worden definitief verwijderd. Dit kan niet ongedaan worden gemaakt.
 }
empty-folder-body-destroy-no-trash = Dit account heeft geen prullenbak, dus { $count ->
    [one] het bericht in { $name } wordt definitief verwijderd.
   *[other] alle { $count } berichten in { $name } worden definitief verwijderd.
 }
empty-folder-confirm-trash = Naar prullenbak verplaatsen
empty-folder-confirm-destroy = Definitief verwijderen
empty-folder-cancel = Annuleren
empty-folder-already-empty = “{ $name }” is al leeg.
empty-folder-prepare-failed = “{ $name }” kon niet worden geteld, dus er is niets gewijzigd. Controleer uw verbinding en probeer het opnieuw.
empty-folder-syncing = “{ $name }” wordt nog gesynchroniseerd en kan daarom nog niet worden geteld. Er is niets gewijzigd; probeer het over een moment opnieuw.
empty-folder-running = Er wordt al een map geleegd. Stop die via de melding voordat u een andere start.
empty-folder-progress-trash = Berichten uit { $name } naar de prullenbak verplaatsen: { $done } van { $total }.
empty-folder-progress-destroy = Berichten uit { $name } verwijderen: { $done } van { $total }.
empty-folder-stop = Stoppen
empty-folder-stopping = Stoppen na de huidige batch. Tot nu toe { $done } van { $total } gedaan.
empty-folder-done-trash = { $done ->
    [one] 1 bericht uit { $name } naar de prullenbak verplaatst.
   *[other] { $done } berichten uit { $name } naar de prullenbak verplaatst.
 }
empty-folder-done-destroy = { $done ->
    [one] 1 bericht uit { $name } definitief verwijderd.
   *[other] { $done } berichten uit { $name } definitief verwijderd.
 }
empty-folder-stopped = Gestopt. { $done } van { $total } berichten gedaan; { $remaining } nog in { $name }.
empty-folder-refused = Gestopt. { $done } van { $total } berichten gedaan; de mailserver weigerde er { $refused }. { $remaining } nog in { $name }.
empty-folder-failed = Het legen van { $name } is na { $done } berichten met een fout gestopt. De laatste batch is mogelijk wel of niet voltooid; controleer de map voordat u het opnieuw probeert.
