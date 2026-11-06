# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: es · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Administración completa de todas las funciones, incluidos administradores, roles, seguridad, enrutamiento del correo y configuración del servicio. Reserve este rol para operadores de confianza responsables de todo el despliegue.
admin-role-purpose-domain_admin = Administración diaria de usuarios y dominios: crear, actualizar, desactivar o eliminar usuarios; gestionar cuotas, funciones de usuario, reglas de inicio de sesión del dominio, ajustes e identidad de marca; revocar sesiones y gestionar bloqueos. También incluye el diagnóstico del correo y las exportaciones de uso y de datos. No concede la gestión de roles de administrador ni cambios en los servidores de origen.
admin-role-purpose-helpdesk = Soporte de cuentas y clasificación de incidencias: consultar usuarios, actividad de auditoría y uso; cambiar indicadores de usuario, desactivar cuentas y revocar sesiones; ver bloqueos y ejecutar simulaciones de inicio de sesión. No concede la creación ni eliminación de usuarios, cambios de cuota ni escritura de ajustes generales.
admin-role-purpose-auditor = Inspección e informes: concede todas las capacidades que terminan en .read, además de la exportación de auditoría y de uso. No concede cambios de configuración ni modificaciones de cuentas. Los informes exportados pueden contener información administrativa sensible.
admin-role-purpose-security_officer = Operaciones de seguridad: gestionar las políticas de seguridad y MFA, la política de inicio de sesión y los bloqueos; desactivar usuarios, revocar sesiones o claves de API; consultar y exportar la actividad de auditoría; leer los registros de autenticación y ejecutar diagnósticos. La configuración de SSO es legible, pero este rol no concede la edición de SSO ni la gestión de roles de administrador.
admin-role-purpose-custom = Un rol personalizado con { $count } permisos seleccionados. Su autoridad proviene de la lista exacta de permisos de abajo, no de su nombre. Revise cada acción antes de asignarlo.
admin-role-purpose-scope = El ámbito de dominio se asigna cuando este rol se concede a un administrador. Las excepciones de permitir/denegar a nivel de cuenta pueden cambiar el acceso efectivo. Una descripción no concede permisos adicionales.
admin-role-purpose-editor = Elija las tareas que este administrador necesita realizar y luego seleccione los permisos correspondientes. El acceso de lectura, la exportación de datos y la modificación de registros son acciones distintas. Revise los identificadores exactos de las capacidades antes de guardar.
admin-role-purpose-selection = Estas descripciones explican las acciones seleccionadas. No amplían los permisos que se envían al servidor.
admin-role-purpose-read = Ver información de esta área.
admin-role-purpose-export = Exportar información de esta área.
admin-role-purpose-revoke = Revocar las sesiones o credenciales que nombra este permiso.
admin-role-purpose-delete = Eliminar los registros que abarca este permiso.
admin-role-purpose-disable = Desactivar las cuentas que abarca este permiso.
admin-role-purpose-create = Crear los registros que abarca este permiso.
admin-role-purpose-specific = Realizar la acción concreta que nombra esta capacidad; revise su identificador antes de concederla.
admin-role-purpose-description = Finalidad y responsabilidades
admin-role-purpose-description-help = Explique quién debe recibir este rol personalizado y qué tareas se encarga de realizar (hasta 1024 caracteres). Este texto no cambia sus permisos.
