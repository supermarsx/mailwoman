admin-cache-title = Manutenção da cache
admin-cache-help = Por predefinição, o conteúdo de correio em cache expira após 60 dias sem utilização. Ao abrir uma mensagem expirada, o conteúdo é transferido novamente. As mensagens originais, os rascunhos e o conteúdo local protegido são preservados.
admin-cache-expiry = Duração das mensagens em cache
admin-cache-orphans = Corpos sem referência e recuperação de espaço
admin-cache-advanced = Limites e temporização dos lotes
admin-cache-settings-help = As alterações seguem a política, a validação e o registo de auditoria normais das definições de administração.
admin-cache-expiry-enabled = Expirar automaticamente os corpos das mensagens em cache
admin-cache-retention-days = Dias sem utilização até à expiração
admin-cache-expiry-status = Expiração dos corpos em cache
admin-cache-expiry-result = Analisados: { $scanned } · Referências de cache desligadas: { $detached } · Protegidos: { $protected }
admin-cache-expiry-help = Apenas são elegíveis os corpos cuja recuperação a partir da caixa de correio de origem foi confirmada. As análises e as atualizações dos cabeçalhos não renovam o prazo de inatividade.
admin-cache-inspect = Inspecionar o armazenamento da cache
admin-cache-inspect-help = Abra a observabilidade da base de dados para medir o conteúdo em cache, os corpos referenciados e as cópias órfãs.
admin-cache-no-status = Ainda não foi concluído nenhum lote de expiração neste processo do servidor.
admin-cache-settings-unavailable = Não foi possível carregar as definições da cache.
admin-cache-change-log = Histórico de sincronização
admin-cache-change-log-help = Por predefinição, os registos de alterações antigos expiram após 60 dias. Os clientes que usam tokens de sincronização mais antigos recebem uma resposta de ressincronização completa.
admin-cache-no-sources = Nenhuma caixa de correio ligada tem atualmente uma via de nova transferência confirmada. O conteúdo protegido é preservado.
admin-cache-wal-recycled = Espaço WAL alocado reciclado; a limpeza pode continuar.
admin-cache-wal-recovery = A limpeza está à espera da recuperação ou do checkpoint do WAL. Os dados pendentes do diário são preservados.

admin-cache-removed-payload = Conteúdo em cache removido
admin-cache-oversized = Limpeza de corpos grandes adiada
admin-cache-change-removed = Registos do histórico de sincronização removidos

admin-cache-parsed-copies = Cópias do conteúdo processado removidas
admin-cache-parsed-bytes = Dados de conteúdo processado removidos
admin-cache-change-chunked = Estados de sincronização limpos por partes
