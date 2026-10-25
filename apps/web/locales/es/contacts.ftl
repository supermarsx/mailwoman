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
