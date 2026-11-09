# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Conservación
retention-intro = Reglas que mueven o eliminan su propio correo pasado un número de días. Una regla puede fijarse en una carpeta, una etiqueta, un remitente, un tamaño, si el mensaje está leído o si tiene un archivo adjunto. El correo marcado con bandera y el etiquetado como keep se deja intacto, salvo que lo cambie. El correo eliminado va primero a la papelera y se purga al terminar el plazo para deshacer.
retention-disabled = Las reglas de conservación están desactivadas en este servidor. Sus reglas existentes no se ejecutan.
retention-never = nunca
retention-error = No se pudieron cargar o guardar los ajustes de conservación. Inténtelo de nuevo.
retention-last-run = Última ejecución: { $when }
retention-next-run = próxima ejecución en unos { $days } día(s)
retention-next-run-unknown = próxima ejecución aún sin programar
retention-enabled = Ejecutar mis reglas de conservación
retention-trash-first = Mover primero el correo eliminado a la papelera
retention-trash-forced = Este servidor siempre mueve el correo primero a la papelera y lo purga al terminar el plazo para deshacer.
retention-undo = Plazo para deshacer en días
retention-undo-help = El correo que una regla movió a la papelera permanece allí este tiempo antes de purgarse. De 0 a { $max } días.
retention-pause = Pausar todas las reglas durante 30 días
retention-paused-until = En pausa hasta el { $when }
retention-resume = Reanudar
retention-policy = Lo que permite este servidor
retention-limits-both = Una regla que elimina correo espera al menos { $min } días y como máximo { $max } días.
retention-limits-min = Una regla que elimina correo espera al menos { $min } días.
retention-max-rules = Puede conservar hasta { $count } reglas.
retention-permanent-allowed = Una regla puede eliminar correo de forma permanente si desactiva el paso de la papelera.
retention-permanent-denied = Las reglas no pueden eliminar correo de forma permanente. El correo va a la papelera y se purga tras { $days } días.
retention-protected = Carpetas que las reglas nunca tocan: { $folders }.
retention-mandatory = Establecido por el administrador: el correo de { $folder } con más de { $days } días se elimina.
retention-rules = Sus reglas
retention-add = Nueva regla
retention-no-rules = Aún no tiene reglas. Adopte una plantilla a continuación o añada una.
retention-rule-paused = en pausa
retention-rule-on = Activada
retention-rule-enabled = La regla { $name } está activada
retention-edit-short = Editar
retention-delete = Eliminar
retention-new = Nueva regla
retention-edit = Editar regla
retention-name = Nombre
retention-action = Qué hacer
retention-action-keep = Conservar siempre
retention-action-delete = Eliminar
retention-action-trash = Mover a la papelera
retention-action-archive = Mover al archivo
retention-action-move = Mover a una carpeta
retention-action-markRead = Marcar como leído
retention-days = Tras (días)
retention-target = Carpeta
retention-choose-folder = Elegir una carpeta
retention-scope = Qué correo
retention-folder = Carpeta
retention-any-folder = Cualquier carpeta
retention-subfolders = Incluir subcarpetas
retention-labels = Etiquetas (separadas por comas)
retention-senders = Direcciones de remitente (separadas por comas)
retention-domains = Dominios de remitente (separados por comas)
retention-min-size = Tamaño mínimo en KB
retention-max-size = Tamaño máximo en KB
retention-read = Estado de lectura
retention-read-any = Leído o no leído
retention-read-read = Solo leídos
retention-read-unread = Solo no leídos
retention-attachment = Archivos adjuntos
retention-attachment-any = Con o sin
retention-attachment-with = Con archivo adjunto
retention-attachment-without = Sin archivo adjunto
retention-exclude = Dejar intacto
retention-exclude-flagged = Correo marcado con bandera (destacado)
retention-exclude-unread = Correo no leído
retention-exclude-tags = Correo con estas etiquetas (separadas por comas)
retention-exclude-note = El correo sujeto a una retención legal nunca se toca, sea cual sea este ajuste.
retention-role-inbox = Bandeja de entrada
retention-role-sent = Enviados
retention-role-archive = Archivo
retention-role-junk = Correo no deseado
retention-role-trash = Papelera
retention-role-drafts = Borradores
retention-btn-preview = Vista previa
retention-save = Guardar
retention-cancel = Cancelar
retention-preview-none = Ningún mensaje cambiaría.
retention-preview-count = Esto cambiaría { $count } mensaje(s).
retention-preview-truncated = Para esta vista previa solo se leyó una parte del buzón.
retention-preview-sample = Ejemplos
retention-no-subject = (sin asunto)
retention-do-trash = a la papelera
retention-do-move = movidos
retention-do-delete = eliminados
retention-do-markRead = marcados como leídos
retention-skipped-legalHold = Su cuenta está sujeta a una retención legal. No se cambia nada.
retention-skipped-restricted = El tratamiento de su cuenta está limitado. No se cambia nada.
retention-skipped-noRules = No tiene reglas activas y el administrador no ha definido ninguna.
retention-error-name = Dé un nombre a la regla.
retention-error-days = Los días deben ser un número entero de 0 a 3650.
retention-error-min = Una regla que elimina correo debe esperar al menos { $days } días.
retention-error-max = Una regla que elimina correo puede esperar como máximo { $days } días.
retention-error-target = Elija la carpeta a la que mover el correo.
retention-summary-keep = conservar siempre
retention-summary-delete = tras { $days } días, a la papelera y luego purgado
retention-summary-delete-permanent = tras { $days } días, eliminado definitivamente
retention-summary-trash = tras { $days } días, a la papelera
retention-summary-archive = tras { $days } días, al archivo
retention-summary-move = tras { $days } días, a una carpeta
retention-summary-markRead = tras { $days } días, marcado como leído
retention-templates = Plantillas
retention-templates-help = Una plantilla añade una regla ya preparada. Compruebe sus etiquetas y días antes de fiarse de ella.
retention-adopt = Añadir esta regla
retention-run-heading = Ejecutar ahora
retention-preview-all = Vista previa de todas las reglas
retention-btn-run = Ejecutar mis reglas ahora
retention-run-result = { $count } mensaje(s) cambiado(s).
retention-history = Ejecuciones recientes
retention-history-when = Cuándo
retention-history-matched = Coincidencias
retention-history-changed = Cambiados
retention-history-errors = Errores
retention-context-action = Conservación…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Calendarios

pim-cal-days = Días después de que terminó un evento

pim-cal-enabled = Eliminar eventos finalizados

pim-cal-help = Elimina eventos en los calendarios que elijas una cantidad determinada de días después de que finalizaron. Los eventos recurrentes nunca se eliminan.

pim-cal-title = Eventos del calendario

pim-days-range = Elige entre { $min } y { $max } días.

pim-held-title = Notas eliminadas que puedes restaurar

pim-held-until = Mantenido hasta { $when }

pim-limits = Entre { $min } y { $max } días.

pim-notes-days = Días desde la última vez que se cambió una nota

pim-notes-enabled = Eliminar notas intactas

pim-notes-help = Elimina las notas que no hayas cambiado durante un número determinado de días. Las notas fijadas se conservan. Una nota eliminada se puede restaurar durante { $days } días.

pim-notes-title = Notas

pim-off = Desactivado por su administrador.

pim-preview = Vista previa

pim-restore = Restaurar

pim-result = { $events } evento(s) y { $notes } nota(s).

pim-run = Corre ahora

pim-run-title = Corre ahora

pim-save = Guardar

pim-skipped = Se omitieron { $count } opciones porque su administrador las desactivó.

pim-untitled = Nota sin título

retention-action-stripAttachments = Quitar archivos adjuntos grandes

retention-do-stripAttachments = archivos adjuntos eliminados

retention-error-strip-disabled = Su administrador deshabilita la eliminación de archivos adjuntos.

retention-error-strip-min = Una regla que elimina archivos adjuntos debe esperar al menos { $days } días.

retention-fd-action = ¿Qué pasa con el correo { $folder }?

retention-fd-days = Días antes de que se procese { $folder }

retention-fd-help = Establezca cuánto tiempo permanece el correo en una carpeta antes de moverlo o eliminarlo. Una carpeta configurada como "Heredar" no tiene ningún valor predeterminado propio; Las reglas anteriores y los límites del administrador aún se aplican. Las subcarpetas siguen a su carpeta.

retention-fd-inherit = heredar

retention-fd-mandatory = El administrador conserva esta carpeta durante { $days } días

retention-fd-protected = Protegido por el administrador

retention-fd-save = Guardar valores predeterminados de carpeta

retention-fd-title = Valores predeterminados de carpeta

retention-rule-strip-off = Desactivado por su administrador

retention-strip-allowed = Una regla puede eliminar archivos adjuntos de al menos { $kb } KB de correo con al menos { $days } días de antigüedad. Una breve nota reemplaza cada archivo adjunto.

retention-strip-denied = Su administrador desactiva la eliminación de archivos adjuntos.

retention-strip-done = Se eliminaron archivos adjuntos de { $count } mensajes, liberando { $size }.

retention-strip-inline-kept = Las imágenes que se muestran dentro del mensaje se conservan.

retention-strip-inline-removed = Las imágenes que se muestran dentro del mensaje también se eliminan.

retention-strip-limits = Elimina archivos adjuntos de al menos { $kb } KB del correo con al menos { $days } días de antigüedad. El original permanece en la Papelera durante { $undo } días.

retention-strip-preview = Los mensajes { $count } perderían los archivos adjuntos y se liberarían { $size }.

retention-strip-preview-estimate = Aproximadamente { $count } mensajes perderían sus archivos adjuntos, liberándose hasta { $size }.

retention-strip-skipped = { $count } reglas que eliminan archivos adjuntos se omitieron porque su administrador las desactivó.

retention-summary-stripAttachments = después de { $days } días, se eliminaron los archivos adjuntos grandes
