# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contactos
contacts-search = Buscar contactos
contacts-new = Nuevo contacto
contacts-find-duplicates = Buscar duplicados
contacts-loading = Cargando…
contacts-none = No hay contactos.
contacts-list = Lista de contactos
contacts-favorite = Marcar { $name } como favorito
contacts-export-vcard = Exportar vCard
contacts-export-csv = Exportar CSV
contacts-select-hint = Seleccione un contacto para ver su ficha.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Libretas de direcciones y grupos
contacts-address-books = Libretas de direcciones
contacts-all = Todos los contactos
contacts-favorites = Favoritos
contacts-groups = Grupos
contacts-new-group = Nuevo grupo
contacts-new-group-name = Nombre del nuevo grupo
contacts-group-name-placeholder = Nombre del grupo
contacts-create = Crear
contacts-data = Datos
contacts-import = Importar…

# -- Business card -----------------------------------------------------------
contacts-card = Contacto { $name }
contacts-favorited = En favoritos
contacts-favorite-action = Favorito
contacts-email = Correo
contacts-phone = Teléfono
contacts-dates = Fechas
contacts-groups-label = Grupos
contacts-notes = Notas
contacts-security-key = Clave de seguridad
contacts-key-on-file = Hay una clave o certificado guardado (solo se muestra hasta que llegue PGP/S-MIME).
contacts-no-key = No hay ninguna clave guardada.
contacts-directory-security = Seguridad del directorio
contacts-member-of = Miembro de
contacts-group-membership = Pertenencia a { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Nuevo contacto
contacts-edit-contact = Editar contacto
contacts-full-name = Nombre completo
contacts-given = Nombre
contacts-given-placeholder = Nombre
contacts-surname = Apellidos
contacts-surname-placeholder = Apellidos
contacts-organization = Organización
contacts-job-title = Cargo
contacts-job-title-placeholder = Cargo
contacts-email-n = Correo { $n }
contacts-email-label = Etiqueta del correo { $n }
contacts-email-label-placeholder = trabajo
contacts-remove-email = Quitar correo { $n }
contacts-add-email = Añadir correo
contacts-phone-n = Teléfono { $n }
contacts-remove-phone = Quitar teléfono { $n }
contacts-add-phone = Añadir teléfono
contacts-security-key-label = Clave de seguridad (marcador opaco)
contacts-security-key-placeholder = Clave PGP / certificado S-MIME (solo se muestra)
contacts-favorite-word = Favorito

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importar contactos
contacts-imported = Se importaron { $count ->
    [one] { $count } contacto
    [many] { $count } de contactos
   *[other] { $count } contactos
}.
contacts-import-hint = Pegue un vCard (.vcf) o CSV, o elija un archivo. El formato se detecta automáticamente.
contacts-import-file = Importar archivo
contacts-paste = Pegar vCard o CSV
contacts-preview = Vista previa
contacts-map-columns = Asignar columnas
contacts-col = Columna
contacts-maps-to = Se asigna a
contacts-sample = Muestra
contacts-map-column = Asignar columna { $header }
contacts-preview-count = Vista previa ({ $count })
contacts-import-preview = Vista previa de la importación
contacts-no-name = (sin nombre)
contacts-no-email = sin correo
contacts-import-n = Importar { $count ->
    [one] { $count } contacto
    [many] { $count } de contactos
   *[other] { $count } contactos
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Combinar duplicados
contacts-merge-review-hint = Revise la ficha combinada. Las demás fichas se marcan como eliminadas (reversible).
contacts-merged-preview = Vista previa combinada
contacts-emails = Correos
contacts-phones = Teléfonos
contacts-merge-confirm = Combinar contactos
contacts-no-duplicates = No se encontraron duplicados.
contacts-possible-duplicates = { $count } posibles duplicados
contacts-review-merge = Revisar combinación

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Datos del contacto
contacts-address-recipients = Mostrar destinatarios
contacts-address-save = Guardar contacto
contacts-address-compose = Enviar correo
contacts-address-unsaved = Esta dirección no está guardada en sus contactos.
contacts-address-book = Libreta de direcciones
contacts-address-lookup-failed = No se pudo cargar la información del contacto. Inténtelo de nuevo.
contacts-address-save-failed = No se pudo guardar el contacto. Inténtelo de nuevo.
contacts-address-saved = Contacto guardado.
contacts-address-loading = Cargando contacto…
contacts-editor-person = Nombre y trabajo
contacts-editor-reach = Correo y teléfono
contacts-editor-other = Notas y preferencias
contacts-editor-save-error = No se pudo guardar el contacto. Sus cambios se conservan. Inténtelo de nuevo.
contacts-group-editor-help = Cree un grupo en una libreta de direcciones y añada miembros desde sus fichas de contacto.
contacts-group-details = Detalles del grupo
contacts-group-save-error = No se pudo guardar el grupo. Sus cambios se conservan. Inténtelo de nuevo.
contacts-edit-group = Editar grupo de contactos
contacts-edit-named-group = Editar grupo { $name }
contacts-group-name = Nombre del grupo
contacts-group-edit-help = Cambie el nombre de este grupo. Su libreta de direcciones y sus miembros no cambian.
contacts-back-to-list = Volver a contactos
contacts-membership-title = Pertenencia al grupo
contacts-membership-help = Elija un grupo y añada o quite sus miembros. Los cambios se guardan al hacerlos.
contacts-membership-no-groups = Aún no hay grupos. Cree uno para empezar a añadir miembros.
contacts-membership-pick = Seleccione un grupo para ver sus miembros.
contacts-membership-of = Miembros de { $name }
contacts-membership-members = Miembros
contacts-membership-add = Escriba un nombre o una dirección para añadir un miembro
contacts-membership-add-group = Escriba el nombre de un grupo para añadir este contacto
contacts-membership-count = { $count ->
    [one] 1 miembro
   *[other] { $count } miembros
}
contacts-membership-failed = No se pudieron guardar algunos cambios de pertenencia. La lista muestra lo que está guardado.
contacts-membership-manage = Gestionar la pertenencia al grupo
contacts-import-source = Origen
contacts-import-book = Libreta de direcciones
contacts-import-review = Revisar
contacts-import-empty = Elija un archivo o pegue texto vCard o CSV y seleccione Vista previa. Los contactos encontrados se muestran aquí y puede excluir cualquiera de ellos.
contacts-merge-sets = Posibles duplicados
contacts-merge-set = Duplicados de { $name }
contacts-merge-keep = Ficha que se conserva
contacts-merge-done = Combinado con { $name }.
contacts-address-actions = Acciones del contacto
contacts-address-save-to = Guardar en contactos
contacts-address-goto = Ir al contacto
contacts-address-copy = Copiar información del contacto
contacts-address-copied = Información del contacto copiada.
contacts-address-copy-failed = No se pudo copiar la información del contacto.
contacts-share = Compartir contacto
contacts-share-email = Enviar contacto por correo
contacts-share-copy = Copiar como vCard
contacts-share-download = Descargar .vcf
contacts-share-native = Compartir…
contacts-share-copied = vCard copiada.
contacts-share-copy-failed = No se pudo copiar la vCard.
contacts-share-downloaded = vCard descargada.
contacts-share-failed = No se pudo compartir el contacto.
contacts-share-attach-failed = No se pudo adjuntar el archivo .vcf, así que la vCard está en el texto del mensaje.
contacts-address-saved-favorite = Contacto guardado y añadido a favoritos.
contacts-favorite-add = Añadir a favoritos
contacts-favorite-remove = Quitar de favoritos
contacts-favorite-save = Guardar y añadir a favoritos
contacts-favorite-added = { $name } añadido a favoritos.
contacts-favorite-removed = { $name } quitado de favoritos.
contacts-favorite-failed = No se pudieron actualizar los favoritos.
contacts-select-row = Seleccionar { $name }
contacts-selected-count = { $count } seleccionados
contacts-select-clear = Borrar selección
contacts-bulk-actions = Acciones para los contactos seleccionados
contacts-bulk-favorites-added = { $count } añadidos a favoritos.
contacts-bulk-favorites-removed = { $count } quitados de favoritos.
contacts-row-menu = Acciones para { $name }
contacts-favorites-empty = Aún no hay favoritos. Use la estrella de un contacto para añadir uno.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } ya está en tus contactos.

contacts-save-already-badge = Ya guardado

contacts-save-already-many = Todos los { $count } contactos ya están guardados.

contacts-save-automated = Parece un remitente automático (sin respuesta, notificaciones).

contacts-save-book = libreta de direcciones

contacts-save-book-readonly = { $name } (solo lectura)

contacts-save-cancel = Cancelar

contacts-save-choose = Elige contactos…

contacts-save-contacts = Guardar contactos de este mensaje

contacts-save-dialog-title = Guardar contactos de este mensaje

contacts-save-email = Correo electrónico

contacts-save-failed = No se pudo guardar el contacto. Intentar otra vez.

contacts-save-group = grupo

contacts-save-group-none = Ningún grupo

contacts-save-loading = Cargando direcciones…

contacts-save-name = Nombre

contacts-save-new = Guardar nuevos contactos

contacts-save-no-book = Ninguna libreta de direcciones puede aceptar nuevos contactos.

contacts-save-no-sender = Este mensaje no tiene dirección de remitente.

contacts-save-note = Guardado de un mensaje el { $date }

contacts-save-nothing = Nadie a quien salvar: todos ya están en tus contactos, eres tú o parecen automatizados.

contacts-save-open = Abierto

contacts-save-open-contact = Contacto abierto

contacts-save-open-link = Contacto abierto

contacts-save-own = Esta es una de sus propias direcciones.

contacts-save-partial = Guardado { $saved } de { $total } contactos. Los demás no pudieron salvarse.

contacts-save-pick = Guardar { $email }

contacts-save-recipients = Guardar todos los destinatarios

contacts-save-removed = Se eliminaron los contactos guardados.

contacts-save-role-bcc = Cco

contacts-save-role-cc = cc

contacts-save-role-from = De

contacts-save-role-replyTo = Responder a

contacts-save-role-sender = Remitente

contacts-save-role-to = a

contacts-save-saved-heading = Ya en tus contactos

contacts-save-saved-many = { $count } contactos guardados en { $book }

contacts-save-saved-one = Guardado { $name } en { $book }

contacts-save-sender = Guardar remitente como contacto

contacts-save-settings-book = libreta de direcciones

contacts-save-settings-book-default = Libreta de direcciones predeterminada

contacts-save-settings-enabled = Oferta Guardar contactos de este mensaje

contacts-save-settings-enabled-help = Agrega el elemento a los menús de mensajes y al menú Más del lector.

contacts-save-settings-note = Añade una nota con la fecha.

contacts-save-settings-note-help = La nota dice "Guardado desde un mensaje el" seguido de la fecha.

contacts-save-settings-reply-to = Utilice Responder a para remitentes de listas y notificaciones

contacts-save-settings-reply-to-help = Guarda la dirección Responder a en lugar de De para esos remitentes.

contacts-save-settings-scope = Marcado primero en el diálogo

contacts-save-settings-scope-all = Todos en el mensaje

contacts-save-settings-scope-sender = Sólo remitente

contacts-save-settings-sender = Oferta Guardar remitente como contacto

contacts-save-settings-sender-help = Un elemento de un solo clic en los menús de mensajes.

contacts-save-settings-skip = Saltar remitentes automáticos

contacts-save-settings-skip-help = Omite direcciones de no respuesta, notificación y rebote a menos que las marques.

contacts-save-settings-title = Guardar contactos de mensajes

contacts-save-show-automated = Mostrar direcciones automatizadas y sin respuesta

contacts-save-submit = { $count ->
        [one] Guardar { $count } contacto
        [many] Guardar { $count } contactos
       *[other] Guardar { $count } contactos
    }

contacts-save-undo = Deshacer
