# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: rules. Source: ../en/rules.ftl

rules-title = Regras e filtros
rules-intro = Gira os scripts de filtragem de correio e as regras visuais. As funcionalidades disponíveis dependem desta conta e das definições do administrador.
rules-empty = Ainda não existem regras.
rules-unavailable = Este servidor de correio não suporta regras de correio.
rules-load-failed = Não foi possível carregar as regras.
rules-new = Nova regra
rules-list-label = Regras de correio
rules-enabled = Ativada
rules-delete-named = Eliminar regra { $name }

# Builder
rules-builder-title = Construtor de regras
rules-builder-hint = Selecione uma regra para editar ou crie uma nova.
rules-name-label = Nome da regra
rules-match-legend = Quando executar
rules-match-all = Cumprir todas as condições
rules-match-any = Cumprir qualquer condição
rules-conditions-legend = Condições
rules-actions-legend = Ações
rules-add-condition = Adicionar condição
rules-add-action = Adicionar ação
rules-remove-condition = Remover condição
rules-remove-action = Remover ação
rules-cond-type-label = Campo
rules-cond-op-label = Operador
rules-cond-value-label = Valor a corresponder
rules-act-type-label = Ação
rules-act-value-label = Valor da ação
rules-save = Guardar regra
rules-editing = A editar { $name }
rules-save-refused = A regra não foi guardada: { $reason }

# Condition fields and operators
rules-cond-from = De
rules-cond-to = Para
rules-cond-subject = Assunto
rules-cond-thread-unsupported = Conversa { $op } { $value }: as condições de conversa não são suportadas, pelo que a regra só pode ser guardada depois de remover esta condição.
rules-op-contains = contém
rules-op-is = é

# Actions
rules-act-move = Mover para a caixa de correio
rules-act-tag = Adicionar etiqueta
rules-act-archive = Arquivar
rules-act-suppress = Suprimir notificação
rules-act-stop = Parar o processamento

# Where it runs
rules-runs-server = Servidor (Sieve)
rules-runs-server-detail = Carregado para o servidor de correio e aplicado lá.
rules-runs-engine = Motor
rules-runs-engine-detail = Aplicado quando o motor sincroniza as novas mensagens da caixa de entrada; as respostas e movimentações aguardam pela sincronização seguinte.

# Panes
rules-panes-label = Ferramentas de regras
rules-tab-builder = Construtor
rules-tab-raw = Sieve em bruto
rules-tab-dryrun = Simulação

# Raw editor
rules-raw-label = Código Sieve em bruto
rules-raw-reset = Repor o gerado
rules-lint-clean = Não foram encontrados problemas.

# Dry run
rules-dryrun-title = Simulação
rules-dryrun-help = Introduza uma mensagem de exemplo para pré-visualizar as regras que seriam acionadas.
rules-dryrun-results = Resultados da simulação
rules-dryrun-none = Não existem regras ativadas para avaliar.
rules-dryrun-nomatch = Sem correspondência
rules-dryrun-stopped = Não alcançada (uma regra anterior parou o processamento)

rules-scripts-label = Scripts Sieve
rules-script-active = Ativo
rules-script-new = Novo script
rules-script-export = Exportar script
rules-script-import = Importar script
rules-script-too-large = O script excede o limite de tamanho.
rules-script-read-only = O seu administrador desativou a edição de scripts em bruto.
rules-script-validate = Validar script
rules-script-deactivate = Desativar
rules-script-activate = Ativar
rules-script-saved = Script guardado.

rules-vacation-title = Resposta automática de ausência
rules-vacation-enabled = Ativar respostas automáticas
rules-vacation-from = Início (UTC)
rules-vacation-to = Fim (UTC)
rules-vacation-body = Texto da resposta
rules-vacation-saved = Resposta automática guardada.
rules-vacation-saved-off = Guardado. As respostas automáticas estão desativadas.
rules-vacation-runs-server = O seu servidor de correio envia esta resposta quando chega correio.
rules-vacation-runs-engine = O Mailwoman envia esta resposta quando sincroniza correio novo na caixa de entrada, pelo que uma resposta pode aguardar a próxima sincronização.
rules-vacation-unavailable = As respostas automáticas de ausência não estão disponíveis para esta conta: o servidor de correio não as oferece ou o administrador desativou-as.
rules-script-template = Modelo de script
rules-forward-title = Reencaminhamento
rules-forward-address = Reencaminhar para
rules-forward-copy = Manter uma cópia nesta conta
rules-forward-refused = O seu administrador não permite o reencaminhamento para este domínio.
