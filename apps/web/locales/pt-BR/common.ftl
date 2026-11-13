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

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = Tentando novamente…
common-waiting-online = Aguardando uma conexão…
common-failure-offline = Você está offline. Reconecte-se e tente novamente.
common-failure-network = Não foi possível acessar o servidor.
common-failure-server = O servidor não respondeu a tempo.
common-failure-rate = Muitas solicitações em pouco tempo. Aguarde um instante e tente novamente.
common-failure-conflict = Isto foi alterado em outro lugar ao mesmo tempo. Recarregue e tente novamente.
common-failure-account = A conta conectada mudou. Recarregue a página.
common-failure-linked = Indisponível enquanto uma caixa de correio vinculada estiver aberta.
common-failure-refused = O servidor recusou a alteração.
common-new-version = Há uma nova versão do Mailwoman disponível. Ao fechar este aviso, você será lembrado novamente em uma hora.
common-new-version-reload = Recarregar agora
common-toast-close = Fechar notificação
common-toast-seconds = { $seconds } s
common-attach-refresh = Atualizar anexos
common-attach-open-message = Abrir a mensagem que contém este anexo
common-attach-page-filter-note = Os filtros se aplicam a esta página. Use Próxima para ver mensagens mais antigas com anexos.
common-attach-pages = Páginas de anexos
common-attach-previous = Anterior
common-attach-next = Próxima
common-attach-page = Página { $page }
common-attach-column-name = Arquivo
common-attach-column-type = Tipo
common-attach-column-size = Tamanho
common-attach-column-sender = Remetente
common-attach-column-subject = Assunto
common-attach-column-date = Recebido
common-pdf-preview = Pré-visualização de PDF
common-pdf-link-title = Abrir link deste PDF?
common-pdf-link-help = O destino será aberto em uma nova guia, fora do Mailwoman.
common-pdf-link-open = Abrir link
common-open-event = Abrir evento
common-message-preview-too-large = As pré-visualizações de anexos de mensagens são limitadas a 4 MiB. Você ainda pode baixar o anexo original.
common-message-preview-failed = Não foi possível pré-visualizar este anexo de mensagem. O arquivo pode não ser compatível, estar danificado ou o serviço de pré-visualização pode estar indisponível. Você ainda pode baixar o original.
common-message-preview-attachments = Arquivos dentro desta mensagem
common-message-preview-download = Baixar arquivo
common-message-preview-size-bytes = { $count } bytes
common-attach-cat-message = Mensagens / anexos do Outlook
common-error-retry-help = Esta seção está temporariamente indisponível. Tente novamente para recarregá-la.
common-error-network-help = Verifique sua conexão e tente novamente. O servidor pode estar temporariamente indisponível.
common-error-page = Não foi possível carregar esta página
common-error-page-help = Não foi possível baixar um recurso da página. Tente novamente. Se o problema continuar, recarregue o aplicativo para obter a versão mais recente.
common-reload-app = Recarregar aplicativo
common-new-version-reloading = Há uma nova versão disponível, recarregando…
common-attach-account-changed = A conta mudou durante o carregamento dos anexos. Abra Anexos novamente para a conta atual.
common-attach-search-account = Pesquisar em toda a conta
common-attach-scanning = Pesquisando… { $pages } páginas de mensagens verificadas · { $count } arquivos encontrados
common-attach-scan-cancelled = Pesquisa cancelada · { $count } arquivos encontrados em { $pages } páginas. Os resultados estão incompletos.
common-attach-scan-incomplete = Pesquisa interrompida · { $count } arquivos encontrados. Tente novamente para pesquisar em toda a conta.
common-attach-scan-limited = Exibindo os primeiros 1.000 arquivos correspondentes. Refine os filtros para encontrar arquivos mais específicos.
common-attach-scan-complete = Pesquisa concluída · { $count } arquivos encontrados em { $pages } páginas de mensagens
common-attach-browse-pages = Navegar pelas páginas
common-attach-cancel-search = Cancelar pesquisa
multiselect-selected = Selecionado: { $label }
multiselect-remove = Remover { $label }
multiselect-no-match = Nenhuma opção correspondente
common-notification-permission-title = Permitir notificações?
common-notification-permission-help = Receba notificações sobre novas mensagens neste dispositivo. Você pode alterar isso em Configurações.
common-notification-permission-allow = Permitir notificações
common-notification-permission-later = Agora não
common-notification-permission-error = Não foi possível ativar as notificações. Verifique as permissões e as configurações de notificação e tente novamente.
common-notifications-enabled = Notificações ativadas neste dispositivo.

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
    [one] 1 selecionado
   *[other] { $count } selecionados
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = O servidor está ocupado. Aguarde alguns segundos e tente novamente.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = Nenhuma correspondência

combobox-toggle = Mostrar opções para { $label }

common-attach-cat-archive = Arquivos

common-attach-cat-document = Documentos

common-attach-cat-spreadsheet = Planilhas

common-conn-reconnecting = Reconectando…

common-viewer-pdf-details = Detalhes técnicos

common-viewer-pdf-details-help = Inclua essas linhas ao relatar o problema. Eles não contêm conteúdo de documento.

timezone-group-other = Outro

timezone-use-browser = Use o fuso horário do navegador ({ $zone })

common-context-actions = Ações para { $name }
common-open = Abrir
common-new = Novo item
common-refresh = Atualizar
common-duplicate = Duplicar
common-copy-fingerprint = Copiar impressão digital
common-export-public-key = Exportar chave pública ou certificado
common-context-show = Mostrar
common-context-show-only = Mostrar apenas este calendário
common-context-color = Cor
common-context-import = Importar
common-context-export = Exportar
common-context-pinned = Itens fixados
common-context-unpin = Desafixar
common-context-my-day = Adicionar a Meu Dia
common-context-remove-my-day = Remover de Meu Dia
common-notify-new-count = Novas mensagens: { $count }
common-network-internal = Rede interna
common-module-notifications = Notificações
common-actions = Ações
