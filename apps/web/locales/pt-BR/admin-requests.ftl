# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Não encontrado (404) por intervalo
admin-history-nf-total = Não encontrado (404) neste período
admin-history-nf-share = { $share } das solicitações
admin-history-nf-not-tracked-window = Os erros 404 não foram registrados neste período.
admin-history-nf-not-tracked-before = Os erros 404 não foram registrados antes de { $time }. Os períodos anteriores aparecem como indisponíveis, não como zero.
admin-history-nf-paths-title = Caminhos não encontrados mais frequentes
admin-history-nf-paths-help = Até { $limit } caminhos por hora, de { $from } a { $to }, arredondados para horas inteiras. As strings de consulta são removidas e endereços de e-mail, números, IDs, tokens e valores longos são substituídos por marcadores como {"{"}token{"}"}. Os erros esperados do próprio aplicativo, como solicitações de ícones do navegador, não são contados.
admin-history-nf-paths-approximate = Chegaram mais caminhos distintos do que os mantidos, então algumas contagens são um intervalo em vez de um valor exato.
admin-history-nf-paths-empty = Nenhuma resposta de não encontrado neste período.
admin-history-nf-paths-error = Não foi possível carregar os caminhos não encontrados. Atualize para tentar novamente.
admin-history-nf-paths-method = Método
admin-history-nf-paths-path = Caminho
admin-history-nf-paths-count = Contagem
admin-history-nf-paths-range = { $min } a { $max }
admin-history-nf-paths-client = Dos clientes do próprio aplicativo
admin-history-nf-paths-client-help = Caminhos solicitados pelo aplicativo web, pelo service worker dele ou pelos aplicativos desktop e móveis: arquivos de build, script do worker, API, JMAP, DAV e rotas do console. Um erro aqui geralmente significa que um cliente está desatualizado ou que uma rota está errada.
admin-history-nf-paths-external = Outro tráfego
admin-history-nf-paths-external-help = Caminhos que o aplicativo nunca solicita, como scanners procurando arquivos PHP, dotfiles ou outros softwares, e links de outros lugares.
admin-history-nf-paths-all = Caminhos
admin-history-nf-paths-all-help = Este servidor não distingue os caminhos dos clientes de outro tráfego.
