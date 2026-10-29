# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: settings. Source: ../en/settings.ftl

settings-title = Definições
settings-appearance = Aspeto
settings-close = Fechar definições

settings-theme = Tema
settings-density = Densidade
settings-accent = Cor de destaque
settings-badge-color = Cor do contador de não lidas
settings-badge-color-accent = Igual à cor de destaque
settings-badge-color-option = Contadores: { $color }
settings-font = Tipo de letra da interface
settings-layout = Disposição
settings-appearance-theme-card = Tema e alternância
settings-appearance-theme-card-help = Escolha um tema e se este acompanha o sistema ou a hora do dia.
settings-appearance-colours-card = Cores
settings-appearance-colours-card-help = A cor de destaque usada nas ações e na seleção, e a cor dos contadores de não lidas.
settings-appearance-text-card = Texto e disposição
settings-appearance-text-card-help = A densidade da interface, o tipo de letra que utiliza e a disposição do correio.

# Density options
settings-density-compact = Compacta
settings-density-cozy = Confortável
settings-density-relaxed = Espaçosa

# Interface-font options
settings-font-default = Predefinido
settings-font-system = Sistema
settings-font-serif = Serifado
settings-font-mono = Monoespaçado

# Layout options
settings-layout-default = Predefinida
settings-layout-ribbon = Friso

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Cancelar
settings-edit = Editar
settings-delete = Eliminar
settings-save = Guardar
settings-saved = Guardado

# Two-factor authentication (S1)
settings-2fa-title = Autenticação de dois fatores
settings-2fa-intro = Adicione um segundo fator para exigir mais do que uma palavra-passe ao iniciar sessão.
settings-2fa-policy-required = A sua organização exige um segundo fator nesta conta.
settings-2fa-enabled = Ativada
settings-2fa-recovery-title = Códigos de recuperação
settings-2fa-recovery-once = Guarde-os agora. São mostrados uma única vez e não podem ser obtidos novamente. Cada código funciona uma vez.
settings-2fa-recovery-ack = Guardei estes códigos
settings-2fa-totp-title = Aplicação de autenticação
settings-2fa-totp-enrol = Configurar uma aplicação de autenticação
settings-2fa-totp-scan = Adicione este segredo à sua aplicação de autenticação e, em seguida, introduza o código que esta mostra.
settings-2fa-totp-uri-link = Abrir numa aplicação de autenticação
settings-2fa-totp-code-label = Código de autenticação
settings-2fa-totp-confirm = Confirmar
settings-2fa-totp-disable = Remover aplicação de autenticação
settings-2fa-passkey-title = Chaves de acesso
settings-2fa-passkey-remove = Remover
settings-2fa-passkey-unsupported = Este navegador não suporta chaves de acesso.
settings-2fa-passkey-label-placeholder = Nome da chave de acesso (opcional)
settings-2fa-passkey-add = Adicionar uma chave de acesso
settings-2fa-recovery-remaining = { $count } { $count ->
        [one] código de recuperação restante
       *[other] códigos de recuperação restantes
    }
settings-2fa-recovery-regenerate = Gerar novos códigos de recuperação
settings-2fa-recovery-code-label = Código de recuperação
settings-2fa-error-generic = Não funcionou. Tente novamente.
settings-2fa-error-code = Não foi possível verificar o código.
settings-2fa-error-passkey = Não foi possível registar a chave de acesso.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Autenticação de dois fatores
settings-2fa-challenge-intro = Confirme o seu segundo fator para concluir o início de sessão.
settings-2fa-challenge-failed = Não foi possível verificar. Tente novamente.
settings-2fa-challenge-use-passkey = Utilizar uma chave de acesso
settings-2fa-challenge-method = Escolha um método
settings-2fa-challenge-totp-tab = Código de autenticação
settings-2fa-challenge-recovery-tab = Código de recuperação
settings-2fa-challenge-verify = Verificar

# Active sessions (S11)
settings-sessions-title = Sessões ativas
settings-sessions-intro = Sessões atualmente iniciadas nesta conta.
settings-sessions-last-seen = Última atividade: { $when }
settings-sessions-revoke = Terminar sessão
settings-sessions-current = Esta sessão
settings-sessions-revoke-others = Terminar { $count } { $count ->
        [one] outra sessão
       *[other] outras sessões
    }
settings-sessions-error = Não foi possível alterar a sessão.

# Signatures (W12)
settings-sig-title = Assinaturas
settings-sig-intro = Texto acrescentado às mensagens que envia. Uma das assinaturas pode ser a predefinida.
settings-sig-default = Predefinida
settings-sig-new = Nova assinatura
settings-sig-name-label = Nome
settings-sig-body-label = Assinatura
settings-sig-default-label = Utilizar como predefinida
settings-sig-error-name = Atribua um nome à assinatura.
settings-sig-error-generic = Não foi possível guardar a assinatura.

# Identities
settings-ident-title = Identidades
settings-ident-intro = O nome e o endereço com que envia. Cada identidade pode utilizar a sua própria assinatura.
settings-ident-new = Nova identidade
settings-ident-name-label = Nome a apresentar
settings-ident-email-label = Endereço de e-mail
settings-ident-replyto-label = Endereço de resposta (opcional)
settings-ident-signature-label = Assinatura predefinida
settings-ident-signature-none = Nenhuma
settings-ident-error-fields = Introduza um nome e um endereço de e-mail válido.
settings-ident-error-generic = Não foi possível guardar a identidade.

# Notification rules + quiet hours (W15)
settings-notif-title = Notificações
settings-notif-intro = Escolha que mensagens novas geram notificações e quando ficar em silêncio.
settings-notif-enabled-label = Mostrar notificações de correio novo
settings-notif-quiet-title = Horas de silêncio
settings-notif-quiet-enabled-label = Silenciar notificações num horário definido
settings-notif-quiet-start = De
settings-notif-quiet-end = Até
settings-notif-rules-title = Regras
settings-notif-rule-match-placeholder = Remetente, caixa de correio ou assunto contém…
settings-notif-rule-action-label = Ação
settings-notif-rule-notify = Notificar
settings-notif-rule-mute = Silenciar
settings-notif-rule-add = Adicionar uma regra
settings-notif-error = Não foi possível guardar as definições de notificação.

# Saved searches → search folders (W13)
settings-search-title = Pesquisas guardadas
settings-search-intro = Mostrar uma pesquisa guardada como pasta na lista de caixas de correio.
settings-search-empty = Ainda não tem pesquisas guardadas.
settings-search-as-folder = Mostrar { $name } como pasta
settings-search-as-folder-label = Mostrar como pasta
settings-search-error = Não foi possível alterar a pesquisa guardada.

# Keyboard shortcuts (W14)
settings-kbd-title = Atalhos de teclado
settings-kbd-preset-title = Conjunto de atalhos
settings-kbd-intro = Escolha um conjunto de atalhos. As alterações aplicam-se neste dispositivo.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Desativados (sem atalhos)
settings-kbd-action-compose = Escrever
settings-kbd-action-archive = Arquivar
settings-kbd-action-reply = Responder
settings-kbd-action-next = Mensagem seguinte
settings-kbd-action-previous = Mensagem anterior
settings-kbd-action-search = Pesquisar

# Offline cache (W16)
settings-offline-title = Cache offline
settings-offline-intro = Quanto correio manter neste dispositivo para utilização offline e como libertar espaço.
settings-offline-budget-label = Limite da cache (MB)
settings-offline-retention-label = Manter durante (dias)
settings-offline-strategy-label = Quando o limite for atingido
settings-offline-lru = Remover o utilizado há mais tempo
settings-offline-oldest = Remover primeiro o mais antigo
settings-offline-manual = Só quando eu a limpar
settings-offline-purge = Limpar a cache offline agora

# Interface direction (W20)
settings-dir-title = Direção da interface
settings-dir-intro = Seguir o idioma ou forçar uma direção. A direção da direita para a esquerda espelha a disposição.
settings-dir-auto = Automática
settings-dir-ltr = Da esquerda para a direita
settings-dir-rtl = Da direita para a esquerda
settings-dir-preview-title = Pré-visualização
settings-dir-preview-body = Este texto fica espelhado quando a direção é da direita para a esquerda.
settings-editor-discard-title = Rejeitar alterações não guardadas?
settings-editor-discard-detail = As suas alterações não foram guardadas. Continue a editar ou rejeite este rascunho para sair do editor.
settings-editor-keep = Continuar a editar
settings-editor-discard = Rejeitar alterações
settings-editor-missing = Este item já não existe. Volte à lista e escolha outro item.
settings-editor-back = Voltar à lista
settings-sig-edit-title = Editar assinatura
settings-ident-edit-title = Editar identidade

settings-sections = Secções das definições
# t66: category headings in the Settings navigation.
settings-category-account = Conta
settings-category-mail = Correio
settings-category-interface = Interface
settings-category-integrations = Integrações
settings-category-about = Acerca
settings-nav-preferences = Teclado e apresentação
settings-nav-rules = Regras de correio
settings-nav-access = Acesso à API
settings-nav-metadata = Informações do servidor

settings-folders-title = Pastas e barra lateral
settings-folders-help = Afixe pastas acima das outras, escolha um ícone e arraste ou utilize os botões de mover para ordenar as pastas dentro de cada grupo. Estas opções são sincronizadas com esta conta e não movem mensagens nem mudam o nome das pastas.
settings-folders-pin = Afixar { $name }
settings-folders-icon = Ícone de { $name }
settings-folders-automatic = Função automática da pasta
settings-folders-drag = Arrastar { $name } para reordenar
settings-folders-up = Mover { $name } para cima
settings-folders-down = Mover { $name } para baixo
settings-folders-sync-idle = Inicie sessão para sincronizar as preferências de pastas.
settings-folders-sync-loading = A carregar as preferências da conta…
settings-folders-sync-saving = A guardar as preferências da conta…
settings-folders-sync-synced = Preferências sincronizadas com esta conta.
settings-folders-sync-error = Não foi possível sincronizar as preferências. As suas alterações mantêm-se nesta sessão e a sincronização será repetida quando a ligação voltar.
settings-folder-icon-folder = Pasta
settings-folder-icon-star = Estrela
settings-folder-icon-flag = Bandeira
settings-folder-icon-tag = Etiqueta
settings-folder-icon-clock = Relógio
settings-folder-icon-calendar = Calendário
settings-folder-icon-file = Documento
settings-folder-icon-image = Imagem
settings-folder-icon-lock = Cadeado
settings-folder-icon-users = Pessoas
settings-folder-icon-archive = Arquivo
settings-folder-icon-mail = Envelope
settings-folder-icon-attachment = Anexo
settings-folder-icon-check = Visto
settings-folder-icon-alert = Alerta

settings-search-new = Nova pasta de pesquisa
settings-search-editor = Pasta de pesquisa
settings-search-editor-help = As mensagens têm de cumprir todas as condições. Deixe uma condição vazia para a ignorar. As pastas de pesquisa mostram as mensagens correspondentes sem as mover nem copiar.
settings-search-name = Nome da pasta
settings-search-name-required = Introduza um nome para esta pasta de pesquisa.
settings-search-invalid = Esta pesquisa guardada tem um filtro inválido. Não pode ser editada em segurança.
settings-search-saved = Pasta de pesquisa «{ $name }» guardada.
settings-search-field-text = Pesquisar todo o texto da mensagem
settings-search-field-from = O remetente contém
settings-search-field-to = O destinatário contém
settings-search-field-subject = O assunto contém
settings-search-field-body = O corpo da mensagem contém
settings-search-field-filename = O nome do ficheiro anexo contém
settings-search-field-inMailbox = Limitar ao identificador da caixa de correio (opcional)
settings-search-field-after = Recebida depois de (data e hora UTC)
settings-search-field-before = Recebida antes de (data e hora UTC)
settings-search-attachment = Anexos
settings-search-any = Qualquer mensagem
settings-search-with-attachments = Com anexos
settings-search-without-attachments = Sem anexos
settings-search-unread = Só mensagens não lidas
settings-search-flagged = Só mensagens sinalizadas

settings-sessions-search = Pesquisar conta, dispositivo ou IP
settings-sessions-account = Conta
settings-sessions-device = Dispositivo e cliente
settings-sessions-location = IP e último país conhecido
settings-sessions-location-unavailable = País indisponível
settings-sessions-activity = Atividade
settings-sessions-actions = Ações
settings-sessions-revoke-device = Terminar sessão em { $device }

settings-font-inter = Inter
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost

settings-compose-title = Escrita e respostas
settings-quote-style = Estilo do cabeçalho de resposta e reencaminhamento
settings-quote-classic = Resposta clássica com citação
settings-quote-outlook = Cabeçalho ao estilo do Outlook
settings-quote-help = O estilo Outlook acrescenta De, Enviado, Para, Cc e Assunto acima da mensagem original, em texto formatado e em texto simples. Os rascunhos existentes não são alterados.
settings-external-warning = Avisos de destinatários externos
settings-external-off = Desativados
settings-external-warn = Mostrar um aviso
settings-external-confirm = Exigir confirmação antes de enviar
settings-external-help = Compara Para, Cc e Bcc com o domínio da identidade selecionada e com a lista de domínios de confiança. Isto protege os envios feitos neste editor de mensagens; não é uma regra de entrega do servidor.
settings-trusted-domains = Domínios de destinatários de confiança
settings-trusted-domains-help = Introduza até 100 domínios exatos, um por linha. Os subdomínios têm de ser indicados em separado. O domínio da própria identidade de envio é sempre de confiança. Não são aceites URLs, endereços de e-mail nem carateres universais.
settings-trusted-domains-invalid = Introduza até 100 nomes de domínio válidos, sem URLs, endereços de e-mail nem carateres universais.
settings-trusted-domains-save = Guardar domínios de confiança

settings-shortcuts-enabled = Ativar atalhos da aplicação
settings-shortcuts-action = Ação
settings-shortcuts-binding = Atribuição
settings-shortcuts-edit = Editar atalho
settings-shortcuts-edit-action = Editar o atalho de { $action }
settings-shortcuts-help = Utilize inherit para o conjunto selecionado, disabled para nenhum atalho, ou uma combinação como Ctrl+Shift+K. Ctrl corresponde a Cmd no macOS. As atribuições em conflito ficam inativas. Os campos de texto e as caixas de diálogo mantêm as teclas habituais; as combinações reservadas pelo navegador podem não estar disponíveis.
settings-shortcuts-invalid = Introduza inherit, disabled ou uma única tecla válida, com os modificadores opcionais Ctrl, Alt e Shift.
settings-shortcuts-conflict = Duas ações utilizam o mesmo atalho. Altere uma das atribuições; os atalhos em conflito não serão executados.
settings-shortcuts-discard = Rejeitar a alteração de atalho não guardada?
settings-shortcuts-save = Guardar atalho
settings-shortcuts-back = Voltar aos atalhos
settings-shortcuts-off = O conjunto Desativados desativa todos os atalhos, mesmo as atribuições personalizadas. Escolha outro conjunto em Preferências para utilizar estas atribuições.
settings-kbd-action-replyAll = Responder a todos
settings-kbd-action-forward = Reencaminhar
settings-kbd-action-refresh = Atualizar mensagens
settings-kbd-action-settings = Abrir definições
settings-kbd-action-trash = Mover para o lixo
settings-kbd-action-markRead = Marcar como lida
settings-kbd-action-markUnread = Marcar como não lida
settings-kbd-action-spam = Marcar como spam
settings-kbd-action-zoomIn = Ampliar mensagem
settings-kbd-action-zoomOut = Reduzir mensagem
settings-kbd-action-zoomReset = Repor zoom da mensagem
settings-kbd-action-renameFolder = Mudar o nome da pasta

settings-notif-device-title = Permissão neste dispositivo
settings-notif-device-help = As suas preferências de notificação acompanham a sua conta. A permissão do navegador pertence a este dispositivo; o Mailwoman só a pede quando opta por permiti-la.
settings-notif-permission-default = A permissão do navegador ainda não foi pedida.
settings-notif-permission-granted = As notificações do navegador estão permitidas neste dispositivo.
settings-notif-permission-denied = As notificações estão bloqueadas. Altere a permissão deste site nas definições do navegador para as permitir.
settings-notif-permission-unsupported = Este navegador não suporta notificações do sistema aqui.
settings-notif-permission-request = Permitir notificações neste dispositivo
settings-notif-permission-error = O navegador não conseguiu pedir a permissão de notificações.
settings-notif-timezone = Fuso horário das horas de silêncio (IANA)
settings-notif-timezone-device = Utilizar o fuso horário deste dispositivo

settings-permissions-background-title = Notificações em segundo plano neste navegador
settings-permissions-background-help = Guarde primeiro as preferências de notificação e permita as notificações do navegador. Registe este navegador para receber a atividade da conta quando a aplicação estiver fechada.
settings-permissions-push-unsupported = O Web Push em segundo plano não está disponível neste navegador.
settings-permissions-push-registered = Este navegador tem uma subscrição push. Registe-a para a associar à conta atual.
settings-permissions-push-unregistered = Este navegador não tem nenhuma subscrição push.
settings-permissions-push-register = Registar notificações em segundo plano
settings-permissions-push-remove = Remover subscrição do navegador
settings-permissions-push-policy = O seu administrador não permite notificações push em segundo plano.
settings-permissions-storage-title = Armazenamento do navegador
settings-permissions-storage-help = O armazenamento persistente protege os dados locais deste site contra a remoção automática, quando o navegador o concede. Isto aplica-se apenas a este dispositivo e não altera a retenção no servidor. Pode limpar os dados do site nas definições do navegador; as alterações offline não enviadas podem perder-se.
settings-permissions-storage-unsupported = Os pedidos de armazenamento persistente não estão disponíveis neste navegador.
settings-permissions-storage-persistent = O navegador concedeu armazenamento persistente a este site.
settings-permissions-storage-evictable = O navegador pode remover automaticamente os dados locais deste site.
settings-permissions-storage-usage = Armazenamento atual do site: { $mib } MiB.
settings-permissions-storage-request = Pedir armazenamento persistente
settings-permissions-storage-policy = O seu administrador não permite pedidos de armazenamento persistente.
settings-permissions-error = Não foi possível concluir a ação do navegador. Verifique as definições guardadas da conta e as permissões deste site no navegador.

settings-notif-background-rules = As regras de conteúdo são avaliadas enquanto o Mailwoman está aberto. As notificações do sistema em segundo plano são suprimidas enquanto existirem regras de conteúdo configuradas, porque um despertar opaco em segundo plano não contém os detalhes da mensagem.

settings-time-title = Fusos horários
settings-time-help = Escolha como são apresentadas as horas das mensagens e, opcionalmente, um fuso de comparação para agendamentos. A localização nunca é deduzida a partir dos endereços de e-mail.
settings-time-displayZone = Fuso horário de apresentação
settings-time-displayZone-help = Utilize local para este dispositivo ou um nome IANA, como Europe/Lisbon.
settings-time-comparisonZone = Fuso horário de comparação
settings-time-comparisonZone-help = Introduza um nome IANA, como America/New_York, ou deixe vazio para ocultar as comparações.
settings-time-helpers = Mostrar auxiliares de fuso horário
settings-time-save = Guardar fuso horário
settings-time-invalid = Introduza um fuso horário IANA válido. Utilize local apenas no fuso de apresentação, ou deixe vazio o fuso de comparação.
settings-time-discard = Rejeitar as alterações de fuso horário não guardadas?

settings-folders-name = Pasta
settings-folders-actions = Ações
settings-folders-edit = Editar o aspeto de { $name }
settings-folders-icon-help = Escolha um de cinquenta ícones temáticos ou mantenha a seleção automática com base na finalidade da pasta. Esta opção acompanha a sua conta e não muda o nome da caixa de correio.
settings-folders-save = Guardar aspeto da pasta
settings-folders-back = Voltar às pastas
settings-folders-discard = Rejeitar as alterações de aspeto da pasta não guardadas?
settings-folder-labels = Etiquetas da pasta
settings-folder-labels-help = Organize esta pasta com até 32 etiquetas da conta. Isto não etiqueta nem move as mensagens que contém. Faça a gestão dos nomes e das cores das etiquetas em Etiquetas.

settings-folders-leave-pending = Estas alterações ainda estão em fila para sincronização. Sair deste editor?
settings-folders-retry = Tentar guardar novamente
settings-folder-role-title = Pastas especiais
settings-folder-role-help = Escolha a pasta real utilizada para cada função de correio. A opção Automática utiliza a deteção do servidor. A opção Utilizar a predefinição do administrador remove a sua substituição. Destinos em conflito ou em falta desativam essa função, em vez de ser escolhida outra pasta.
settings-folder-role-auto = Deteção automática
settings-folder-role-inherit = Utilizar a predefinição do administrador
settings-folder-role-path = Caminho configurado: { $path }
settings-folder-role-missing = A pasta configurada não está disponível
settings-folder-role-conflict = Este destino está em falta ou entra em conflito com outra função. Essa função fica indisponível até o mapeamento ser corrigido.
settings-folder-role-refresh-error = Guardado, mas não foi possível atualizar a lista de pastas. Atualize as pastas antes de utilizar o novo mapeamento.
settings-folder-role-unsupported = Estas substituições exigem pastas de correio nativas. As contas JMAP externas mantêm as funções de pasta do respetivo servidor.
settings-folder-role-inbox = Caixa de entrada
settings-folder-role-drafts = Rascunhos
settings-folder-role-sent = Enviados
settings-folder-role-trash = Lixo
settings-folder-role-junk = Correio indesejado e spam
settings-folder-role-archive = Arquivo

settings-sessions-select = Selecionar
settings-sessions-select-device = Selecionar a sessão de { $device }
settings-sessions-select-visible = Selecionar todas as outras sessões visíveis
settings-sessions-current-protected = Esta sessão mantém-se iniciada e não pode ser selecionada aqui. Utilize Terminar sessão para a encerrar.
settings-sessions-selected-title = Terminar as sessões selecionadas ({ $count })
settings-sessions-selected-confirm = Encerrar apenas as outras sessões selecionadas. Esta sessão mantém-se iniciada. Os pedidos falhados ou ignorados serão comunicados, sem novas tentativas automáticas.
settings-sessions-selected-action = Terminar as sessões selecionadas
settings-sessions-cancel = Cancelar
settings-sessions-selected-result = Sessões terminadas: { $revoked }. Já encerradas: { $unchanged }. Sem confirmação: { $failed }. Não tentadas: { $skipped }.
settings-sessions-refresh-failed = Não foi possível atualizar a lista de sessões. Recarregue-a antes de selecionar mais sessões.
settings-sig-format = Formato da assinatura
settings-sig-format-plain = Texto simples
settings-sig-format-html = Texto formatado (HTML)
settings-sig-html-help = O HTML está limitado a 32 KiB e é limpo antes de ser guardado e enviado. As imagens remotas e o conteúdo ativo são removidos.
settings-sig-html-source = Editar código HTML
settings-sig-editor-loading = A carregar o editor de assinaturas…
settings-sig-rich-body = Assinatura formatada
settings-sig-plain-alternative = Alternativa em texto simples
settings-sig-plain-help = As mensagens em texto simples utilizam este texto. As edições formatadas atualizam-no até o personalizar; as edições do código HTML preservam a sua alternativa.
settings-sig-account-changed = A conta com sessão iniciada mudou. Volte a abrir esta assinatura na conta correta.
settings-sig-image-type = { $name } não é uma imagem PNG, JPEG, GIF ou WebP.
settings-sig-image-size = { $name } não cabe: o HTML de uma assinatura, incluindo imagens, está limitado a 32 KiB.
settings-sig-too-large = Esta assinatura tem mais de 32 KiB de HTML. Remova uma imagem ou alguma formatação.
settings-sig-images-help = As imagens ficam guardadas na assinatura e são enviadas como imagens incorporadas em cada mensagem.

settings-folder-icon-inbox = Caixa de entrada
settings-folder-icon-sent = Enviados
settings-folder-icon-edit = Editar
settings-folder-icon-delete = Eliminar
settings-folder-icon-spam = Spam
settings-folder-icon-search = Pesquisar
settings-folder-icon-pin = Pionés
settings-folder-icon-link = Ligação
settings-folder-icon-shield = Escudo
settings-folder-icon-unlock = Desbloquear
settings-folder-icon-fileAudio = Áudio
settings-folder-icon-fileVideo = Vídeo
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Código
settings-folder-icon-print = Imprimir
settings-folder-icon-download = Transferir
settings-folder-icon-preferences = Preferências
settings-folder-icon-dashboard = Painel
settings-folder-icon-analytics = Análises
settings-folder-icon-nextcloud = Ficheiros na nuvem
settings-folder-icon-domains = Domínios
settings-folder-icon-twofactor = Autenticação
settings-folder-icon-assistant = Assistente
settings-folder-icon-integrations = Integrações
settings-folder-icon-logs = Registos
settings-folder-icon-home = Casa
settings-folder-icon-briefcase = Pasta executiva
settings-folder-icon-heart = Coração
settings-folder-icon-shopping = Compras
settings-folder-icon-education = Educação
settings-folder-icon-travel = Viagens
settings-folder-icon-coffee = Café
settings-folder-icon-gift = Presente
settings-folder-icon-bank = Banco
settings-folder-icon-book = Livro

settings-nav-keyboard-display = Teclado e apresentação

settings-2fa-passkey-policy-disabled = O seu administrador desativou o início de sessão e o registo de chaves de acesso nesta conta. As credenciais existentes continuam listadas para que as possa gerir ou remover.

settings-mail-interactions = Seleção e movimentação de mensagens
settings-mail-modifier-selection = Selecionar mensagens com teclas modificadoras
settings-mail-modifier-selection-help = Ctrl ou Command seleciona ou desmarca mensagens individuais. Shift seleciona um intervalo e Shift com as teclas de seta expande-o. O botão de seleção visível continua disponível quando esta opção está desativada.
settings-mail-drag-folder = Arrastar mensagens para pastas
settings-mail-drag-folder-help = Arraste uma mensagem selecionada para mover a seleção, ou uma mensagem não selecionada para mover apenas essa mensagem. Só são aceites pastas desta conta. Quem utiliza ecrã tátil ou teclado pode utilizar a barra de ferramentas de seleção.
settings-search-sections = Pesquisar definições
settings-search-no-sections = Nenhuma secção de definições corresponde.
settings-time-local = Fuso horário local do dispositivo
settings-time-none = Sem fuso horário de comparação

settings-folder-counts = Contadores das pastas
settings-folder-counts-none = Ocultar contadores
settings-folder-counts-unread = Mensagens não lidas
settings-folder-counts-total = Total de mensagens
settings-folder-counts-both = Não lidas e total de mensagens
settings-folder-counts-unreadIfAny = Mensagens não lidas, apenas quando existirem
settings-folder-count-scope = Pastas com contagens
settings-folder-count-scope-all = Todas as pastas
settings-folder-count-scope-main = Apenas as pastas principais
settings-folder-count-scope-help = As pastas principais são a Caixa de entrada, Rascunhos, Enviados, Arquivo, Lixo eletrónico, Reciclagem, Caixa de saída e outras pastas de utilização especial. As pastas personalizadas e as respetivas subpastas não mostram contagem.
settings-account-unread-badge = Mostrar a contagem de não lidas junto ao nome da caixa de correio
settings-folder-counts-compact = Contadores mais pequenos fora da Caixa de entrada
settings-folder-counts-reset = Herdar contadores das pastas
settings-folder-counts-reset-size = Herdar tamanho do contador
settings-mail-resume-tabs = Retomar os meus separadores abertos
settings-mail-resume-tabs-help = Restaura as referências dos separadores da conta ao voltar a abrir o Mailwoman. O corpo das mensagens e os URLs dos anexos não são guardados no registo da área de trabalho.

settings-draft-save = Rascunhos da conta
settings-draft-save-help = Guarda na pasta Rascunhos da sua conta para acesso noutro dispositivo. O armazenamento é silencioso; só as falhas mostram uma notificação. As contas não suportadas e o conteúdo desencriptado protegido nunca são carregados sem aviso.
settings-draft-save-mode = Guardar automaticamente
settings-draft-realtime = Após uma pausa na escrita
settings-draft-periodic = A intervalos regulares
settings-draft-manual = Só quando eu escolher Guardar
settings-draft-debounce = Pausa na escrita (milissegundos)
settings-draft-interval = Intervalo de gravação (segundos)
settings-permissions-push-other-registration = Este navegador tem uma subscrição que não está registada nesta conta. Remova-a antes de voltar a registar.

settings-show-from = Mostrar De ao escrever
settings-show-from-help = Começar com o campo De visível. Também o pode mostrar para uma única mensagem a partir do friso. Só são aceites endereços de envio autorizados.
settings-sessions-table = Tabela de sessões ativas
settings-folders-table = Lista de pastas
settings-kbd-reference = Ver atribuições do conjunto
settings-kbd-reference-help = Apenas referência do conjunto. As atribuições personalizadas e a política do administrador podem alterar os atalhos ativos.
settings-sessions-client-details = Detalhes do cliente

# t39-e6: política de chaves de acesso do administrador
settings-2fa-passkey-rules = A sua organização só aceita chaves de acesso que:
settings-2fa-passkey-rule-platform = estejam guardadas neste dispositivo (Windows Hello, Touch ID, bloqueio de ecrã do Android)
settings-2fa-passkey-rule-cross-platform = estejam numa chave de segurança ou num telemóvel
settings-2fa-passkey-rule-user-verification = verifiquem o seu PIN, impressão digital ou rosto
settings-2fa-passkey-rule-resident-key = estejam guardadas no autenticador (detetáveis)
settings-2fa-passkey-gap-allowed = Adicionada antes da política atual e não a cumpre. Continua a funcionar; adicione uma nova chave de acesso e remova esta.
settings-2fa-passkey-gap-refused = Não cumpre a política atual e deixou de ser aceite no início de sessão. Adicione uma nova chave de acesso e remova esta.
settings-2fa-passkey-browser-refused-policy = O navegador não encontrou nenhum autenticador que cumpra a política de chaves de acesso da sua organização, ou o pedido foi cancelado.
settings-2fa-policy-user-verification = A sua organização exige uma chave de acesso que verifique o seu PIN, impressão digital ou rosto. Este autenticador não o verificou.
settings-2fa-policy-platform = A sua organização só aceita chaves de acesso guardadas neste dispositivo (como Windows Hello, Touch ID ou bloqueio de ecrã do Android). Chaves de segurança e telemóveis não são aceites.
settings-2fa-policy-cross-platform = A sua organização só aceita chaves de segurança ou chaves de acesso de telemóvel. Chaves de acesso guardadas neste dispositivo não são aceites.
settings-2fa-policy-attachment-unknown = O navegador não indicou que tipo de autenticador é este, por isso não é possível verificá-lo face à política de chaves de acesso da sua organização. Experimente outro navegador.
settings-2fa-policy-resident-key = A sua organização exige uma chave de acesso guardada no autenticador. Este autenticador não guardou nenhuma.
settings-2fa-policy-legacy = Esta chave de acesso foi adicionada antes da política atual da sua organização e deixou de a cumprir. Inicie sessão com outro fator ou um código de recuperação e depois adicione uma nova chave de acesso.
settings-2fa-policy-last-factor-recovery = Gere códigos de recuperação antes de remover o seu último segundo fator.
settings-2fa-policy-last-factor-blocked = A sua organização não permite remover o seu último segundo fator. Adicione outro primeiro.
settings-2fa-policy-last-factor-required = A sua organização exige autenticação de dois fatores. Adicione outro fator antes de remover este.
