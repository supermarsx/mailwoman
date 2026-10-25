# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = Asistente de IA
assist-heading = Asistente de IA
assist-transcript-label = Conversación con el asistente de IA
assist-proposed-action = Acción propuesta
assist-outbox-note = Esto colocaría un mensaje en su Bandeja de salida. No se envía nada hasta que lo confirme allí.
assist-review-outbox = Revisar en la Bandeja de salida
assist-review = Revisar
assist-empty = Pida al asistente de IA que resuma, redacte o busque cosas. Puede proponer acciones, pero usted siempre las confirma.
assist-error = El asistente de IA no ha podido responder en este momento.
assist-input-label = Escribir al asistente de IA
assist-input-placeholder = Pregunte al asistente de IA…
assist-thinking = Pensando…
assist-ask = Preguntar

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Ayuda para escribir
assist-composer-toolbar = Herramientas del editor
assist-tool-grammar = Corregir gramática
assist-tool-rewrite = Reescribir
assist-tool-tone = Ajustar tono
assist-tool-translate = Traducir
assist-toolarg-tone = Tono
assist-toolarg-translate = Idioma
assist-tone-neutral = Neutro
assist-tone-friendly = Cordial
assist-tone-formal = Formal
assist-tone-concise = Conciso
assist-tone-assertive = Firme
assist-busy = { $label }…
assist-composer-empty-error = Escriba algo primero.
assist-composer-error = El asistente de IA no ha podido completarlo en este momento.
assist-suggested-edit = Edición sugerida
assist-apply-draft = Aplicar al borrador
assist-discard = Descartar

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Etiquetas sugeridas
assist-apply-auto = Aplicar automáticamente
assist-apply-label = Aplicar { $label }
assist-remove-label = Quitar { $label }
assist-apply = Aplicar
assist-undo = Deshacer

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Mantener pulsado para dictar
assist-dictate-stop = Detener dictado
assist-dictate-hold-text = 🎙 Mantener pulsado para dictar
assist-dictate-listening = ● Escuchando…
assist-dictate-endpoint-note = El audio lo transcribe el endpoint de su asistente de IA.
assist-dictate-err = Error de dictado.
assist-transcribe-err = No se pudo transcribir el audio.
assist-mic-err = Micrófono no disponible.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Búsqueda semántica
assist-semantic-note = El texto de la consulta se envía al endpoint de su asistente de IA para ordenar por significado.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Qué puede salir de este dispositivo
assist-left-1 = el texto del mensaje seleccionado (asunto + cuerpo) para la capacidad elegida
assist-left-2 = el host del endpoint al que se envió
assist-left-3 = nunca: contenido cifrado de extremo a extremo (excluido de forma predeterminada)
assist-left-4 = nunca: archivos adjuntos (excluidos de forma predeterminada)
assist-left-5 = nunca: sus credenciales ni otras cuentas
assist-disclosure-off = El asistente de IA está desactivado. Ningún contenido de mensajes sale de este dispositivo.
assist-disclosure-sentence = Cuando usa una herramienta del asistente de IA, el texto del mensaje seleccionado se reenvía a { $host }. { $excl } El envío nunca es automático: usted siempre confirma antes de que algo salga de su Bandeja de salida.
assist-disclosure-excl-one = Nunca envía { $a }.
assist-disclosure-excl-two = Nunca envía { $a } ni { $b }.
assist-disclosure-admin-allowed = Su administrador ha permitido enviar contenido cifrado y archivos adjuntos.
assist-excluded-e2ee = contenido cifrado de extremo a extremo
assist-excluded-attachments = archivos adjuntos

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = Asistente de IA
assist-admin-intro = El asistente de IA envía el texto de los mensajes que seleccionan los usuarios al endpoint de IA configurado aquí. No puede enviar, eliminar ni aceptar correo. El contenido cifrado de extremo a extremo y los archivos adjuntos se retienen salvo que los permita abajo.
assist-admin-load-unreadable = No se puede leer la configuración almacenada del asistente de IA: su clave de API no se abre con la clave de este servidor (MW_SERVER_KEY). Se guardó con otra clave o ha sido modificada. El asistente de IA está desactivado. Rellene el formulario, incluida la clave de API, y guarde para sustituirla.
assist-admin-load-error = No se pudo cargar la configuración del asistente de IA. Guardar está deshabilitado para no sobrescribir la configuración almacenada.
assist-admin-status-running = El asistente de IA está en funcionamiento. Las solicitudes van a { $host }.
assist-admin-status-running-pending = El asistente de IA funciona con la configuración con la que se inició el servidor. Los cambios guardados surten efecto cuando se reinicie el servidor.
assist-admin-status-off = El asistente de IA está desactivado. Se rechazan todas las solicitudes al asistente de IA.
assist-admin-status-pending = El asistente de IA no está en funcionamiento. La configuración guardada surte efecto cuando se reinicie el servidor.
assist-admin-status-local-program-not-allowed = El asistente de IA no está en funcionamiento. El programa configurado { $program } no figura en MW_ASSIST_LOCAL_PROGRAMS en el servidor, por lo que el servidor no lo inició.
assist-admin-status-no-endpoint = El asistente de IA está activado, pero no en funcionamiento porque no hay ningún endpoint utilizable configurado.
assist-admin-stop = Detener el asistente de IA ahora
assist-admin-stop-note = La detención surte efecto de inmediato para todos los usuarios y se mantiene tras los reinicios.
assist-admin-resume = Activar el asistente de IA
assist-admin-resume-note = Si el servidor se inició con el asistente de IA desactivado, activarlo surte efecto cuando se reinicie el servidor.
assist-admin-kill-error = No se pudo cambiar si el asistente de IA está activado.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = El servidor contacta con este endpoint; los navegadores nunca lo hacen. Los cambios surten efecto cuando se reinicie el servidor.
assist-admin-kind = Tipo de endpoint
assist-admin-kind-none = Ninguno
assist-admin-kind-open-ai-compatible = Compatible con OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Programa local
assist-admin-local-program-enable = Para que el asistente de IA ejecute un programa en este servidor, establezca MW_ASSIST_LOCAL_PROGRAMS con la ruta absoluta del programa en el entorno donde se inicia el servidor (separe varias rutas con comas) y reinicie el servidor.
assist-admin-local-program-note = El servidor inicia este programa para cada solicitud al asistente de IA y le pasa la solicitud por la entrada estándar, incluido el texto del mensaje seleccionado por el usuario. Lee la respuesta de la salida estándar del programa. Solo se pueden elegir los programas que figuran en MW_ASSIST_LOCAL_PROGRAMS en el servidor.
assist-admin-local-program = Programa
assist-admin-local-program-unlisted = { $program } (no figura en la lista)
assist-admin-local-program-unlisted-note = Este programa no figura en MW_ASSIST_LOCAL_PROGRAMS, por lo que el servidor no lo ejecuta y no lo aceptará al guardar. Elija un programa de la lista u otro tipo de endpoint.
assist-admin-local-program-required = Elija el programa que se ejecutará.
assist-admin-local-args = Argumentos, uno por línea
assist-admin-local-args-note = Cada línea se pasa al programa como un argumento. No interviene ningún shell.
assist-admin-base-url = URL base
assist-admin-base-url-required = Introduzca la URL base del endpoint, empezando por http:// o https://.
assist-admin-api-key = Clave de API
assist-admin-api-key-stored = Hay una clave almacenada. El servidor no vuelve a mostrarla.
assist-admin-api-key-replace = Sustituir la clave de API
assist-admin-api-key-replace-placeholder = Dejar vacío para conservar la clave almacenada
assist-admin-api-key-remove = Quitar la clave almacenada
assist-admin-api-key-keep = Conservar la clave almacenada
assist-admin-api-key-remove-pending = La clave almacenada se quitará al guardar.
assist-admin-chat-model = Modelo de chat
assist-admin-embed-model = Modelo de embeddings
assist-admin-stt-model = Modelo de voz a texto
assist-admin-model = Modelo
assist-admin-anthropic-version = Versión de la API
assist-admin-max-tokens = Tokens máximos por respuesta
assist-admin-default-placeholder = Valor predeterminado del servidor
assist-admin-grants = Capacidades
assist-admin-grants-note = Solo se pueden usar las capacidades marcadas aquí.
assist-admin-cap-summarize = Resumir
assist-admin-cap-draft = Redactar y reescribir
assist-admin-cap-grammar = Gramática
assist-admin-cap-dictation = Dictado
assist-admin-cap-search-semantic = Búsqueda semántica
assist-admin-cap-auto-tag = Etiquetado automático
assist-admin-cap-recap = Recapitulación
assist-admin-cap-assistant = Chat con el asistente de IA
assist-admin-ceilings = Límites de datos
assist-admin-ceilings-note = Estos límites se aplican a todas las solicitudes, sea cual sea la capacidad.
assist-admin-accounts = Ids de cuenta cuyo correo puede enviarse, uno por línea
assist-admin-accounts-note = Si no hay ninguna cuenta en la lista, no se envía contenido de correo al endpoint.
assist-admin-folders = Ids de carpeta a los que limitar el envío, uno por línea
assist-admin-folders-note = Si no hay ninguna carpeta en la lista, se permiten todas las carpetas de las cuentas indicadas.
assist-admin-allow-e2ee = Permitir enviar contenido cifrado de extremo a extremo
assist-admin-allow-attachments = Permitir enviar archivos adjuntos
assist-admin-save = Guardar configuración
assist-admin-saved = Guardado.
assist-admin-saved-pending = Guardado. El cambio surte efecto cuando se reinicie el servidor.
assist-admin-save-error = La configuración no se guardó.
assist-admin-save-not-allowed = La configuración no se guardó porque el servidor no permite este programa. Solo se pueden usar los programas que figuran en MW_ASSIST_LOCAL_PROGRAMS.
