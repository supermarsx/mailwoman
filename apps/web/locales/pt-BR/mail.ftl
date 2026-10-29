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
mail-toast-pinned = Fixada
mail-toast-unpinned = Desafixada
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
