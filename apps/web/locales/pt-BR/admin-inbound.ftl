# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = Acesso de entrada
admin-inbound-help = Restrinja os IPs de origem confiáveis do console de administração e do webmail separadamente. As duas políticas começam desativadas. Estas regras não alteram proxies confiáveis, escopos de API nem o acesso de saída.
admin-inbound-unscoped = Esta política válida para toda a implantação exige um administrador com acesso de segurança sem escopo.
admin-inbound-admin = Console de administração e API
admin-inbound-admin-help = Inclui o login e este editor. A autenticação de administrador existente não contorna a política de rede.
admin-inbound-webmail = Webmail e APIs de conta
admin-inbound-webmail-help = Abrange e-mail, DAV, conteúdo assinado, SSO, OIDC e rotas de API. As conexões já atualizadas mantêm a decisão tomada no handshake.
admin-inbound-enabled = Restringir esta superfície à lista de permissões
admin-inbound-cidrs = IPs e CIDRs de resgate
admin-inbound-cidrs-help = Um IP ou CIDR literal por linha, até 128. Pelo menos um é obrigatório quando ativado. Inclua uma origem confiável que você possa usar para recuperação.
admin-inbound-hostnames = Nomes de host DNS permitidos
admin-inbound-hostnames-help = Um nome DNS ASCII em minúsculas por linha. Use IDNA ASCII para nomes internacionais. Sem URLs, portas, curingas, IPs literais nem pontos finais. Os nomes concedem acesso apenas aos endereços resolvidos no momento.
admin-inbound-dns = Limites de resolução DNS
admin-inbound-dns-help = Nomes novos, expirados e com falha não concedem acesso. As respostas bem-sucedidas respeitam o TTL de DNS e o limite de cache configurado. As solicitações nunca esperam pela resolução DNS.
admin-inbound-servers = Servidores DNS recursivos
admin-inbound-servers-help = Deixe vazio para usar os resolvedores do sistema ou informe até quatro sockets IP:porta, um por linha (por exemplo 10.0.0.53:53 ou [fd00::53]:53). Servidores explícitos substituem os resolvedores do sistema sem fallback público. A confiança no resolvedor é de sua responsabilidade; não se afirma validação DNSSEC independente.
admin-inbound-dns-timeoutMs = Tempo limite de consulta (milissegundos)
admin-inbound-dns-cacheSeconds = Duração máxima do cache (segundos)
admin-inbound-dns-maxHosts = Máximo de nomes de host distintos
admin-inbound-dns-maxAddresses = Máximo de endereços por nome de host
admin-inbound-dns-concurrency = Consultas simultâneas
admin-inbound-bounds = Intervalo permitido: { $min }–{ $max }.
admin-inbound-lockout = Esta política pode bloquear imediatamente o seu próprio acesso, inclusive a este editor. Confirme que os IPs de resgate cobrem a sua origem real, conforme determinada pelos proxies confiáveis configurados. Não há bypass automático para loopback nem para o administrador com login. A recuperação de um bloqueio exige restauração do lado da implantação; o login comum não consegue contorná-lo.
admin-inbound-confirm-title = Aplicar a política de acesso de entrada?
admin-inbound-confirm-word = APLICAR
admin-inbound-save = Aplicar a política confirmada
admin-inbound-review = Revisar alterações
admin-inbound-refresh = Atualizar a política salva e o status do DNS
admin-inbound-reset = Redefinir o rascunho para a política carregada
admin-inbound-saved = O servidor aceitou a política de acesso de entrada.
admin-inbound-conflict = A política salva mudou enquanto você a editava. Redefina o rascunho e atualize para carregar a revisão atual antes de editar novamente.
admin-inbound-no-revision = O servidor não retornou uma revisão da política. Atualize antes de salvar.
admin-inbound-status = Concessões de DNS atuais
admin-inbound-status-help = É um instantâneo da última atualização de status, não uma contagem regressiva nem um teste de conectividade. Atualize para ver as concessões atuais. Um erro pode coexistir com uma resposta anterior ainda válida até a expiração original.
admin-inbound-refreshing = Uma atualização de DNS limitada está em andamento.
admin-inbound-idle = Nenhuma atualização de DNS estava em andamento na última leitura de status.
admin-inbound-no-hosts = Nenhum nome de host está ativo nas políticas ativadas.
admin-inbound-hostname = Nome de host
admin-inbound-state = Concessão de acesso
admin-inbound-addresses = Endereços utilizáveis
admin-inbound-validity = Validade na última atualização
admin-inbound-error = Código de resultado do DNS
admin-inbound-usable = Utilizável
admin-inbound-unavailable = Nenhuma concessão atual
admin-inbound-seconds = { $seconds } segundos
admin-inbound-on = Ativada
admin-inbound-off = Desativada
admin-inbound-invalid-dns = Os limites de DNS ou o número de resolvedores estão fora dos limites permitidos.
admin-inbound-rescue-required = Cada superfície ativada exige pelo menos um IP ou CIDR de resgate.
admin-inbound-too-many = A política excede o limite configurado de nomes de host ou endereços literais.
admin-inbound-invalid-host = Use nomes DNS ASCII em minúsculas sem URLs, portas, curingas, IPs literais nem pontos finais.
