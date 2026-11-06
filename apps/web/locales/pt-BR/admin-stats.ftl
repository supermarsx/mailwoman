# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Números principais
admin-stats-charts = Gráficos
admin-stats-tables = Tabelas
admin-stats-group-traffic = Tráfego neste período
admin-stats-group-storage = Armazenamento e entrega, agora
admin-stats-refresh = Atualizar estatísticas
admin-stats-loading = Carregando estatísticas…
admin-stats-requests-total = Solicitações
admin-stats-errors-total = Erros do servidor (5xx)
admin-stats-p95-peak = Percentil 95 mais lento
admin-stats-requests = Solicitações
admin-stats-errors = Erros do servidor (5xx)
admin-stats-not-found = Não encontrado (404)
admin-stats-latency = Tempo de resposta
admin-stats-series-p50 = Mediana
admin-stats-series-p95 = Percentil 95
admin-stats-unit-count = Contagem por { $bucket }
admin-stats-unit-duration = Tempo por { $bucket }, na unidade exibida
admin-stats-unit-accounts = Contas bloqueadas por domínio
admin-stats-chart-about = Gráfico de linhas de { $from } a { $to }.
admin-stats-bars-about = { $count ->
    [one] Gráfico de barras com um domínio.
   *[other] Gráfico de barras com { $count } domínios.
}
admin-stats-view-table = Ver como tabela
admin-stats-legend = Legenda
admin-stats-empty-chart = Nada foi medido neste período ainda. Escolha um período mais longo ou volte depois que o servidor tiver tratado algumas solicitações.
admin-stats-empty-blocked = Nenhuma conta está bloqueada.
admin-stats-delta-vs = em relação ao período anterior
admin-stats-delta-same = Igual ao período anterior
admin-stats-delta-sr-up = Aumento de { $change } em relação ao período anterior.
admin-stats-delta-sr-down = Queda de { $change } em relação ao período anterior.
admin-stats-delta-sr-same = Igual ao período anterior.
admin-ct-group = Tempos de carregamento do cliente
admin-ct-metric-timeToUsable = Tempo até ficar utilizável
admin-ct-metric-ttfb = Primeiro byte
admin-ct-metric-dcl = DOM pronto
admin-ct-metric-load = Carregamento da página
admin-ct-metric-fcp = First contentful paint
admin-ct-metric-lcp = Largest contentful paint
admin-ct-metric-folderOpen = Abertura de pasta
admin-ct-metric-folderLoadCold = Carregamento de pasta, pela rede
admin-ct-metric-folderLoadCached = Carregamento de pasta, do cache
admin-ct-metric-messageLoadCold = Carregamento de mensagem, pela rede
admin-ct-metric-messageLoadCached = Carregamento de mensagem, do cache ou prefetch
admin-ct-device-desktop = Desktop
admin-ct-device-mobile = Dispositivo móvel
admin-ct-device-unknown = Desconhecido
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } amostra
       *[other] { $count } amostras
    }
admin-ct-no-samples = Nenhuma amostra neste período.
admin-ct-few-samples = Apenas { $count } amostras; são necessárias pelo menos { $min }.
admin-ct-chart-usable = Tempo até ficar utilizável
admin-ct-group-folder = Tempos de carregamento de pastas
admin-ct-group-message = Tempos de carregamento de mensagens
admin-ct-chart-folder-cold = Carregamento de pasta pela rede
admin-ct-chart-folder-cached = Carregamento de pasta do cache
admin-ct-chart-message-cold = Carregamento de mensagem pela rede
admin-ct-chart-message-cached = Carregamento de mensagem do cache ou prefetch
admin-ct-folder-off = Os tempos de carregamento de pastas vêm desativados por padrão. Somente as pessoas que os ativaram são medidas.
admin-ct-message-off = Os tempos de carregamento de mensagens vêm desativados por padrão. Somente as pessoas que os ativaram são medidas.
admin-ct-chart-note = Os percentis só aparecem para intervalos com pelo menos { $min } amostras.
admin-ct-empty-chart = Nenhum intervalo tem as { $min } amostras necessárias para mostrar uma mediana. Escolha um período mais longo.
admin-ct-col-samples = Amostras
admin-ct-col-metric = Medida
admin-ct-col-device = Dispositivo
admin-ct-table-title = Tempos de carregamento do cliente por dispositivo
admin-ct-table-caption = Amostras, mediana e percentil 95 para cada medida e classe de dispositivo.
admin-ct-empty-table = Nenhum tempo de carregamento de navegador foi relatado neste período.
admin-ct-failed = Não foi possível carregar os tempos de carregamento do cliente.
admin-ct-disabled = A coleta dos tempos de carregamento do cliente está desativada. Os números abaixo são de antes de ela ser desativada.
admin-ct-note = Somente de navegadores com login, { $rate }% das sessões amostradas, anônimas e agregadas. O tempo até ficar utilizável vai do início da navegação até a lista de mensagens mostrar as primeiras linhas. Os números são estimativas, dentro de cerca de { $error }% do valor real.
