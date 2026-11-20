admin-cache-title = Manutenção do cache
admin-cache-help = Por padrão, o conteúdo de e-mail em cache expira após 60 dias sem uso. Abrir uma mensagem expirada baixa seu conteúdo novamente. As mensagens originais, os rascunhos e o conteúdo local protegido são preservados.
admin-cache-expiry = Tempo de vida das mensagens em cache
admin-cache-orphans = Corpos sem referência e recuperação de espaço
admin-cache-advanced = Limites e agendamento dos lotes
admin-cache-settings-help = As alterações seguem a política, a validação e a trilha de auditoria normais das configurações administrativas.
admin-cache-expiry-enabled = Expirar automaticamente os corpos das mensagens em cache
admin-cache-retention-days = Dias sem uso antes da expiração
admin-cache-expiry-status = Expiração dos corpos em cache
admin-cache-expiry-result = Verificados: { $scanned } · Referências de cache desvinculadas: { $detached } · Protegidos: { $protected }
admin-cache-expiry-help = Somente corpos cuja recuperação da caixa de correio de origem foi confirmada são elegíveis. Verificações e atualizações dos cabeçalhos não renovam o prazo de inatividade.
admin-cache-inspect = Inspecionar o armazenamento do cache
admin-cache-inspect-help = Abra a observabilidade do banco de dados para medir o conteúdo em cache, os corpos referenciados e as cópias órfãs.
admin-cache-no-status = Nenhum lote de expiração foi concluído neste processo do servidor.
admin-cache-settings-unavailable = Não foi possível carregar as configurações do cache.
admin-cache-change-log = Histórico de sincronização
admin-cache-change-log-help = Por padrão, os registros antigos de alterações expiram após 60 dias. Clientes que usam tokens de sincronização mais antigos recebem uma resposta de ressincronização completa.
admin-cache-no-sources = Nenhuma caixa de correio conectada tem atualmente um caminho confirmado para baixar novamente. O conteúdo protegido é mantido.
admin-cache-wal-recycled = Espaço WAL alocado reciclado; a limpeza pode continuar.
admin-cache-wal-recovery = A limpeza está aguardando a recuperação ou o checkpoint do WAL. Os dados pendentes do diário são preservados.

admin-cache-removed-payload = Conteúdo em cache removido
admin-cache-oversized = Limpeza de corpos grandes adiada
admin-cache-change-removed = Registros do histórico de sincronização removidos

admin-cache-parsed-copies = Cópias do conteúdo processado removidas
admin-cache-parsed-bytes = Dados de conteúdo processado removidos
admin-cache-change-chunked = Estados de sincronização limpos em partes

# Public static representations only; process-local counters.
admin-cache-assets-title = Cache RAM de recursos estáticos públicos
admin-cache-assets-clear = Limpar cache estático
admin-cache-assets-help = Armazena apenas recursos públicos. O orçamento cobre dados próprios residentes, não respostas em andamento nem a memória total do processo. A página inicial e o service worker publicados podem permanecer em cache durante o intervalo de revalidação.
admin-cache-assets-clear-help = A limpeza libera entradas retidas; as respostas ativas terminam normalmente. O pré-carregamento ativo pode preencher o cache novamente. Os contadores abrangem este processo do servidor.
admin-cache-assets-stale = A atualização da política falhou; a última configuração aplicada permanece ativa.
admin-cache-assets-applied = Configuração aplicada: { $time }
admin-cache-assets-resident = Dados próprios / orçamento
admin-cache-assets-entries = Entradas / limite
admin-cache-assets-file = Limite efetivo por arquivo próprio
admin-cache-assets-loads = Em execução / limite; na fila / limite
admin-cache-assets-hits = Acertos / carregamentos aceitos / esperas compartilhadas
admin-cache-assets-evictions = Removidas / expiradas / grandes demais
admin-cache-assets-failures = Recusas por ocupação / tarefas com falha
admin-cache-assets-preload = Modo de pré-carregamento; retidas / ignoradas
