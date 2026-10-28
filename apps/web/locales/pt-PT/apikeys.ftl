# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = Chaves de API
apikeys-heading = Chaves de API
apikeys-intro = Crie chaves com âmbito limitado para scripts e integrações. Cada chave é guardada como hash, mostrada uma única vez e pode ser revogada individualmente. Conceda o âmbito mínimo que funcione.

apikeys-label = Etiqueta
apikeys-label-aria = Etiqueta da chave
apikeys-label-placeholder = p. ex. script de cópia de segurança

apikeys-create = Criar chave
apikeys-saved = Já a guardei

apikeys-reveal-warning = Copie este segredo agora — é mostrado uma única vez e não pode ser recuperado.
apikeys-copy = Copiar
apikeys-copy-aria = Copiar o segredo para a área de transferência
apikeys-copied = Copiado para a área de transferência

apikeys-error-need-label = atribua uma etiqueta à chave para a poder reconhecer mais tarde
apikeys-error-create = não foi possível criar a chave
apikeys-error-revoke = não foi possível revogar a chave

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Chaves existentes
apikeys-existing = Chaves existentes
apikeys-none = Ainda não há chaves.
apikeys-created = criada em { $date }
apikeys-last-used = última utilização em { $date }
apikeys-never-used = nunca utilizada
apikeys-revoked-at = revogada em { $date }
apikeys-revoke = Revogar
apikeys-status-active = Ativa
apikeys-status-revoked = Revogada

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Âmbito
apikeys-capabilities = Capacidades
apikeys-cap-read = Ler
apikeys-cap-send = Enviar
apikeys-cap-delete = Eliminar
apikeys-surface = Área
apikeys-surface-mail-label = Correio
apikeys-surface-pim-label = PIM (calendário / tarefas / notas / contactos)
apikeys-accounts = Contas
apikeys-all-accounts = Todas as contas
apikeys-folders = Pastas
apikeys-all-folders = Todas as pastas
apikeys-folder-ids = Ids das pastas (separados por vírgulas)
apikeys-folder-subset-aria = Subconjunto de pastas
apikeys-constraints = Restrições
apikeys-expiry = Expiração (RFC 3339, vazio = sem expiração)
apikeys-expiry-aria = Expiração
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Limite de pedidos (pedidos/min, vazio = ilimitado)
apikeys-rate-limit-aria = Limite de pedidos
apikeys-ip-allowlist = Lista de permissões de IP (CIDR/IP, separados por vírgulas; vazio = qualquer)
apikeys-ip-allowlist-aria = Lista de permissões de IP
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = ler
apikeys-verb-send = enviar
apikeys-verb-delete = eliminar
apikeys-summary-mail = correio
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = sem ações
apikeys-summary-nothing = nada
apikeys-summary-none = nenhum
apikeys-summary-verbs-on = { $verbs } em { $surfaces }
apikeys-summary-all-accounts = todas as contas
apikeys-summary-accounts = contas: { $ids }
apikeys-summary-all-folders = todas as pastas
apikeys-summary-folders = pastas: { $ids }
apikeys-summary-expires = expira em { $date }
apikeys-summary-rate = { $n } pedidos/min
apikeys-summary-ips = IPs: { $ips }
apikeys-summary-mcp-tools = Ferramentas MCP: { $tools }
apikeys-summary-unattended = Envio SEM SUPERVISÃO

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = Chaves MCP
apikeys-mcp-heading = Chaves MCP
apikeys-mcp-intro = Conceda a um agente de IA (através do Model Context Protocol) exatamente as ferramentas que escolher. Os corpos das mensagens devolvidos ao agente são marcados como entrada não fidedigna. Cada ferramenta é concedida individualmente.
apikeys-mcp-label-aria = Etiqueta da chave MCP
apikeys-mcp-label-placeholder = p. ex. agente assistente
apikeys-mcp-create = Criar chave MCP
apikeys-mcp-reveal-warning = Copie este segredo agora — é mostrado uma única vez.
apikeys-mcp-error-need-label = atribua uma etiqueta à chave MCP
apikeys-mcp-error-need-tool = conceda pelo menos uma ferramenta
apikeys-mcp-error-create = não foi possível criar a chave MCP

apikeys-tools = Ferramentas
apikeys-tools-group = Ferramentas MCP
apikeys-outbox-suffix = (sujeito à Caixa de saída)

apikeys-unattended-aria = Envio sem supervisão
apikeys-unattended-allow = Permitir envio sem supervisão (ignora a Caixa de saída — exige a contra-assinatura de um administrador)
apikeys-unattended-pending = O envio sem supervisão fica inativo até um administrador contra-assinar esta chave.
apikeys-unattended-disclosure = Por predefinição, um envio concedido fica na sua Caixa de saída e aguarda que o confirme na aplicação — a automatização não pode enviar sozinha. O envio sem supervisão REMOVE esse passo de confirmação humana, para que esta chave possa enviar correio sem confirmação. Exige ainda que um administrador contra-assine a chave. Conceda-o apenas a automatizações em que confie totalmente; uma chave de envio sem supervisão comprometida pode enviar correio em seu nome sem qualquer aviso.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Pesquisar correio
apikeys-tool-mail-search-desc = Pesquisar mensagens. Os resultados são marcados como entrada não fidedigna.
apikeys-tool-mail-read-label = Ler correio
apikeys-tool-mail-read-desc = Ler o corpo de uma mensagem. Os corpos são marcados como entrada não fidedigna.
apikeys-tool-folders-list-label = Listar pastas
apikeys-tool-folders-list-desc = Listar as pastas da caixa de correio.
apikeys-tool-drafts-create-label = Criar rascunhos
apikeys-tool-drafts-create-desc = Criar um rascunho de mensagem (nunca enviado automaticamente).
apikeys-tool-mail-send-label = Enviar correio
apikeys-tool-mail-send-desc = Colocar uma mensagem na Caixa de saída para confirmação na aplicação.
apikeys-tool-calendar-read-label = Ler calendário
apikeys-tool-calendar-read-desc = Ler eventos do calendário.
apikeys-tool-calendar-propose-label = Propor eventos
apikeys-tool-calendar-propose-desc = Propor (sem confirmar) eventos do calendário.
apikeys-tool-tasks-read-label = Ler tarefas
apikeys-tool-tasks-read-desc = Ler tarefas.
apikeys-tool-tasks-write-label = Escrever tarefas
apikeys-tool-tasks-write-desc = Criar ou atualizar tarefas.
apikeys-tool-contacts-read-label = Ler contactos
apikeys-tool-contacts-read-desc = Ler contactos.
apikeys-surface-scripts-label = Gerir scripts Sieve
