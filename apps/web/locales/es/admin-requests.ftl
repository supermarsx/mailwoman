# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = No encontrado (404) por intervalo
admin-history-nf-total = No encontrado (404) en este periodo
admin-history-nf-share = { $share } de las solicitudes
admin-history-nf-not-tracked-window = Los errores 404 no se registraron durante este periodo.
admin-history-nf-not-tracked-before = Los errores 404 no se registraban antes de { $time }. Los periodos anteriores se muestran como no disponibles, no como cero.
admin-history-nf-paths-title = Rutas no encontradas más frecuentes
admin-history-nf-paths-help = Hasta { $limit } rutas por hora, de { $from } a { $to }, redondeadas a horas completas. Se eliminan las cadenas de consulta y las direcciones de correo, números, ID, tokens y valores largos se sustituyen por marcadores como {"{"}token{"}"}. No se cuentan los fallos esperados de la propia aplicación, como las solicitudes de iconos del navegador.
admin-history-nf-paths-approximate = Llegaron más rutas distintas de las que se conservan, por lo que algunos recuentos son un intervalo y no una cifra exacta.
admin-history-nf-paths-empty = No hay respuestas de no encontrado en este periodo.
admin-history-nf-paths-error = No se pudieron cargar las rutas no encontradas. Actualice para volver a intentarlo.
admin-history-nf-paths-method = Método
admin-history-nf-paths-path = Ruta
admin-history-nf-paths-count = Recuento
admin-history-nf-paths-range = { $min } a { $max }
admin-history-nf-paths-client = De los clientes propios de la aplicación
admin-history-nf-paths-client-help = Rutas que solicitan la aplicación web, su service worker o las aplicaciones de escritorio y móviles: archivos de compilación, el script del worker, y rutas de API, JMAP, DAV y consola. Un fallo aquí suele significar que un cliente está desactualizado o que una ruta es incorrecta.
admin-history-nf-paths-external = Otro tráfico
admin-history-nf-paths-external-help = Rutas que la aplicación nunca solicita, como escáneres que buscan archivos PHP, archivos ocultos u otro software, y enlaces de otros sitios.
admin-history-nf-paths-all = Rutas
admin-history-nf-paths-all-help = Este servidor no distingue las rutas de los clientes del resto del tráfico.
