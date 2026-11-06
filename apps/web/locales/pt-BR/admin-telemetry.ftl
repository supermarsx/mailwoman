# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Telemetria operacional
admin-telemetry-help = Exporte traces e métricas operacionais para um coletor compatível com OpenTelemetry, como um OpenTelemetry Collector auto-hospedado. A exportação vem desativada por padrão. As alterações salvas são aplicadas após reiniciar o servidor.
admin-telemetry-back = Voltar para observabilidade
admin-telemetry-saved = Configuração de telemetria salva. Reinicie o servidor para aplicar as alterações.
admin-telemetry-restart = A configuração salva difere do servidor em execução. É necessário reiniciar.
admin-telemetry-reload = Recarregar configuração
admin-telemetry-save = Salvar configuração de telemetria
admin-telemetry-collector = Coletor e sinais
admin-telemetry-collector-help = Somente um administrador pode escolher este destino válido para todo o servidor. Use um coletor privado ou HTTPS; HTTP transmite a telemetria sem criptografia.
admin-telemetry-enabled = Ativar a exportação OTLP
admin-telemetry-traces = Exportar traces
admin-telemetry-metrics = Exportar métricas
admin-telemetry-endpoint = Endpoint OTLP gRPC
admin-telemetry-endpoint-help = Endereço HTTP ou HTTPS absoluto do coletor, geralmente a porta 4317. Não inclua credenciais, consulta nem fragmento. É um endpoint gRPC, não a porta OTLP HTTP 4318.
admin-telemetry-serviceName = Nome do serviço
admin-telemetry-serviceName-help = Rótulo de serviço estável usado para identificar esta implantação do Mailwoman no seu coletor. Não use endereço de e-mail nem outras informações pessoais.
admin-telemetry-environment = Ambiente
admin-telemetry-environment-help = Rótulo opcional da implantação, como production ou staging. É enviado junto com a telemetria exportada.
admin-telemetry-pinned = Os campos desativados são controlados por variáveis de ambiente da implantação. Altere essas variáveis para substituir os valores.
admin-telemetry-limits = Amostragem e limites de exportação
admin-telemetry-limits-help = Limite a memória e o tráfego para o coletor. O tamanho do lote não pode exceder o tamanho da fila. Filas cheias podem descartar spans em vez de bloquear as solicitações de e-mail.
admin-telemetry-sampleRatio = Proporção de amostragem de traces
admin-telemetry-sampleRatio-help = Fração dos novos traces raiz a manter, de 0 a 1. Os spans filhos seguem a decisão de amostragem do pai.
admin-telemetry-exportTimeoutMs = Tempo limite de exportação (milissegundos)
admin-telemetry-exportTimeoutMs-help = Duração máxima de uma solicitação de exportação ao coletor, de 100 a 30000 milissegundos.
admin-telemetry-metricsIntervalSecs = Intervalo das métricas (segundos)
admin-telemetry-metricsIntervalSecs-help = Tempo entre as exportações de métricas, de 10 a 3600 segundos.
admin-telemetry-spanQueueSize = Tamanho da fila de spans
admin-telemetry-spanQueueSize-help = Número máximo de spans aguardando exportação, de 128 a 32768.
admin-telemetry-spanBatchSize = Tamanho do lote de spans
admin-telemetry-spanBatchSize-help = Número máximo de spans em uma exportação, de 1 a 4096 e não maior que o tamanho da fila.
admin-telemetry-spanExportDelayMs = Atraso do lote de spans (milissegundos)
admin-telemetry-spanExportDelayMs-help = Espera normal entre lotes de spans, de 100 a 60000 milissegundos.
admin-telemetry-prometheus-help = Um endpoint autenticado separado para um coletor compatível com Prometheus. O acesso sempre exige o bearer token MW_METRICS_TOKEN da implantação; o valor dele nunca é devolvido a esta página.
admin-telemetry-prometheusEnabled = Ativar a coleta do Prometheus
admin-telemetry-token-missing = Nenhum token de coleta está configurado. O endpoint de métricas continua inacessível até o operador definir MW_METRICS_TOKEN.
