# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Chaves de API
apikeys-heading = Chaves de API
apikeys-intro = Crie chaves com escopo para scripts e integrações. Cada chave é armazenada como hash, mostrada uma única vez e pode ser revogada individualmente. Conceda o menor escopo que funcione.

apikeys-label = Rótulo
apikeys-label-aria = Rótulo da chave
apikeys-label-placeholder = ex.: script de backup

apikeys-create = Criar chave
apikeys-saved = Já salvei

apikeys-reveal-warning = Copie este segredo agora — ele é mostrado uma única vez e não pode ser recuperado depois.
apikeys-copy = Copiar
apikeys-copy-aria = Copiar segredo para a área de transferência
apikeys-copied = Copiado para a área de transferência

apikeys-error-need-label = dê um rótulo à chave para reconhecê-la depois
apikeys-error-create = não foi possível criar a chave
apikeys-error-revoke = não foi possível revogar a chave

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Chaves existentes
apikeys-existing = Chaves existentes
apikeys-none = Nenhuma chave ainda.
apikeys-created = criada em { $date }
apikeys-last-used = último uso em { $date }
apikeys-never-used = nunca usada
apikeys-revoked-at = revogada em { $date }
apikeys-revoke = Revogar
apikeys-status-active = Ativa
apikeys-status-revoked = Revogada

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Escopo
apikeys-capabilities = Permissões
apikeys-cap-read = Ler
apikeys-cap-send = Enviar
apikeys-cap-delete = Excluir
apikeys-surface = Área
apikeys-surface-mail-label = E-mail
apikeys-surface-pim-label = PIM (agenda / tarefas / notas / contatos)
apikeys-accounts = Contas
apikeys-all-accounts = Todas as contas
apikeys-folders = Pastas
apikeys-all-folders = Todas as pastas
apikeys-folder-ids = IDs das pastas (separados por vírgula)
apikeys-folder-subset-aria = Subconjunto de pastas
apikeys-constraints = Restrições
apikeys-expiry = Expiração (RFC 3339, vazio = sem expiração)
apikeys-expiry-aria = Expiração
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Limite de taxa (solicitações/min, vazio = ilimitado)
apikeys-rate-limit-aria = Limite de taxa
apikeys-ip-allowlist = Lista de IPs permitidos (CIDR/IP, separados por vírgula; vazio = qualquer)
apikeys-ip-allowlist-aria = Lista de IPs permitidos
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = ler
apikeys-verb-send = enviar
apikeys-verb-delete = excluir
apikeys-summary-mail = e-mail
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = nenhuma ação
apikeys-summary-nothing = nada
apikeys-summary-none = nenhum
apikeys-summary-verbs-on = { $verbs } em { $surfaces }
apikeys-summary-all-accounts = todas as contas
apikeys-summary-accounts = contas: { $ids }
apikeys-summary-all-folders = todas as pastas
apikeys-summary-folders = pastas: { $ids }
apikeys-summary-expires = expira em { $date }
apikeys-summary-rate = { $n } solic./min
apikeys-summary-ips = IPs: { $ips }
apikeys-summary-mcp-tools = Ferramentas MCP: { $tools }
apikeys-summary-unattended = Envio SEM SUPERVISÃO

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Chaves MCP
apikeys-mcp-heading = Chaves MCP
apikeys-mcp-intro = Conceda a um agente de IA (pelo Model Context Protocol) exatamente as ferramentas que você escolher. Os corpos de mensagens devolvidos ao agente são marcados como entrada não confiável. Cada ferramenta é concedida individualmente.
apikeys-mcp-label-aria = Rótulo da chave MCP
apikeys-mcp-label-placeholder = ex.: agente assistente
apikeys-mcp-create = Criar chave MCP
apikeys-mcp-reveal-warning = Copie este segredo agora — ele é mostrado uma única vez.
apikeys-mcp-error-need-label = dê um rótulo à chave MCP
apikeys-mcp-error-need-tool = conceda pelo menos uma ferramenta
apikeys-mcp-error-create = não foi possível criar a chave MCP

apikeys-tools = Ferramentas
apikeys-tools-group = Ferramentas MCP
apikeys-outbox-suffix = (passa pela Caixa de saída)

apikeys-unattended-aria = Envio sem supervisão
apikeys-unattended-allow = Permitir envio sem supervisão (ignora a Caixa de saída — exige a contra-assinatura de um administrador)
apikeys-unattended-pending = O envio sem supervisão permanece inativo até que um administrador contra-assine esta chave.
apikeys-unattended-disclosure = Por padrão, um envio concedido vai para a sua Caixa de saída e aguarda a sua confirmação no app — a automação não consegue enviar sozinha. O envio sem supervisão REMOVE essa etapa de confirmação humana, para que esta chave possa enviar e-mails sem confirmação. Ele também exige que um administrador contra-assine a chave. Conceda-o apenas a automações em que você confia totalmente; uma chave de envio sem supervisão comprometida pode enviar e-mails em seu nome sem nenhuma solicitação.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Pesquisar e-mails
apikeys-tool-mail-search-desc = Pesquisa mensagens. Os resultados são marcados como entrada não confiável.
apikeys-tool-mail-read-label = Ler e-mails
apikeys-tool-mail-read-desc = Lê o corpo de uma mensagem. Os corpos são marcados como entrada não confiável.
apikeys-tool-folders-list-label = Listar pastas
apikeys-tool-folders-list-desc = Lista as pastas da caixa postal.
apikeys-tool-drafts-create-label = Criar rascunhos
apikeys-tool-drafts-create-desc = Cria um rascunho de mensagem (nunca enviado automaticamente).
apikeys-tool-mail-send-label = Enviar e-mails
apikeys-tool-mail-send-desc = Coloca uma mensagem na Caixa de saída para confirmação no app.
apikeys-tool-calendar-read-label = Ler agenda
apikeys-tool-calendar-read-desc = Lê eventos da agenda.
apikeys-tool-calendar-propose-label = Propor eventos
apikeys-tool-calendar-propose-desc = Propõe (sem confirmar) eventos da agenda.
apikeys-tool-tasks-read-label = Ler tarefas
apikeys-tool-tasks-read-desc = Lê tarefas.
apikeys-tool-tasks-write-label = Gravar tarefas
apikeys-tool-tasks-write-desc = Cria ou atualiza tarefas.
apikeys-tool-contacts-read-label = Ler contatos
apikeys-tool-contacts-read-desc = Lê contatos.
apikeys-surface-scripts-label = Gerenciar scripts Sieve
