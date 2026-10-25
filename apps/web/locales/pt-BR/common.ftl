# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Cancelar
common-close = Fechar
common-save = Salvar
common-delete = Excluir
common-remove = Remover
common-edit = Editar
common-add = Adicionar
common-back = Voltar
common-next = Avançar
common-done = Concluído
common-apply = Aplicar
common-confirm = Confirmar
common-retry = Tentar novamente
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Se isso continuar acontecendo, entre em contato com o administrador.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Este app não foi configurado para um servidor. Entre em contato com o administrador.
common-copy = Copiar
common-search = Pesquisar
common-more = Mais
common-yes = Sim
common-no = Não

# -- Generic states ----------------------------------------------------------
common-loading = Carregando…
common-saving = Salvando…
common-empty = Nada aqui ainda
common-error = Algo deu errado
common-error-network = Não foi possível acessar o servidor. Verifique sua conexão e tente novamente.
common-offline = Você está offline

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Conectando…
common-conn-degraded = Atualizações em tempo real pausadas — reconectando
common-conn-offline = Você está offline — as alterações serão sincronizadas quando você se reconectar
common-conn-auth-expired = Sua sessão expirou — entre novamente
common-conn-reconnect = Reconectar
common-conn-sign-in-again = Entrar novamente
common-conn-reconnected = Reconectado

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Nova mensagem
common-notify-no-subject = (sem assunto)

# -- Generic labels ----------------------------------------------------------
common-required = Obrigatório
common-optional = Opcional

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Abrir pastas e apps
common-nav-close = Fechar pastas e apps

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Anexos
common-attach-search = Pesquisar anexos
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filtrar por tipo
common-attach-loading = Carregando anexos…
common-attach-empty = Nenhum anexo corresponde.
common-attach-cat-all = todos
common-attach-cat-image = imagem
common-attach-cat-pdf = pdf
common-attach-cat-text = texto
common-attach-cat-audio = áudio
common-attach-cat-video = vídeo
common-attach-cat-other = outro

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Mostrar senha
common-password-hide = Ocultar senha

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = E-mail
common-module-calendar = Agenda
common-module-tasks = Tarefas
common-module-notes = Notas
common-module-contacts = Contatos
common-module-keys = Chaves

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Carregando visualizador…
common-viewer-loading-audio = Carregando áudio…
common-viewer-loading-image = Carregando imagem…
common-viewer-loading-video = Carregando vídeo…
common-viewer-rendering-pdf = Renderizando PDF…
common-viewer-pdf-failed = Não foi possível renderizar este PDF.
common-viewer-no-preview = Sem pré-visualização para este tipo de arquivo.
common-viewer-unknown-type = tipo desconhecido
common-viewer-download = Baixar
common-viewer-attachments = Anexos
