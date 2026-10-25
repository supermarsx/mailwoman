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
