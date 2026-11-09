# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: it · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Persone e immagini dei contatti
avatars-mode = Immagini di mittenti e contatti
avatars-off = Nascondi immagini
avatars-initials = Genera iniziali in locale
avatars-contacts = Usa le immagini dei contatti locali, con le iniziali come alternativa
avatars-help = Le iniziali vengono generate su questo dispositivo. Le immagini dei contatti usano solo foto PNG o JPEG incorporate già salvate nelle sue rubriche, fino a 512 × 512 pixel e 128 KiB. Gli URL di immagini remote e i servizi di avatar esterni non vengono mai richiesti. Questa preferenza è salvata con il suo account e può essere fissata dall'amministratore.
avatars-default = Ripristina preferenza ereditata
avatars-sender-icons = Mostra le icone incluse dei domini dei mittenti
avatars-sender-icons-help = I domini riconosciuti di social, sviluppo, IA, shopping e pubbliche amministrazioni usano icone incluse in questa app. Le sue immagini dei contatti hanno la priorità. I mittenti sconosciuti usano iniziali generate in locale con 192 colori. Le icone sono indizi sul dominio, non la prova che un messaggio sia autentico. Non viene contattato alcun servizio di loghi esterno.
avatars-icons-default = Ripristina preferenza ereditata per le icone
avatars-brand-avatars = Mostra i loghi dei marchi accanto ai mittenti verificati
avatars-brand-avatars-help = Nel lettore, un'azienda riconosciuta mostra il proprio logo accanto al mittente solo se il messaggio ha superato DKIM e un controllo DMARC allineato per il dominio di quell'azienda. In caso contrario viene usato l'avatar normale. I loghi sono inclusi in questa app; non viene richiesto nulla al mittente.
avatars-brand-avatars-default = Ripristina preferenza ereditata per i loghi dei marchi
avatars-domain-icon = Icona del dominio { $name } · l'identità del mittente non è verificata da questa icona
avatars-favorites-first = Mostra prima i contatti preferiti nei suggerimenti dei destinatari
avatars-favorites-first-help = I contatti con la stella compaiono prima delle altre corrispondenze quando digiti un destinatario. Cambia solo l'ordine; nessun suggerimento viene nascosto.
avatars-favorites-first-default = Ripristina preferenza ereditata per i preferiti

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Autenticazione necessaria per i loghi del marchio

avatars-require-auth-always = Sempre (richiede un controllo superato)

avatars-require-auth-default = Ripristina la preferenza di autenticazione ereditata

avatars-require-auth-help = Scegli quando il logo di un'azienda riconosciuta può sostituire le iniziali accanto al mittente. Mai: mostralo ogni volta che il dominio del mittente corrisponde. Solo se un controllo fallisce (impostazione predefinita): mostralo a meno che il messaggio non sia fallito DKIM o DMARC. Sempre: mostralo solo quando il messaggio ha superato DKIM e un controllo DMARC allineato per il dominio di quella società, quindi la posta non ancora controllata mostra le iniziali. La corrispondenza del dominio include sottodomini come mail.example.com. Questa preferenza viene salvata con il tuo account e può essere mantenuta dal tuo amministratore.

avatars-require-auth-known-failure = Solo se un controllo fallisce

avatars-require-auth-never = Mai (corrisponde solo al dominio)

avatars-unverified-sender = Mittente non verificato · non viene mostrata alcuna icona del marchio perché questo messaggio non ha superato il controllo di autenticazione
