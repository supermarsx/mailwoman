# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Telemetría operativa
admin-telemetry-help = Exporte trazas y métricas operativas a un recolector compatible con OpenTelemetry, como un OpenTelemetry Collector alojado por usted. La exportación está desactivada de forma predeterminada. Los cambios guardados se aplican tras reiniciar el servidor.
admin-telemetry-back = Volver a observabilidad
admin-telemetry-saved = Configuración de telemetría guardada. Reinicie el servidor para aplicar los cambios.
admin-telemetry-restart = La configuración guardada difiere del servidor en ejecución. Es necesario reiniciar.
admin-telemetry-reload = Recargar configuración
admin-telemetry-save = Guardar configuración de telemetría
admin-telemetry-collector = Recolector y señales
admin-telemetry-collector-help = Solo un administrador puede elegir este destino para todo el servidor. Use un recolector privado o HTTPS; HTTP transmite la telemetría sin cifrar.
admin-telemetry-enabled = Activar la exportación OTLP
admin-telemetry-traces = Exportar trazas
admin-telemetry-metrics = Exportar métricas
admin-telemetry-endpoint = Punto de conexión OTLP gRPC
admin-telemetry-endpoint-help = Dirección absoluta HTTP o HTTPS del recolector, normalmente el puerto 4317. No incluya credenciales, consulta ni fragmento. Es un punto de conexión gRPC, no el puerto 4318 de OTLP HTTP.
admin-telemetry-serviceName = Nombre del servicio
admin-telemetry-serviceName-help = Etiqueta de servicio estable que identifica este despliegue de Mailwoman en su recolector. No use una dirección de correo ni otros datos personales.
admin-telemetry-environment = Entorno
admin-telemetry-environment-help = Etiqueta opcional del despliegue, como production o staging. Se envía junto con la telemetría exportada.
admin-telemetry-pinned = Los campos desactivados los controlan variables de entorno del despliegue. Cambie esas variables para anular sus valores.
admin-telemetry-limits = Muestreo y límites de exportación
admin-telemetry-limits-help = Limitan la memoria y el tráfico hacia el recolector. El tamaño del lote no puede superar el tamaño de la cola. Las colas llenas pueden descartar spans en lugar de bloquear las solicitudes de correo.
admin-telemetry-sampleRatio = Proporción de muestreo de trazas
admin-telemetry-sampleRatio-help = Fracción de trazas raíz nuevas que se conservan, de 0 a 1. Los spans hijos siguen la decisión de muestreo del padre.
admin-telemetry-exportTimeoutMs = Tiempo de espera de exportación (milisegundos)
admin-telemetry-exportTimeoutMs-help = Duración máxima de una solicitud de exportación al recolector, de 100 a 30000 milisegundos.
admin-telemetry-metricsIntervalSecs = Intervalo de métricas (segundos)
admin-telemetry-metricsIntervalSecs-help = Tiempo entre exportaciones de métricas, de 10 a 3600 segundos.
admin-telemetry-spanQueueSize = Tamaño de la cola de spans
admin-telemetry-spanQueueSize-help = Número máximo de spans a la espera de exportación, de 128 a 32768.
admin-telemetry-spanBatchSize = Tamaño del lote de spans
admin-telemetry-spanBatchSize-help = Número máximo de spans en una exportación, de 1 a 4096 y sin superar el tamaño de la cola.
admin-telemetry-spanExportDelayMs = Retraso del lote de spans (milisegundos)
admin-telemetry-spanExportDelayMs-help = Espera normal entre lotes de spans, de 100 a 60000 milisegundos.
admin-telemetry-prometheus-help = Un punto de conexión autenticado independiente para un recolector compatible con Prometheus. El acceso siempre requiere el token de portador MW_METRICS_TOKEN del despliegue; su valor nunca se devuelve a esta página.
admin-telemetry-prometheusEnabled = Activar la recopilación de Prometheus
admin-telemetry-token-missing = No hay ningún token de recopilación configurado. El punto de conexión de métricas sigue inaccesible hasta que el operador defina MW_METRICS_TOKEN.
