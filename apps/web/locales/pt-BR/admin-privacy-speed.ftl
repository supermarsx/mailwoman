# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Velocidade
privacy-speed-unavailable = Os números de velocidade não estão disponíveis neste servidor.
privacy-speed-empty = Nenhuma imagem foi servida neste período ainda.
privacy-speed-off = O monitoramento de velocidade está desativado (configuração: Monitoramento de velocidade do proxy de imagens). Nada novo é registrado.
privacy-speed-verdict-healthy = Saudável
privacy-speed-verdict-slow = Lento
privacy-speed-verdict-degraded = Degradado
privacy-speed-verdict-noData = Dados insuficientes
privacy-speed-reason-ok = As imagens buscadas e os acertos de cache estão dentro dos limites abaixo e poucas solicitações falharam.
privacy-speed-reason-failureRate = { $failurePct } % das solicitações terminaram ocupadas, com tempo esgotado ou com falha upstream.
privacy-speed-reason-missP95 = As imagens que precisaram ser buscadas estão lentas: o percentil 95 delas está acima do limite abaixo.
privacy-speed-reason-hitP95 = As imagens em cache estão lentas para responder: o percentil 95 delas está acima do limite abaixo.
privacy-speed-reason-fewRequests = Menos de { $min } solicitações neste período, poucas demais para avaliar.
privacy-speed-thresholds = Lento a partir de: percentil 95 da imagem buscada { $slowMiss }, percentil 95 da imagem em cache { $slowHit } ou { $slowFail } % ocupado, com tempo esgotado ou com falha. Degradado a partir de: percentil 95 da imagem buscada { $degradedMiss } ou { $degradedFail } % com falha. Limites fixos, não configurações.
privacy-speed-median = Mediana
privacy-speed-p95 = Percentil 95
privacy-speed-p99 = Percentil 99
privacy-speed-max = Mais lenta
privacy-speed-requests = Solicitações medidas
privacy-speed-busy = Respostas de ocupado
privacy-speed-timeouts = Tempos esgotados
privacy-speed-upstream-errors = Erros upstream
privacy-speed-hit = Acertos de cache
privacy-speed-miss = Imagens buscadas (falhas de cache)
privacy-speed-split-count = { $count } solicitações
privacy-speed-split-times = Mediana { $p50 }, percentil 95 { $p95 }, mais lenta { $max }.
privacy-speed-chart = Tempo para servir uma imagem
privacy-speed-chart-miss = Tempo para servir uma imagem buscada
privacy-speed-chart-unit = Da solicitação recebida à resposta pronta
privacy-speed-phases = Para onde vai o tempo
privacy-speed-phases-help = Para as solicitações que não foram acertos de cache: quanto cada etapa durou e sua parcela do tempo total.
privacy-speed-phase = Etapa
privacy-speed-share = Parcela do tempo
privacy-speed-phase-queue = Aguardando capacidade
privacy-speed-phase-cache = Consulta ao cache
privacy-speed-phase-upstream = Busca upstream
privacy-speed-phase-decode = Decodificar e recodificar
privacy-speed-phase-other = Todo o resto
privacy-speed-note = Medido somente neste processo do servidor, mantido em memória e vazio após uma reinicialização. Os percentis têm precisão de até { $error } %. Nenhum URL, host ou conta é armazenado.
