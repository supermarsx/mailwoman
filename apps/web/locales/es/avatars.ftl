# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Personas e imágenes de contacto
avatars-mode = Imágenes de remitentes y contactos
avatars-off = Ocultar imágenes
avatars-initials = Generar iniciales localmente
avatars-contacts = Usar imágenes de contacto locales, con iniciales como alternativa
avatars-help = Las iniciales se generan en este dispositivo. Las imágenes de contacto solo usan fotos PNG o JPEG incrustadas que ya están guardadas en sus propias libretas de direcciones, de hasta 512 × 512 píxeles y 128 KiB. Nunca se solicitan URL de imágenes remotas ni servicios de avatares externos. Esta preferencia se guarda con su cuenta y su administrador puede fijarla.
avatars-default = Restaurar la preferencia heredada
avatars-sender-icons = Mostrar los iconos incluidos de dominios de remitentes
avatars-sender-icons-help = Los dominios reconocidos de redes sociales, desarrollo, IA, compras y administraciones públicas usan iconos incluidos en esta aplicación. Sus propias imágenes de contacto tienen prioridad. Los remitentes desconocidos usan iniciales generadas localmente con 192 colores. Los iconos son indicios del dominio, no una prueba de que un mensaje sea auténtico. No se contacta con ningún servicio externo de logotipos.
avatars-icons-default = Restaurar la preferencia de iconos heredada
avatars-brand-avatars = Mostrar logotipos de marca junto a los remitentes verificados
avatars-brand-avatars-help = En el lector, una empresa reconocida muestra su logotipo junto al remitente solo si el mensaje superó DKIM y una comprobación DMARC alineada para el dominio propio de esa empresa. En caso contrario se usa el avatar normal. Los logotipos están incluidos en esta aplicación; no se solicita nada al remitente.
avatars-brand-avatars-default = Restaurar la preferencia de logotipos de marca heredada
avatars-domain-icon = Icono del dominio { $name } · este icono no verifica la identidad del remitente
avatars-favorites-first = Mostrar primero los contactos favoritos en las sugerencias de destinatarios
avatars-favorites-first-help = Los contactos marcados con estrella aparecen antes que las demás coincidencias al escribir un destinatario. Solo cambia el orden; no se oculta ninguna sugerencia.
avatars-favorites-first-default = Restaurar la preferencia de favoritos heredada

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Se necesita autenticación para los logotipos de marcas.

avatars-require-auth-always = Siempre (requiere un cheque aprobado)

avatars-require-auth-default = Restaurar la preferencia de autenticación heredada

avatars-require-auth-help = Elija cuándo el logotipo de una empresa reconocida puede reemplazar las iniciales junto al remitente. Nunca: muéstralo siempre que coincida el dominio del remitente. Solo si falló una verificación (predeterminado): muéstrelo a menos que el mensaje falló DKIM o DMARC. Siempre: muéstrelo solo cuando el mensaje pasó DKIM y una verificación DMARC alineada para el dominio propio de esa empresa, de modo que el correo aún no verificado muestre las iniciales. La coincidencia de dominio incluye subdominios como mail.example.com. Esta preferencia se guarda con su cuenta y su administrador puede mantenerla.

avatars-require-auth-known-failure = Sólo si un control falló

avatars-require-auth-never = Nunca (coincidencia solo en el dominio)

avatars-unverified-sender = Remitente no verificado · no se muestra ningún ícono de marca porque este mensaje no pasó la verificación de autenticación
