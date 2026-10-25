# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Cerrar

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = No disponible — { $reason }
admin-ux-unavailable-default = este servidor todavía no lo ofrece.
admin-ux-state-code = Código de error: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Abrir
admin-ux-list-back = Volver a la lista

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = No hay cambios sin guardar
admin-ux-form-changes =
    { $count ->
        [one] 1 cambio sin guardar
       *[other] { $count } cambios sin guardar
    }
admin-ux-form-discard = Descartar cambios
admin-ux-form-saving = Guardando…
admin-ux-form-saved =
    { $count ->
        [one] Se ha guardado 1 cambio.
       *[other] Se han guardado { $count } cambios.
    }
admin-ux-form-fix =
    { $count ->
        [one] No se ha guardado nada. Corrija el campo marcado abajo.
       *[other] No se ha guardado nada. Corrija los { $count } campos marcados abajo.
    }
admin-ux-learn-more = Más información
admin-ux-learn-more-field = sobre { $label }
admin-ux-propagation-hot = Se aplica en 30 segundos en todos los procesos del servidor.
admin-ux-propagation-restart = Se aplica después de reiniciar el servidor.
admin-ux-secret-set = Hay un valor almacenado. Escriba uno nuevo para reemplazarlo.
admin-ux-secret-unset = No hay ningún valor almacenado.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Introduzca un valor.
admin-ux-err-number = Introduzca un número.
admin-ux-err-whole = Introduzca un número entero.
admin-ux-err-min = Introduzca { $min } o más.
admin-ux-err-max = Introduzca { $max } o menos.
admin-ux-err-max-length = Use como máximo { $max } caracteres.
admin-ux-err-url = Introduzca una dirección completa que empiece por https:// o http://.
admin-ux-err-email = Introduzca una dirección de correo como name@example.org.
admin-ux-err-pattern = Este valor no tiene el formato esperado.
admin-ux-err-option = Elija una de las opciones de la lista.
admin-ux-err-out-of-range = El servidor rechazó este valor porque está fuera del rango permitido.
admin-ux-err-too-long = El servidor rechazó este valor porque es demasiado largo.
admin-ux-err-invalid = El servidor rechazó este valor. Revíselo y vuelva a guardar.
admin-ux-err-pinned = Este valor está establecido en el entorno del servidor y no se puede cambiar aquí.
admin-ux-err-locked = Este valor está bloqueado en un ámbito más amplio y no se puede cambiar aquí.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Predeterminado
admin-ux-source-stored = Almacenado
admin-ux-source-env-pin = Establecido por { $name }
admin-ux-source-env-fallback = Valor alternativo de { $name }
admin-ux-source-build = Fijado al compilar
admin-ux-source-locked = Bloqueado por { $scope }
admin-ux-source-read-only = Solo lectura
admin-ux-source-restart = Requiere reinicio
admin-ux-source-env-pin-reason = El entorno del servidor establece este valor con { $name }. Cámbielo allí y reinicie el servidor.
admin-ux-source-build-reason = Este valor se fija al compilar el servidor.
admin-ux-source-locked-reason = Un bloqueo en el ámbito { $scope } fija este valor. Quite el bloqueo allí para cambiarlo.
admin-ux-source-read-only-reason = La consola muestra este valor, pero no puede cambiarlo. La documentación indica dónde se establece.

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = Último cambio de { $who } el { $time }
admin-ux-history-changed-by = Último cambio de { $who }
admin-ux-history-changed-at = Último cambio el { $time }
admin-ux-history-never = No se ha cambiado desde la consola
admin-ux-history-link = Historial

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = Predeterminado: { $value }
admin-ux-revert = Restablecer el predeterminado
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Se ha restablecido el valor predeterminado ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } cambió esto el { $time } mientras usted lo editaba.
admin-ux-stale-by = { $who } cambió esto mientras usted lo editaba.
admin-ux-stale-unknown = Otra persona cambió esto mientras usted lo editaba.
admin-ux-stale-choice = Vuelva a cargar para ver la versión de la otra persona y perder sus cambios, o conserve sus cambios y guárdelos encima de esa versión.
admin-ux-stale-reload = Volver a cargar
admin-ux-stale-keep = Conservar los míos

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Comprobando a qué afecta este cambio…
admin-ux-impact-heading = Este cambio afecta a
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] y 1 más
       *[other] y { $count } más
    }
admin-ux-impact-none = No cambia nada más.
admin-ux-impact-unavailable = Vista previa del impacto no disponible — el servidor no indicó a qué afecta este cambio.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filtrar
admin-ux-stream-pause = Pausar
admin-ux-stream-resume = Reanudar
admin-ux-stream-clear = Borrar
admin-ux-stream-announce = Leer en voz alta las entradas nuevas
admin-ux-stream-connecting = Conectando…
admin-ux-stream-live = En directo
admin-ux-stream-ended = Finalizado
admin-ux-stream-lost = Conexión perdida
admin-ux-stream-reconnect = Volver a conectar
admin-ux-stream-paused =
    { $count ->
        [0] En pausa
        [one] En pausa — 1 entrada nueva retenida
       *[other] En pausa — { $count } entradas nuevas retenidas
    }
admin-ux-stream-no-match = Ninguna entrada coincide con el filtro.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Campo

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Apariencia
admin-ux-appearance-title = Apariencia de la consola
admin-ux-appearance-note = Estas opciones solo cambian la consola de administración, en este navegador.
admin-ux-appearance-following = La consola sigue el tema del correo web de este navegador hasta que elija aquí. Estas opciones solo cambian la consola de administración, en este navegador.
admin-ux-appearance-mode = Modo
admin-ux-appearance-mode-light = Claro
admin-ux-appearance-mode-dark = Oscuro
admin-ux-appearance-mode-system = Igual que el sistema
admin-ux-appearance-theme = Tema
admin-ux-appearance-accent = Color de acento
admin-ux-appearance-light = Claro
admin-ux-appearance-dark = Oscuro
admin-ux-appearance-reset = Usar el tema del correo web
