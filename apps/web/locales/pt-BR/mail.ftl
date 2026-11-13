# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Configurações
mail-compose = Escrever
mail-nav-mailboxes = Pastas
mail-nav-attachments = Anexos
mail-nav-outbox = Caixa de saída
mail-nav-sharing = Compartilhar pasta
mail-nav-apps = Apps
mail-logout = Sair
mail-offline = Off-line
mail-module-loading = Carregando { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Pesquisar
mail-search-label = Pesquisar e-mails
mail-search-placeholder = Pesquisar e-mails — from:alice subject:invoice larger:1mb
mail-search-clear = Limpar

# -- Message list ------------------------------------------------------------
mail-list-label = Mensagens
mail-loading = Carregando mensagens…
mail-empty = Nenhuma mensagem
mail-unknown-sender = (remetente desconhecido)
mail-no-subject = (sem assunto)
mail-pinned = Fixada
mail-has-attachment = Tem anexo
mail-answered = Respondida
mail-forwarded = Encaminhada
mail-unread = Não lida
mail-flagged = Sinalizada
# Announced on a selected row; the row also shows a check mark.
mail-selected = Selecionada
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Mensagem { $pos } de { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } mensagem
   *[other] { $count } mensagens
}
mail-thread-expand = Expandir conversa, { $count ->
    [one] { $count } mensagem
   *[other] { $count } mensagens
}
mail-thread-collapse = Recolher conversa, { $count ->
    [one] { $count } mensagem
   *[other] { $count } mensagens
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Opções de exibição
mail-reading-pane = Painel de leitura
mail-reading-pane-right = À direita
mail-reading-pane-bottom = Embaixo
mail-reading-pane-off = Desativado

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Mensagens selecionadas
mail-bulk-count = { $count ->
    [one] { $count } selecionada
   *[other] { $count } selecionadas
}
mail-mark-read = Marcar como lida
mail-mark-unread = Marcar como não lida
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Sinalizar
mail-unflag-message = Remover sinalização
mail-move-to = Mover para a pasta
mail-move-to-placeholder = Mover para…
mail-select-all = Selecionar tudo
mail-clear-selection = Limpar seleção

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Filtro da caixa de entrada
mail-inbox-focused = Prioritárias
mail-inbox-other = Outras

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Abas abertas
mail-subtab-pin = Fixar { $title }
mail-subtab-unpin = Desafixar { $title }
mail-subtab-tearoff = Abrir { $title } em uma nova janela
mail-subtab-close = Fechar { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Fixar
mail-unpin = Desafixar
mail-snooze = Adiar
mail-snooze-menu = Adiar até
mail-snooze-later = Mais tarde hoje
mail-snooze-tomorrow = Amanhã
mail-snooze-next-week = Próxima semana
mail-unsnooze = Cancelar adiamento
mail-label = Marcador
mail-labels-menu = Marcadores
mail-flag = Marcar para acompanhamento
mail-clear-flag = Limpar acompanhamento
mail-archive = Arquivar
mail-delete = Excluir
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Mais ações

# -- Reader ------------------------------------------------------------------
mail-reader-label = Mensagem
mail-reader-empty = Selecione uma mensagem para ler
mail-back = Voltar
mail-reader-from = De: { $addr }
mail-reader-to = Para: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Ações da mensagem
mail-reply = Responder
mail-reply-all = Responder a todos
mail-forward = Encaminhar
mail-reply-failed = Não foi possível preparar a mensagem. Tente novamente.
mail-reply-unthreaded = Não foi possível ler o ID da mensagem original. Esta resposta não ficará na mesma conversa que ela.
mail-spam = Spam
mail-export = Exportar
mail-sweep-sender = Limpar remetente
mail-message-body = Corpo da mensagem
mail-no-content = Sem conteúdo
mail-sanitizing = Higienizando…
mail-attachments = Anexos
mail-attachment-unnamed = (sem nome)
mail-attachment-open = Anexo { $name }
mail-attachment-close = Fechar anexo
mail-attachment-loading = Carregando anexo…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Mensagem criptografada
mail-encrypted-note = Esta mensagem tem criptografia de ponta a ponta. Desbloqueie-a neste dispositivo para lê-la.
mail-key-passphrase = Frase secreta da chave
mail-decrypt = Descriptografar
mail-decrypting = Descriptografando…
mail-decrypt-no-key = Nenhuma chave privada disponível para descriptografar esta mensagem.
mail-decrypt-failed = Falha ao descriptografar

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = Em { $date }, { $name } escreveu:
mail-quote-attribution-undated = { $name } escreveu:
# The header block above a forwarded original.
mail-forward-heading = ---------- Mensagem encaminhada ----------
mail-forward-from = De: { $addr }
mail-forward-date = Data: { $date }
mail-forward-subject = Assunto: { $subject }
mail-forward-to = Para: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Escrever mensagem
mail-compose-title = Nova mensagem
mail-compose-title-reply = Responder
mail-compose-title-reply-all = Responder a todos
mail-compose-title-forward = Encaminhar
mail-compose-close = Fechar
mail-compose-from = De
mail-compose-from-default = Padrão
mail-compose-to = Para
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Sugestões de contatos
mail-compose-cc = Cc
mail-compose-bcc = Cco
mail-compose-show-cc-bcc = Adicionar Cc ou Cco
mail-compose-bcc-encrypted = Uma mensagem criptografada lista a chave de cada destinatário, então um destinatário em Cco ficaria visível para os demais. Remova os destinatários em Cco ou desative a criptografia.
mail-compose-plaintext-confirm-title = Enviar texto descriptografado sem criptografia?
mail-compose-plaintext-confirm-body = Esta mensagem cita texto de uma mensagem criptografada. Ela não está configurada para ser criptografada, então o texto citado seria enviado sem proteção.
mail-compose-plaintext-confirm-send = Enviar sem criptografia
mail-compose-plaintext-confirm-cancel = Voltar
mail-compose-subject = Assunto
mail-compose-body = Corpo
mail-compose-attachments = Anexos
mail-compose-remove-attachment = Remover { $name }
mail-compose-attach-nextcloud = Anexar do Nextcloud
mail-compose-close-nextcloud = Fechar Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Anexar um arquivo
mail-compose-uploading = Enviando…
mail-compose-upload-unavailable = O envio de arquivos não está disponível no momento.
mail-compose-upload-failed = Não foi possível enviar { $name }.
mail-compose-upload-too-large = { $name } tem { $size } MB. O tamanho máximo de envio é { $max } MB.
mail-compose-send-later = Enviar mais tarde
mail-compose-cancel = Cancelar
mail-compose-send = Enviar
mail-compose-schedule = Agendar
mail-compose-sending = Enviando…
mail-compose-encrypted-subject = Mensagem criptografada
mail-compose-dlp-blocked = O envio está bloqueado por uma regra de prevenção contra perda de dados (veja o aviso acima).
mail-compose-send-failed = Falha no envio
mail-compose-send-later-past = Escolha um horário de envio no futuro ou limpe o campo para enviar agora.
mail-compose-options = Opções de envio
mail-compose-request-receipt = Solicitar confirmação de leitura
mail-compose-request-receipt-hint = O programa de e-mail do destinatário pode perguntar a ele, enviar uma confirmação sem perguntar ou ignorar a solicitação.
mail-compose-track-opens = Registrar quando esta mensagem for aberta
mail-compose-track-opens-hint = Uma imagem de 1×1 carregada deste servidor é adicionada à mensagem. Muitos programas de e-mail bloqueiam imagens ou as carregam antecipadamente, então uma abertura registrada não prova que a mensagem foi lida, e a ausência de registro não prova que não foi. O destinatário não é avisado.
mail-compose-image-type = { $name } não pode ser colocada no texto. Use uma imagem PNG, JPEG, GIF ou WebP.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Você não está conectado.
mail-send-no-draft-folder = Esta conta não tem uma pasta para guardar o rascunho.
mail-send-no-recipient = Adicione pelo menos um destinatário.
mail-send-bad-recipient = { $count ->
    [one] Este não é um endereço de e-mail: { $addresses }
   *[other] Estes não são endereços de e-mail: { $addresses }
}
mail-send-later-past = O horário de envio não está no futuro. Nada foi enviado.
mail-send-tracking-failed = Não é possível registrar aberturas agora, então nada foi enviado. Desmarque “Registrar quando esta mensagem for aberta” para enviar sem isso.
mail-send-draft-refused = O servidor recusou esta mensagem: { $reason }. Nada foi enviado.
mail-send-submission-refused = O servidor se recusou a enviar esta mensagem: { $reason }. Nada foi enviado.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Desbloquear chave de assinatura
mail-compose-sign-unlock-note = Digite a frase secreta da sua chave para assinar esta mensagem. Ela fica desbloqueada apenas para esta janela de composição.
mail-compose-sign-unlock = Desbloquear
mail-compose-sign-unlocking = Desbloqueando…
mail-compose-sign-no-key = Nenhuma chave de assinatura disponível neste dispositivo.
mail-compose-sign-unlock-failed = Não foi possível desbloquear a chave de assinatura.
mail-compose-sign-unlock-required = Desbloqueie sua chave de assinatura para enviar esta mensagem assinada.

# Rich-text editor (W1)
mail-compose-format-plain = Texto simples
mail-compose-format-rich = Texto formatado
mail-compose-rt-toolbar = Formatação
mail-compose-rt-bold = Negrito
mail-compose-rt-italic = Itálico
mail-compose-rt-underline = Sublinhado
mail-compose-rt-strike = Tachado
mail-compose-rt-bullet = Lista com marcadores
mail-compose-rt-ordered = Lista numerada
mail-compose-rt-quote = Citação
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Aplicar
mail-compose-rt-image = Inserir uma imagem

# Signature picker (W12)
mail-compose-signature = Assinatura
mail-compose-signature-none = Adicionar uma assinatura…

# Recall (W10)
mail-compose-recall = Recolher
mail-compose-recall-action = Recolher
mail-compose-recall-holding = Retida, aguardando envio
mail-compose-recall-scheduled = Agendada para { $when }

# Drafts drawer (W9)
mail-compose-drafts = Rascunhos
mail-compose-drafts-empty = Nenhum rascunho salvo
mail-compose-drafts-delete = Excluir rascunho
mail-compose-drafts-no-recipient = (sem destinatário)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Caixa de saída
mail-outbox-refresh = Atualizar
mail-outbox-empty = Nada aguardando envio
mail-outbox-send-now = Enviar agora
mail-outbox-cancel = Cancelar
mail-outbox-scheduled = Agendada
mail-outbox-holding = Envio em breve
mail-outbox-sent = Enviada
mail-outbox-canceled = Cancelada
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Retida
mail-outbox-failed = Não enviada
mail-outbox-release = Liberar
mail-outbox-discard = Descartar
mail-outbox-held-note = Retida. Não será enviada até ser liberada.
mail-outbox-held-note-api-key = Retida — criada pela chave de API { $name }. Não será enviada até ser liberada.
mail-outbox-held-note-oauth = Retida — criada pelo app conectado { $name }. Não será enviada até ser liberada.
mail-outbox-origin-api-key = Criada pela chave de API { $name }
mail-outbox-origin-oauth = Criada pelo app conectado { $name }
mail-outbox-to = Para { $recipients }
mail-outbox-no-subject = (sem assunto)
mail-outbox-attempts = Tentativas: { $count }. Último erro: { $error }
mail-outbox-error = Erro: { $error }
mail-outbox-toast-canceled = Envio cancelado
mail-outbox-toast-discarded = Descartada. A mensagem não foi enviada.
mail-outbox-toast-sent = Enviada
mail-outbox-toast-retrying = Ainda não enviada: { $error }. Haverá uma nova tentativa.
mail-outbox-toast-failed = Não enviada: { $error }
mail-outbox-toast-not-released = Não foi possível enviar: { $error }
mail-outbox-toast-not-canceled = Não foi possível cancelar: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Conexão perdida — tentando novamente…
mail-net-back = Conectado novamente
mail-queue-sent = { $count ->
    [one] { $count } alteração pendente enviada
   *[other] { $count } alterações pendentes enviadas
}
mail-queue-failed = { $count ->
    [one] { $count } alteração pendente falhou
   *[other] { $count } alterações pendentes falharam
}
mail-queue-notice-label = Alterações que não puderam ser aplicadas
mail-queue-notice-title = { $count ->
    [one] { $count } alteração feita off-line não pôde ser aplicada
   *[other] { $count } alterações feitas off-line não puderam ser aplicadas
}
mail-queue-kind-send = Enviar “{ $subject }”
mail-queue-kind-send-untitled = Enviar uma mensagem sem assunto
mail-queue-kind-move = Mover uma mensagem
mail-queue-kind-flag = Alterar um marcador ou sinalização
mail-queue-kind-draft = Salvar um rascunho
mail-queue-kind-pim = Alteração de agenda, tarefa, nota ou contato
mail-queue-reason = Motivo: { $reason }
mail-queue-reason-unknown = O servidor não informou o motivo.
mail-queue-retry = Tentar novamente
mail-queue-discard = Descartar
mail-queue-retry-item = Tentar novamente: { $what }
mail-queue-discard-item = Descartar: { $what }
mail-queue-discarded = Descartada. A alteração não foi aplicada.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Limpar mensagens
mail-sweep-title = Limpar { $sender }
mail-sweep-all = Excluir todas deste remetente
mail-sweep-keep-latest = Manter a mais recente e excluir as demais
mail-sweep-older-than = Excluir as mais antigas que N dias
mail-sweep-block = Excluir todas e bloquear este remetente
mail-sweep-days = Dias
mail-sweep-count = { $count ->
    [one] { $count } mensagem será movida para a Lixeira
   *[other] { $count } mensagens serão movidas para a Lixeira
}
mail-sweep-cancel = Cancelar
mail-sweep-run = Limpar { $count }
mail-sweeping = Limpando…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Dispensar
mail-undo-action = Desfazer
mail-undo-send-cancel = Cancelar
mail-undo-failed = Não foi possível desfazer

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Marcador adicionado
mail-toast-label-removed = Marcador removido
mail-toast-pinned = Mensagem “{ $target }” fixada
mail-toast-unpinned = Mensagem “{ $target }” desafixada
mail-toast-snoozed = Adiada
mail-toast-follow-up-set = Acompanhamento definido
mail-toast-follow-up-cleared = Acompanhamento removido
mail-toast-archived = Arquivada
mail-toast-trashed = Movida para a Lixeira
mail-toast-marked-spam = Marcada como spam
mail-toast-moved-to = Movida para { $folder }
mail-toast-moved = Movida para a pasta
mail-toast-archived-count = { $count } mensagens arquivadas
mail-toast-trashed-count = { $count } mensagens movidas para a Lixeira
mail-toast-marked-spam-count = { $count } mensagens marcadas como spam
mail-toast-moved-count = { $count } mensagens movidas
mail-toast-moved-to-count = { $count } mensagens movidas para { $folder }
mail-toast-marked-read = { $count ->
    [one] Marcada como lida
   *[other] { $count } mensagens marcadas como lidas
}
mail-toast-marked-unread = { $count ->
    [one] Marcada como não lida
   *[other] { $count } mensagens marcadas como não lidas
}
mail-toast-flagged = { $count ->
    [one] Sinalizada
   *[other] { $count } mensagens sinalizadas
}
mail-toast-unflagged = { $count ->
    [one] Sinalização removida
   *[other] Sinalização removida de { $count } mensagens
}
mail-toast-move-failed = Não foi possível mover as mensagens. Nada foi movido.
mail-toast-update-failed = Não foi possível alterar as mensagens. Nada foi alterado.
mail-toast-update-refused = { $count ->
    [one] O servidor recusou a alteração de uma mensagem
   *[other] O servidor recusou a alteração de { $count } mensagens
}
mail-toast-no-archive-folder = Nenhuma pasta Arquivo
mail-toast-no-trash-folder = Nenhuma pasta Lixeira
mail-toast-no-spam-folder = Nenhuma pasta Spam
mail-toast-sweep-nothing = Nada para limpar
mail-toast-swept = { $count ->
    [one] { $count } mensagem limpa
   *[other] { $count } mensagens limpas
}
mail-toast-load-more-failed = Não foi possível carregar mais mensagens
mail-toast-export-nothing = Nada para exportar
mail-toast-exported = .eml exportado
mail-toast-export-failed = Falha na exportação
mail-toast-send-queued = Na fila — será enviada quando a conexão voltar
mail-toast-tracking-not-bound = A mensagem foi enviada, mas as aberturas dela não serão registradas.
mail-toast-send-scheduled = Envio agendado
mail-toast-sent = Mensagem enviada
mail-toast-send-canceled = Envio cancelado

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Faixa de opções
mail-ribbon-tab-home = Início
mail-ribbon-tab-view = Exibir
mail-ribbon-tab-folder = Pasta
mail-ribbon-expand = Expandir
mail-ribbon-collapse = Recolher
mail-ribbon-group-new = Novo
mail-ribbon-compose = Escrever
mail-ribbon-group-session = Sessão
mail-ribbon-logout = Sair
mail-ribbon-group-theme = Tema
mail-ribbon-group-density = Densidade
mail-ribbon-group-settings = Configurações
mail-ribbon-more = Mais…
mail-ribbon-group-folders = Pastas
mail-density-compact = compacta
mail-density-cozy = confortável
mail-density-relaxed = espaçosa

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Nível de segurança da mensagem
mail-maxsec-full = Completo
mail-maxsec-full-hint = HTML higienizado com imagens e estilos.
mail-maxsec-no-media = Sem mídia
mail-maxsec-no-media-hint = HTML higienizado sem nenhuma imagem ou mídia.
mail-maxsec-plain-text = Texto simples
mail-maxsec-plain-text-hint = Nenhum HTML renderizado — a mensagem é exibida apenas como texto simples.
mail-feature-off = Desativado pelas suas configurações ou pelo administrador
mail-send-disabled = O envio está desativado pelas suas configurações ou pelo administrador. Você ainda pode salvar um rascunho.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Eventos
mail-events-upcoming = Somente próximos
mail-events-filter = Filtro de eventos
mail-loading-more = Carregando mensagens mais antigas…
mail-event-state-invitation = Convite
mail-event-state-updated = Atualizado
mail-event-state-cancelled = Cancelado
mail-event-state-reply = Resposta
mail-event-state-counter = Novo horário proposto
mail-event-state-event = Evento
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Mostrar mais ações
mail-remote-moved = Esta mensagem foi movida para { $folder } a partir de outro dispositivo ou aplicativo. Você ainda está lendo a cópia que abriu.
mail-remote-moved-unknown = Esta mensagem foi movida a partir de outro dispositivo ou aplicativo. Você ainda está lendo a cópia que abriu.
mail-remote-deleted = Esta mensagem foi excluída a partir de outro dispositivo ou aplicativo. Você pode continuar lendo esta cópia até fechá-la.
mail-remote-dismiss = Dispensar
mail-reply-draft = Resposta em andamento
mail-reply-draft-open = Abrir rascunho
mail-draft-load-failed = Não foi possível abrir o rascunho. Tente novamente.
mail-reply-images-omitted =
    { $count ->
        [one] Uma imagem não pôde ser incluída na resposta.
       *[other] { $count } imagens não puderam ser incluídas na resposta.
}
mail-export-forbidden = Seu administrador desativou a exportação de mensagens para esta conta.
mail-export-open-failed = Não foi possível abrir as opções de exportação. Feche uma aba do espaço de trabalho e tente novamente.
mail-headers-title = Cabeçalhos brutos da mensagem
mail-headers-action-help = Veja os cabeçalhos originais da mensagem, incluindo detalhes de entrega e autenticação.
mail-headers-help = Cabeçalhos originais da mensagem. As linhas dobradas são preservadas; apenas os cabeçalhos são mostrados aqui.
mail-headers-loading = Carregando os cabeçalhos da mensagem…
mail-headers-failed = Não foi possível carregar os cabeçalhos da mensagem. Verifique sua conexão e tente novamente.
mail-headers-copy = Copiar cabeçalhos
mail-headers-copied = Cabeçalhos copiados.
mail-headers-copy-failed = Não foi possível copiar os cabeçalhos. Selecione o texto para copiá-lo manualmente.
mail-attachment-failed = Não foi possível baixar este anexo. Tente novamente quando houver conexão.
mail-compose-rt-rule = Linha horizontal
mail-compose-signature-new = Novas mensagens
mail-compose-signature-replies = Respostas
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Tamanho do corpo
mail-editor-body-size-help = Tamanho do corpo atual em UTF-8, incluindo a marcação HTML e as imagens incorporadas. Os cabeçalhos da mensagem e os anexos separados não estão incluídos.
mail-outbox-load-failed = Não foi possível verificar a Caixa de saída. Tente Atualizar depois de verificar sua conexão.
mail-outbox-unavailable = Caixa de saída indisponível. Use Atualizar para tentar novamente.
mail-maxsec-view = Exibição: { $mode }
mail-maxsec-menu = Modo de exibição
mail-toast-tab-pinned = Aba “{ $target }” fixada
mail-toast-tab-unpinned = Aba “{ $target }” desafixada
mail-workspace-message = Mensagem
mail-opened-announcement = Aberta: { $subject }
mail-workspace-layout = Layout do espaço de trabalho
mail-workspace-tabs = Abas
mail-workspace-side-by-side = Lado a lado
mail-workspace-stack = Empilhado
mail-workspace-refresh = Atualizar pasta
mail-workspace-load-error = Não foi possível carregar esta mensagem. Feche a aba e tente novamente.
mail-resize-folders = Redimensionar a barra lateral de pastas
mail-resize-list = Redimensionar a lista de mensagens
mail-resize-tiles = Redimensionar os painéis do espaço de trabalho
mail-show-folders = Mostrar a barra lateral de pastas
mail-hide-folders = Ocultar a barra lateral de pastas
mail-compact-folders = Ícones de pasta compactos
mail-toggle-list = Mostrar ou ocultar a lista de mensagens
mail-toggle-reader = Mostrar ou ocultar o painel de leitura
mail-folder-counts = { $unread } não lidas, { $total } mensagens
mail-print = Imprimir / Salvar como PDF
mail-go-to-top = Ir para o topo
mail-print-help = Imprima esta mensagem ou escolha Salvar como PDF na caixa de diálogo de impressão. As imagens externas continuam bloqueadas.
mail-file-details = Detalhes e impressão digital do arquivo
mail-file-type = Tipo de arquivo
mail-file-extension = Extensão
mail-file-detected = Assinatura detectada
mail-file-unknown = Nenhuma assinatura reconhecida
mail-file-size = Tamanho
mail-file-size-bytes = { $count } bytes
mail-file-hash-unavailable = Impressão digital indisponível (exige um navegador seguro e um arquivo de até 64 MiB).
mail-file-fingerprint-help = Uma impressão digital identifica exatamente esses bytes. Ela não prova que o arquivo é seguro nem quem o criou.
mail-file-warning-executable = Este arquivo pode executar programas ou alterar configurações do sistema. Baixe-o somente se confiar na origem.
mail-file-warning-macro = Este documento pode conter macros. Não ative macros sem ter verificado o remetente e a finalidade.
mail-file-warning-disguisedName = O nome do arquivo pode disfarçar sua extensão real. Verifique os detalhes do arquivo antes de baixar.
mail-file-warning-typeMismatch = A assinatura detectada do arquivo não corresponde ao tipo declarado. A pré-visualização automática foi bloqueada.
mail-file-warning-activeDocument = Este formato pode conter conteúdo ativo. A pré-visualização é isolada; abrir uma cópia baixada pode se comportar de forma diferente.
mail-file-preview-blocked = A pré-visualização automática está desativada para este tipo de arquivo ou para tipos incompatíveis.
mail-quote-header-from = De:
mail-quote-header-sent = Enviada:
mail-quote-header-to = Para:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Assunto:
mail-spam-score = Análise de spam
mail-spam-score-loading = Verificando os cabeçalhos de spam…
mail-spam-score-unavailable = Pontuação de spam indisponível
mail-spam-score-unverified = Informado pelos cabeçalhos da mensagem, sem verificação independente. Cabeçalhos fornecidos pelo remetente podem ser forjados.
mail-spam-score-positive = O cabeçalho indica spam.
mail-spam-score-negative = O cabeçalho indica um resultado de não spam.
mail-spam-score-unknown = Nenhum veredito foi fornecido com esta pontuação.
mail-external-recipients = Destinatários externos
mail-external-count = { $count } destinatário(s) fora dos domínios confiáveis
mail-external-help = Verifique os destinatários em Para, Cc e Cco antes de enviar informações confidenciais. A confiança se baseia na identidade de envio selecionada e na sua política de domínios efetiva.
mail-external-reviewed = Verifiquei estes destinatários externos.
mail-external-confirm-required = Revise e confirme os destinatários externos antes de enviar.
mail-quick-reply = Resposta rápida
mail-quick-reply-help = Oferece uma aba de resposta compacta com os destinatários já preenchidos. Desativado por padrão. As verificações de segurança normais e as opções de envio continuam valendo.
mail-quick-reply-expand = Editar destinatários e assunto
mail-attachment-policy-loading = Verificando a política de anexos…
mail-attachment-policy-unavailable = Não foi possível verificar a política de anexos. Tente novamente antes de abrir este arquivo.
mail-attachment-policy-blocked = Seu administrador bloqueou este anexo ou o download do original.
mail-attachment-policy-warning = Seu administrador recomenda cautela com este anexo. Continue somente se confiar na origem.
mail-attachment-policy-acknowledge = Entendi — continuar
mail-attachment-policy-no-preview = As pré-visualizações de anexos estão desativadas pelo seu administrador. Os downloads dos originais ainda podem estar disponíveis.
mail-time-sent = Enviada
mail-time-received = Recebida
mail-time-unknown = Não informado
mail-time-comparison = Horário em { $zone }
mail-time-privacy = O fuso de comparação é uma escolha sua. Os carimbos de data e hora das mensagens não identificam a localização de ninguém.
mail-time-event-comparison = Comparação do horário do evento
mail-time-ambiguous = Este horário local ocorre duas vezes quando os relógios mudam. Os dois instantes possíveis são mostrados; verifique o deslocamento pretendido do evento antes de salvar.
mail-bulk-add-label = Adicionar marcador
mail-select-messages = Selecionar mensagens
mail-bulk-remove-label = Remover marcador
mail-mark-spam = Marcar como spam
mail-bulk-forward = Encaminhar as mensagens selecionadas como anexos
mail-bulk-download = Baixar as mensagens selecionadas como mbox
mail-bulk-forward-subject = Mensagens encaminhadas ({ $count })
mail-bulk-transfer-failed = Não foi possível preparar as mensagens selecionadas. Selecione até 100 mensagens, com no máximo 32 MiB no total, e verifique as permissões de exportação da sua conta.
mail-toast-labels-added = Marcador { $label } adicionado a { $count } mensagens
mail-toast-labels-removed = Marcador { $label } removido de { $count } mensagens
mail-action-help-reply = Responder ao remetente em uma nova aba de redação.
mail-action-help-reply-all = Responder ao remetente e aos demais destinatários em uma nova aba de redação.
mail-action-help-forward = Revisar e encaminhar esta mensagem aos destinatários que você escolher.
mail-action-help-pin = Manter esta mensagem fixada para acesso fácil ou remover a fixação.
mail-action-help-read = Alterar se esta mensagem está marcada como lida. Você pode desfazer a alteração.

# -- Machine translation (2026-11): added to match en ---------------------
mail-action-help-flag = Definir ou remover a sinalização da mensagem para acompanhamento.
mail-action-help-archive = Mover esta mensagem para a pasta de arquivo configurada.
mail-action-help-delete = Mover esta mensagem para a lixeira. Você pode desfazer a movimentação.
mail-action-help-spam = Marcar esta mensagem como lixo eletrônico e movê-la para a pasta de spam configurada.
mail-action-help-export = Abrir a página de exportação para baixar esta mensagem em um formato de sua escolha.
mail-action-help-sweep = Revise as outras mensagens deste remetente e escolha o que limpar (excluir tudo, manter a mais recente, excluir as mais antigas ou bloquear o remetente) antes de aplicar qualquer coisa.
mail-empty-trash = Esvaziar a lixeira
mail-trash-help = Revise e exclua permanentemente as mensagens que estão na Lixeira. Isso não pode ser desfeito.
mail-trash-preparing = Preparando um instantâneo das mensagens na Lixeira…
mail-trash-prepare-failed = Não foi possível preparar a pré-visualização da Lixeira. Verifique sua conexão e abra-a novamente. Nenhuma mensagem foi excluída.
mail-trash-confirm = Excluir permanentemente { $count } mensagens da Lixeira? Isso não pode ser desfeito. As mensagens que chegarem depois desta pré-visualização são excluídas da operação; as mensagens movidas para fora da Lixeira são ignoradas.
mail-trash-limit = Esta operação inclui até { $count } mensagens. Mais mensagens permanecem na Lixeira; revise outra operação depois para removê-las.
mail-trash-permanent = Excluir permanentemente
mail-trash-stop = Parar após a solicitação atual
mail-trash-progress = Excluídas: { $deleted }; ignoradas: { $skipped } mensagens que não estão mais na Lixeira; restantes nesta pré-visualização: { $remaining }.
mail-trash-stopped = Exclusão interrompida. Exclusão confirmada: { $deleted }; ignoradas: { $skipped }; restantes: { $remaining }; resultado desconhecido: { $uncertain }. Verifique a Lixeira antes de iniciar outra operação.
mail-links-title = Links da mensagem
mail-links-review = Revisar links ({ $count })
mail-links-help = Os links da mensagem só abrem depois que você revisa o destino aqui. Nenhum destino é buscado automaticamente.
mail-links-inert = Use Revisar links acima da mensagem para inspecionar e abrir este destino.
mail-links-limit = Os 100 primeiros destinos distintos com suporte estão listados. Os links adicionais permanecem inativos na mensagem.
mail-links-destination = Destino a abrir
mail-links-external = Este destino vem do remetente da mensagem. Abri-lo sai do Mailwoman; revise o endereço antes de continuar.
mail-links-open = Abrir em uma nova aba
mail-links-compose = Revisar uma nova mensagem
mail-links-warning-http = Este endereço usa HTTP sem criptografia.
mail-links-warning-address = Este destino usa um endereço IP em vez de um nome de domínio.
mail-links-warning-international = Este domínio contém um nome internacionalizado. Verifique com atenção a grafia em ASCII.
mail-links-warning-mismatch = O endereço web visível indica um host diferente do destino real.
mail-links-warning-direction = O texto do link contém caracteres que mudam a direção do texto e podem disfarçar o endereço.
mail-outbox-uncertain = Status de entrega desconhecido
mail-outbox-uncertain-note = O envio começou, mas a entrega não pôde ser confirmada. Esta mensagem não será reenviada automaticamente. Verifique Enviados ou confirme com o destinatário antes de enviar novamente.
mail-conversation-title = Ver conversa
mail-conversation-help = Navegue pelas mensagens de conversa disponíveis desta conta em todas as pastas. Abrir uma mensagem não altera a pasta selecionada.
mail-conversation-count = Mostrando { $loaded } de { $total } mensagens da conversa.
mail-conversation-other-folder = Outra pasta
mail-conversation-loading = Carregando a conversa…
mail-conversation-missing = { $count } mensagem(ns) não está(ão) mais disponível(is).
mail-conversation-error = Não foi possível carregar a conversa. Tente novamente.
mail-conversation-more = Carregar mais mensagens
mail-conversation-limit = Esta exibição é limitada a { $count } mensagens.
mail-compose-request-delivery = Solicitar notificações de entrega
mail-compose-request-delivery-hint = Pede aos servidores de e-mail receptores que informem entrega, falha ou atraso. Isso não solicita nem comprova que alguém leu a mensagem. Os relatórios devolvidos incluem cabeçalhos, não o corpo completo.
mail-compose-delivery-unavailable = As notificações de entrega estão indisponíveis ou desativadas pelo seu administrador para esta conta. Uma notificação solicitada explicitamente não será removida silenciosamente.
mail-compose-account-changed = Esta redação pertence a outro login. Entre na conta correta e reabra o rascunho.
mail-context-open-message = Abrir mensagem
mail-context-open-folder = Abrir pasta
mail-context-download = Baixar mensagem
mail-context-refresh = Atualizar pasta
mail-context-folder-settings = Configurações da pasta
mail-context-failed = Não foi possível concluir a ação. Tente novamente.
mail-sharing-unavailable = O compartilhamento de pastas está indisponível ou você não tem mais permissão para gerenciá-lo.
mail-trash-cancelling = Parando após a solicitação atual. Exclusão confirmada: { $deleted }; ignoradas: { $skipped }; restantes: { $remaining }. As exclusões já aceitas não podem ser desfeitas.
mail-trash-already-running = A exclusão da Lixeira já está em andamento. Use a notificação de progresso para interromper os próximos lotes.
mail-context-search-sender = Pesquisar este remetente
mail-context-search-subject = Pesquisar o mesmo assunto
mail-drag-too-many = Arraste até { $count } mensagens por vez. Selecione um grupo menor antes de mover.
mail-subtab-reorder-help = Arraste para reordenar as abas ou pressione Ctrl+Shift+Seta para a esquerda ou para a direita na aba ativa.
mail-compose-ribbon = Faixa de opções do editor
mail-compose-ribbon-format = Formatar texto
mail-compose-ribbon-insert = Inserir
mail-compose-font = Fonte
mail-compose-font-size = Tamanho da fonte
mail-compose-font-inherit = Padrão
mail-compose-undo = Desfazer
mail-compose-redo = Refazer
mail-compose-undo-tip = Desfazer ({ $shortcut })
mail-compose-redo-tip = Refazer ({ $shortcut })
mail-workspace-limit = Feche uma aba antes de abrir outra. É possível manter até 32 abas.
mail-compose-private-storage-unavailable = A recuperação local de rascunhos não está disponível nesta sessão. Mantenha este editor aberto até enviar ou copiar sua mensagem.
mail-offline-storage-unavailable = As alterações offline ficam indisponíveis até que esta sessão tenha verificado a autoridade de armazenamento. Reconecte-se e entre novamente.
mail-draft-status-idle = Ainda não salvo na conta
mail-draft-status-pending = Alterações não salvas
mail-draft-status-saving = Salvando em Rascunhos…
mail-draft-status-saved = Salvo em Rascunhos da conta
mail-draft-status-error = O rascunho não foi salvo. Sua cópia de recuperação local continua disponível.
mail-draft-status-uncertain = A confirmação do salvamento não foi recebida. Verifique Rascunhos antes de criar outra cópia; o salvamento automático está pausado.
mail-draft-status-conflict = Este rascunho foi alterado em outro dispositivo. Reabra a cópia da conta antes de continuar.
mail-draft-status-unsupported = No momento, esta conta não consegue confirmar salvamentos em sua pasta Rascunhos.
mail-draft-protected = O conteúdo citado criptografado ou descriptografado não é salvo como texto simples. Qualquer rascunho anterior da conta permanece até que você o descarte explicitamente.
mail-draft-save-now = Salvar em Rascunhos
mail-draft-save-failed = Não foi possível salvar este rascunho na sua conta. Verifique o editor antes de fechá-lo.
mail-draft-uncertain = O rascunho pode ter sido salvo. As novas tentativas automáticas estão pausadas; verifique sua pasta Rascunhos.
mail-draft-conflict = Outro dispositivo alterou este rascunho. Suas edições não sobrescreveram a cópia dele.
mail-draft-cleanup-pending = O novo rascunho foi salvo, mas não foi possível remover a cópia anterior. Revise Rascunhos antes de salvar novamente.
mail-draft-account-list = Rascunhos da conta
mail-draft-loading = Carregando os rascunhos da conta…
mail-draft-close-recovery = Fechar e manter apenas a recuperação local
mail-draft-discard-account = Descartar
mail-draft-discard-pending = Não foi possível confirmar a remoção do rascunho. O registro dele na conta permanece; tente Descartar novamente depois de revisar a pasta Rascunhos.
mail-draft-limit = Esta conta já tem 100 rascunhos salvos. Descarte um rascunho de conta indesejado antes de salvar um novo.
mail-draft-quota-help = São mantidos até 100 rascunhos de conta. Descartar remove a cópia confirmada no servidor antes de liberar a vaga.
mail-draft-discard-confirm = Confirmar a remoção do rascunho
mail-draft-discard-confirm-detail = Remover permanentemente “{ $subject }” dos Rascunhos da conta? Fechar o editor, em vez disso, mantém o rascunho.
mail-draft-discard-title = Descartar este rascunho?
mail-draft-keep-editing = Continuar editando
mail-draft-discard-action = Descartar
mail-draft-close-title = Descartar alterações não salvas?
mail-draft-close-protected = Esta mensagem contém conteúdo criptografado ou descriptografado, por isso não é salva como rascunho. Fechá-la descarta estas alterações. Um rascunho anterior da conta, se houver, permanece em Rascunhos.
mail-draft-close-unstored = Os rascunhos não podem ser salvos nesta sessão, por isso fechar esta mensagem a descarta.
mail-workspace-view-mode = Disposição das exibições
mail-workspace-tabs-help = Mostrar uma exibição ativa e manter as outras abas abertas.
mail-workspace-side-by-side-help = Mostrar duas exibições selecionadas lado a lado. Arraste uma aba para qualquer exibição para substituí-la.
mail-workspace-stack-help = Mostrar duas exibições selecionadas uma acima da outra. Arraste uma aba para qualquer exibição para substituí-la.
mail-workspace-masonry-side-by-side = Mosaico lado a lado
mail-workspace-masonry-side-by-side-help = Organiza todas as exibições abertas em colunas escalonadas. Telas pequenas mostram a aba ativa.
mail-workspace-masonry-stack = Mosaico empilhado
mail-workspace-masonry-stack-help = Organiza todas as exibições abertas em linhas escalonadas. Telas pequenas mostram a aba ativa.
mail-workspace-view-first = Primeira exibição
mail-workspace-view-second = Segunda exibição
mail-workspace-view-empty = Arraste uma aba aberta até aqui para escolher esta exibição.
mail-popup-blocked = Permita pop-ups para o Mailwoman para destacar esta aba. Sua aba permanece aqui.
mail-popup-unavailable = Não foi possível carregar a janela destacada. Sua aba e as alterações não salvas permanecem na janela principal. Tente novamente depois de atualizar o aplicativo.
mail-popup-reattach = Reanexar à janela principal
mail-popup-action-reattached = Retornou à janela principal para concluir esta ação.
mail-popup-detached = Esta aba está aberta em uma janela separada. Fechar essa janela a traz de volta para cá.
mail-compose-from-custom = Outro endereço autorizado…
mail-compose-from-alias = Endereço De desta mensagem
mail-compose-from-alias-help = Use um alias a partir do qual você tem autorização para enviar. Isso altera apenas este rascunho; a autorização é verificada antes do envio.
mail-compose-from-refused = Este endereço não está autorizado para envio a partir desta conta.
mail-compose-from-unavailable = Não foi possível verificar este endereço de remetente. Sua mensagem não foi enviada.
mail-compose-from-invalid = Informe um endereço de e-mail válido para o remetente.
mail-compose-from-toggle = Mostrar ou ocultar o remetente desta mensagem
mail-compose-ribbon-security = Segurança
mail-compose-ribbon-delivery = Entrega
mail-compose-ribbon-recipients = Destinatários
mail-compose-ribbon-message-type = Tipo de mensagem
mail-compose-recall-help = Cancele mensagens ainda retidas para desfazer ou para entrega agendada. As mensagens já entregues não podem ser recuperadas.
mail-compose-recall-loading = Verificando as mensagens na fila…
mail-compose-recall-empty = Não há mensagens na fila disponíveis para cancelar.

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = Não foi possível confirmar o status da fila. Feche e reabra este painel para atualizá-lo antes de outra ação.
mail-compose-importance = Importância
mail-compose-importance-high = Alta
mail-compose-importance-normal = Normal
mail-compose-importance-low = Baixa
mail-insert-account-title = Inserir item da conta
mail-insert-account-help = Anexe uma exportação de calendário, um cartão de visita de contato ou uma mensagem original desta conta.
mail-insert-account-close = Fechar o seletor de itens da conta
mail-insert-account-kind = Tipo de item
mail-insert-account-event = Evento de calendário (.ics)
mail-insert-account-calendar = Calendário (.ics)
mail-insert-account-contact = Cartão de visita do contato (.vcf)
mail-insert-account-message = Mensagem original (.eml)
mail-insert-account-search = Filtrar os itens listados
mail-insert-account-selection = Selecione um item
mail-insert-account-scope = Somente os itens da conta atual são listados. As mensagens originais seguem a política de exportação do servidor.
mail-insert-account-limited = Mostrando até 100 itens. O filtro pesquisa somente nesta lista.
mail-insert-account-unavailable = Nenhuma exportação de conta com suporte está disponível.
mail-insert-account-empty = Nenhum item correspondente.
mail-insert-account-attach = Anexar o item selecionado
mail-insert-account-loading = Carregando o item da conta…
mail-insert-account-error = Não foi possível anexar o item. Verifique seu acesso e a política de exportação e tente novamente. Calendários grandes podem exigir a exportação de eventos individuais.
mail-compose-html-source = Código-fonte HTML
mail-editor-clear-format = Limpar formatação
mail-compose-format-html = HTML
mail-compose-html-visual = Editor visual
mail-compose-html-help = Edite o HTML diretamente. Somente a formatação de e-mail com suporte é salva e enviada; scripts, imagens externas e marcação insegura são removidos.
mail-compose-html-processing = Preparando HTML seguro…
mail-compose-html-failed = Não foi possível preparar o HTML. Edite-o ou tente novamente antes de salvar ou enviar.
mail-compose-check-names = Verificar nomes
mail-compose-check-names-help = Resolve nomes exatos a partir dos seus contatos carregados e verifica a formatação dos endereços. Isso não verifica a entrega na caixa de correio.
mail-compose-names-unresolved = Escolha um endereço de e-mail completo para estes nomes não resolvidos ou ambíguos: { $names }
mail-compose-names-checked = Formatação dos destinatários verificada; { $count } nome(s) de contato resolvido(s). A entrega na caixa de correio não foi verificada.
mail-compose-ribbon-table = Tabela
mail-compose-group-clipboard = Área de transferência
mail-compose-group-paragraph = Parágrafo
mail-compose-group-editing = Edição
mail-compose-group-view = Exibição
mail-compose-dark-background = Fundo escuro
mail-compose-dark-background-help = Mostra a mensagem em um fundo escuro enquanto você escreve. Somente a sua tela muda: a mensagem é enviada sem cor de fundo.
mail-compose-group-assist = Assistente de escrita
mail-editor-counts = Estatísticas do editor
mail-editor-words = { $count ->
    [one] { $count } palavra
   *[other] { $count } palavras
}
mail-editor-paragraphs = { $count ->
    [one] { $count } parágrafo
   *[other] { $count } parágrafos
}
mail-editor-characters = { $count ->
    [one] { $count } caractere
   *[other] { $count } caracteres
}
mail-editor-selection-required = Selecione um texto ou células de tabela adequados antes de usar esta ação.
mail-editor-clipboard-unavailable = O acesso à área de transferência não estava disponível. Use os atalhos Copiar, Recortar ou Colar do navegador.
mail-editor-selection-changed = A seleção mudou. Selecione o texto desejado e tente novamente.
mail-editor-clipboard = Área de transferência e formatação
mail-editor-copy = Copiar
mail-editor-cut = Recortar
mail-editor-paste = Colar
mail-editor-copy-format = Copiar formatação
mail-editor-apply-format = Aplicar formatação
mail-editor-style = Estilo do parágrafo
mail-editor-text-color = Cor da fonte
mail-editor-clear-color = Cor padrão
mail-editor-highlight = Realce
mail-editor-clear-highlight = Remover realce
mail-editor-paragraph = Opções de parágrafo
mail-editor-alignment = Alinhamento
mail-editor-line-height = Altura da linha
mail-editor-space-before = Espaço antes
mail-editor-space-after = Espaço depois
mail-editor-indent = Recuo
mail-editor-shading = Sombreamento do parágrafo
mail-editor-clear-shading = Remover sombreamento
mail-editor-case = Alterar maiúsculas e minúsculas
mail-editor-sort = Ordenar os parágrafos selecionados
mail-editor-sort-alpha = Alfabética
mail-editor-sort-numeric = Ordem numérica
mail-editor-table-rows = Linhas da tabela
mail-editor-table-columns = Colunas da tabela
mail-editor-table-insert = Inserir tabela
mail-editor-table-limits = Até 20 linhas, 10 colunas e 200 células por tabela; no máximo 10 tabelas. Tabelas aninhadas não são aceitas.
mail-editor-table-row-before = Linha acima
mail-editor-table-row-after = Linha abaixo
mail-editor-table-row-delete = Excluir linha
mail-editor-table-column-before = Coluna antes
mail-editor-table-column-after = Coluna depois
mail-editor-table-column-delete = Excluir coluna
mail-editor-table-merge = Mesclar células
mail-editor-table-split = Dividir célula
mail-editor-table-header = Alternar linha de cabeçalho
mail-editor-table-delete = Excluir tabela
mail-editor-table-shading = Sombreamento da célula
mail-editor-find-replace = Localizar e substituir
mail-editor-find = Localizar texto
mail-editor-replacement = Substituir por
mail-editor-match-case = Diferenciar maiúsculas de minúsculas
mail-editor-previous = Ocorrência anterior
mail-editor-next = Próxima ocorrência
mail-editor-replace = Substituir a ocorrência selecionada
mail-editor-replace-all = Substituir todas as ocorrências
mail-editor-matches = { $count } ocorrências (até 1.000)
mail-editor-select-all = Selecionar tudo
mail-editor-clear-selection = Limpar seleção
mail-editor-history = Histórico de desfazer
mail-editor-undo-groups = Desfazer os últimos { $count } grupos de alterações
mail-editor-zoom = Zoom do editor
mail-editor-markers = Marcas de parágrafo
mail-editor-style-normal = Normal
mail-editor-style-no-spacing = Sem espaçamento
mail-editor-style-title = Título
mail-editor-style-subtitle = Subtítulo
mail-editor-style-heading1 = Título 1
mail-editor-style-heading2 = Título 2
mail-editor-style-heading3 = Título 3
mail-editor-style-heading4 = Título 4
mail-editor-style-heading5 = Título 5
mail-editor-style-heading6 = Título 6
mail-editor-align-left = Esquerda
mail-editor-align-center = Centro
mail-editor-align-right = Direita
mail-editor-align-justify = Justificado
mail-editor-case-lower = minúsculas
mail-editor-case-upper = MAIÚSCULAS
mail-editor-case-title = Iniciais Maiúsculas
mail-editor-case-sentence = Primeira letra maiúscula
mail-editor-action-history = Histórico de { $action }
mail-editor-history-empty = Nenhuma ação disponível
mail-editor-redo-groups = Refazer { $count } alterações
mail-compose-identities-failed = Não foi possível carregar as identidades de envio. Feche e reabra este rascunho para tentar novamente.
mail-folders-loading = Carregando pastas…
mail-initial-sync-error = Suas mensagens ainda não puderam ser sincronizadas.
mail-initial-sync-error-help = A conexão com o seu servidor de e-mail foi interrompida. Tente novamente ou peça ao administrador que verifique os logs de conexão se isso continuar acontecendo.
mail-links-trusted = Domínio confiável: abre sem revisão
mail-links-trust-domain = Confiar em { $domain }
mail-links-trust-failed = Não foi possível salvar o domínio confiável. Tente novamente.
mail-focus-always-focused = Sempre mover para Foco
mail-focus-always-other = Sempre mover para Outras

# -- Machine translation (2026-11): added to match en ---------------------
mail-focus-moved-focused = As mensagens de { $sender } agora vão para Foco.
mail-focus-moved-other = As mensagens de { $sender } agora vão para Outras.
mail-focus-save-failed = Não foi possível salvar a escolha de Caixa de entrada com foco para este remetente. Tente novamente.
mail-context-search-domain = Pesquisar mensagens deste domínio
mail-loading-mailbox = Carregando { $name }…
mail-user-folders = Pastas do usuário
mail-context-hide-folder = Ocultar da barra lateral
mail-folder-fallback = pasta
mail-toast-moving = { $count ->
    [one] Movendo uma mensagem para { $folder }…
   *[other] Movendo { $count } mensagens para { $folder }…
}
mail-toast-move-queueing = { $count ->
    [one] Enfileirando uma mensagem para mover para { $folder }…
   *[other] Enfileirando { $count } mensagens para mover para { $folder }…
}
mail-toast-move-queued = { $count ->
    [one] Uma mensagem na fila para mover para { $folder }
   *[other] { $count } mensagens na fila para mover para { $folder }
}
mail-toast-move-partial = { $count } de { $total } mensagens movidas para { $folder }; { $failed } não puderam ser movidas
mail-send-copy-folder-unavailable = A pasta selecionada para a cópia enviada não está disponível. Escolha outra pasta antes de enviar.
mail-send-copy-failed = A mensagem foi aceita, mas não foi possível arquivar a cópia enviada. Não a envie novamente.
mail-compose-attach-public-key = Anexar minha chave pública ou certificado
mail-compose-include-hashes = Incluir hashes SHA-256 dos arquivos anexados
mail-compose-hashes-pending = Calculando os hashes dos anexos. Envie quando estiver pronto.
mail-compose-hashes-failed = Não foi possível verificar os bytes dos anexos. Desative os hashes ou tente novamente ativando-os de novo.
mail-compose-crypto-pending = A criptografia está sendo atualizada. Aguarde antes de enviar.
mail-compose-public-key-unavailable = Nenhuma chave pública ou certificado atual corresponde ao endereço de envio.
mail-compose-sent-copy = Salvar a mensagem enviada em
mail-compose-sent-copy-default = Padrão da conta
folder-create = Criar pasta
folder-name = Nome da pasta
folder-parent = Pasta superior
folder-top-level = Nível superior
folder-created = Pasta criada.
folder-create-unavailable = A criação de pastas não está disponível para esta conta ou pasta superior.
folder-create-invalid = Escolha um nome de pasta válido e exclusivo, sem separadores de hierarquia nem caracteres de controle.
folder-create-failed = Não foi possível criar a pasta. Atualize e tente novamente.
folder-cleanup = Limpar pasta…
folder-cleanup-title = Limpar { $name }
folder-cleanup-help = Encontre mensagens mais antigas que a idade escolhida. Revise a contagem antes de excluir. As subpastas são excluídas da operação.
folder-cleanup-age = Mais antigas que
folder-age-days = Dias
folder-age-weeks = Semanas
folder-age-months = Meses
folder-age-years = Anos
folder-cleanup-preview = Pré-visualizar a limpeza
folder-cleanup-confirm = Excluir as mensagens correspondentes
folder-cleanup-permanent = Excluir permanentemente { $count } mensagens correspondentes. Isso não pode ser desfeito.
folder-cleanup-trash = Mover { $count } mensagens correspondentes para a Lixeira.
folder-cleanup-before = Recebidas antes de { $date }
folder-cleanup-done = { $count } mensagens limpas.
mail-from = Remetente
mail-subject = Assunto
mail-date = Data
mail-cancel = Cancelar
mail-bulk-selected = { $count } conversas selecionadas
mail-bulk-select-folder = Selecionar tudo em { $folder }
mail-list-order = Ordem das mensagens
mail-list-order-newest = Mais recentes primeiro
mail-list-order-oldest = Mais antigas primeiro
mail-list-order-sender = Remetente
mail-list-order-subject = Assunto
mail-list-order-error = Não foi possível salvar a ordem das mensagens.
mail-bulk-selection-load-failed = Não foi possível selecionar a pasta inteira. Verifique sua conexão e tente novamente.
mail-privacy-bar = Privacidade da mensagem
mail-privacy-trackers-allow = Permitir rastreadores nesta mensagem
mail-privacy-trackers-block = Bloquear rastreadores novamente
mail-privacy-images-loading = { $loaded }/{ $total } imagens exibidas · { $pending } carregando
mail-privacy-images-failed = { $loaded }/{ $total } imagens exibidas · { $failed } com falha
mail-privacy-images-loaded = { $loaded }/{ $total } imagens exibidas
mail-privacy-images-loading-failed = { $loaded }/{ $total } imagens exibidas · { $pending } carregando · { $failed } com falha
mail-reader-folder-title = Pasta: { $folder }
mail-reader-move-to = Mover para…
mail-action-help-move = Mover esta mensagem para outra pasta.
folder-ops-new-subfolder = Nova subpasta
folder-ops-mark-read = Marcar tudo como lido
folder-ops-this-folder = Esta pasta
folder-ops-with-subfolders = Incluir subpastas
folder-ops-no-unread = Nenhuma mensagem não lida.
folder-ops-read-unavailable = Você não pode alterar as mensagens desta pasta.
folder-ops-more = Mais
folder-ops-system-folder = Uma pasta do sistema não pode ser alterada desta forma.
folder-ops-sort-subfolders = Ordenar subpastas por
folder-ops-sort-folders = Ordenar pastas por
folder-ops-sort-name = Nome
folder-ops-sort-unread = Quantidade de não lidas
folder-ops-sort-total = Quantidade total
folder-ops-sort-date = Última atividade
folder-ops-sort-custom = Ordem personalizada
folder-ops-sort-asc = Crescente
folder-ops-sort-desc = Decrescente
folder-ops-delete-with-subfolders = Excluir pasta e subpastas
folder-ops-delete-unavailable = Esta pasta não pode ser excluída.
folder-ops-delete-tree-body = Isso exclui permanentemente { $folders } pastas e as { $count } mensagens que elas contêm. Isso não pode ser desfeito.
folder-ops-copy-all-menu = Copiar todos os itens para…
folder-ops-move-all-menu = Mover todos os itens para…
folder-ops-move-folder-menu = Mover pasta para…
folder-ops-move-up = Mover para cima
folder-ops-move-down = Mover para baixo
folder-ops-move-unavailable = Você não pode mover esta pasta nem os itens dela.
folder-ops-move-duplicate = O destino já tem uma pasta com este nome.
folder-ops-move-folder-title = Mover { $name }
folder-ops-move-folder-body = A pasta é movida com suas subpastas e mensagens. Escolha a pasta dentro da qual ela ficará, ou o nível superior.
folder-ops-move-here = Mover para cá
folder-ops-moved-folder = { $name } movida.
folder-ops-choose-folder = Escolha uma pasta de destino.
folder-ops-destination = Pasta de destino
folder-ops-include-subfolders = Incluir as mensagens de { $count } subpastas
folder-ops-copy-all-title = Copiar todos os itens de { $name }
folder-ops-move-all-title = Mover todos os itens de { $name }
folder-ops-copy-all-body = Adiciona { $count } mensagens à pasta de destino. Elas continuam onde estão.
folder-ops-move-all-body = Move { $count } mensagens para a pasta de destino. Você pode desfazer depois.
folder-ops-copy-all = Copiar itens
folder-ops-move-all = Mover itens
folder-ops-undo = Desfazer
folder-ops-stop = Parar
folder-ops-marking = Marcando { $name } como lida…
folder-ops-marking-progress = Marcando { $name } como lida: { $done } de { $total }
folder-ops-marked = { $count } mensagens em { $name } marcadas como lidas.
folder-ops-copying = Copiando { $name }…
folder-ops-copying-progress = Copiando { $name }: { $done } de { $total }
folder-ops-copied = { $count } mensagens copiadas de { $name }.
folder-ops-moving = Movendo { $name }…
folder-ops-moving-progress = Movendo { $name }: { $done } de { $total }
folder-ops-moved = { $count } mensagens movidas de { $name }.
folder-ops-stopped = { $name } interrompida após { $count } de { $total } mensagens.
folder-ops-partial = { $count } de { $total } mensagens em { $name } foram processadas; { $refused } foram recusadas.
folder-ops-failed = Não foi possível concluir { $name }. Atualize e tente novamente.
folder-ops-error-forbidden = Você não tem os direitos necessários para esta alteração.
folder-ops-error-exists = Já existe ali uma pasta com este nome.
folder-ops-error-mailboxHasEmail = A pasta ainda tem mensagens.
folder-ops-error-notFound = A pasta não existe mais.
folder-ops-error-failed = Não foi possível fazer a alteração. Atualize e tente novamente.
folder-ops-expand = Mostrar as subpastas de { $name }
folder-ops-collapse = Ocultar as subpastas de { $name }
mail-context-reply = Responder
mail-context-reply-all = Responder a todos
mail-context-forward = Encaminhar

# -- Machine translation (2026-11): added to match en ---------------------
mail-context-quick-print = Impressão rápida
mail-context-copy = Copiar
mail-context-copy-to-folder = Copiar mensagem para a pasta…
mail-context-copy-sender = Copiar endereço do remetente
mail-context-copy-subject = Copiar assunto
mail-context-copy-link = Copiar link da mensagem
mail-context-copied = Copiado.
mail-context-copy-failed = Não foi possível copiar para a área de transferência.
mail-context-print-failed = Não foi possível imprimir a mensagem.
mail-reply-images-title = Imagens da mensagem original
mail-reply-images-ask = Incluir as imagens da mensagem original nesta resposta?
mail-reply-images-include = Incluir imagens
mail-reply-images-not-now = Agora não
mail-reply-images-trust = Sempre confiar em { $domain }
mail-reply-images-loading = Adicionando imagens…
mail-reply-images-cancelled = Carregamento cancelado. As imagens não foram adicionadas.
mail-reply-images-failed = Não foi possível adicionar as imagens.
mail-reply-images-retry = Carregar novamente
mail-image-cancel = Cancelar
mail-compose-from-why-match = Enviando de { $identity } porque a mensagem foi enviada para { $address }.
mail-compose-from-why-match-plus = Enviando de { $identity } porque a mensagem foi enviada para { $address }, um endereço com sinal de mais dele.
mail-compose-from-why-ambiguous = Várias identidades correspondem a { $address }. Enviando de { $identity }; escolha outra:
mail-compose-from-why-default = Enviando da sua identidade padrão, { $identity }, porque nenhuma corresponde aos destinatários originais.
mail-compose-from-why-forced-default = Enviando da sua identidade padrão, { $identity }, conforme definido nas suas configurações.
mail-compose-from-why-keep = Enviando de { $identity }.
mail-compose-from-change = Alterar
reader-attachments-reason-encrypted = Esta mensagem está criptografada. Os anexos aparecem depois que você a desbloqueia.
reader-attachments-reason-list-missing = Não foi possível carregar a lista de anexos desta mensagem, por isso a barra de anexos está oculta.
reader-attachments-reason-inline-only = Os anexos desta mensagem ({ $count }) são imagens embutidas exibidas na própria mensagem.
reader-attachments-reason-unavailable-parts = { $count ->
    [one] Um anexo desta mensagem não tem conteúdo para download, por isso a barra de anexos está oculta.
   *[other] { $count } anexos desta mensagem não têm conteúdo para download, por isso a barra de anexos está oculta.
}
reader-attachments-reason-policy-blocked = Sua política de anexos bloqueia o download { $count ->
    [one] deste anexo.
   *[other] destes anexos.
}
reader-attachments-reason-policy-unavailable = Não foi possível verificar a política de anexos, por isso os anexos ainda não podem ser abertos.
mail-drafts-popup-count = { $count ->
    [one] 1 rascunho
   *[other] { $count } rascunhos
}
mail-drafts-popup-search = Pesquisar por assunto ou destinatário
mail-drafts-popup-tab-local = Rascunhos locais
mail-drafts-popup-local-hint = Os rascunhos locais ficam somente neste navegador.
mail-drafts-popup-badge-account = Conta
mail-drafts-popup-badge-local = Local
mail-drafts-popup-current = Aberto agora
mail-drafts-popup-select = Selecionar
mail-drafts-popup-select-all = Selecionar tudo
mail-drafts-popup-clear = Limpar
mail-drafts-popup-done = Concluído
mail-drafts-popup-selected = { $count } selecionados
mail-drafts-popup-select-row = Selecionar { $subject }
mail-drafts-popup-open = Abrir
mail-drafts-popup-discard = Descartar
mail-drafts-popup-discard-selected = Descartar selecionados
mail-drafts-popup-more = Mais ações para { $subject }
mail-drafts-popup-retry = Tentar novamente
mail-drafts-popup-load-failed = Não foi possível carregar os rascunhos da conta.
mail-drafts-popup-empty-account = Nenhum rascunho da conta
mail-drafts-popup-no-match = Nenhum rascunho corresponde a “{ $query }”
mail-drafts-popup-discarding = Descartando…
mail-drafts-popup-row-failed = Não foi possível descartar. A cópia da conta permanece.
mail-drafts-popup-confirm-title = { $count ->
    [one] Descartar este rascunho?
   *[other] Descartar { $count } rascunhos?
}
mail-drafts-popup-confirm-account = Os rascunhos da conta são removidos primeiro da pasta Rascunhos da sua conta, e a vaga deles só é liberada depois que essa remoção for confirmada.
mail-drafts-popup-confirm-local = Os rascunhos locais são removidos deste navegador.
mail-drafts-popup-announce-done = { $count ->
    [one] 1 rascunho descartado
   *[other] { $count } rascunhos descartados
}
mail-drafts-popup-announce-failed = { $done } descartados, { $failed } não puderam ser descartados
mail-check-title = Verificar antes de enviar
mail-check-intro = Estes itens podem merecer uma segunda olhada:
mail-check-invalid = Alguns endereços de destinatário parecem inválidos: { $list }
mail-check-subject = O assunto está vazio.
mail-check-attachment = A mensagem menciona um anexo, mas nada está anexado.
mail-check-external = Destinatários fora dos domínios confiáveis: { $list }
mail-check-send-anyway = Enviar mesmo assim
mail-check-review = Revisar
mail-check-remember = Não perguntar novamente para estes
mail-check-denied-invalid = O envio está bloqueado pelas suas configurações. Corrija os endereços de destinatário inválidos: { $list }
mail-check-denied-subject = O envio está bloqueado pelas suas configurações. Adicione um assunto.
mail-check-denied-attachment = O envio está bloqueado pelas suas configurações. Anexe o arquivo mencionado ou reformule a mensagem.
mail-check-denied-external = O envio está bloqueado pelas suas configurações. Fora dos domínios confiáveis: { $list }
mail-alerts-label = Problemas na mensagem
mail-alert-invalid = Endereço inválido: { $address }
mail-alert-subject = Sem assunto
mail-recipient-chip-remove = Remover { $address }
mail-recipient-chips = Destinatários em { $field }
mail-recipient-invalid-title = Não é um endereço de e-mail válido
mail-recipient-chip-name = { $name }, { $address }. Enter para ver os detalhes do contato, Backspace ou F2 para editar, Delete para remover
mail-recipient-chip-name-invalid = { $address }, não é um endereço válido. Enter para editar, Delete para remover
mail-recipient-added = { $name } adicionado
mail-recipient-removed = { $name } removido
mail-recipient-removed-many = { $count ->
    [one] { $count } destinatário removido
   *[other] { $count } destinatários removidos
}
mail-recipient-editing = Editando { $name }
mail-recipient-all-selected = { $count ->
    [one] { $count } destinatário selecionado
   *[other] { $count } destinatários selecionados
}
mail-external-line = { $count } fora dos domínios confiáveis: { $domains }
mail-external-dismiss = Dispensar
mail-status-label = Status do editor
mail-status-more = Mais { $count } avisos
mail-status-more-label = Mais avisos
mail-status-no-recipients = Sem destinatários
mail-status-no-attachment = Sem anexo
mail-status-external = Externos: { $count }
mail-status-external-reviewed = Destinatários externos revisados
mail-status-external-review = Marcar como revisado
mail-status-from-match = De { $identity }
mail-status-from-match-plus = De { $identity }
mail-status-from-ambiguous = De { $identity } (várias correspondem)
mail-status-from-default = Da identidade padrão
mail-status-from-forced-default = Da identidade padrão
mail-status-from-keep = De { $identity }
mail-status-held = { $count } na fila
mail-status-storage = Sem recuperação local
mail-status-protected = Não salvo como texto simples
mail-status-saving = Salvando…
mail-status-saved = Salvo { $time }
mail-status-saved-full = Salvo em Rascunhos da conta · { $time }
mail-status-not-saved = Rascunho não salvo
mail-status-retry = Tentar novamente
mail-status-conflict = Alterado em outro dispositivo
mail-status-uncertain = Salvamento não confirmado
mail-status-close-recovery = Fechar, manter recuperação
mail-status-unsupported = Salvamento do rascunho não confirmado
mail-compose-header = Cabeçalho da mensagem
mail-header-label = Cabeçalho da mensagem
mail-header-collapse = Recolher o cabeçalho da mensagem
mail-header-expand = Expandir o cabeçalho da mensagem: { $summary }
mail-header-from-chip = De { $address }. Alterar remetente
mail-compose-group-timing = Momento do envio
mail-compose-group-priority = Prioridade
mail-compose-group-receipts = Confirmações e rastreamento
mail-compose-group-keys-files = Chaves e arquivos
mail-compose-group-sent-copy = Cópia enviada
mail-editor-style-quote = Citação
mail-editor-style-code = Código
mail-compose-ribbon-table-layout = Layout da tabela
mail-compose-ribbon-picture = Imagem
mail-editor-table-group-rows = Linhas
mail-editor-table-group-columns = Colunas
mail-editor-table-group-cells = Células
mail-editor-table-row-height = Altura da linha
mail-editor-table-column-width = Largura da coluna
mail-editor-table-size-auto = Automático
mail-editor-table-padding = Margem interna da célula
mail-editor-table-padding-compact = Compacta
mail-editor-table-padding-normal = Normal
mail-editor-table-padding-roomy = Ampla
mail-editor-table-align-left = Alinhar célula à esquerda

# -- Machine translation (2026-11): added to match en ---------------------
mail-editor-table-align-center = Centralizar célula
mail-editor-table-align-right = Alinhar célula à direita
mail-picture-group-size = Tamanho
mail-picture-group-arrange = Organizar
mail-picture-group-text = Texto
mail-picture-group-style = Estilo
mail-picture-group-edit = Editar
mail-picture-width = Largura
mail-picture-height = Altura
mail-picture-lock = Manter proporções
mail-picture-reset-size = Tamanho original
mail-picture-align-inline = Alinhada ao texto
mail-picture-align-left = Alinhar à esquerda
mail-picture-align-center = Centralizar
mail-picture-align-right = Alinhar à direita
mail-picture-alt = Texto alternativo
mail-picture-title = Título
mail-picture-border = Borda
mail-picture-radius = Cantos
mail-picture-none = Nenhum
mail-picture-round = Arredondados
mail-picture-rotate-ccw = Girar para a esquerda
mail-picture-rotate-cw = Girar para a direita
mail-picture-flip-h = Inverter horizontalmente
mail-picture-flip-v = Inverter verticalmente
mail-picture-crop = Cortar
mail-picture-crop-left = Esquerda (%)
mail-picture-crop-top = Superior (%)
mail-picture-crop-right = Direita (%)
mail-picture-crop-bottom = Inferior (%)
mail-picture-crop-apply = Cortar
mail-picture-replace = Substituir
mail-picture-remove = Remover
mail-picture-failed-no-image = Selecione uma imagem primeiro.
mail-picture-failed-no-source = Esta imagem não tem cópia local para editar. Substitua-a.
mail-picture-failed-failed = Não foi possível alterar a imagem.
mail-context-download-attachments = Baixar anexos
mail-context-download-named = Baixar { $name }
mail-context-download-all = Baixar tudo como ZIP
mail-context-loading-attachments = Carregando anexos…
mail-context-attachments-failed = Não foi possível carregar os anexos.
mail-context-searching = Procurando a mensagem…
mail-context-go-reply = Ir para a resposta
mail-context-go-forward = Ir para o encaminhamento
mail-context-reply-not-found = Resposta não encontrada
mail-context-forward-not-found = Encaminhamento não encontrado
mail-no-recipients = (sem destinatários)
mail-attachment-downloaded = { $name } baixado
mail-attachment-download-failed = O download falhou.
mail-attachments-zip-done = { $count } anexos baixados em um ZIP.
mail-attachments-zip-capped = O ZIP contém os primeiros { $count } anexos; o limite é de { $max } arquivos ou { $size }.
mail-focus-not-found = Mensagem não encontrada em { $folder }
mail-focus-go-to = Ir para { $folder }
mail-nav-views-queues = Exibições e filas
mail-badge-unread = { $unread ->
    [one] { $unread } mensagem não lida
   *[other] { $unread } mensagens não lidas
}
mail-badge-total = { $total ->
    [one] { $total } mensagem nesta pasta
   *[other] { $total } mensagens nesta pasta
}
mail-badge-both = { $unread ->
    [one] { $unread } mensagem não lida
   *[other] { $unread } mensagens não lidas
} de { $total ->
    [one] { $total } nesta pasta
   *[other] { $total } nesta pasta
}
mail-badge-drafts = { $total ->
    [one] Rascunhos: { $total } mensagem
   *[other] Rascunhos: { $total } mensagens
}
mail-badge-compact-hint = As pastas diferentes da Caixa de entrada usam um selo menor. Altere isso em Configurações, Pastas e barra lateral.
mail-badge-outbox = { $count ->
    [one] Caixa de saída: { $count } mensagem aguardando envio
   *[other] Caixa de saída: { $count } mensagens aguardando envio
}
mail-color-menu = Cores da mensagem
mail-color-button = Cores da mensagem: { $mode }
mail-color-use-default = Usar meu padrão ({ $mode })
mail-color-make-default = Tornar este o meu padrão
mail-color-locked = Bloqueado pelo seu administrador
mail-color-confirm-title = Alterar suas cores padrão de mensagem?
mail-color-confirm-body = Todas as mensagens usarão "{ $mode }", a menos que você altere para uma mensagem específica.
mail-color-confirm = Tornar padrão
mail-color-default-failed = Não foi possível salvar suas cores padrão de mensagem.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = As mensagens selecionadas não possuem anexos para download.

mail-attachments-zip-all-blocked = Nada foi baixado: a política de anexo bloqueia todos os arquivos { $skipped }.

mail-attachments-zip-gathering = Coletando os anexos de { $count } mensagens…

mail-attachments-zip-skipped = { $skipped ->
        [0] { $skipped } arquivos bloqueados pela política de anexos foram deixados de fora.
        [one] { $skipped } arquivo bloqueado pela política de anexos foi deixado de fora.
        [many] { $skipped } arquivos bloqueados pela política de anexos foram deixados de fora.
       *[other] { $skipped } arquivos bloqueados pela política de anexos foram deixados de fora.
    }

mail-attachments-zip-unreadable = Os anexos das mensagens { $count } não puderam ser lidos e foram deixados de fora.

mail-bulk-download-attachments = Baixe anexos como ZIP

mail-compose-from-via = Correspondeu ao cabeçalho { $header }.

mail-compose-from-why-match-domain = Enviando de { $identity } porque a mensagem foi entregue para { $address }, um endereço no mesmo domínio.

mail-compose-from-why-match-pattern = Enviando de { $identity } porque a mensagem foi enviada para { $address }, que se enquadra no padrão de endereço dessa identidade.

mail-compose-ribbon-autohide = Ocultar automaticamente a faixa de opções (Ctrl+F1)

mail-compose-ribbon-pin = Sempre mostrar a faixa de opções (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
        [0] Baixe anexos de { $count } mensagens selecionadas como ZIP
        [one] Baixe os anexos da mensagem selecionada como ZIP
        [many] Baixe anexos de { $count } mensagens selecionadas como ZIP
       *[other] Baixe anexos de { $count } mensagens selecionadas como ZIP
    }

mail-context-download-thread-attachments = Baixe todos os { $count } anexos da conversa como ZIP

mail-context-open-new-tab = Abrir em uma nova aba

mail-delete-sender-confirm = Mover { $count } mensagens para a Lixeira

mail-delete-sender-detail = Mover { $count } mensagens de { $sender } para a Lixeira? As mensagens que já estão na Lixeira são excluídas. Você pode desfazer esta ação.

mail-delete-sender-empty = Não há mensagens deste remetente fora da Lixeira.

mail-delete-sender-failed = Não foi possível verificar todas as mensagens correspondentes e suas permissões. Nada foi movido; tente novamente.

mail-delete-sender-help = Encontre as mensagens deste remetente em todas as pastas e mova-as para a Lixeira após a confirmação.

mail-delete-sender-title = Exclua todas as mensagens deste remetente

mail-draft-continue = Continuar editando

mail-draft-edit = Editar rascunho

mail-draft-edit-failed = Não foi possível abrir o rascunho para edição

mail-draft-edit-help = Abra este rascunho no compositor para continuar editando-o

mail-draft-notice = Este é um rascunho

mail-file-copied = Copiado para a área de transferência.

mail-file-copy-failed = Não foi possível copiar para a área de transferência.

mail-file-copy-fingerprint = Copiar impressão digital

mail-file-copy-name = Copiar nome do arquivo

mail-file-created = Criado

mail-file-creator = Criado com

mail-file-details-button = Detalhes do arquivo

mail-file-details-tooltip = Mostrar detalhes do arquivo e impressão digital

mail-file-encryption = Proteção por senha

mail-file-encryption-none = Nenhuma senha foi solicitada

mail-file-encryption-password = Foi necessária uma senha para abri-lo

mail-file-fingerprint = Impressão digital

mail-file-name = Nome do arquivo

mail-file-pages = Páginas

mail-file-pdf-version = Versão PDF

mail-file-producer = Produtor

mail-file-verify-help = Compare esses grupos com uma impressão digital que o remetente forneceu de outra forma, como uma ligação ou seu site. Se todos os grupos corresponderem, você terá exatamente os mesmos bytes.

mail-file-verify-title = Verifique a impressão digital

mail-follow-up-finished = Acompanhamento concluído

mail-group-by = Agrupar por

mail-header-attachments = { $count ->
        [0] { $count } anexos
        [one] { $count } anexo
        [many] { $count } anexos
       *[other] { $count } anexos
    }

mail-header-expand-short = Expandir cabeçalho da mensagem (Alt+H)

mail-list-group-date-unknown = Data desconhecida

mail-list-group-importance = Importância

mail-list-group-locked = O agrupamento de mensagens é bloqueado pelo seu administrador.

mail-list-group-no-subject = (Sem assunto)

mail-list-group-none = Sem agrupamento

mail-list-group-priority-high = Alta importância

mail-list-group-priority-low = Baixa importância

mail-list-group-priority-normal = Importância normal

mail-list-group-received = Data de recebimento

mail-list-group-sender = De

mail-list-group-sent = Data de envio

mail-list-group-size = Tamanho

mail-list-group-size-huge = 10 MB e acima

mail-list-group-size-large = 1–10MB

mail-list-group-size-medium = 100 KB – 1 MB

mail-list-group-size-small = 25–100 KB

mail-list-group-size-tiny = Menos de 25 KB

mail-list-group-subject = Assunto

mail-list-order-importance = Importância, alta primeiro

mail-list-order-importance-asc = Importância, baixa primeiro

mail-list-order-locked = A ordem das mensagens é bloqueada pelo seu administrador.

mail-list-order-sender-desc = Remetente, Z a A

mail-list-order-sent-newest = Enviado, mais novo primeiro

mail-list-order-sent-oldest = Enviado, mais antigo primeiro

mail-list-order-size = Tamanho, maior primeiro

mail-list-order-size-asc = Tamanho, o menor primeiro

mail-list-order-subject-desc = Assunto, Z a A

mail-mark-answered = Marcar como respondido

mail-mark-as = Marcar como

mail-mark-finished = Marcar como concluído

mail-mark-forwarded = Marcar como encaminhado

mail-mark-not-forwarded = Marcar como não encaminhado

mail-mark-unanswered = Marcar como sem resposta

mail-mentions-me = Menciona-me

mail-mentions-me-help = Pesquise mensagens que mencionem um dos seus endereços de e-mail

mail-move-search = Pesquisar pastas

mail-order-by = Encomendar por

mail-reader-folder-go = Armazenado em { $folder } · Vá para a pasta

mail-reply-draft-discard = Descartar resposta

mail-reply-draft-discard-detail = Remover permanentemente esta resposta em andamento? Isto não pode ser desfeito.

mail-reply-draft-discard-title = Descartar esta resposta?

mail-status-from-match-domain = De { $identity }

mail-status-from-match-pattern = De { $identity }

mail-status-from-via = Respondendo de { $identity } porque correspondeu a { $header }

mail-toast-follow-up-finished = Acompanhamento marcado como concluído

mail-toast-marked-answered = { $count ->
        [0] { $count } mensagens marcadas como respondidas
        [one] { $count } mensagem marcada como respondida
        [many] { $count } mensagens marcadas como respondidas
       *[other] { $count } mensagens marcadas como respondidas
    }

mail-toast-marked-forwarded = { $count ->
        [0] { $count } mensagens marcadas como encaminhadas
        [one] { $count } mensagem marcada como encaminhada
        [many] { $count } mensagens marcadas como encaminhadas
       *[other] { $count } mensagens marcadas como encaminhadas
    }

mail-toast-marked-not-forwarded = { $count ->
        [0] { $count } mensagens marcadas como não encaminhadas
        [one] { $count } mensagem marcada como não encaminhada
        [many] { $count } mensagens marcadas como não encaminhadas
       *[other] { $count } mensagens marcadas como não encaminhadas
    }

mail-toast-marked-unanswered = { $count ->
        [0] { $count } mensagens marcadas como não respondidas
        [one] { $count } mensagem marcada como não respondida
        [many] { $count } mensagens marcadas como não respondidas
       *[other] { $count } mensagens marcadas como não respondidas
    }

mail-toast-move-failed-reason = Não foi possível mover as mensagens ({ $reason }). Nada foi movido.

mail-toast-update-refused-reason = { $count ->
        [0] O servidor recusou a mudança por { $count } mensagens: { $reason }
        [one] O servidor recusou a mudança para uma mensagem: { $reason }
        [many] O servidor recusou a mudança por { $count } mensagens: { $reason }
       *[other] O servidor recusou a mudança por { $count } mensagens: { $reason }
    }

reader-badge-hide = Ocultar este selo

reader-badge-locked = Definido pelo seu administrador

reader-badge-menu = Selo de mensagem

reader-badge-notable = Mostrar apenas quando for notável

reader-sanitized-always = O conteúdo ativo, como scripts, permanece desativado.

reader-sanitized-clean = Mensagem HTML não precisou de alterações

reader-sanitized-clean-detail = Nada foi removido ou bloqueado do HTML desta mensagem.

reader-sanitized-counts = { $blocked } itens remotos bloqueados, { $trackers } itens de rastreamento.

reader-sanitized-detail = A mensagem HTML foi higienizada antes de ser exibida. O conteúdo remoto segue suas configurações de privacidade.

reader-sanitized-hosts = Hosts remotos

reader-sanitized-remote = Conteúdo remoto bloqueado

reader-sanitized-short = Higienizado HTML

reader-sanitized-short-blocked = HTML bloqueado

reader-sanitized-short-clean = HTML limpar

reader-sanitized-trackers = Itens de rastreamento

mail-draft-view = Ver rascunho
mail-reply-draft-view = Ver resposta
