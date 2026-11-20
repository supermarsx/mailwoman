admin-cache-title = Mantenimiento de caché
admin-cache-help = De forma predeterminada, el contenido de correo en caché caduca tras 60 días sin uso. Abrir un mensaje caducado descarga su contenido de nuevo. Se conservan los mensajes originales, los borradores y el contenido local protegido.
admin-cache-expiry = Duración de los mensajes en caché
admin-cache-orphans = Cuerpos sin referencia y recuperación de espacio
admin-cache-advanced = Límites y programación de lotes
admin-cache-settings-help = Los cambios siguen la política, la validación y el registro de auditoría habituales de la configuración administrativa.
admin-cache-expiry-enabled = Caducar automáticamente los cuerpos de mensajes en caché
admin-cache-retention-days = Días sin uso antes de caducar
admin-cache-expiry-status = Caducidad de cuerpos en caché
admin-cache-expiry-result = Analizados: { $scanned } · Referencias de caché desvinculadas: { $detached } · Protegidos: { $protected }
admin-cache-expiry-help = Solo son aptos los cuerpos cuya recuperación desde el buzón de origen está confirmada. Los análisis y las actualizaciones de cabeceras no renuevan el plazo de inactividad.
admin-cache-inspect = Inspeccionar el almacenamiento de caché
admin-cache-inspect-help = Abra la observabilidad de la base de datos para medir el contenido en caché, los cuerpos referenciados y las copias huérfanas.
admin-cache-no-status = No se ha completado ningún lote de caducidad en este proceso del servidor.
admin-cache-settings-unavailable = No se pudo cargar la configuración de caché.
admin-cache-change-log = Historial de sincronización
admin-cache-change-log-help = Los registros antiguos de cambios caducan tras 60 días de forma predeterminada. Los clientes con tokens de sincronización anteriores reciben una respuesta de resincronización completa.
admin-cache-no-sources = Actualmente ningún buzón conectado tiene una vía confirmada para volver a descargar. Se conserva el contenido protegido.
admin-cache-wal-recycled = Se ha reciclado espacio WAL asignado; la limpieza puede continuar.
admin-cache-wal-recovery = La limpieza espera la recuperación o el punto de control del WAL. Se conservan los datos pendientes del diario.

admin-cache-removed-payload = Contenido en caché eliminado
admin-cache-oversized = Limpieza de cuerpos grandes aplazada
admin-cache-change-removed = Registros del historial de sincronización eliminados

admin-cache-parsed-copies = Copias del cuerpo procesado eliminadas
admin-cache-parsed-bytes = Datos del cuerpo procesado eliminados
admin-cache-change-chunked = Estados de sincronización limpiados por partes
