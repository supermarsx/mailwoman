# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-analytics. Source: ../en/admin-analytics.ftl

admin-analytics-title = Análises de utilização
admin-analytics-help = Contagens de eventos opcionais, desativadas por predefinição. Apenas os utilizadores de caixas de correio que consentem contribuem com nomes de ecrãs fixos. Não são executados scripts externos na aplicação. O conteúdo do correio, os endereços, as pesquisas, os detalhes das rotas e os identificadores de utilizadores nunca são incluídos. As opções Do Not Track e Global Privacy Control do navegador sobrepõem-se ao consentimento.
admin-analytics-collector = Configuração do coletor
admin-analytics-enabled = Ativar a recolha opcional de análises
admin-analytics-provider = Fornecedor
admin-analytics-endpoint = Endpoint do coletor
admin-analytics-endpoint-help = Utilize um URL HTTPS absoluto que termine em /api/send para o Umami ou /matomo.php para o Matomo. Credenciais, parâmetros de consulta e fragmentos são recusados.
admin-analytics-site = ID do website ou do site
admin-analytics-site-help = O Umami utiliza um UUID de website. O Matomo utiliza um ID de site numérico positivo.
admin-analytics-http = Permitir HTTP não encriptado para este coletor
admin-analytics-save = Guardar configuração
admin-analytics-reload = Recarregar
admin-analytics-saved = Configuração de análises guardada.
admin-analytics-counts = Estado do exportador
admin-analytics-status-label = Estado
admin-analytics-status-count = Eventos
admin-analytics-counts-help = As contagens locais do processo são repostas quando o servidor reinicia. «Aceite» significa que o coletor aceitou o pedido, não que reteve um evento. Visitantes únicos, dispositivos cliente e geografia dos clientes não estão disponíveis; os coletores veem a ligação do exportador.
admin-analytics-queued = Em fila
admin-analytics-delivered = Aceite pelo coletor
admin-analytics-dropped = Descartados
admin-analytics-private = Destinos privados do coletor
admin-analytics-private-help = Permita um anfitrião ou rede específicos apenas para o tráfego de análises. Os endereços de metadados, de loopback e link-local continuam bloqueados. O encaminhamento por proxy continua a aplicar-se. Remover uma entrada afeta os pedidos futuros.
admin-analytics-destination = Anfitrião ou rede, opcionalmente com porta
admin-analytics-allow = Permitir destino
admin-analytics-remove = Remover
admin-analytics-ga-help = Envia contagens de ecrãs consentidas através do servidor com o Measurement Protocol do Google Analytics 4. Não são carregados scripts da Google nem cookies de rastreio. O consentimento publicitário é recusado. Os relatórios de visitantes, sessões e envolvimento não são suportados.
admin-analytics-ga-global = Endpoint global da Google
admin-analytics-ga-eu = Endpoint de recolha da Google na UE
admin-analytics-ga-id = ID de medição
admin-analytics-ga-secret = Segredo de API do Measurement Protocol
admin-analytics-ga-secret-stored = Está guardado um segredo de forma segura. Deixe em branco para o manter. Alterar o endpoint ou o ID de medição requer um segredo de substituição.
admin-analytics-ga-secret-needed = Crie um segredo de API do Measurement Protocol no seu fluxo de dados web do Google Analytics. O servidor guarda-o encriptado e nunca o devolve a este formulário.
admin-analytics-ga-secret-clear = Remover o segredo de API guardado (desative primeiro a recolha)
