# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-stats. Source: ../en/admin-stats.ftl

admin-stats-key-figures = Números principais
admin-stats-charts = Gráficos
admin-stats-tables = Tabelas
admin-stats-group-traffic = Tráfego neste período
admin-stats-group-storage = Armazenamento e entrega, agora
admin-stats-refresh = Atualizar estatísticas
admin-stats-loading = A carregar estatísticas…
admin-stats-requests-total = Pedidos
admin-stats-errors-total = Erros do servidor (5xx)
admin-stats-p95-peak = Percentil 95 mais lento
admin-stats-requests = Pedidos
admin-stats-errors = Erros do servidor (5xx)
admin-stats-not-found = Não encontrado (404)
admin-stats-latency = Tempo de resposta
admin-stats-series-p50 = Mediana
admin-stats-series-p95 = Percentil 95
admin-stats-unit-count = Contagem por { $bucket }
admin-stats-unit-duration = Tempo por { $bucket }, na unidade indicada
admin-stats-unit-accounts = Contas bloqueadas por domínio
admin-stats-chart-about = Gráfico de linhas de { $from } a { $to }.
admin-stats-bars-about = { $count ->
    [one] Gráfico de barras com um domínio.
   *[other] Gráfico de barras com { $count } domínios.
}
admin-stats-view-table = Ver como tabela
admin-stats-legend = Legenda
admin-stats-empty-chart = Ainda não foi medido nada neste período. Escolha um período mais longo ou volte quando o servidor tiver tratado alguns pedidos.
admin-stats-empty-blocked = Não há contas bloqueadas.
admin-stats-delta-vs = vs. período anterior
admin-stats-delta-same = Igual ao período anterior
admin-stats-delta-sr-up = Mais { $change } do que no período anterior.
admin-stats-delta-sr-down = Menos { $change } do que no período anterior.
admin-stats-delta-sr-same = Igual ao período anterior.
admin-ct-group = Tempos de carregamento no navegador
admin-ct-metric-timeToUsable = Tempo até estar utilizável
admin-ct-metric-ttfb = Primeiro byte
admin-ct-metric-dcl = DOM pronto
admin-ct-metric-load = Carregamento da página
admin-ct-metric-fcp = Primeira renderização de conteúdo
admin-ct-metric-lcp = Maior renderização de conteúdo
admin-ct-metric-folderOpen = Abertura de pasta
admin-ct-device-desktop = Computador
admin-ct-device-mobile = Telemóvel
admin-ct-device-unknown = Desconhecido
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } amostra
       *[other] { $count } amostras
    }
admin-ct-no-samples = Sem amostras neste período.
admin-ct-few-samples = Apenas { $count } amostras; são precisas pelo menos { $min }.
admin-ct-chart-usable = Tempo até estar utilizável
admin-ct-chart-note = Os percentis só aparecem em intervalos com pelo menos { $min } amostras.
admin-ct-empty-chart = Nenhum intervalo tem as { $min } amostras necessárias para mostrar a mediana. Escolha um período mais longo.
admin-ct-col-samples = Amostras
admin-ct-col-metric = Medida
admin-ct-col-device = Dispositivo
admin-ct-table-title = Tempos de carregamento por dispositivo
admin-ct-table-caption = Amostras, mediana e percentil 95 para cada medida e tipo de dispositivo.
admin-ct-empty-table = Não foram comunicados tempos de carregamento neste período.
admin-ct-failed = Não foi possível carregar os tempos de carregamento.
admin-ct-disabled = A recolha dos tempos de carregamento está desligada. Os valores abaixo são anteriores ao desligar.
admin-ct-note = Apenas de navegadores com sessão iniciada, com { $rate }% das sessões amostradas, de forma anónima e agregada. O tempo até estar utilizável vai do início da navegação até a lista de mensagens mostrar as primeiras linhas. Os valores são estimativas, a cerca de { $error }% do valor real.
