# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: rules. Source: ../en/rules.ftl

rules-title = Regras e filtros
rules-intro = Gerencie scripts de filtragem e regras visuais. Os recursos disponíveis dependem da conta e das configurações do administrador.
rules-empty = Nenhuma regra ainda.
rules-unavailable = Este servidor de e-mail não oferece suporte a regras.
rules-load-failed = Não foi possível carregar as regras.
rules-new = Nova regra
rules-list-label = Regras de e-mail
rules-enabled = Ativada
rules-delete-named = Excluir a regra { $name }

# Builder
rules-builder-title = Editor de regras
rules-builder-hint = Selecione uma regra para editar ou crie uma nova.
rules-name-label = Nome da regra
rules-match-legend = Quando executar
rules-match-all = Atender a todas as condições
rules-match-any = Atender a qualquer condição
rules-conditions-legend = Condições
rules-actions-legend = Ações
rules-add-condition = Adicionar condição
rules-add-action = Adicionar ação
rules-remove-condition = Remover condição
rules-remove-action = Remover ação
rules-cond-type-label = Campo
rules-cond-op-label = Operador
rules-cond-value-label = Valor a comparar
rules-act-type-label = Ação
rules-act-value-label = Valor da ação
rules-save = Salvar regra
rules-editing = Editando { $name }
rules-save-refused = A regra não foi salva: { $reason }

# Condition fields and operators
rules-cond-from = De
rules-cond-to = Para
rules-cond-subject = Assunto
rules-cond-thread-unsupported = Conversa { $op } { $value }: condições de conversa não são suportadas, então a regra não pode ser salva até que esta condição seja removida.
rules-op-contains = contém
rules-op-is = é

# Actions
rules-act-move = Mover para a pasta
rules-act-tag = Adicionar etiqueta
rules-act-archive = Arquivar
rules-act-suppress = Suprimir notificação
rules-act-stop = Parar o processamento

# Where it runs
rules-runs-server = Servidor (Sieve)
rules-runs-server-detail = Enviada ao servidor de e-mail e aplicada lá.
rules-runs-engine = Motor
rules-runs-engine-detail = Aplicado ao sincronizar novas mensagens; respostas e movimentações aguardam a próxima sincronização.

# Panes
rules-panes-label = Ferramentas de regras
rules-tab-builder = Editor
rules-tab-raw = Sieve bruto
rules-tab-dryrun = Simulação

# Raw editor
rules-raw-label = Código-fonte Sieve bruto
rules-raw-reset = Restaurar o gerado
rules-lint-clean = Nenhum problema encontrado.

# Dry run
rules-dryrun-title = Simulação
rules-dryrun-help = Insira uma mensagem de exemplo para ver quais regras seriam acionadas.
rules-dryrun-results = Resultados da simulação
rules-dryrun-none = Nenhuma regra ativada para avaliar.
rules-dryrun-nomatch = Sem correspondência
rules-dryrun-stopped = Não alcançada (uma regra anterior parou o processamento)

rules-scripts-label = Scripts Sieve
rules-script-active = Ativo
rules-script-new = Novo script
rules-script-export = Exportar script
rules-script-import = Importar script
rules-script-too-large = O script excede o limite de tamanho.
rules-script-read-only = A edição direta de scripts foi desativada pelo administrador.
rules-script-validate = Validar script
rules-script-deactivate = Desativar
rules-script-activate = Ativar
rules-script-saved = Script salvo.

rules-vacation-title = Resposta de ausência
rules-vacation-enabled = Ativar respostas automáticas
rules-vacation-from = Início (UTC)
rules-vacation-to = Fim (UTC)
rules-vacation-body = Texto da resposta
rules-vacation-saved = Resposta de ausência salva.
rules-script-template = Modelo de script
rules-forward-title = Encaminhamento
rules-forward-address = Encaminhar para
rules-forward-copy = Manter uma cópia nesta conta
rules-forward-refused = O administrador não permite encaminhar para este domínio.
