# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-requests. Source: ../en/admin-requests.ftl

admin-history-not-found = Não encontrado (404) por intervalo
admin-history-nf-total = Não encontrado (404) neste período
admin-history-nf-share = { $share } dos pedidos
admin-history-nf-not-tracked-window = Os 404 não eram registados durante este período.
admin-history-nf-not-tracked-before = Os 404 não eram registados antes de { $time }. Os períodos anteriores aparecem como indisponíveis, não como zero.
admin-history-nf-paths-title = Caminhos não encontrados mais frequentes
admin-history-nf-paths-help = Até { $limit } caminhos por hora, de { $from } a { $to }, alargado a horas completas. As query strings são removidas e os endereços de email, números, IDs, tokens e valores longos são substituídos por marcadores como {"{"}token{"}"}. As falhas esperadas da própria aplicação, como os pedidos de ícones do navegador, não são contadas.
admin-history-nf-paths-approximate = Chegaram mais caminhos distintos do que os que são guardados, pelo que algumas contagens são um intervalo e não um valor exato.
admin-history-nf-paths-empty = Nenhuma resposta 404 neste período.
admin-history-nf-paths-error = Não foi possível carregar os caminhos não encontrados. Atualize para tentar novamente.
admin-history-nf-paths-method = Método
admin-history-nf-paths-path = Caminho
admin-history-nf-paths-count = Contagem
admin-history-nf-paths-range = { $min } a { $max }
admin-history-nf-paths-client = Dos clientes da própria aplicação
admin-history-nf-paths-client-help = Caminhos que a aplicação web, o seu service worker ou as aplicações de computador e móveis pedem: ficheiros da compilação, o script do worker e rotas de API, JMAP, DAV e consola. Uma falha aqui indica normalmente um cliente desatualizado ou uma rota errada.
admin-history-nf-paths-external = Outro tráfego
admin-history-nf-paths-external-help = Caminhos que a aplicação nunca pede, como scanners à procura de ficheiros PHP, ficheiros ocultos ou outro software, e ligações de outros sítios.
admin-history-nf-paths-all = Caminhos
admin-history-nf-paths-all-help = Este servidor não distingue caminhos de clientes de outro tráfego.
