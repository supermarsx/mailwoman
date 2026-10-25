# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: rules. Source: ../en/rules.ftl

rules-title = Regras e filtros
rules-intro = Organize, etiquete ou processe automaticamente as mensagens recebidas. As regras que usam apenas testes padrão rodam no servidor de e-mail como Sieve; as demais rodam no motor no momento da entrega.
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
rules-runs-engine-detail = Aplicada pelo motor no momento da entrega.

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
