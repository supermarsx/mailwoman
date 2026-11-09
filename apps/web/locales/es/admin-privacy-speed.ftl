# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Velocidad
privacy-speed-unavailable = Este servidor no dispone de cifras de velocidad.
privacy-speed-empty = Aún no se han servido imágenes en este intervalo.
privacy-speed-off = El seguimiento de la velocidad está desactivado (ajuste: seguimiento de la velocidad del proxy de imágenes). No se registra nada nuevo.
privacy-speed-verdict-healthy = Correcto
privacy-speed-verdict-slow = Lento
privacy-speed-verdict-degraded = Degradado
privacy-speed-verdict-noData = Datos insuficientes
privacy-speed-reason-ok = Las imágenes obtenidas y los aciertos de caché están dentro de los límites indicados abajo, y pocas solicitudes fallaron.
privacy-speed-reason-failureRate = El { $failurePct } % de las solicitudes terminó como ocupado, con tiempo de espera agotado o con un error del servidor de origen.
privacy-speed-reason-missP95 = Las imágenes que hubo que obtener son lentas: su percentil 95 supera el límite indicado abajo.
privacy-speed-reason-hitP95 = Las imágenes en caché tardan en responder: su percentil 95 supera el límite indicado abajo.
privacy-speed-reason-fewRequests = Menos de { $min } solicitudes en este intervalo, demasiado pocas para valorar.
privacy-speed-thresholds = Lento a partir de: percentil 95 de imagen obtenida { $slowMiss }, percentil 95 de imagen en caché { $slowHit } o { $slowFail } % ocupado, con tiempo agotado o fallido. Degradado a partir de: percentil 95 de imagen obtenida { $degradedMiss } o { $degradedFail } % fallido. Límites fijos, no ajustes.
privacy-speed-median = Mediana
privacy-speed-p95 = Percentil 95
privacy-speed-p99 = Percentil 99
privacy-speed-max = Más lenta
privacy-speed-requests = Solicitudes medidas
privacy-speed-busy = Respuestas de ocupado
privacy-speed-timeouts = Tiempos de espera agotados
privacy-speed-upstream-errors = Errores del servidor de origen
privacy-speed-hit = Aciertos de caché
privacy-speed-miss = Imágenes obtenidas (fallos de caché)
privacy-speed-split-count = { $count } solicitudes
privacy-speed-split-times = Mediana { $p50 }, percentil 95 { $p95 }, más lenta { $max }.
privacy-speed-chart = Tiempo para servir una imagen
privacy-speed-chart-miss = Tiempo para servir una imagen obtenida
privacy-speed-chart-unit = Desde la solicitud recibida hasta la respuesta lista
privacy-speed-phases = En qué se va el tiempo
privacy-speed-phases-help = Para las solicitudes que no fueron aciertos de caché: cuánto duró cada paso y qué parte del tiempo total representa.
privacy-speed-phase = Paso
privacy-speed-share = Parte del tiempo
privacy-speed-phase-queue = Espera de capacidad
privacy-speed-phase-cache = Consulta de la caché
privacy-speed-phase-upstream = Obtención del origen
privacy-speed-phase-decode = Decodificar y volver a codificar
privacy-speed-phase-other = Todo lo demás
privacy-speed-note = Medido solo en este proceso del servidor, se guarda en memoria y queda vacío tras un reinicio. Los percentiles tienen una precisión del { $error } %. No se almacenan URL, hosts ni cuentas.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Recuperaciones unidas y solicitudes de prioridad

privacy-work-coalesced-series = Se une a uno en vuelo.

privacy-work-handle = Manejar la tasa de aciertos de la caché

privacy-work-handle-series = Manejar visitas de caché

privacy-work-per-interval = Por intervalo de muestreo

privacy-work-percent = Proporción de búsquedas de identificadores

privacy-work-priority-series = Solicitudes prioritarias

privacy-work-resized = Imágenes reducidas

privacy-work-resized-series = Redimensionado

privacy-work-revalidated-series = Revalidado

privacy-work-saved = Bytes ahorrados mediante la reducción de escala

privacy-work-saved-series = Bytes guardados

privacy-work-stale = Imágenes obsoletas servidas y revalidadas

privacy-work-stale-series = servido rancio

privacy-work-title = Obra evitada y remodelada
