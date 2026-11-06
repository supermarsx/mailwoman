# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Annulla invio
send-delay-help = Dopo aver premuto Invia, il messaggio resta in Posta in uscita per questo tempo. Fino ad allora può richiamarlo e continuare a modificarlo. Il server lo trattiene, quindi chiudere il browser non lo invia in anticipo né lo perde.
send-delay-label = Trattieni i messaggi inviati per
send-delay-off = Disattivato (invio immediato)
send-delay-seconds = { $n } secondi
send-delay-minutes = { $n ->
    [one] 1 minuto
   *[other] { $n } minuti
}
send-delay-custom = Personalizzato…
send-delay-custom-label = Ritardo personalizzato in secondi
send-delay-custom-help = Secondi interi, da 0 a { $max }.
send-delay-capped = L'amministratore limita questo valore a { $max }.
send-delay-disabled = Il richiamo dei messaggi inviati è disattivato dall'amministratore. I messaggi vengono inviati subito.
send-delay-invalid = Inserisci un numero intero di secondi da 0 a { $max }.
send-later-button = Invia più tardi
send-later-menu = Opzioni di invio posticipato
send-later-preset-hour = Tra 1 ora
send-later-preset-today = Più tardi oggi, 17:00
send-later-preset-tomorrow = Domani mattina, 09:00
send-later-preset-monday = Lunedì mattina, 09:00
send-later-custom = Scegli data e ora…
send-later-custom-label = Invia alle
send-later-zone = Fuso orario: { $zone }
send-later-schedule = Pianifica invio
send-later-cancel = Annulla
send-later-err-invalid = Inserisci una data e un'ora valide.
send-later-err-past = Scegli un orario futuro.
send-later-err-horizon = È troppo lontano. I messaggi possono essere pianificati fino a { $days } giorni in anticipo.
send-later-err-off = L'invio pianificato è disattivato dall'amministratore.
send-queued = Messaggio in coda — verrà inviato tra
send-queued-undo = Annulla
send-queued-now = Invia ora
send-queued-recalled = Messaggio richiamato. È tornato nelle bozze.
send-queued-too-late = Troppo tardi per richiamarlo: il messaggio è già in invio.
send-queued-sent-now = Invio in corso.
send-queued-scheduled = Pianificato per { $when } ({ $zone }).
outbox-scheduled-badge = Pianificato
outbox-sends-in = Verrà inviato tra { $time }
outbox-sends-at = Verrà inviato { $when }
outbox-recall-edit = Richiama e modifica
outbox-recall-too-late = Già in invio; non può più essere richiamato.
outbox-failed-drafts = Non è stato possibile inviarlo ed è stato conservato nelle bozze.
outbox-queued-count = { $count ->
    [one] 1 messaggio in attesa di invio
   *[other] { $count } messaggi in attesa di invio
}
