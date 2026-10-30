# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Definições
mail-compose = Escrever
mail-nav-mailboxes = Caixas de correio
mail-nav-attachments = Anexos
mail-nav-events = Eventos
mail-events-upcoming = Só os próximos
mail-events-filter = Filtro de eventos
mail-nav-outbox = Caixa de saída
mail-nav-sharing = Partilhar pasta
mail-nav-apps = Aplicações
mail-logout = Terminar sessão
mail-offline = Offline
mail-module-loading = A carregar { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Pesquisar
mail-search-label = Pesquisar correio
mail-search-placeholder = Pesquisar correio — from:alice subject:fatura larger:1mb
mail-search-clear = Limpar

# -- Message list ------------------------------------------------------------
mail-list-label = Mensagens
mail-loading = A carregar mensagens…
mail-loading-more = A carregar mensagens mais antigas…
mail-empty = Sem mensagens
mail-unknown-sender = (remetente desconhecido)
mail-no-subject = (sem assunto)
mail-pinned = Afixada
mail-has-attachment = Tem anexo
mail-event-state-invitation = Convite
mail-event-state-updated = Atualizado
mail-event-state-cancelled = Cancelado
mail-event-state-reply = Resposta
mail-event-state-counter = Nova hora proposta
mail-event-state-event = Evento
mail-event-hint = { $state }, { $when }
mail-answered = Respondida
mail-forwarded = Reencaminhada
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
mail-thread-collapse = Fechar conversa, { $count ->
    [one] { $count } mensagem
   *[other] { $count } mensagens
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Opções de vista
mail-reading-pane = Painel de leitura
mail-reading-pane-right = À direita
mail-reading-pane-bottom = Em baixo
mail-reading-pane-off = Desativado

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Mensagens selecionadas
mail-bulk-count = { $count } selecionadas
mail-mark-read = Marcar como lida
mail-mark-unread = Marcar como não lida
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Sinalizar
mail-unflag-message = Remover sinalização
mail-move-to = Mover para pasta
mail-move-to-placeholder = Mover para…
mail-select-all = Selecionar tudo
mail-clear-selection = Limpar seleção

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Filtro da caixa de entrada
mail-inbox-focused = Destaques
mail-inbox-other = Outras

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Separadores abertos
mail-subtab-pin = Afixar { $title }
mail-subtab-unpin = Desafixar { $title }
mail-subtab-tearoff = Abrir { $title } numa nova janela
mail-subtab-close = Fechar { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Afixar
mail-unpin = Desafixar
mail-snooze = Adiar
mail-snooze-menu = Adiar até
mail-snooze-later = Mais logo
mail-snooze-tomorrow = Amanhã
mail-snooze-next-week = Próxima semana
mail-unsnooze = Cancelar adiamento
mail-label = Etiqueta
mail-labels-menu = Etiquetas
mail-flag = Sinalizar para seguimento
mail-clear-flag = Limpar seguimento
mail-archive = Arquivar
mail-delete = Eliminar
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = Mais ações

# -- Reader ------------------------------------------------------------------
mail-reader-label = Mensagem
mail-reader-empty = Selecione uma mensagem para a ler
# A mensagem aberta mudou noutro lado (t39): o leitor mantém a cópia e avisa.
mail-remote-moved = Esta mensagem foi movida para { $folder } noutro dispositivo ou aplicação. Continua a ler a cópia que abriu.
mail-remote-moved-unknown = Esta mensagem foi movida noutro dispositivo ou aplicação. Continua a ler a cópia que abriu.
mail-remote-deleted = Esta mensagem foi eliminada noutro dispositivo ou aplicação. Pode continuar a ler esta cópia até a fechar.
mail-remote-dismiss = Fechar aviso
mail-back = Voltar
mail-reader-from = De: { $addr }
mail-reader-to = Para: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Ações da mensagem
mail-reply = Responder
mail-reply-all = Responder a todos
mail-forward = Reencaminhar
mail-reply-failed = Não foi possível preparar a mensagem. Tente novamente.
mail-reply-unthreaded = Não foi possível ler o ID da mensagem original. Esta resposta não ficará na mesma conversa.
mail-spam = Spam
mail-export = Exportar
mail-sweep-sender = Limpar remetente
mail-message-body = Corpo da mensagem
mail-no-content = Sem conteúdo
mail-sanitizing = A limpar o conteúdo…
mail-attachments = Anexos
mail-attachment-unnamed = (sem nome)
mail-attachment-open = Anexo { $name }
mail-attachment-close = Fechar anexo
mail-attachment-loading = A carregar anexo…
mail-attachment-failed = Não foi possível transferir este anexo. Tente novamente quando houver ligação.

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Mensagem encriptada
mail-encrypted-note = Esta mensagem está encriptada ponto a ponto. Desbloqueie-a neste dispositivo para a ler.
mail-key-passphrase = Frase-passe da chave
mail-decrypt = Desencriptar
mail-decrypting = A desencriptar…
mail-decrypt-no-key = Não há nenhuma chave privada disponível para desencriptar esta mensagem.
mail-decrypt-failed = Falha ao desencriptar

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = Em { $date }, { $name } escreveu:
mail-quote-attribution-undated = { $name } escreveu:
# The header block above a forwarded original.
mail-forward-heading = ---------- Mensagem reencaminhada ----------
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
mail-compose-title-forward = Reencaminhar
mail-compose-close = Fechar
mail-compose-from = De
mail-compose-from-default = Predefinido
mail-compose-to = Para
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Sugestões de contactos
mail-compose-cc = Cc
mail-compose-bcc = Bcc
mail-compose-show-cc-bcc = Adicionar Cc ou Bcc
mail-compose-bcc-encrypted = Uma mensagem encriptada indica a chave de cada destinatário, pelo que um destinatário em Bcc ficaria visível para os outros. Remova os destinatários em Bcc ou desative a encriptação.
mail-compose-plaintext-confirm-title = Enviar texto desencriptado sem encriptação?
mail-compose-plaintext-confirm-body = Esta mensagem cita texto de uma mensagem encriptada. Não está definida para ser encriptada, pelo que o texto citado seria enviado sem proteção.
mail-compose-plaintext-confirm-send = Enviar sem encriptação
mail-compose-plaintext-confirm-cancel = Voltar
mail-compose-subject = Assunto
mail-compose-body = Corpo
mail-compose-attachments = Anexos
mail-compose-remove-attachment = Remover { $name }
mail-compose-attach-nextcloud = Anexar do Nextcloud
mail-compose-close-nextcloud = Fechar Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Anexar um ficheiro
mail-compose-uploading = A carregar…
mail-compose-upload-unavailable = O carregamento de ficheiros não está disponível neste momento.
mail-compose-upload-failed = Não foi possível carregar { $name }.
mail-compose-upload-too-large = { $name } tem { $size } MB. O tamanho máximo de carregamento é { $max } MB.
mail-compose-send-later = Enviar mais tarde
mail-compose-cancel = Cancelar
mail-compose-send = Enviar
mail-compose-schedule = Agendar
mail-compose-sending = A enviar…
mail-compose-encrypted-subject = Mensagem encriptada
mail-compose-dlp-blocked = O envio está bloqueado por uma regra de prevenção de perda de dados (veja o aviso acima).
mail-compose-send-failed = Falha no envio
mail-compose-send-later-past = Escolha uma hora de envio no futuro ou limpe o campo para enviar já.
mail-compose-options = Opções de envio
mail-compose-request-receipt = Pedir confirmação de leitura
mail-compose-request-receipt-hint = O programa de correio do destinatário pode perguntar-lhe, enviar uma sem perguntar ou ignorar o pedido.
mail-compose-track-opens = Registar quando esta mensagem é aberta
mail-compose-track-opens-hint = É adicionada à mensagem uma imagem de 1×1 carregada a partir deste servidor. Muitos programas de correio bloqueiam imagens ou carregam-nas antecipadamente, pelo que uma abertura registada não prova que a mensagem foi lida, e a ausência de registo não prova que não foi. O destinatário não é informado.
mail-compose-image-type = Não é possível colocar { $name } no texto. Utilize uma imagem PNG, JPEG, GIF ou WebP.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = Não tem sessão iniciada.
mail-send-no-draft-folder = Esta conta não tem nenhuma pasta para guardar o rascunho.
mail-send-no-recipient = Adicione pelo menos um destinatário.
mail-send-bad-recipient = { $count ->
    [one] Isto não é um endereço de e-mail: { $addresses }
   *[other] Estes não são endereços de e-mail: { $addresses }
}
mail-send-later-past = A hora de envio não está no futuro. Nada foi enviado.
mail-send-tracking-failed = Não é possível registar aberturas neste momento, pelo que nada foi enviado. Desmarque «Registar quando esta mensagem é aberta» para enviar sem esta opção.
mail-send-draft-refused = O servidor recusou esta mensagem: { $reason }. Nada foi enviado.
mail-send-submission-refused = O servidor recusou enviar esta mensagem: { $reason }. Nada foi enviado.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Desbloquear chave de assinatura
mail-compose-sign-unlock-note = Introduza a frase-passe da chave para assinar esta mensagem. Fica desbloqueada apenas para este editor de mensagens.
mail-compose-sign-unlock = Desbloquear
mail-compose-sign-unlocking = A desbloquear…
mail-compose-sign-no-key = Não há nenhuma chave de assinatura disponível neste dispositivo.
mail-compose-sign-unlock-failed = Não foi possível desbloquear a chave de assinatura.
mail-compose-sign-unlock-required = Desbloqueie a chave de assinatura para enviar esta mensagem assinada.

# Rich-text editor (W1)
mail-compose-format-plain = Texto simples
mail-compose-format-rich = Texto formatado
mail-compose-rt-toolbar = Formatação
mail-compose-rt-bold = Negrito
mail-compose-rt-italic = Itálico
mail-compose-rt-underline = Sublinhado
mail-compose-rt-strike = Rasurado
mail-compose-rt-bullet = Lista com marcas
mail-compose-rt-ordered = Lista numerada
mail-compose-rt-quote = Citação
mail-compose-rt-link = Ligação
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Aplicar
mail-compose-rt-image = Inserir uma imagem
mail-compose-rt-rule = Linha horizontal

# Signature picker (W12)
mail-compose-signature = Assinatura
mail-compose-signature-none = Adicionar uma assinatura…

# Recall (W10)
mail-compose-recall = Recolher
mail-compose-recall-action = Recolher
mail-compose-recall-holding = Retida, a aguardar envio
mail-compose-recall-scheduled = Agendada para { $when }

# Drafts drawer (W9)
mail-compose-drafts = Rascunhos
mail-compose-drafts-empty = Sem rascunhos guardados
mail-compose-drafts-delete = Eliminar rascunho
mail-compose-drafts-no-recipient = (sem destinatário)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Caixa de saída
mail-outbox-refresh = Atualizar
mail-outbox-empty = Nada a aguardar envio
mail-outbox-send-now = Enviar já
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
mail-outbox-release = Libertar
mail-outbox-discard = Descartar
mail-outbox-held-note = Retida. Não é enviada até ser libertada.
mail-outbox-held-note-api-key = Retida — criada pela chave de API { $name }. Não é enviada até ser libertada.
mail-outbox-held-note-oauth = Retida — criada pela aplicação ligada { $name }. Não é enviada até ser libertada.
mail-outbox-origin-api-key = Criada pela chave de API { $name }
mail-outbox-origin-oauth = Criada pela aplicação ligada { $name }
mail-outbox-to = Para { $recipients }
mail-outbox-no-subject = (sem assunto)
mail-outbox-attempts = Tentativas: { $count }. Último erro: { $error }
mail-outbox-error = Erro: { $error }
mail-outbox-toast-canceled = Envio cancelado
mail-outbox-toast-discarded = Descartada. A mensagem não foi enviada.
mail-outbox-toast-sent = Enviada
mail-outbox-toast-retrying = Ainda não enviada: { $error }. Será tentada novamente.
mail-outbox-toast-failed = Não enviada: { $error }
mail-outbox-toast-not-released = Não foi possível enviar: { $error }
mail-outbox-toast-not-canceled = Não foi possível cancelar: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Ligação perdida — a tentar novamente…
mail-net-back = Novamente online
mail-queue-sent = { $count ->
    [one] { $count } alteração em fila enviada
   *[other] { $count } alterações em fila enviadas
}
mail-queue-failed = { $count ->
    [one] Falhou { $count } alteração em fila
   *[other] Falharam { $count } alterações em fila
}
mail-queue-notice-label = Alterações que não foi possível aplicar
mail-queue-notice-title = { $count ->
    [one] Não foi possível aplicar { $count } alteração feita offline
   *[other] Não foi possível aplicar { $count } alterações feitas offline
}
mail-queue-kind-send = Enviar «{ $subject }»
mail-queue-kind-send-untitled = Enviar uma mensagem sem assunto
mail-queue-kind-move = Mover uma mensagem
mail-queue-kind-flag = Alterar uma etiqueta ou sinalização
mail-queue-kind-draft = Guardar um rascunho
mail-queue-kind-pim = Alteração de calendário, tarefa, nota ou contacto
mail-queue-reason = Motivo: { $reason }
mail-queue-reason-unknown = O servidor não indicou o motivo.
mail-queue-retry = Tentar novamente
mail-queue-discard = Descartar
mail-queue-retry-item = Tentar novamente: { $what }
mail-queue-discard-item = Descartar: { $what }
mail-queue-discarded = Descartada. A alteração não foi aplicada.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Limpar mensagens
mail-sweep-title = Limpar { $sender }
mail-sweep-all = Eliminar todas deste remetente
mail-sweep-keep-latest = Manter a mais recente, eliminar as restantes
mail-sweep-older-than = Eliminar as com mais de N dias
mail-sweep-block = Eliminar todas e bloquear este remetente
mail-sweep-days = Dias
mail-sweep-count = { $count ->
    [one] { $count } mensagem será movida para o Lixo
   *[other] { $count } mensagens serão movidas para o Lixo
}
mail-sweep-cancel = Cancelar
mail-sweep-run = Limpar { $count }
mail-sweeping = A limpar…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Dispensar
mail-undo-action = Anular
mail-undo-send-cancel = Cancelar
mail-undo-failed = Não foi possível anular

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Etiqueta adicionada
mail-toast-label-removed = Etiqueta removida
mail-toast-pinned = Mensagem «{ $target }» afixada
mail-toast-unpinned = Mensagem «{ $target }» desafixada
mail-toast-snoozed = Adiada
mail-toast-follow-up-set = Seguimento definido
mail-toast-follow-up-cleared = Seguimento limpo
mail-toast-archived = Arquivada
mail-toast-trashed = Movida para o Lixo
mail-toast-marked-spam = Marcada como spam
mail-toast-moved-to = Movida para { $folder }
mail-toast-moved = Movida para a pasta
mail-toast-archived-count = { $count } mensagens arquivadas
mail-toast-trashed-count = { $count } mensagens movidas para o Lixo
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
mail-toast-no-archive-folder = Sem pasta Arquivo
mail-toast-no-trash-folder = Sem pasta Lixo
mail-toast-no-spam-folder = Sem pasta Spam
mail-toast-sweep-nothing = Nada para limpar
mail-toast-swept = { $count ->
    [one] { $count } mensagem limpa
   *[other] { $count } mensagens limpas
}
mail-toast-load-more-failed = Não foi possível carregar mais mensagens
mail-toast-export-nothing = Nada para exportar
mail-toast-exported = .eml exportado
mail-toast-export-failed = Falha na exportação
mail-toast-send-queued = Em fila — será enviada quando voltar a estar online
mail-toast-tracking-not-bound = A mensagem foi enviada, mas as aberturas não serão registadas.
mail-toast-send-scheduled = Envio agendado
mail-toast-sent = Mensagem enviada
mail-toast-send-canceled = Envio cancelado

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Friso
mail-ribbon-tab-home = Base
mail-ribbon-tab-view = Ver
mail-ribbon-tab-folder = Pasta
mail-ribbon-expand = Expandir
mail-ribbon-collapse = Fechar
mail-ribbon-group-new = Novo
mail-ribbon-compose = Escrever
mail-ribbon-group-session = Sessão
mail-ribbon-logout = Terminar sessão
mail-ribbon-group-theme = Tema
mail-ribbon-group-density = Densidade
mail-ribbon-group-settings = Definições
mail-ribbon-more = Mais…
mail-ribbon-group-folders = Pastas
mail-density-compact = compacta
mail-density-cozy = confortável
mail-density-relaxed = espaçosa

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Nível de segurança da mensagem
mail-maxsec-full = Completo
mail-maxsec-full-hint = HTML limpo com imagens e estilos.
mail-maxsec-no-media = Sem multimédia
mail-maxsec-no-media-hint = HTML limpo sem imagens nem conteúdos multimédia.
mail-maxsec-plain-text = Texto simples
mail-maxsec-plain-text-hint = Sem HTML — a mensagem é apresentada apenas como texto simples.
mail-maxsec-view = Vista: { $mode }
mail-maxsec-menu = Modo de visualização

mail-toast-tab-pinned = Separador «{ $target }» afixado
mail-toast-tab-unpinned = Separador «{ $target }» desafixado

mail-workspace-message = Mensagem
mail-opened-announcement = Mensagem aberta: { $subject }
mail-workspace-layout = Disposição da área de trabalho
mail-workspace-tabs = Separadores
mail-workspace-side-by-side = Lado a lado
mail-workspace-stack = Empilhado
mail-workspace-refresh = Atualizar pasta
mail-workspace-load-error = Não foi possível carregar esta mensagem. Feche o separador e tente novamente.

mail-resize-folders = Redimensionar barra lateral de pastas
mail-resize-list = Redimensionar lista de mensagens
mail-resize-tiles = Redimensionar painéis da área de trabalho
mail-show-folders = Mostrar barra lateral de pastas
mail-hide-folders = Ocultar barra lateral de pastas
mail-compact-folders = Ícones de pastas compactos
mail-toggle-list = Mostrar ou ocultar lista de mensagens
mail-toggle-reader = Mostrar ou ocultar painel de leitura

mail-folder-counts = { $unread } não lidas, { $total } mensagens

mail-print = Imprimir / Guardar como PDF
mail-print-help = Imprima esta mensagem ou escolha «Guardar como PDF» na caixa de diálogo de impressão. As imagens externas continuam bloqueadas.

mail-file-details = Detalhes e impressão digital do ficheiro
mail-file-type = Tipo de ficheiro
mail-file-extension = Extensão
mail-file-detected = Assinatura detetada
mail-file-unknown = Nenhuma assinatura reconhecida
mail-file-size = Tamanho
mail-file-size-bytes = { $count } bytes
mail-file-hash-unavailable = Impressão digital indisponível (requer um navegador seguro e um ficheiro com até 64 MiB).
mail-file-fingerprint-help = Uma impressão digital identifica exatamente estes bytes. Não prova que o ficheiro é seguro nem quem o criou.
mail-file-warning-executable = Este ficheiro pode executar programas ou alterar definições do sistema. Transfira-o apenas se confiar na origem.
mail-file-warning-macro = Este documento pode conter macros. Não ative macros a menos que tenha verificado o remetente e o objetivo.
mail-file-warning-disguisedName = O nome do ficheiro pode disfarçar a extensão real. Verifique os detalhes do ficheiro antes de o transferir.
mail-file-warning-typeMismatch = A assinatura detetada do ficheiro não corresponde ao tipo declarado. A pré-visualização automática foi bloqueada.
mail-file-warning-activeDocument = Este formato pode conter conteúdo ativo. A pré-visualização está isolada; abrir uma cópia transferida pode comportar-se de outra forma.
mail-file-preview-blocked = A pré-visualização automática está desativada para este tipo de ficheiro ou discrepância de tipo.

mail-quote-header-from = De:
mail-quote-header-sent = Enviada:
mail-quote-header-to = Para:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Assunto:

mail-spam-score = Análise de spam
mail-spam-score-loading = A verificar cabeçalhos de spam…
mail-spam-score-unavailable = Pontuação de spam indisponível
mail-spam-score-unverified = Indicado pelos cabeçalhos da mensagem, sem verificação independente. Os cabeçalhos fornecidos pelo remetente podem ser falsificados.
mail-spam-score-positive = O cabeçalho indica spam.
mail-spam-score-negative = O cabeçalho indica um resultado que não é spam.
mail-spam-score-unknown = Não foi fornecido nenhum veredicto com esta pontuação.

mail-external-recipients = Destinatários externos
mail-external-count = { $count } destinatário(s) fora dos domínios de confiança
mail-external-help = Verifique os destinatários em Para, Cc e Bcc antes de enviar informações confidenciais. A confiança baseia-se na identidade de envio selecionada e na política de domínios em vigor.
mail-external-reviewed = Verifiquei estes destinatários externos.
mail-external-confirm-required = Reveja e confirme os destinatários externos antes de enviar.
mail-quick-reply = Resposta rápida
mail-quick-reply-help = Oferece um separador de resposta compacto com os destinatários já preenchidos. Desativado por predefinição. As verificações de segurança e opções de envio habituais continuam a aplicar-se.
mail-quick-reply-expand = Editar destinatários e assunto

mail-attachment-policy-loading = A verificar a política de anexos…
mail-attachment-policy-unavailable = Não foi possível verificar a política de anexos. Tente novamente antes de abrir este ficheiro.
mail-attachment-policy-blocked = O administrador bloqueou este anexo ou a transferência do original.
mail-attachment-policy-warning = O administrador recomenda cautela com este anexo. Continue apenas se confiar na origem.
mail-attachment-policy-acknowledge = Compreendo — continuar
mail-attachment-policy-no-preview = As pré-visualizações de anexos foram desativadas pelo administrador. A transferência do original pode continuar disponível.

mail-time-sent = Enviada
mail-time-received = Recebida
mail-time-unknown = Não indicado
mail-time-comparison = Hora em { $zone }
mail-time-privacy = O fuso de comparação é escolhido por si. As datas e horas das mensagens não identificam a localização de ninguém.
mail-time-event-comparison = Comparação da hora do evento
mail-time-ambiguous = Esta hora local ocorre duas vezes quando a hora muda. São apresentados os dois instantes possíveis; verifique o desvio pretendido do evento antes de guardar.
mail-bulk-add-label = Adicionar etiqueta
mail-select-messages = Selecionar mensagens
mail-bulk-remove-label = Remover etiqueta
mail-mark-spam = Marcar como spam
mail-bulk-forward = Reencaminhar as mensagens selecionadas como anexos
mail-bulk-download = Transferir as mensagens selecionadas como mbox
mail-bulk-forward-subject = Mensagens reencaminhadas ({ $count })
mail-bulk-transfer-failed = Não foi possível preparar as mensagens selecionadas. Selecione até 100 mensagens com um total máximo de 32 MiB e verifique as permissões de exportação da sua conta.
mail-toast-labels-added = Etiqueta { $label } adicionada a { $count } mensagens
mail-toast-labels-removed = Etiqueta { $label } removida de { $count } mensagens

mail-action-help-reply = Responder ao remetente num novo separador de escrita.
mail-action-help-reply-all = Responder ao remetente e aos outros destinatários num novo separador de escrita.
mail-action-help-forward = Rever e reencaminhar esta mensagem para os destinatários que escolher.
mail-action-help-pin = Manter esta mensagem afixada para acesso fácil ou remover a afixação.
mail-action-help-read = Alterar se esta mensagem está marcada como lida. Pode anular a alteração.
mail-action-help-flag = Definir ou remover a sinalização de seguimento da mensagem.
mail-action-help-archive = Mover esta mensagem para a pasta de arquivo configurada.
mail-action-help-delete = Mover esta mensagem para a pasta Lixo. Pode anular a ação.
mail-action-help-spam = Marcar esta mensagem como indesejada e movê-la para a pasta de spam configurada.
mail-action-help-export = Abrir a página de exportação para transferir esta mensagem no formato que escolher.
mail-action-help-sweep = Rever ações para outras mensagens deste remetente antes de as aplicar.

mail-empty-trash = Esvaziar lixo
mail-trash-help = Reveja e elimine permanentemente as mensagens que estão no Lixo. Esta ação não pode ser anulada.
mail-trash-preparing = A preparar um instantâneo das mensagens no Lixo…
mail-trash-prepare-failed = Não foi possível preparar a pré-visualização do Lixo. Verifique a ligação e abra-a novamente. Nenhuma mensagem foi eliminada.
mail-trash-confirm = Eliminar permanentemente { $count } mensagens do Lixo? Esta ação não pode ser anulada. As mensagens que chegarem depois desta pré-visualização não são incluídas; as mensagens retiradas do Lixo são ignoradas.
mail-trash-limit = Esta operação inclui até { $count } mensagens. Restam mais no Lixo; reveja outra operação depois para as remover.
mail-trash-permanent = Eliminar permanentemente
mail-trash-stop = Parar após o pedido atual
mail-trash-progress = Eliminadas: { $deleted }; ignoradas { $skipped } mensagens que já não estão no Lixo; { $remaining } ainda nesta pré-visualização.
mail-trash-stopped = Eliminação parada. Eliminadas com confirmação: { $deleted }; ignoradas: { $skipped }; restantes: { $remaining }; resultado desconhecido: { $uncertain }. Verifique o Lixo antes de iniciar outra operação.
mail-links-title = Ligações da mensagem
mail-links-review = Rever ligações ({ $count })
mail-links-help = As ligações da mensagem só abrem depois de rever o destino aqui. Nenhum destino é obtido automaticamente.
mail-links-inert = Utilize «Rever ligações» acima da mensagem para inspecionar e abrir este destino.
mail-links-limit = São listados os primeiros 100 destinos suportados distintos. As restantes ligações continuam inativas na mensagem.
mail-links-destination = Destino a abrir
mail-links-external = Este destino vem do remetente da mensagem. Ao abri-lo, sai do Mailwoman; reveja o endereço antes de continuar.
mail-links-open = Abrir num novo separador
mail-links-compose = Rever uma nova mensagem
mail-links-warning-http = Este endereço utiliza HTTP sem encriptação.
mail-links-warning-address = Este destino utiliza um endereço IP em vez de um nome de domínio.
mail-links-warning-international = Este domínio contém um nome internacionalizado. Verifique com cuidado a sua grafia em ASCII.
mail-links-warning-mismatch = O endereço web visível indica um anfitrião diferente do destino real.
mail-links-warning-direction = O texto da ligação contém carateres de mudança de direção que podem disfarçar o endereço.

mail-outbox-uncertain = Estado da entrega desconhecido
mail-outbox-uncertain-note = O envio começou, mas não foi possível confirmar a entrega. Esta mensagem não será reenviada automaticamente. Verifique a pasta Enviados ou confirme com o destinatário antes de enviar novamente.

mail-conversation-title = Ver conversa
mail-conversation-help = Percorra as mensagens disponíveis desta conversa na conta, em todas as pastas. Abrir uma mensagem não altera a pasta selecionada.
mail-conversation-count = A mostrar { $loaded } de { $total } mensagens da conversa.
mail-conversation-other-folder = Outra pasta
mail-conversation-loading = A carregar conversa…
mail-conversation-missing = { $count } mensagem(ns) já não está(ão) disponível(eis).
mail-conversation-error = Não foi possível carregar a conversa. Tente novamente.
mail-conversation-more = Carregar mais mensagens
mail-conversation-limit = Esta vista está limitada a { $count } mensagens.


mail-compose-request-delivery = Pedir notificações de entrega
mail-compose-request-delivery-hint = Pede aos servidores de correio de destino que comuniquem a entrega, a falha ou o atraso. Não pede nem prova que alguém leu a mensagem. Os relatórios devolvidos incluem os cabeçalhos, não o corpo completo.
mail-compose-delivery-unavailable = As notificações de entrega não estão disponíveis ou foram desativadas pelo administrador para esta conta. Uma notificação pedida explicitamente não será removida sem aviso.

mail-compose-account-changed = Esta mensagem pertence a outro início de sessão. Inicie sessão na conta correta e volte a abrir o rascunho.
mail-context-open-message = Abrir mensagem
mail-context-open-folder = Abrir pasta
mail-context-download = Transferir mensagem
mail-context-refresh = Atualizar pasta
mail-context-folder-settings = Definições da pasta
mail-context-failed = Não foi possível concluir a ação. Tente novamente.
mail-sharing-unavailable = A partilha de pastas não está disponível ou já não tem permissão para a gerir.

mail-trash-cancelling = A parar após o pedido atual. Eliminadas com confirmação: { $deleted }; ignoradas: { $skipped }; restantes: { $remaining }. As eliminações já aceites não podem ser anuladas.
mail-trash-already-running = A eliminação do Lixo já está em curso. Utilize a respetiva notificação de progresso para parar os lotes seguintes.

mail-context-search-sender = Pesquisar este remetente
mail-context-search-subject = Pesquisar o mesmo assunto
mail-drag-too-many = Arraste até { $count } mensagens de cada vez. Selecione um grupo mais pequeno antes de mover.

mail-subtab-reorder-help = Arraste para reordenar os separadores ou prima Ctrl+Shift+Esquerda ou Direita no separador ativo.

mail-compose-ribbon = Friso do editor
mail-compose-ribbon-format = Formatar texto
mail-compose-ribbon-insert = Inserir
mail-compose-font = Tipo de letra
mail-compose-font-size = Tamanho
mail-compose-font-inherit = Padrão
mail-compose-undo = Anular
mail-compose-redo = Refazer
mail-compose-undo-tip = Anular ({ $shortcut })
mail-compose-redo-tip = Refazer ({ $shortcut })
mail-workspace-limit = Feche um separador antes de abrir outro. Podem ser mantidos até 32 separadores.
mail-compose-private-storage-unavailable = A recuperação local de rascunhos não está disponível nesta sessão. Mantenha este editor aberto até enviar ou copiar a mensagem.
mail-offline-storage-unavailable = As alterações offline não estão disponíveis até esta sessão ter verificado a autoridade de armazenamento. Restabeleça a ligação e inicie sessão novamente.

mail-draft-status-idle = Ainda não guardado na conta
mail-draft-status-pending = Alterações não guardadas
mail-draft-status-saving = A guardar em Rascunhos…
mail-draft-status-saved = Guardado nos Rascunhos da conta
mail-draft-status-error = O rascunho não foi guardado. A sua cópia de recuperação local continua disponível.
mail-draft-status-uncertain = Falta a confirmação do guardar. Verifique os Rascunhos antes de criar outra cópia; o guardar automático está em pausa.
mail-draft-status-conflict = Este rascunho foi alterado noutro dispositivo. Volte a abrir a cópia da conta antes de continuar.
mail-draft-status-unsupported = De momento, esta conta não consegue confirmar que guarda na pasta Rascunhos.
mail-draft-protected = O conteúdo citado encriptado ou desencriptado não é guardado como texto simples. Qualquer rascunho anterior na conta permanece até o descartar explicitamente.
mail-draft-save-now = Guardar em Rascunhos
mail-draft-save-failed = Não foi possível guardar este rascunho na sua conta. Verifique o editor de mensagens antes de o fechar.
mail-draft-uncertain = O rascunho pode ter sido guardado. As novas tentativas automáticas estão em pausa; verifique a pasta Rascunhos.
mail-draft-conflict = Outro dispositivo alterou este rascunho. As suas edições não substituíram essa cópia.
mail-draft-cleanup-pending = O novo rascunho foi guardado, mas não foi possível remover a cópia anterior. Reveja os Rascunhos antes de guardar novamente.
mail-draft-account-list = Rascunhos da conta
mail-draft-loading = A carregar rascunhos da conta…
mail-draft-close-recovery = Fechar e manter só a recuperação local
mail-draft-discard-account = Descartar rascunho da conta
mail-draft-discard-pending = Não foi possível confirmar a remoção do rascunho. O registo na conta permanece; tente «Descartar» novamente depois de rever a pasta Rascunhos.
mail-draft-limit = Esta conta já tem 100 rascunhos guardados. Descarte um rascunho da conta que não pretenda antes de guardar um novo.
mail-draft-quota-help = São mantidos até 100 rascunhos na conta. «Descartar» remove a cópia confirmada no servidor antes de libertar o espaço.
mail-draft-discard-confirm = Confirmar remoção do rascunho
mail-draft-discard-confirm-detail = Remover permanentemente «{ $subject }» dos Rascunhos da conta? Se, em vez disso, fechar o editor de mensagens, o rascunho é mantido.
# The discard confirmation dialog (t42): Keep editing is the safe default.
mail-draft-discard-title = Descartar este rascunho?
mail-draft-keep-editing = Continuar a editar
mail-draft-discard-action = Descartar
mail-draft-close-title = Descartar alterações não guardadas?
mail-draft-close-protected = Esta mensagem contém conteúdo encriptado ou desencriptado, pelo que não é guardada como rascunho. Fechá-la descarta estas alterações. Um rascunho anterior na conta, se existir, permanece nos Rascunhos.
mail-draft-close-unstored = Não é possível guardar rascunhos nesta sessão, pelo que fechar esta mensagem descarta-a.

mail-workspace-view-mode = Disposição das vistas
mail-workspace-tabs-help = Mostra uma vista ativa e mantém os outros separadores abertos.
mail-workspace-side-by-side-help = Mostra duas vistas selecionadas lado a lado. Arraste um separador para qualquer uma das vistas para a substituir.
mail-workspace-stack-help = Mostra duas vistas selecionadas uma por cima da outra. Arraste um separador para qualquer uma das vistas para a substituir.
mail-workspace-masonry-side-by-side = Mosaico lado a lado
mail-workspace-masonry-side-by-side-help = Dispõe todas as vistas abertas em colunas desfasadas. Em ecrãs pequenos é mostrado o separador ativo.
mail-workspace-masonry-stack = Mosaico empilhado
mail-workspace-masonry-stack-help = Dispõe todas as vistas abertas em linhas desfasadas. Em ecrãs pequenos é mostrado o separador ativo.
mail-workspace-view-first = Primeira vista
mail-workspace-view-second = Segunda vista
mail-workspace-view-empty = Arraste um separador aberto para aqui para escolher esta vista.

mail-popup-blocked = Permita janelas de pop-up para o Mailwoman para destacar este separador. O separador permanece aqui.
mail-popup-unavailable = Não foi possível carregar a janela destacada. O separador e as alterações não guardadas permanecem na janela principal. Tente novamente depois de atualizar a aplicação.
mail-popup-reattach = Voltar a juntar à janela principal
mail-popup-action-reattached = Voltou à janela principal para concluir esta ação.
mail-popup-detached = Este separador está aberto numa janela separada. Ao fechar essa janela, regressa aqui.

mail-compose-from-custom = Outro endereço autorizado…
mail-compose-from-alias = Endereço do remetente desta mensagem
mail-compose-from-alias-help = Utilize um alias a partir do qual tenha autorização para enviar. Altera apenas este rascunho; a autorização é verificada antes do envio.
mail-compose-from-refused = Este endereço não está autorizado para envio a partir desta conta.
mail-compose-from-unavailable = Não foi possível verificar este endereço de remetente. A mensagem não foi enviada.
mail-compose-from-invalid = Introduza um único endereço de e-mail válido para o remetente.

mail-compose-from-toggle = Mostrar ou ocultar o remetente desta mensagem

mail-compose-ribbon-security = Segurança
mail-compose-ribbon-delivery = Entrega
mail-compose-ribbon-recipients = Destinatários
mail-compose-ribbon-message-type = Tipo de mensagem

mail-compose-recall-help = Cancele mensagens ainda retidas para anulação ou com entrega agendada. As mensagens já entregues não podem ser recolhidas.
mail-compose-recall-loading = A verificar mensagens em fila…
mail-compose-recall-empty = Não há mensagens em fila que possam ser canceladas.
mail-compose-recall-failed = Não foi possível confirmar o estado da fila. Feche e volte a abrir este painel para o atualizar antes de outra ação.

mail-compose-importance = Importância
mail-compose-importance-high = Alta
mail-compose-importance-normal = Normal
mail-compose-importance-low = Baixa

mail-insert-account-title = Inserir item da conta
mail-insert-account-help = Anexe uma exportação de calendário, um cartão de visita de contacto ou uma mensagem original desta conta.
mail-insert-account-close = Fechar seletor de itens da conta
mail-insert-account-kind = Tipo de item
mail-insert-account-event = Evento de calendário (.ics)
mail-insert-account-calendar = Calendário (.ics)
mail-insert-account-contact = Cartão de visita de contacto (.vcf)
mail-insert-account-message = Mensagem original (.eml)
mail-insert-account-search = Filtrar itens listados
mail-insert-account-selection = Selecione um item
mail-insert-account-scope = Só são listados itens da conta atual. As mensagens originais seguem a política de exportação do servidor.
mail-insert-account-limited = A mostrar até 100 itens. O filtro pesquisa apenas nesta lista.
mail-insert-account-unavailable = Não há exportações de conta suportadas disponíveis.
mail-insert-account-empty = Nenhum item correspondente.
mail-insert-account-attach = Anexar item selecionado
mail-insert-account-loading = A carregar item da conta…
mail-insert-account-error = Não foi possível anexar o item. Verifique o seu acesso e a política de exportação e tente novamente. Calendários grandes podem exigir a exportação de eventos individuais.

mail-compose-html-source = Código HTML
mail-compose-html-help = Edite o HTML diretamente. Só é guardada e enviada formatação de e-mail suportada; scripts, imagens externas e marcação insegura são removidos.

mail-compose-check-names = Verificar nomes
mail-compose-check-names-help = Resolve nomes exatos a partir dos contactos carregados e verifica a formatação dos endereços. Não verifica a entrega na caixa de correio.
mail-compose-names-unresolved = Escolha um endereço de e-mail completo para estes nomes não resolvidos ou ambíguos: { $names }
mail-compose-names-checked = Formatação dos destinatários verificada; { $count } nome(s) de contacto resolvido(s). A entrega na caixa de correio não foi verificada.

mail-compose-ribbon-table = Tabela
# Captions of the composer ribbon's 3-row groups (t38); also the folded group's dropdown text.
mail-compose-group-clipboard = Copiar e colar
mail-compose-group-paragraph = Parágrafo
mail-compose-group-editing = Edição
mail-compose-group-view = Ver
mail-compose-dark-background = Fundo escuro
mail-compose-dark-background-help = Mostra a mensagem sobre um fundo escuro enquanto escreve. Só muda o seu ecrã: a mensagem é enviada sem cor de fundo.
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
    [one] { $count } caráter
   *[other] { $count } carateres
}
mail-editor-selection-required = Selecione texto ou células de tabela adequados antes de utilizar esta ação.
mail-editor-clipboard-unavailable = Não foi possível aceder à área de transferência. Utilize o atalho Copiar, Cortar ou Colar do navegador.
mail-editor-selection-changed = A seleção mudou. Selecione o texto pretendido e tente novamente.
mail-editor-clipboard = Área de transferência e formatação
mail-editor-copy = Copiar
mail-editor-cut = Cortar
mail-editor-paste = Colar
mail-editor-copy-format = Copiar formatação
mail-editor-apply-format = Aplicar formatação
mail-editor-style = Estilo de parágrafo
mail-editor-text-color = Cor do texto
mail-editor-clear-color = Cor predefinida
mail-editor-highlight = Realce
mail-editor-clear-highlight = Limpar realce
mail-editor-paragraph = Opções de parágrafo
mail-editor-alignment = Alinhamento
mail-editor-line-height = Entrelinha
mail-editor-space-before = Espaço antes
mail-editor-space-after = Espaço depois
mail-editor-indent = Avanço
mail-editor-shading = Sombreado do parágrafo
mail-editor-clear-shading = Limpar sombreado
mail-editor-case = Maiúsc./minúsc.
mail-editor-sort = Ordenar parágrafos
mail-editor-sort-alpha = Alfabética
mail-editor-sort-numeric = Ordem numérica
mail-editor-table-rows = Linhas da tabela
mail-editor-table-columns = Colunas da tabela
mail-editor-table-insert = Inserir tabela
mail-editor-table-limits = Até 20 linhas, 10 colunas e 200 células por tabela; no máximo 10 tabelas. Não são suportadas tabelas aninhadas.
mail-editor-table-row-before = Linha acima
mail-editor-table-row-after = Linha abaixo
mail-editor-table-row-delete = Eliminar linha
mail-editor-table-column-before = Coluna antes
mail-editor-table-column-after = Coluna depois
mail-editor-table-column-delete = Eliminar coluna
mail-editor-table-merge = Unir células
mail-editor-table-split = Dividir célula
mail-editor-table-header = Linha de cabeçalho
mail-editor-table-delete = Eliminar tabela
mail-editor-table-shading = Sombreado
mail-editor-find-replace = Localizar e substituir
mail-editor-find = Localizar texto
mail-editor-replacement = Substituir por
mail-editor-match-case = Maiúsculas/minúsculas
mail-editor-previous = Resultado anterior
mail-editor-next = Resultado seguinte
mail-editor-replace = Substituir resultado selecionado
mail-editor-replace-all = Substituir todos os resultados
mail-editor-matches = { $count } resultados (até 1000)
mail-editor-select-all = Selecionar tudo
mail-editor-clear-selection = Limpar seleção
mail-editor-history = Histórico de anulação
mail-editor-undo-groups = Anular os últimos { $count } grupos de alterações
mail-editor-zoom = Zoom do editor
mail-editor-markers = Marcas de parágrafo
mail-editor-style-normal = Normal
mail-editor-style-no-spacing = Sem espaçamento
mail-editor-style-title = Título
mail-editor-style-subtitle = Subtítulo
mail-editor-style-heading1 = Cabeçalho 1
mail-editor-style-heading2 = Cabeçalho 2
mail-editor-style-heading3 = Cabeçalho 3
mail-editor-style-heading4 = Cabeçalho 4
mail-editor-style-heading5 = Cabeçalho 5
mail-editor-style-heading6 = Cabeçalho 6
mail-editor-align-left = Esquerda
mail-editor-align-center = Centro
mail-editor-align-right = Direita
mail-editor-align-justify = Justificar
mail-editor-case-lower = minúsculas
mail-editor-case-upper = MAIÚSCULAS
mail-editor-case-title = Inicial Maiúscula
mail-editor-case-sentence = Tipo frase

mail-editor-action-history = Histórico: { $action }
mail-editor-history-empty = Nenhuma ação disponível
mail-editor-redo-groups = Refazer { $count } alterações
mail-compose-identities-failed = Não foi possível carregar as identidades de envio. Feche e volte a abrir este rascunho para tentar novamente.
mail-folders-loading = A carregar pastas…
mail-initial-sync-error = Ainda não foi possível sincronizar as suas mensagens.
mail-initial-sync-error-help = A ligação ao seu servidor de correio foi interrompida. Tente novamente ou, se isto continuar a acontecer, peça ao administrador que verifique os registos de ligação.
mail-links-trusted = Domínio de confiança: abre sem revisão
mail-links-trust-domain = Confiar em { $domain }
mail-links-trust-failed = Não foi possível guardar o domínio de confiança. Tente novamente.
mail-focus-always-focused = Mover sempre para Destaques
mail-focus-always-other = Mover sempre para Outras
mail-focus-moved-focused = As mensagens de { $sender } passam a ir para Destaques.
mail-focus-moved-other = As mensagens de { $sender } passam a ir para Outras.
mail-focus-save-failed = Não foi possível guardar a escolha da caixa de entrada destacada para este remetente. Tente novamente.

mail-context-search-domain = Pesquisar mensagens deste domínio

mail-loading-mailbox = A carregar { $name }…
mail-user-folders = Pastas do utilizador
mail-context-hide-folder = Ocultar da barra lateral
mail-folder-fallback = pasta
mail-toast-moving = { $count ->
    [one] A mover uma mensagem para { $folder }…
   *[other] A mover { $count } mensagens para { $folder }…
    }
mail-toast-move-queueing = { $count ->
    [one] A colocar uma mensagem na fila para mover para { $folder }…
   *[other] A colocar { $count } mensagens na fila para mover para { $folder }…
    }
mail-toast-move-queued = { $count ->
    [one] Uma mensagem na fila para mover para { $folder }
   *[other] { $count } mensagens na fila para mover para { $folder }
    }
mail-toast-move-partial = { $count } de { $total } mensagens movidas para { $folder }; não foi possível mover { $failed }
