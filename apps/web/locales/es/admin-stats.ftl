# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Cifras clave
admin-stats-charts = Gráficos
admin-stats-tables = Tablas
admin-stats-group-traffic = Tráfico en este periodo
admin-stats-group-storage = Almacenamiento y entrega, ahora
admin-stats-refresh = Actualizar estadísticas
admin-stats-loading = Cargando estadísticas…
admin-stats-requests-total = Solicitudes
admin-stats-errors-total = Errores del servidor (5xx)
admin-stats-p95-peak = Percentil 95 más lento
admin-stats-requests = Solicitudes
admin-stats-errors = Errores del servidor (5xx)
admin-stats-not-found = No encontrado (404)
admin-stats-latency = Tiempo de respuesta
admin-stats-series-p50 = Mediana
admin-stats-series-p95 = Percentil 95
admin-stats-unit-count = Recuento por { $bucket }
admin-stats-unit-duration = Tiempo por { $bucket }, en la unidad mostrada
admin-stats-unit-accounts = Cuentas bloqueadas por dominio
admin-stats-chart-about = Gráfico de líneas de { $from } a { $to }.
admin-stats-bars-about = { $count ->
    [one] Gráfico de barras con un dominio.
   *[other] Gráfico de barras con { $count } dominios.
}
admin-stats-view-table = Ver como tabla
admin-stats-legend = Leyenda
admin-stats-empty-chart = Todavía no se ha medido nada en este periodo. Elija un periodo más largo o vuelva cuando el servidor haya gestionado algunas solicitudes.
admin-stats-empty-blocked = No hay cuentas bloqueadas.
admin-stats-delta-vs = frente al periodo anterior
admin-stats-delta-same = Igual que el periodo anterior
admin-stats-delta-sr-up = { $change } más que el periodo anterior.
admin-stats-delta-sr-down = { $change } menos que el periodo anterior.
admin-stats-delta-sr-same = Igual que el periodo anterior.
admin-ct-group = Tiempos de carga del cliente
admin-ct-metric-timeToUsable = Tiempo hasta que se puede usar
admin-ct-metric-ttfb = Primer byte
admin-ct-metric-dcl = DOM listo
admin-ct-metric-load = Carga de la página
admin-ct-metric-fcp = Primer renderizado de contenido
admin-ct-metric-lcp = Mayor renderizado de contenido
admin-ct-metric-folderOpen = Apertura de carpeta
admin-ct-metric-folderLoadCold = Carga de carpeta, desde la red
admin-ct-metric-folderLoadCached = Carga de carpeta, desde la caché
admin-ct-metric-messageLoadCold = Carga de mensaje, desde la red
admin-ct-metric-messageLoadCached = Carga de mensaje, desde la caché o la precarga
admin-ct-device-desktop = Escritorio
admin-ct-device-mobile = Móvil
admin-ct-device-unknown = Desconocido
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } muestra
       *[other] { $count } muestras
    }
admin-ct-no-samples = No hay muestras en este periodo.
admin-ct-few-samples = Solo { $count } muestras; se necesitan al menos { $min }.
admin-ct-chart-usable = Tiempo hasta que se puede usar
admin-ct-group-folder = Tiempos de carga de carpetas
admin-ct-group-message = Tiempos de carga de mensajes
admin-ct-chart-folder-cold = Carga de carpeta desde la red
admin-ct-chart-folder-cached = Carga de carpeta desde la caché
admin-ct-chart-message-cold = Carga de mensaje desde la red
admin-ct-chart-message-cached = Carga de mensaje desde la caché o la precarga
admin-ct-folder-off = Los tiempos de carga de carpetas están desactivados de forma predeterminada. Solo se mide a las personas que los activaron.
admin-ct-message-off = Los tiempos de carga de mensajes están desactivados de forma predeterminada. Solo se mide a las personas que los activaron.
admin-ct-chart-note = Los percentiles solo se muestran en los intervalos con al menos { $min } muestras.
admin-ct-empty-chart = Ningún intervalo tiene las { $min } muestras necesarias para mostrar una mediana. Elija un periodo más largo.
admin-ct-col-samples = Muestras
admin-ct-col-metric = Medida
admin-ct-col-device = Dispositivo
admin-ct-table-title = Tiempos de carga del cliente por dispositivo
admin-ct-table-caption = Muestras, mediana y percentil 95 de cada medida y clase de dispositivo.
admin-ct-empty-table = No se informó de tiempos de carga del navegador en este periodo.
admin-ct-failed = No se pudieron cargar los tiempos de carga del cliente.
admin-ct-disabled = La recopilación de tiempos de carga del cliente está desactivada. Las cifras de abajo son de antes de desactivarla.
admin-ct-note = Solo de navegadores con sesión iniciada, muestreo del { $rate } % de las sesiones, anónimo y agregado. El tiempo hasta que se puede usar transcurre desde el inicio de la navegación hasta que la lista de mensajes muestra sus primeras filas. Las cifras son estimaciones, con una desviación de aproximadamente el { $error } % respecto al valor real.
