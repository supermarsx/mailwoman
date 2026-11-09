# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: settings. Source: ../en/settings.ftl

settings-title = Configurações
settings-appearance = Aparência
settings-close = Fechar configurações

settings-theme = Tema
settings-density = Densidade
settings-accent = Cor de destaque
settings-font = Fonte da interface
settings-layout = Layout

# Density options
settings-density-compact = Compacta
settings-density-cozy = Confortável
settings-density-relaxed = Espaçosa

# Interface-font options
settings-font-default = Padrão
settings-font-system = Sistema
settings-font-serif = Serifada
settings-font-mono = Monoespaçada

# Layout options
settings-layout-default = Padrão
settings-layout-ribbon = Faixa de opções

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Cancelar
settings-edit = Editar
settings-delete = Excluir
settings-save = Salvar
settings-saved = Salvo

# Two-factor authentication (S1)
settings-2fa-title = Autenticação de dois fatores
settings-2fa-intro = Adicione um segundo fator para exigir mais do que uma senha ao entrar.
settings-2fa-policy-required = Sua organização exige um segundo fator nesta conta.
settings-2fa-enabled = Ativada
settings-2fa-recovery-title = Códigos de recuperação
settings-2fa-recovery-once = Guarde-os agora. Eles são exibidos uma única vez e não podem ser recuperados depois. Cada código funciona uma vez.
settings-2fa-recovery-ack = Guardei estes códigos
settings-2fa-totp-title = App autenticador
settings-2fa-totp-enrol = Configurar um app autenticador
settings-2fa-totp-scan = Adicione este segredo ao seu app autenticador e depois digite o código que ele mostrar.
settings-2fa-totp-uri-link = Abrir em um app autenticador
settings-2fa-totp-code-label = Código do autenticador
settings-2fa-totp-confirm = Confirmar
settings-2fa-totp-disable = Remover autenticador
settings-2fa-passkey-title = Chaves de acesso
settings-2fa-passkey-remove = Remover
settings-2fa-passkey-unsupported = Este navegador não oferece suporte a chaves de acesso.
settings-2fa-passkey-label-placeholder = Nome da chave de acesso (opcional)
settings-2fa-passkey-add = Adicionar uma chave de acesso
settings-2fa-recovery-remaining = { $count ->
        [one] { $count } código de recuperação restante
       *[other] { $count } códigos de recuperação restantes
    }
settings-2fa-recovery-regenerate = Gerar novos códigos de recuperação
settings-2fa-recovery-code-label = Código de recuperação
settings-2fa-error-generic = Não funcionou. Tente novamente.
settings-2fa-error-code = Esse código não foi validado.
settings-2fa-error-passkey = Não foi possível registrar a chave de acesso.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Autenticação de dois fatores
settings-2fa-challenge-intro = Confirme seu segundo fator para concluir a entrada.
settings-2fa-challenge-failed = Não foi possível validar. Tente novamente.
settings-2fa-challenge-use-passkey = Usar uma chave de acesso
settings-2fa-challenge-method = Escolha um método
settings-2fa-challenge-totp-tab = Código do autenticador
settings-2fa-challenge-recovery-tab = Código de recuperação
settings-2fa-challenge-verify = Verificar

# Active sessions (S11)
settings-sessions-title = Sessões ativas
settings-sessions-intro = Sessões conectadas a esta conta no momento.
settings-sessions-last-seen = Última atividade { $when }
settings-sessions-revoke = Sair
settings-sessions-current = Esta sessão
settings-sessions-revoke-others = { $count ->
        [one] Sair de { $count } outra sessão
       *[other] Sair de { $count } outras sessões
    }
settings-sessions-error = Não foi possível alterar a sessão.

# Signatures (W12)
settings-sig-title = Assinaturas
settings-sig-intro = Texto adicionado ao final das mensagens que você envia. Uma assinatura pode ser a padrão.
settings-sig-default = Padrão
settings-sig-new = Nova assinatura
settings-sig-name-label = Nome
settings-sig-body-label = Assinatura
settings-sig-default-label = Usar como padrão
settings-sig-error-name = Dê um nome à assinatura.
settings-sig-error-generic = Não foi possível salvar a assinatura.

# Identities
settings-ident-title = Identidades
settings-ident-intro = O nome e o endereço com que você envia. Cada identidade pode usar sua própria assinatura.
settings-ident-new = Nova identidade
settings-ident-name-label = Nome de exibição
settings-ident-email-label = Endereço de e-mail
settings-ident-replyto-label = Endereço de resposta (opcional)
settings-ident-signature-label = Assinatura padrão
settings-ident-signature-none = Nenhuma
settings-ident-error-fields = Digite um nome e um endereço de e-mail válido.
settings-ident-error-generic = Não foi possível salvar a identidade.

# Notification rules + quiet hours (W15)
settings-notif-title = Notificações
settings-notif-intro = Escolha quais mensagens novas geram notificação e quando ficar em silêncio.
settings-notif-enabled-label = Mostrar notificações de novos e-mails
settings-notif-quiet-title = Horário de silêncio
settings-notif-quiet-enabled-label = Silenciar notificações durante um horário definido
settings-notif-quiet-start = De
settings-notif-quiet-end = Até
settings-notif-rules-title = Regras
settings-notif-rule-match-placeholder = Remetente, pasta ou assunto contém…
settings-notif-rule-action-label = Ação
settings-notif-rule-notify = Notificar
settings-notif-rule-mute = Silenciar
settings-notif-rule-add = Adicionar uma regra
settings-notif-error = Não foi possível salvar as configurações de notificação.

# Saved searches → search folders (W13)
settings-search-title = Pesquisas salvas
settings-search-intro = Mostre uma pesquisa salva como pasta na sua lista de pastas.
settings-search-empty = Você ainda não tem pesquisas salvas.
settings-search-as-folder = Mostrar { $name } como pasta
settings-search-as-folder-label = Mostrar como pasta
settings-search-error = Não foi possível alterar a pesquisa salva.

# Keyboard shortcuts (W14)
settings-kbd-title = Atalhos de teclado
settings-kbd-intro = Escolha um conjunto de atalhos. As alterações valem neste dispositivo.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Desativado (sem atalhos)
settings-kbd-action-compose = Escrever
settings-kbd-action-archive = Arquivar
settings-kbd-action-reply = Responder
settings-kbd-action-next = Próxima mensagem
settings-kbd-action-previous = Mensagem anterior
settings-kbd-action-search = Pesquisar

# Offline cache (W16)
settings-offline-title = Cache off-line
settings-offline-intro = Quanto e-mail manter neste dispositivo para uso off-line e como liberar espaço.
settings-offline-budget-label = Limite do cache (MB)
settings-offline-retention-label = Manter por (dias)
settings-offline-strategy-label = Quando o limite for atingido
settings-offline-lru = Remover os menos usados recentemente
settings-offline-oldest = Remover os mais antigos primeiro
settings-offline-manual = Somente quando eu limpar
settings-offline-purge = Limpar o cache off-line agora

# Interface direction (W20)
settings-dir-title = Direção da interface
settings-dir-intro = Seguir o idioma ou forçar uma direção. Da direita para a esquerda espelha o layout.
settings-dir-auto = Automática
settings-dir-ltr = Da esquerda para a direita
settings-dir-rtl = Da direita para a esquerda
settings-dir-preview-title = Pré-visualização
settings-dir-preview-body = Este texto é espelhado quando a direção é da direita para a esquerda.
settings-mail-features = Recursos de e-mail
settings-mail-feature-allowSend = Permitir o envio de mensagens
settings-mail-feature-allowSend-help = Quando desativado, Enviar e seu atalho ficam desabilitados. Os rascunhos ainda podem ser salvos.
settings-mail-feature-allowReply = Permitir responder e responder a todos
settings-mail-feature-allowReply-help = Quando desativado, as ações Responder e Responder a todos e seus atalhos não são oferecidas.
settings-mail-feature-allowForward = Permitir encaminhar
settings-mail-feature-allowForward-help = Quando desativado, as ações Encaminhar e seu atalho não são oferecidas.
settings-mail-feature-allowDownload = Permitir baixar mensagens e anexos
settings-mail-feature-allowDownload-help = Quando desativado, o download de mensagens e anexos não é oferecido.
settings-mail-feature-allowPrint = Permitir imprimir
settings-mail-feature-allowPrint-help = Quando desativado, a ação Imprimir não é oferecida.
settings-mail-feature-showSpam = Mostrar a pasta Spam
settings-mail-feature-showSpam-help = Quando desativado, a pasta Spam e as mensagens marcadas como spam ficam ocultas da lista de pastas e dos resultados de pesquisa. Nada é excluído.
settings-mail-feature-showAuthStatus = Mostrar os resultados de autenticação do remetente
settings-mail-feature-showAuthStatus-help = Mostra no leitor os resultados SPF, DKIM e DMARC de uma mensagem.
settings-mail-feature-closeOnDelete = Fechar uma mensagem ao excluí-la
settings-mail-feature-closeOnDelete-help = Fecha a mensagem, a guia ou a janela abertas quando essa mensagem é excluída ou movida para a Lixeira.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Caminho das configurações
settings-crumb-loading = Carregando nome
settings-crumb-new = Novo
settings-crumb-edit = Editar
settings-badge-color = Cor do selo de não lidas
settings-badge-color-accent = Igual ao destaque
settings-badge-color-accent-lighter = Destaque mais claro
settings-badge-color-accent-darker = Destaque mais escuro
settings-badge-color-option = Selos { $color }
settings-appearance-theme-card = Tema e alternância
settings-appearance-theme-card-help = Escolha um tema e se ele segue o sistema ou o horário do dia.
settings-appearance-colours-card = Cores
settings-appearance-colours-card-help = O destaque usado em ações e seleção, e a cor dos contadores de não lidas.
settings-appearance-text-card = Texto e layout
settings-appearance-text-card-help = Quão densa é a interface, qual fonte ela usa e como o e-mail é organizado.
settings-sig-create-variant = Criar variante
settings-sig-error-existing = Já existe uma assinatura com este nome. Escolha outro nome.
settings-sig-save-failed = Não foi possível salvar a assinatura. { $reason }
settings-ident-signature-new = Novas mensagens
settings-ident-signature-account = Usar a assinatura da conta
settings-ident-signature-replies = Respostas
settings-ident-signature-inherit = Usar a assinatura de novas mensagens
settings-ident-signature-unavailable = Assinatura indisponível: { $name }
settings-ident-signature-help = Crie variantes de assinatura em Assinaturas e atribua uma aqui. A mesma variante pode ser usada nos dois casos.
settings-ident-save-failed = Não foi possível salvar a identidade. { $reason }
settings-search-save-failed = Não foi possível salvar a pesquisa salva. { $reason }
settings-kbd-preset-title = Predefinição de atalhos
settings-editor-discard-title = Descartar alterações não salvas?
settings-editor-discard-detail = Suas alterações não foram salvas. Continue editando ou descarte este rascunho para sair do editor.
settings-editor-keep = Continuar editando
settings-editor-discard = Descartar alterações
settings-editor-missing = Este item não existe mais. Volte à lista e escolha outro item.
settings-editor-back = Voltar à lista
settings-sig-edit-title = Editar assinatura
settings-ident-edit-title = Editar identidade
settings-sections = Seções das configurações
settings-category-account = Conta
settings-category-mail = E-mail
settings-category-interface = Interface
settings-category-integrations = Integrações
settings-category-about = Sobre
settings-nav-preferences = Teclado e exibição
settings-nav-rules = Regras de e-mail
settings-nav-access = Acesso à API
settings-nav-metadata = Informações do servidor
settings-folders-title = Pastas e barra lateral
settings-folders-help = Fixe pastas acima das demais, escolha um ícone e arraste ou use os botões de mover para organizar as pastas dentro de cada grupo. Essas escolhas são sincronizadas com esta conta e não movem mensagens nem renomeiam pastas.
settings-folders-pin = Fixar { $name }
settings-folders-icon = Ícone de { $name }
settings-folders-automatic = Função automática da pasta
settings-folders-drag = Arraste { $name } para reordenar
settings-folders-up = Mover { $name } para cima
settings-folders-down = Mover { $name } para baixo
settings-folders-sync-idle = Entre para sincronizar as preferências de pastas.
settings-folders-sync-loading = Carregando as preferências da conta…
settings-folders-sync-saving = Salvando as preferências da conta…
settings-folders-sync-synced = Preferências sincronizadas com esta conta.
settings-folders-sync-error = Não foi possível sincronizar as preferências. Suas alterações permanecem nesta sessão e serão tentadas novamente quando a conexão voltar.
settings-folder-icon-folder = Pasta
settings-folder-icon-star = Estrela
settings-folder-icon-flag = Bandeira
settings-folder-icon-tag = Marcador
settings-folder-icon-clock = Relógio
settings-folder-icon-calendar = Calendário
settings-folder-icon-file = Documento
settings-folder-icon-image = Imagem
settings-folder-icon-lock = Cadeado
settings-folder-icon-users = Pessoas
settings-folder-icon-archive = Arquivo
settings-folder-icon-mail = Envelope
settings-folder-icon-attachment = Anexo
settings-folder-icon-check = Marca de verificação
settings-folder-icon-alert = Alerta
settings-search-new = Nova pasta de pesquisa
settings-search-editor = Pasta de pesquisa
settings-search-editor-help = As mensagens devem corresponder a todas as condições. Deixe uma condição vazia para ignorá-la. As pastas de pesquisa mostram as mensagens correspondentes sem movê-las nem copiá-las.
settings-search-name = Nome da pasta
settings-search-name-required = Informe um nome para esta pasta de pesquisa.
settings-search-invalid = Esta pesquisa salva tem um filtro inválido. Não é possível editá-la com segurança.
settings-search-saved = Pasta de pesquisa “{ $name }” salva.
settings-search-field-text = Pesquisar em todo o texto da mensagem
settings-search-field-from = Remetente contém
settings-search-field-to = Destinatário contém
settings-search-field-subject = Assunto contém
settings-search-field-body = Corpo da mensagem contém
settings-search-field-filename = Nome do arquivo anexo contém
settings-search-field-inMailbox = Limitar ao identificador da caixa de correio (opcional)
settings-search-field-after = Recebida depois de (data e hora UTC)
settings-search-field-before = Recebida antes de (data e hora UTC)
settings-search-attachment = Anexos
settings-search-any = Qualquer mensagem
settings-search-with-attachments = Com anexos
settings-search-without-attachments = Sem anexos
settings-search-unread = Somente mensagens não lidas
settings-search-flagged = Somente mensagens sinalizadas
settings-sessions-search = Pesquisar conta, dispositivo ou IP
settings-sessions-account = Conta
settings-sessions-device = Dispositivo e cliente
settings-sessions-location = IP e último país conhecido
settings-sessions-location-unavailable = País indisponível
settings-sessions-activity = Atividade
settings-sessions-actions = Ações
settings-sessions-revoke-device = Desconectar { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Redação e respostas
settings-quote-style = Estilo do cabeçalho de resposta e encaminhamento
settings-quote-classic = Resposta citada clássica
settings-quote-outlook = Cabeçalho no estilo Outlook
settings-quote-help = O estilo Outlook adiciona De, Enviada, Para, Cc e Assunto acima da mensagem original, em texto formatado e em texto simples. Os rascunhos existentes não são alterados.
settings-external-warning = Avisos de destinatários externos
settings-external-off = Desativado
settings-external-warn = Mostrar um aviso
settings-external-confirm = Exigir confirmação antes de enviar
settings-external-help = Compara Para, Cc e Cco com o domínio da identidade selecionada e com a lista de domínios confiáveis. Isso protege os envios feitos deste editor; não é uma regra de entrega do servidor.
settings-trusted-domains = Domínios de destinatários confiáveis
settings-trusted-domains-help = Informe até 100 domínios exatos, um por linha. Os subdomínios devem ser listados separadamente. O domínio da própria identidade de envio é sempre confiável. URLs, endereços de e-mail e curingas não são aceitos.
settings-trusted-domains-invalid = Informe até 100 nomes de domínio válidos, sem URLs, endereços de e-mail ou curingas.
settings-trusted-domains-save = Salvar domínios confiáveis
settings-shortcuts-enabled = Ativar os atalhos do aplicativo
settings-shortcuts-action = Ação
settings-shortcuts-binding = Atalho
settings-shortcuts-edit = Editar atalho
settings-shortcuts-edit-action = Editar o atalho de { $action }
settings-shortcuts-help = Use inherit para a predefinição selecionada, disabled para nenhum atalho ou uma combinação como Ctrl+Shift+K. Ctrl equivale a Cmd no macOS. Atalhos em conflito ficam inativos. Campos de digitação e caixas de diálogo mantêm as teclas habituais; combinações reservadas pelo navegador podem não estar disponíveis.
settings-shortcuts-invalid = Informe inherit, disabled ou uma tecla única válida, com os modificadores opcionais Ctrl, Alt e Shift.
settings-shortcuts-conflict = Duas ações usam o mesmo atalho. Altere um deles; atalhos em conflito não serão executados.
settings-shortcuts-discard = Descartar a alteração de atalho não salva?
settings-shortcuts-save = Salvar atalho
settings-shortcuts-back = Voltar aos atalhos
settings-shortcuts-off = A predefinição Desativado desativa todos os atalhos, inclusive os personalizados. Escolha outra predefinição em Preferências para usar estes atalhos.
settings-kbd-action-replyAll = Responder a todos
settings-kbd-action-forward = Encaminhar
settings-kbd-action-refresh = Atualizar mensagens
settings-kbd-action-settings = Abrir configurações
settings-kbd-action-trash = Mover para a lixeira
settings-kbd-action-markRead = Marcar como lida

# -- Machine translation (2026-11): added to match en ---------------------
settings-kbd-action-markUnread = Marcar como não lida
settings-kbd-action-spam = Marcar como spam
settings-kbd-action-zoomIn = Aumentar o zoom da mensagem
settings-kbd-action-zoomOut = Diminuir o zoom da mensagem
settings-kbd-action-zoomReset = Redefinir o zoom da mensagem
settings-kbd-action-renameFolder = Renomear pasta
settings-kbd-action-openInTab = Abrir a mensagem em sua própria aba
settings-notif-device-title = Permissão neste dispositivo
settings-notif-device-help = Suas preferências de notificação acompanham sua conta. A permissão do navegador pertence a este dispositivo; o Mailwoman só pede quando você escolhe permitir.
settings-notif-ask-permission = Oferecer permissão de notificação em novos dispositivos
settings-notif-ask-permission-help = Mostra uma solicitação de permissão quando este dispositivo ainda não permitiu nem bloqueou notificações. Ativado por padrão; esta preferência é salva imediatamente.
settings-notif-permission-default = A permissão do navegador ainda não foi solicitada.
settings-notif-permission-granted = As notificações do navegador estão permitidas neste dispositivo.
settings-notif-permission-denied = As notificações estão bloqueadas. Altere a permissão deste site nas configurações do navegador para permiti-las.
settings-notif-permission-unsupported = Este navegador não oferece suporte a notificações do sistema aqui.
settings-notif-permission-request = Permitir notificações neste dispositivo
settings-notif-permission-error = O navegador não conseguiu solicitar a permissão de notificação.
settings-notif-timezone = Fuso horário do horário de silêncio (IANA)
settings-notif-timezone-device = Usar o fuso horário deste dispositivo
settings-permissions-background-title = Notificações em segundo plano neste navegador
settings-permissions-background-help = Salve primeiro as preferências de notificação e permita as notificações do navegador. Registre este navegador para receber a atividade da conta quando o aplicativo estiver fechado.
settings-permissions-push-unsupported = O Web Push em segundo plano não está disponível neste navegador.
settings-permissions-push-registered = Este navegador tem uma assinatura push. Registre-a para vinculá-la à conta atual.
settings-permissions-push-unregistered = Este navegador não tem assinatura push.
settings-permissions-push-register = Registrar notificações em segundo plano
settings-permissions-push-remove = Remover assinatura do navegador
settings-permissions-push-policy = Seu administrador não permite notificações push em segundo plano.
settings-permissions-storage-title = Armazenamento do navegador
settings-permissions-storage-help = O armazenamento persistente protege os dados locais deste site contra remoção automática quando o navegador o concede. Ele é local ao dispositivo e não altera a retenção no servidor. Você pode limpar os dados do site nas configurações do navegador; alterações offline não enviadas podem ser perdidas.
settings-permissions-storage-unsupported = As solicitações de armazenamento persistente não estão disponíveis neste navegador.
settings-permissions-storage-persistent = O navegador concedeu armazenamento persistente a este site.
settings-permissions-storage-evictable = O navegador pode remover automaticamente os dados locais deste site.
settings-permissions-storage-usage = Armazenamento atual do site: { $mib } MiB.
settings-permissions-storage-request = Solicitar armazenamento persistente
settings-permissions-storage-policy = Seu administrador não permite solicitações de armazenamento persistente.
settings-permissions-error = Não foi possível concluir a ação do navegador. Verifique as configurações salvas da conta e as permissões do navegador para este site.
settings-notif-background-rules = As regras de conteúdo são avaliadas enquanto o Mailwoman está aberto. As notificações de sistema em segundo plano ficam suprimidas enquanto houver regras de conteúdo configuradas, pois um despertar opaco em segundo plano não contém detalhes da mensagem.
settings-time-title = Fusos horários
settings-time-help = Escolha como os horários das mensagens aparecem e, opcionalmente, um fuso de comparação para agendamentos. Os locais nunca são inferidos a partir de endereços de e-mail.
settings-time-displayZone = Fuso horário de exibição
settings-time-displayZone-help = Use local para este dispositivo ou um nome IANA, como Europe/Lisbon.
settings-time-comparisonZone = Fuso horário de comparação
settings-time-comparisonZone-help = Informe um nome IANA, como America/New_York, ou deixe vazio para ocultar as comparações.
settings-time-helpers = Mostrar auxílios de fuso horário
settings-time-save = Salvar fuso horário
settings-time-invalid = Informe um fuso horário IANA válido. Use local apenas para o fuso de exibição ou deixe o fuso de comparação vazio.
settings-time-discard = Descartar as alterações de fuso horário não salvas?
settings-folders-name = Pasta
settings-folders-actions = Ações
settings-folders-edit = Editar a aparência de { $name }
settings-folders-icon-help = Escolha um entre cinquenta ícones temáticos ou mantenha a seleção automática com base na finalidade da pasta. Isso acompanha sua conta e não renomeia a caixa de correio.
settings-folders-save = Salvar aparência da pasta
settings-folders-back = Voltar às pastas
settings-folders-discard = Descartar as alterações de aparência da pasta não salvas?
settings-folder-labels = Marcadores da pasta
settings-folder-labels-help = Organize esta pasta com até 32 marcadores da conta. Isso não marca nem move as mensagens dentro dela. Gerencie nomes e cores dos marcadores em Marcadores.
settings-folders-leave-pending = Estas alterações ainda estão na fila de sincronização. Sair deste editor?
settings-folders-retry = Tentar salvar novamente
settings-folder-role-title = Pastas especiais
settings-folder-role-help = Escolha a pasta real usada para cada função de e-mail. Automático usa a descoberta do servidor. Usar o padrão do administrador remove a sua substituição. Destinos em conflito ou ausentes desativam essa função em vez de escolher outra pasta.
settings-folder-role-auto = Descoberta automática
settings-folder-role-inherit = Usar o padrão do administrador
settings-folder-role-path = Caminho configurado: { $path }
settings-folder-role-missing = A pasta configurada não está disponível
settings-folder-role-conflict = Este destino está ausente ou entra em conflito com outra função. Essa função fica indisponível até que o mapeamento seja corrigido.
settings-folder-role-refresh-error = Salvo, mas não foi possível atualizar a lista de pastas. Atualize as pastas antes de usar o novo mapeamento.
settings-folder-role-unsupported = Estas substituições exigem pastas de e-mail nativas. As contas JMAP externas mantêm as funções de pasta do próprio servidor.
settings-folder-role-inbox = Caixa de entrada
settings-folder-role-drafts = Rascunhos
settings-folder-role-sent = E-mails enviados
settings-folder-role-trash = Lixeira
settings-folder-role-junk = Lixo eletrônico e spam
settings-folder-role-archive = Arquivo
settings-sessions-select = Selecionar
settings-sessions-select-device = Selecionar a sessão de { $device }
settings-sessions-select-visible = Selecionar todas as outras sessões visíveis
settings-sessions-current-protected = Esta sessão permanece conectada e não pode ser selecionada aqui. Use Sair para encerrá-la.
settings-sessions-selected-title = Desconectar as sessões selecionadas ({ $count })
settings-sessions-selected-confirm = Encerra apenas as outras sessões selecionadas. Esta sessão permanece conectada. As solicitações com falha ou ignoradas serão informadas, sem novas tentativas automáticas.
settings-sessions-selected-action = Desconectar as sessões selecionadas
settings-sessions-cancel = Cancelar
settings-sessions-selected-result = Desconectadas: { $revoked }. Já encerradas: { $unchanged }. Não foi possível confirmar: { $failed }. Não tentadas: { $skipped }.
settings-sessions-refresh-failed = Não foi possível atualizar a lista de sessões. Recarregue-a antes de selecionar mais sessões.
settings-sig-format = Formato da assinatura
settings-sig-format-plain = Texto simples
settings-sig-format-html = Texto formatado (HTML)
settings-sig-html-help = O HTML é limitado a 2 MiB e é higienizado antes de salvar e enviar. Imagens remotas e conteúdo ativo são removidos.
settings-sig-html-source = Editar o código-fonte HTML
settings-sig-editor-loading = Carregando o editor de assinatura…
settings-sig-rich-body = Assinatura formatada
settings-sig-plain-alternative = Alternativa em texto simples
settings-sig-plain-help = As mensagens em texto simples usam este texto. As edições no texto formatado o atualizam até que você o personalize; as edições no código-fonte HTML preservam a sua alternativa.
settings-sig-account-changed = A conta conectada foi alterada. Reabra esta assinatura na conta correta.
settings-sig-image-type = { $name } não é uma imagem PNG, JPEG, GIF ou WebP.
settings-sig-image-size = { $name } não cabe: o HTML de uma assinatura, incluindo as imagens, é limitado a 2 MiB.
settings-sig-too-large = Esta assinatura tem mais de 2 MiB de HTML. Remova uma imagem ou parte da formatação.
settings-sig-images-help = As imagens ficam armazenadas na assinatura e são enviadas como imagens embutidas em cada mensagem.
settings-folder-icon-inbox = Caixa de entrada
settings-folder-icon-sent = Enviados
settings-folder-icon-edit = Editar
settings-folder-icon-delete = Excluir
settings-folder-icon-spam = Spam
settings-folder-icon-search = Pesquisar
settings-folder-icon-pin = Alfinete
settings-folder-icon-link = Link
settings-folder-icon-shield = Escudo
settings-folder-icon-unlock = Cadeado aberto
settings-folder-icon-fileAudio = Áudio
settings-folder-icon-fileVideo = Vídeo
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Código
settings-folder-icon-print = Imprimir
settings-folder-icon-download = Download
settings-folder-icon-preferences = Preferências
settings-folder-icon-dashboard = Painel
settings-folder-icon-analytics = Análises
settings-folder-icon-nextcloud = Arquivos na nuvem
settings-folder-icon-domains = Domínios
settings-folder-icon-twofactor = Autenticação
settings-folder-icon-assistant = Assistente
settings-folder-icon-integrations = Integrações
settings-folder-icon-logs = Logs
settings-folder-icon-home = Início
settings-folder-icon-briefcase = Maleta
settings-folder-icon-heart = Coração
settings-folder-icon-shopping = Compras
settings-folder-icon-education = Educação
settings-folder-icon-travel = Viagens
settings-folder-icon-coffee = Café
settings-folder-icon-gift = Presente
settings-folder-icon-bank = Banco
settings-folder-icon-book = Livro
settings-nav-keyboard-display = Teclado e exibição
settings-2fa-passkey-policy-disabled = Seu administrador desativou o login e o cadastro com passkey para esta conta. As credenciais existentes continuam listadas para que você possa gerenciá-las ou removê-las.
settings-mail-interactions = Seleção e movimentação de mensagens
settings-mail-modifier-selection = Selecionar mensagens com teclas modificadoras
settings-mail-modifier-selection-help = Ctrl ou Command alterna mensagens separadas. Shift seleciona um intervalo, e Shift com as setas o amplia. O botão de seleção visível continua disponível quando esta opção está desativada.
settings-mail-drag-folder = Arrastar mensagens para pastas
settings-mail-drag-folder-help = Arraste uma mensagem selecionada para mover a seleção, ou uma mensagem não selecionada para mover apenas ela. Só são aceitas pastas desta conta. Quem usa toque ou teclado pode usar a barra de ferramentas de seleção.
settings-search-sections = Pesquisar nas configurações
settings-search-no-sections = Nenhuma seção de configurações correspondente.
settings-time-local = Fuso horário local do dispositivo
settings-time-none = Sem fuso horário de comparação
settings-folder-counts = Contagens das pastas
settings-folder-counts-none = Ocultar contagens
settings-folder-counts-unread = Mensagens não lidas
settings-folder-counts-total = Total de mensagens
settings-folder-counts-both = Mensagens não lidas e total
settings-folder-counts-unreadIfAny = Mensagens não lidas, somente quando houver
settings-folder-count-scope = Pastas com contagem
settings-folder-count-scope-all = Todas as pastas
settings-folder-count-scope-main = Somente as pastas principais
settings-folder-count-scope-help = As pastas principais são Caixa de entrada, Rascunhos, Enviados, Arquivo, Lixo eletrônico, Lixeira, Caixa de saída e outras pastas de uso especial. As pastas personalizadas e suas subpastas não mostram contagem.
settings-account-unread-badge = Mostrar a contagem de não lidas ao lado do nome da caixa de correio
settings-folder-counts-compact = Selos menores fora da Caixa de entrada
settings-focused-inbox = Caixa de entrada com foco
settings-focused-inbox-help = Divide a Caixa de entrada nas abas Foco e Outras acima da lista. As mensagens de newsletters e remetentes automáticos vão para Outras. As demais pastas e os resultados de pesquisa mostram todas as mensagens.
settings-focused-senders-title = Gerenciar remetentes de Foco
settings-focused-senders-help = Remetentes que você escolheu mover sempre para Foco ou Outras. Remova um para que a classificação automática volte a decidir.
settings-focused-senders-empty = Nenhum remetente ainda. Use Sempre mover para Foco ou Sempre mover para Outras em uma mensagem.

# -- Machine translation (2026-11): added to match en ---------------------
settings-focused-senders-to-focused = Sempre em Foco
settings-focused-senders-to-other = Sempre em Outras
settings-focused-senders-remove = Remover
settings-focused-senders-remove-label = Remover { $sender }
settings-focused-senders-remove-failed = Não foi possível remover o remetente. Tente novamente.
settings-folder-counts-reset = Herdar as contagens das pastas
settings-folder-counts-reset-size = Herdar o tamanho do selo
settings-mail-resume-tabs = Retomar minhas abas abertas
settings-mail-resume-tabs-help = Restaura as referências de abas da conta ao reabrir o Mailwoman. Os corpos das mensagens e as URLs dos anexos não são salvos no registro do espaço de trabalho.
settings-row-actions-mode = Ações rápidas da lista de mensagens
settings-row-actions-mode-open = Sempre abertas
settings-row-actions-mode-hover = Mostrar ao passar o mouse
settings-row-actions-mode-click = Mostrar ao clicar
settings-row-actions-mode-off = Desativado
settings-row-actions-mode-help = Como as ações rápidas de cada linha de mensagem aparecem. Ao passar o mouse, mostra três controles e um botão Mais; ao clicar, mostra um botão que os abre; em telas de toque, passar o mouse funciona como clicar. Desativado as remove das linhas (o menu de contexto e a barra de ferramentas continuam com todas as ações).
settings-mail-feature-showGoToTop = Mostrar um botão Ir para o topo na lista de mensagens
settings-mail-feature-showGoToTop-help = Adiciona um botão que rola a lista de mensagens de volta à primeira mensagem. Desativado por padrão.
settings-mail-feature-showCreateFolderButton = Mostrar um botão Criar pasta ao lado de Escrever
settings-mail-feature-showCreateFolderButton-help = Adiciona na barra lateral um botão só com ícone que cria uma pasta. Desativado por padrão.
settings-draft-save = Rascunhos da conta
settings-draft-save-help = Salva na pasta Rascunhos da sua conta para acessar em outro dispositivo. O salvamento é silencioso; apenas falhas mostram uma notificação. Contas sem suporte e conteúdo descriptografado protegido nunca são enviados em silêncio.
settings-draft-save-mode = Salvar automaticamente
settings-draft-realtime = Após uma pausa na digitação
settings-draft-periodic = Em intervalos regulares
settings-draft-manual = Somente quando eu escolher Salvar
settings-draft-debounce = Pausa na digitação (milissegundos)
settings-draft-interval = Intervalo de salvamento (segundos)
settings-permissions-push-other-registration = Este navegador tem uma assinatura que não está registrada nesta conta. Remova-a antes de registrar novamente.
settings-show-from = Mostrar De ao escrever
settings-show-from-help = Começar com o campo De visível. Você também pode mostrá-lo para uma mensagem pela faixa de opções. Somente endereços de envio autorizados são aceitos.
settings-sessions-table = Tabela de sessões ativas
settings-folders-table = Lista de pastas
settings-kbd-reference = Ver atalhos da predefinição
settings-kbd-reference-help = Apenas referência da predefinição. Atalhos personalizados e a política do administrador podem alterar os atalhos ativos.
settings-sessions-client-details = Detalhes do cliente
settings-2fa-passkey-rules = Sua organização aceita apenas passkeys que:
settings-2fa-passkey-rule-platform = são armazenadas neste dispositivo (Windows Hello, Touch ID, bloqueio de tela do Android)
settings-2fa-passkey-rule-cross-platform = estão em uma chave de segurança ou em um celular
settings-2fa-passkey-rule-user-verification = verificam seu PIN, impressão digital ou rosto
settings-2fa-passkey-rule-resident-key = são armazenadas no autenticador (detectáveis)
settings-2fa-passkey-gap-allowed = Adicionada antes da política atual e não a atende. Ainda funciona; adicione uma nova passkey e remova esta.
settings-2fa-passkey-gap-refused = Não atende à política atual e não é mais aceita para login. Adicione uma nova passkey e remova esta.
settings-2fa-passkey-browser-refused-policy = Seu navegador não encontrou nenhum autenticador que atenda à política de passkeys da sua organização, ou a solicitação foi cancelada.
settings-2fa-policy-user-verification = Sua organização exige uma passkey que verifique seu PIN, impressão digital ou rosto. Este autenticador não verificou você.
settings-2fa-policy-platform = Sua organização aceita apenas passkeys armazenadas neste dispositivo (como Windows Hello, Touch ID ou bloqueio de tela do Android). Chaves de segurança e celulares não são aceitos.
settings-2fa-policy-cross-platform = Sua organização aceita apenas chaves de segurança ou passkeys de celular. Passkeys armazenadas neste dispositivo não são aceitas.
settings-2fa-policy-attachment-unknown = Seu navegador não informou que tipo de autenticador é este, por isso ele não pode ser verificado em relação à política de passkeys da sua organização. Tente outro navegador.
settings-2fa-policy-resident-key = Sua organização exige uma passkey armazenada no autenticador. Este autenticador não armazenou uma.
settings-2fa-policy-legacy = Esta passkey foi adicionada antes da política atual de passkeys da sua organização e não a atende mais. Entre com outro fator ou com um código de recuperação e depois adicione uma nova passkey.
settings-2fa-policy-last-factor-blocked = Sua organização não permite remover o seu último segundo fator. Adicione antes outro método de login.
settings-2fa-policy-last-factor-required = Sua organização exige autenticação em dois fatores. Adicione outro método de login antes de remover este.
settings-2fa-last-factor-hint = Adicione antes outro método de login.
settings-2fa-last-factor-tooltip = Adicione antes outro método de login. Sua organização exige pelo menos uma passkey ou um aplicativo autenticador na sua conta.
settings-2fa-challenge-passkey-not-used = A passkey não foi usada: a solicitação foi cancelada ou expirou, ou este autenticador não consegue fazer o que sua organização exige (como verificar PIN, impressão digital ou rosto). Tente novamente ou use outro método.
settings-folders-hidden-help = As pastas ocultas e suas subpastas ficam fora da barra lateral. Elas não são excluídas nem têm o acesso restrito. Mostre uma subpasta aqui para mostrar também as pastas pai ocultas. A Caixa de entrada sempre permanece disponível.
settings-folders-collapse-user = Recolher as pastas do usuário na barra lateral
settings-folders-show = Mostrar { $name } na barra lateral
settings-folders-hide = Ocultar { $name } da barra lateral
settings-folders-show-short = Mostrar
settings-folders-hide-short = Ocultar
settings-folders-search = Pesquisar pastas
settings-folders-select = Selecionar pasta: { $name }
settings-folders-selected = Pastas selecionadas: { $count }
settings-folders-select-visible = Selecionar visíveis
settings-folders-select-all = Selecionar todas as pastas
settings-folders-clear = Limpar seleção
settings-folders-pin-selected = Fixar selecionadas
settings-folders-unpin-selected = Desafixar selecionadas
settings-folders-show-selected = Mostrar selecionadas
settings-folders-hide-selected = Ocultar selecionadas
settings-folders-visible = Visíveis
settings-folders-hidden = Ocultas
settings-folders-pinned = Fixadas
settings-folders-location = Local
settings-folders-counts = { $unread } não lidas / { $total } no total
settings-folders-more = Mais ações para { $name }
settings-folders-empty = Nenhuma pasta corresponde à pesquisa.
settings-folders-root = Nível superior
settings-folders-order-help = Arraste as pastas para reordená-las na barra lateral. As pastas fixadas permanecem juntas.
cache-title = Cache de e-mail e pré-carregamento
cache-help = Mantém pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o limpa. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
cache-enabled = Ativar o cache da sessão
cache-mailboxes = Armazenar em cache as listas de mensagens das pastas
cache-messages = Armazenar em cache as mensagens abertas
cache-freshSeconds = Intervalo de atualidade (segundos)
cache-maxAgeSeconds = Idade máxima do cache (segundos)
cache-maxEntries = Máximo de entradas em cache
cache-maxBytes = Limite de memória (bytes)
cache-prefetchEnabled = Ativar o pré-carregamento especulativo
cache-hoverEnabled = Pré-carregar ao apontar para uma pasta
cache-focusEnabled = Pré-carregar quando uma pasta recebe o foco do teclado
cache-dwellMs = Atraso do ponteiro e do foco (milissegundos)
cache-idleEnabled = Pré-carregar pastas em ociosidade
cache-idleDelayMs = Atraso de ociosidade (milissegundos)
cache-maxConcurrency = Máximo de pré-carregamentos simultâneos
cache-maxQueued = Máximo de pré-carregamentos na fila
cache-preloadMessages = Permitir corpos de mensagens especulativos
cache-excludedFolders = Pastas excluídas do pré-carregamento
cache-exclusions-help = Uma regra por linha: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive, ou id: seguido do ID exato de uma pasta.
cache-refreshOnOpen = Atualizar o conteúdo em cache ao abrir
cache-clear = Limpar o e-mail em cache agora
settings-compose-window = Abrir novas mensagens em
settings-compose-window-tab = Uma aba do espaço de trabalho
settings-compose-window-new = Uma nova janela
settings-compose-window-help = Se o navegador bloquear uma nova janela, o editor abre em uma aba do espaço de trabalho.
settings-compose-read-receipt = Sempre solicitar confirmação de leitura
settings-compose-delivery-receipt = Sempre solicitar confirmação de entrega
settings-compose-reply-copy = Salvar uma cópia das respostas na pasta original
settings-compose-receipts-help = As confirmações são solicitações. O destinatário ou o servidor de e-mail dele pode recusá-las ou ignorá-las.
settings-compose-forward = Formato padrão de encaminhamento
settings-compose-forward-inline = Texto da mensagem no corpo
settings-compose-forward-attachment = Mensagem original como anexo .eml
settings-compose-saving = Salvando a preferência de redação…
settings-compose-saved = Preferência de redação salva
settings-ident-organization = Organização
settings-ident-auto-bcc = Cco automático
settings-ident-auto-bcc-help = Adiciona até 20 endereços em Cco ao enviar a partir desta identidade. Separe os endereços com vírgulas.
settings-ident-pgp-key = Chave OpenPGP
settings-ident-pgp-none = Usar a chave padrão
settings-ident-pgp-unavailable = A chave salva não está disponível — escolha outra chave
settings-ident-pgp-help = Escolha uma das suas chaves de assinatura OpenPGP existentes. O material da chave privada permanece no seu cofre de chaves.
settings-ident-error-optional = Verifique os endereços de Responder para e de Cco automático, a organização e a chave selecionada.
settings-compose-open-tracking = Registrar aberturas de mensagens por padrão
settings-compose-open-tracking-help = Adiciona uma imagem de rastreamento às mensagens formatadas elegíveis. As aberturas dependem de o destinatário carregar imagens e não provam que a mensagem foi lida. Seu administrador pode desativar o rastreamento ou bloquear este padrão.
settings-compose-open-tracking-inherit = Usar o padrão de rastreamento herdado
settings-reply-images = Imagens em respostas e encaminhamentos
settings-reply-images-ask = Perguntar a cada vez
settings-reply-images-allow = Sempre incluir
settings-reply-images-trusted = Incluir para domínios de remetentes confiáveis, perguntar nos demais
settings-reply-images-deny = Nunca incluir
settings-reply-images-help = Escolha se uma resposta ou um encaminhamento leva as imagens da mensagem original. As imagens incluídas são copiadas para o rascunho como anexos.
settings-reply-image-domains = Domínios de remetentes de imagens confiáveis
settings-reply-image-domains-help = Informe até 200 domínios de remetentes exatos, um por linha. Usado quando a opção acima é domínios confiáveis.
settings-reply-image-domains-save = Salvar domínios de imagens confiáveis
settings-ident-group-basics = Básico
settings-ident-group-replies = Tratamento de respostas
settings-ident-group-signature = Assinatura
settings-ident-group-sending = Opções de envio
settings-ident-name-required = Informe um nome de exibição.
settings-ident-email-invalid = Informe um endereço de e-mail válido, como nome@exemplo.org.
settings-ident-email-duplicate = Você já tem uma identidade com este endereço.
settings-ident-replyto-invalid = Informe um endereço de e-mail válido ou deixe vazio.
settings-ident-replyto-help = As respostas às mensagens enviadas a partir desta identidade vão para este endereço, em vez do endereço do remetente.
settings-ident-bcc-invalid = Verifique a lista de Cco: use endereços válidos separados por vírgulas, no máximo 20.
settings-ident-delete-title = Excluir a identidade { $email }?
settings-ident-delete-body = Você não poderá mais enviar a partir desta identidade. Rascunhos e mensagens enviadas não são afetados.
settings-ident-empty = Você ainda não tem identidades. Adicione uma para escolher nome, endereço e assinatura ao escrever.
settings-sig-delete-title = Excluir a assinatura { $name }?
settings-sig-delete-body = As identidades que a usam passam a usar a assinatura da conta. As mensagens já enviadas não são afetadas.
settings-sig-deleted = Assinatura excluída.
settings-reply-from = Responder de
settings-reply-from-match = O endereço para o qual a mensagem foi enviada
settings-reply-from-default = Sempre minha identidade padrão
settings-reply-from-ask = Perguntar quando várias identidades corresponderem
settings-reply-from-keep = Manter a identidade já selecionada
settings-reply-from-help = Ao responder ou encaminhar, escolha qual identidade é usada. Se nenhuma identidade corresponder ao endereço para o qual a mensagem foi enviada, sua identidade padrão é usada. Endereços com sinal de mais, como nome+tag@exemplo.org, correspondem a nome@exemplo.org.
settings-ident-default-badge = Padrão
settings-ident-make-default = Tornar padrão
settings-ident-default-set = { $email } agora é sua identidade padrão.
settings-ident-create-variant = Criar variante

# -- Machine translation (2026-11): added to match en ---------------------
settings-ident-variant-count = { $count } { $count ->
        [one] variante
       *[other] variantes
}
settings-ident-inherits = Usa { $base } para: { $fields }
settings-ident-field-replyTo = responder para
settings-ident-field-autoBcc = Cco automático
settings-ident-field-organization = organização
settings-ident-field-pgpKeyId = chave OpenPGP
settings-ident-field-signatureName = assinatura
settings-ident-delete-default-body = Esta é a sua identidade padrão. Sua identidade restante mais antiga passa a ser a padrão. Você não poderá mais enviar a partir desta identidade. Rascunhos e mensagens enviadas não são afetados.
settings-ident-variants-choice-title = Excluir { $email }?
settings-ident-variants-choice-body = Esta identidade tem { $count } { $count ->
        [one] variante
       *[other] variantes
}. Exclua-{ $count ->
        [one] a
       *[other] as
} também ou mantenha-{ $count ->
        [one] a
       *[other] as
} como identidades separadas. Uma variante mantida copia as configurações que herdou desta identidade.
settings-ident-delete-with-variants = Excluir a identidade e as variantes
settings-ident-delete-keep-variants = Excluir a identidade e manter as variantes
settings-ident-new-variant = Nova variante
settings-ident-edit-variant-title = Editar variante
settings-ident-variant-intro = Uma variante de { $base }. Deixe um campo vazio para usar o valor dessa identidade.
settings-ident-label-label = Rótulo da variante
settings-ident-label-required = Informe um rótulo para esta variante, como Newsletters.
settings-ident-inherit-hint = Vazio: usa { $value } da identidade base.
settings-ident-inherit-none = nada
settings-check-policy-allow = Enviar sem perguntar
settings-check-policy-warn = Perguntar a cada vez
settings-check-policy-warn-remember = Perguntar e permitir que eu pare de perguntar
settings-check-policy-deny = Bloquear o envio
settings-check-invalid = Endereços de destinatário inválidos
settings-check-attachment = Anexo mencionado, mas ausente
settings-check-subject = Assunto vazio
settings-check-help = O que fazer quando a verificação encontra um problema. Perguntar sempre oferece Enviar mesmo assim; somente Bloquear o envio impede o envio de uma mensagem.
settings-external-warn-remember = Perguntar e permitir que eu pare de perguntar
settings-external-deny = Bloquear o envio
settings-auto-chip = Concluir um destinatário quando eu digitar espaço, vírgula ou ponto e vírgula
settings-auto-chip-help = Um endereço válido se torna um destinatário concluído e o separador é adicionado para você.
settings-recipient-separator = Separador de destinatários
settings-recipient-separator-comma = Vírgula
settings-recipient-separator-semicolon = Ponto e vírgula
settings-header-collapse = Cabeçalho da mensagem (Para, Cc, Assunto)
settings-header-collapse-expanded = Sempre mostrar
settings-header-collapse-collapsed = Recolher em uma linha de resumo
settings-header-collapse-auto = Recolher enquanto escrevo o corpo
settings-header-collapse-help = Recolhido, o cabeçalho mostra uma linha como Para: Ana +2 e se abre quando você clica nele ou quando um campo precisa de atenção.
settings-retention-title = Retenção
settings-folder-icon-camera = Câmera
settings-folder-icon-music = Música
settings-folder-icon-film = Filme
settings-folder-icon-lightbulb = Ideia
settings-folder-icon-rocket = Foguete
settings-folder-icon-trophy = Troféu
settings-folder-icon-leaf = Folha
settings-folder-icon-globe = Globo
settings-folder-icon-cloud = Nuvem
settings-folder-icon-bookmark = Favorito
settings-folder-icon-wallet = Carteira
settings-folder-icon-medical = Médico
settings-folder-icon-bell = Sino
settings-folder-icon-key = Chave
settings-folder-icon-receipt = Recibo
settings-folder-icon-cart = Carrinho de compras
settings-folder-icon-package = Pacote
settings-folder-icon-wrench = Ferramentas
settings-folder-icon-chart = Gráfico
settings-folder-icon-team = Equipe
settings-folder-icon-store = Loja
settings-folder-icon-office = Escritório

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-laptop = Notebook
settings-folder-icon-crown = Coroa
settings-folder-icon-family = Família
settings-folder-icon-group-general = Geral
settings-folder-icon-group-mail = E-mail
settings-folder-icon-group-work = Trabalho
settings-folder-icon-group-finance = Finanças e compras
settings-folder-icon-group-travel = Viagens e casa
settings-folder-icon-group-personal = Pessoal e diversão
settings-folder-icon-group-tech = Tecnologia e ferramentas
settings-folder-icon-search-label = Pesquisar ícones
settings-folder-icon-no-match = Nenhum ícone corresponde

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Autenticação (DMARC)

settings-badge-auth-help = O resultado DMARC; ele abre todos os detalhes de segurança.

settings-badge-density = Tamanho do selo

settings-badge-density-comfortable = Confortável

settings-badge-density-compact = Compacto

settings-badge-density-help = Altura e espaçamento compactos ou confortáveis para cada crachá.

settings-badge-folder = Pasta

settings-badge-folder-help = Onde a mensagem é armazenada. Somente quando notável mostra isso para Lixo e Lixo.

settings-badge-labels = Etiquetas de crachá

settings-badge-labels-help = Ícone com texto, ou apenas o ícone (o texto completo fica na dica e para leitores de tela).

settings-badge-labels-icon = Somente ícone

settings-badge-labels-text = Ícone e texto

settings-badge-list = Lista de discussão

settings-badge-list-help = A lista de discussão de onde veio a mensagem; ele abre os detalhes e ações da assinatura. Somente quando notável mostra quando o e-mail continua chegando após você cancelar a inscrição.

settings-badge-mode-always = Sempre

settings-badge-mode-never = Nunca

settings-badge-mode-notable = Somente quando notável

settings-badge-sanitized = Higienizado HTML

settings-badge-sanitized-help = O que o desinfetante HTML fez com a mensagem. Somente quando notável mostra quando algo foi bloqueado.

settings-badge-spam = Pontuação de spam

settings-badge-spam-help = A pontuação que o servidor receptor escreveu nos cabeçalhos das mensagens.

settings-folders-hide-empty = Ocultar pastas vazias na barra lateral

settings-folders-hide-empty-help = As pastas reaparecem quando contêm mensagens. A caixa de entrada, as pastas de pesquisa, as pastas com contagens desconhecidas e os pais das pastas visíveis permanecem exibidos. Substitua pastas individuais em Propriedades, Exibição; cada pasta permanece nesta lista.

settings-folders-list-card = Lista e ordem de pastas

settings-header-auto-collapse = Dobre o cabeçalho depois de escrever (segundos)

settings-header-auto-collapse-help = Quando o corpo estiver em foco por tantos segundos e o cabeçalho tiver um destinatário e um assunto, ele será dobrado na linha de resumo. 0 desativa isso.

settings-message-badges = Emblemas de mensagens

settings-message-badges-help = Escolha quais emblemas aparecem no cabeçalho de uma mensagem aberta. As marcas de prioridade são definidas em Pastas e barra lateral. Você também pode clicar com o botão direito em um emblema para ocultá-lo.

settings-reply-from-headers = Cabeçalhos que escolhem a identidade

settings-reply-from-headers-help = Um nome de cabeçalho por linha, tentado nesta ordem: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc. Um cabeçalho deixado de fora não é lido.

settings-reply-from-headers-invalid = Use apenas estes nomes, um por linha: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc.

settings-reply-from-headers-save = Salvar ordem de cabeçalho

settings-sessions-linked-created = Link criado

settings-sessions-linked-details = Detalhes do link

settings-sessions-linked-kind = Tipo de link

settings-sessions-linked-kind-linked = Caixa de correio vinculada

settings-sessions-linked-last-used = Usado pela última vez

settings-sessions-linked-line = Acesso à caixa de correio vinculada · de { $origin }

settings-sessions-linked-line-hidden = Acesso à caixa de correio vinculada

settings-sessions-linked-origin = Vem de

settings-sessions-linked-removed = Desconhecido (conta removida)

settings-sessions-linked-revoke = Revogar acesso ao link

settings-sessions-linked-revoke-label = Revogar acesso ao link para { $mailbox }

settings-sessions-linked-target = Caixa de correio aberta

settings-sessions-origin-full = Endereço completo

settings-sessions-origin-help = Controla como esta tabela nomeia a conta de onde vem uma sessão de caixa de correio vinculada. Só você vê isso, em suas próprias sessões.

settings-sessions-origin-hidden = Oculto

settings-sessions-origin-masked-local-part = Endereço com a primeira parte mascarada

settings-sessions-origin-setting = Mostrar de onde vem o acesso à caixa de correio vinculada

settings-shortcuts-search = Atalhos de pesquisa
