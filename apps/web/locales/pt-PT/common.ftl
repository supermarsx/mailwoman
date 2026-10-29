# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Cancelar
common-close = Fechar
common-save = Guardar
common-delete = Eliminar
common-remove = Remover
common-edit = Editar
common-add = Adicionar
common-back = Voltar
common-next = Seguinte
common-done = Concluído
common-apply = Aplicar
common-confirm = Confirmar
common-retry = Tentar novamente
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = Se o problema persistir, contacte o seu administrador.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = Esta aplicação não foi configurada para um servidor. Contacte o seu administrador.
common-copy = Copiar
common-search = Pesquisar
common-more = Mais
common-yes = Sim
common-no = Não

# -- Generic states ----------------------------------------------------------
common-loading = A carregar…
common-saving = A guardar…
common-empty = Ainda não há nada aqui
common-error = Ocorreu um erro
common-error-network = Não é possível contactar o servidor. Verifique a ligação e tente novamente.
common-offline = Sem ligação

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = A ligar…
common-conn-degraded = Atualizações em tempo real em pausa — a restabelecer a ligação
common-conn-offline = Está sem ligação — as alterações são sincronizadas quando voltar a ligar-se
common-conn-auth-expired = A sua sessão expirou — inicie sessão novamente
common-conn-reconnect = Voltar a ligar
common-conn-sign-in-again = Iniciar sessão novamente
common-conn-reconnected = Ligação restabelecida

# -- Toasts (components/toast/ToastRegion.tsx), in the webmail and the admin
#    console. The seconds are the time left on an undo window. -----------------
common-toast-close = Fechar notificação
common-toast-seconds = { $seconds } s

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = Nova mensagem
common-notify-no-subject = (sem assunto)

# -- Generic labels ----------------------------------------------------------
common-required = Obrigatório
common-optional = Opcional

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Abrir pastas e aplicações
common-nav-close = Fechar pastas e aplicações

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Anexos
common-attach-refresh = Atualizar anexos
common-attach-open-message = Abrir a mensagem que contém este anexo
common-attach-page-filter-note = Os filtros aplicam-se a esta página. Utilize «Seguinte» para percorrer mensagens mais antigas com anexos.
common-attach-pages = Páginas de anexos
common-attach-previous = Anterior
common-attach-next = Seguinte
common-attach-page = Página { $page }
common-attach-column-name = Ficheiro
common-attach-column-type = Tipo
common-attach-column-size = Tamanho
common-attach-column-sender = Remetente
common-attach-column-subject = Assunto
common-attach-column-date = Receção
common-attach-search = Pesquisar anexos
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filtrar por tipo
common-attach-loading = A carregar anexos…
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
common-password-show = Mostrar palavra-passe
common-password-hide = Ocultar palavra-passe

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Correio
common-module-calendar = Calendário
common-module-tasks = Tarefas
common-module-notes = Notas
common-module-contacts = Contactos
common-module-keys = Chaves

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = A carregar o visualizador…
common-viewer-loading-audio = A carregar o áudio…
common-viewer-loading-image = A carregar a imagem…
common-viewer-loading-video = A carregar o vídeo…
common-viewer-rendering-pdf = A apresentar o PDF…
common-viewer-pdf-failed = Não foi possível apresentar este PDF.
common-viewer-no-preview = Não há pré-visualização para este tipo de ficheiro.
common-viewer-unknown-type = tipo desconhecido
common-viewer-download = Transferir
common-viewer-attachments = Anexos
common-open-event = Abrir evento

common-message-preview-too-large = As pré-visualizações de anexos de mensagem estão limitadas a 4 MiB. Pode ainda transferir o anexo original.
common-message-preview-failed = Não foi possível pré-visualizar este anexo de mensagem. O ficheiro pode não ser suportado ou estar danificado, ou o serviço de pré-visualização pode estar indisponível. Pode ainda transferir o original.
common-message-preview-attachments = Ficheiros nesta mensagem
common-message-preview-download = Transferir ficheiro
common-message-preview-size-bytes = { $count } bytes
common-attach-cat-message = Mensagens / anexos do Outlook

common-error-retry-help = Esta secção está temporariamente indisponível. Tente novamente para a recarregar.
common-error-network-help = Verifique a ligação e tente novamente. O servidor pode estar temporariamente indisponível.

common-error-page = Não foi possível carregar esta página
common-error-page-help = Não foi possível transferir um recurso da página. Tente novamente. Se o problema persistir, recarregue a aplicação para obter a versão mais recente.
common-reload-app = Recarregar aplicação

common-attach-account-changed = A conta mudou durante o carregamento dos anexos. Abra novamente «Anexos» para a conta atual.
common-attach-search-account = Pesquisar em toda a conta
common-attach-scanning = A pesquisar… { $pages } páginas de mensagens verificadas · { $count } ficheiros encontrados
common-attach-scan-cancelled = Pesquisa cancelada · { $count } ficheiros encontrados em { $pages } páginas. Os resultados estão incompletos.
common-attach-scan-incomplete = Pesquisa interrompida · { $count } ficheiros encontrados. Tente novamente para pesquisar toda a conta.
common-attach-scan-limited = A mostrar os primeiros 1000 ficheiros correspondentes. Restrinja os filtros para encontrar ficheiros mais específicos.
common-attach-scan-complete = Pesquisa concluída · { $count } ficheiros encontrados em { $pages } páginas de mensagens
common-attach-browse-pages = Percorrer páginas
common-attach-cancel-search = Cancelar pesquisa
multiselect-selected = Selecionados: { $label }
multiselect-remove = Remover { $label }
multiselect-no-match = Nenhuma opção corresponde
multiselect-count = { $count ->
    [one] 1 selecionado
   *[other] { $count } selecionados
}
