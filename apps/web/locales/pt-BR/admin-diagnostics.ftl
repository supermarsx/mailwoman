# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = Páginas de diagnóstico
admin-diagx-tab-overview = Visão geral
admin-diagx-tab-server = Servidor
admin-diagx-tab-connectivity = Conectividade
admin-diagx-tab-backends = Back-ends de e-mail
admin-diagx-tab-posture = Postura de segurança
admin-diagx-tab-tools = Ferramentas
admin-diagx-tab-history = Histórico
admin-diagx-tab-report = Relatório
admin-diagx-tab-server-intro = Verificações das partes do próprio servidor: o banco de dados, suas migrações e a chave do servidor. Elas são executadas junto com as verificações do servidor.
admin-diagx-tab-connectivity-intro = Se este servidor é acessível pelo seu nome público, consegue alcançar os back-ends salvos e mantém endereços privados fora de alcance. As ferramentas de domínio abaixo consultam registros DNS, testam o TLS e descobrem configurações de um domínio.
admin-diagx-tab-backends-intro = Uma conexão e um handshake de segurança com cada endpoint salvo de e-mail, envio, regras e DAV. Nenhuma credencial é enviada e nenhum e-mail é entregue.
admin-diagx-tab-posture-intro = Configurações que protegem logins e sessões, e como o fail2ban veria o endereço de uma solicitação.
admin-diagx-tab-tools-intro = Outros lugares para verificar o serviço e os limites que se aplicam às execuções de diagnóstico.
admin-diagx-tab-history-intro = Execuções salvas, da mais recente para a mais antiga. Abra uma execução para ver seus resultados e o que mudou desde a execução anterior.
admin-diagx-status-ok = OK
admin-diagx-status-warn = Aviso
admin-diagx-status-fail = Falhou
admin-diagx-status-skipped = Ignorado
admin-diagx-status-not-run = Não executado
admin-diagx-status-running = Em execução
admin-diagx-run = Executar
admin-diagx-run-again = Executar de novo
admin-diagx-running = Executando…
admin-diagx-run-server-note = Executa as verificações do servidor de novo. Todos os cartões dessa execução são atualizados.
admin-diagx-run-backends-note = Verifica de novo todos os back-ends salvos. Todos os cartões de back-end são atualizados.
admin-diagx-how-to-fix = Como corrigir
admin-diagx-details = Detalhes
admin-diagx-copy-details = Copiar detalhes
admin-diagx-copy-value = Copiar { $label }
admin-diagx-copied = Copiado.
admin-diagx-copy-failed = O navegador não permitiu a cópia. Selecione o texto e copie-o manualmente.
admin-diagx-not-run-yet = Ainda não executado.
admin-diagx-not-reported = A última execução do servidor não informou esta verificação; ela não se aplica a esta implantação.
admin-diagx-posture-met = Atendido
admin-diagx-posture-not-met = Não atendido
admin-diagx-loading = Carregando…
admin-diagx-none = Nenhum
admin-diagx-yes = Sim
admin-diagx-no = Não
admin-diagx-open = Abrir
admin-diagx-row-result = Resultado
admin-diagx-row-duration = Duração
admin-diagx-row-checked = Verificado
admin-diagx-row-id = ID da verificação
admin-diagx-row-domain = Domínio
admin-diagx-row-backend = Back-end
admin-diagx-row-protocol = Protocolo
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Capacidades
admin-diagx-row-credentials = Credenciais enviadas
admin-diagx-row-source = Encontrado em
admin-diagx-row-oauth = Requer OAuth
admin-diagx-overview-intro = O resultado mais recente de cada verificação, por grupo. Executar tudo executa as verificações do servidor e depois verifica cada back-end salvo.
admin-diagx-run-all = Executar todas as verificações
admin-diagx-run-all-step = Etapa { $step } de { $steps }: { $suite ->
    [backends] back-ends salvos
   *[server] verificações do servidor
}.
admin-diagx-run-all-done = Todas as verificações terminaram. Os resultados estão salvos em Histórico.
admin-diagx-backends-progress = { $completed } de { $total } back-ends verificados.
admin-diagx-counts = { $ok } OK · { $warn } avisos · { $fail } falhas · { $notRun } não executadas
admin-diagx-attention = Requer atenção
admin-diagx-attention-none = Nenhuma verificação relatou aviso ou falha na sua última execução.
admin-diagx-never-run = Nenhuma verificação foi executada ainda. Execute todas as verificações para obter um primeiro resultado.
admin-diagx-last-server = Últimas verificações do servidor
admin-diagx-last-backends = Últimas verificações de back-end
admin-diagx-never = Nunca
admin-diagx-open-health = Integridade do serviço
admin-diagx-check-database-connectivity = Conexão com o banco de dados
admin-diagx-check-database-connectivity-purpose = O banco de dados responde a uma consulta simples.
admin-diagx-check-database-connectivity-fix = Verifique se o banco de dados está em execução e se MW_DB_PATH ou MW_DATABASE_URL aponta para ele. O log do servidor indica o erro de conexão.
admin-diagx-check-database-write = Gravações no banco de dados
admin-diagx-check-database-write-purpose = Uma linha de teste é gravada e depois revertida.
admin-diagx-check-database-write-fix = Verifique o espaço livre em disco e se o arquivo ou o usuário do banco de dados pode gravar. Um volume somente leitura ou um disco cheio impede as gravações.
admin-diagx-check-database-migrations = Migrações do banco de dados
admin-diagx-check-database-migrations-purpose = O banco de dados tem todas as migrações que esta versão do servidor espera.
admin-diagx-check-database-migrations-fix = Reinicie o servidor para que ele aplique as migrações pendentes e verifique no log se alguma migração falhou. Não execute um servidor mais antigo em um banco de dados mais novo.
admin-diagx-check-server-key-roundtrip = Chave do servidor
admin-diagx-check-server-key-roundtrip-purpose = A chave do servidor sela e abre um valor de teste.
admin-diagx-check-server-key-roundtrip-fix = Verifique se MW_SERVER_KEY (ou o arquivo de chave) é a chave com a qual este banco de dados foi criado. Os segredos armazenados não podem ser abertos com outra chave.
admin-diagx-check-other-purpose = Uma verificação relatada por este servidor.
admin-diagx-check-other-fix = Abra Detalhes para ver o resultado bruto e verifique o log do servidor no momento da execução.
admin-diagx-check-network-public-dns = Nome público
admin-diagx-check-network-public-dns-purpose = O nome de host em MW_PUBLIC_URL é resolvido a partir deste servidor.
admin-diagx-check-network-public-dns-fix = Verifique o registro DNS do nome público ou defina MW_PUBLIC_URL com o endereço que as pessoas usam para acessar este servidor.
admin-diagx-check-network-public-port = Porta pública
admin-diagx-check-network-public-port-purpose = A porta da URL pública aceita uma conexão deste servidor.
admin-diagx-check-network-public-port-fix = Verifique o firewall, o proxy reverso e o redirecionamento de portas. Alguns roteadores não conseguem se conectar ao próprio endereço público (NAT hairpinning); teste também de fora da sua rede.
admin-diagx-check-network-backend-ports = Portas dos back-ends
admin-diagx-check-network-backend-ports-purpose = O host e a porta de cada back-end salvo aceitam uma conexão direta.
admin-diagx-check-network-backend-ports-fix = Verifique se os hosts dos back-ends estão ativos e se um firewall permite que este servidor os alcance. Back-ends de e-mail mostra qual endpoint falha no handshake.
admin-diagx-check-egress-private-policy = Proteção de endereços privados
admin-diagx-check-egress-private-policy-purpose = Por padrão, as conexões de saída para endereços de loopback e privados são recusadas.
admin-diagx-check-egress-private-policy-fix = Revise as configurações de saída em Privacidade. Permita endereços privados apenas para back-ends nomeados que precisem deles.
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = As solicitações chegam a este servidor por HTTPS.
admin-diagx-check-security-https-fix = Sirva o console por HTTPS, ou defina MW_PUBLIC_URL como https e MW_TRUSTED_PROXIES quando um proxy reverso encerra o TLS.
admin-diagx-check-security-secure-cookies = Cookies seguros
admin-diagx-check-security-secure-cookies-purpose = Os cookies de sessão só são enviados por HTTPS.
admin-diagx-check-security-secure-cookies-fix = Defina MW_COOKIE_SECURE como true quando o servidor for acessado por HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Os navegadores são instruídos a usar HTTPS para este site.
admin-diagx-check-security-hsts-fix = Ative o Strict-Transport-Security nas configurações de segurança ou envie-o pelo proxy reverso.
admin-diagx-check-security-csrf = Proteção CSRF
admin-diagx-check-security-csrf-purpose = As solicitações que alteram dados precisam conter o token CSRF.
admin-diagx-check-security-csrf-fix = Defina MW_CSRF_STRICT como true.
admin-diagx-check-security-admin-factors = Segundo fator do administrador
admin-diagx-check-security-admin-factors-purpose = Todo administrador tem um segundo fator de login.
admin-diagx-check-security-admin-factors-fix = Peça a cada administrador sem um que adicione uma passkey ou um app autenticador em Minhas preferências, ou exija isso na Central de segurança.
admin-diagx-check-security-signin-pacing = Atraso entre tentativas de login
admin-diagx-check-security-signin-pacing-purpose = Uma espera curta se segue a um login com falha.
admin-diagx-check-security-signin-pacing-fix = Ative o ritmo de login na política de segurança.
admin-diagx-check-security-credential-locks = Bloqueios de credencial
admin-diagx-check-security-credential-locks-purpose = Logins com falha repetidos bloqueiam a credencial por um tempo.
admin-diagx-check-security-credential-locks-fix = Ative os bloqueios automáticos na Central de segurança.
admin-diagx-check-security-geography = Dados de geografia de login
admin-diagx-check-security-geography-purpose = A política de geografia de login tem os dados de países de que precisa.
admin-diagx-check-security-geography-fix = Instale ou atualize o banco de dados de países, ou desative a política de geografia.
admin-diagx-check-security-render-isolation = Renderização isolada de mensagens
admin-diagx-check-security-render-isolation-purpose = As mensagens são renderizadas no worker separado e isolado.
admin-diagx-check-security-render-isolation-fix = Verifique se MW_RENDER_BIN aponta para o programa mw-render e se ele inicia. A integridade do serviço no Painel mostra o estado do worker.
admin-diagx-check-security-deployment = Configurações da implantação
admin-diagx-check-security-deployment-purpose = As configurações de segurança da implantação podem ser lidas.
admin-diagx-check-security-deployment-fix = Verifique a conexão com o banco de dados e abra a avaliação de segurança da implantação.
admin-diagx-check-security-plaintext-backends = Senhas em conexões não criptografadas
admin-diagx-check-security-plaintext-backends-purpose = Nenhum back-end salvo aceita senhas sem TLS.
admin-diagx-check-security-plaintext-backends-fix = Edite as conexões do domínio e escolha TLS ou STARTTLS. Permita conexões sem criptografia apenas em uma rede privada que você controla.
admin-diagx-check-fail2ban-client-address = Endereço do cliente no fail2ban
admin-diagx-check-fail2ban-client-address-purpose = O endereço que uma linha de log do fail2ban nomearia é o endereço real do cliente.
admin-diagx-check-fail2ban-client-address-fix = Atrás de um proxy reverso, defina MW_TRUSTED_PROXIES com o endereço do proxy, ou o fail2ban bane o proxy em vez do cliente. Um endereço de loopback é esperado quando você abre o console no próprio servidor.
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Endpoint { $protocol }: conexão e handshake de segurança, sem fazer login.
admin-diagx-backend-other-purpose = Uma observação da execução dos back-ends.
admin-diagx-backend-fix = Verifique o nome de host, a porta e o modo TLS salvos para este back-end, se o DNS é resolvido e se o certificado é válido para o nome de host. Depois execute de novo.
admin-diagx-backend-config-fix = Abra as conexões do domínio e salve-as de novo; a lista de back-ends salvos não pôde ser lida.
admin-diagx-backend-results = Verificações de back-end
admin-diagx-backends-not-run = Os back-ends salvos ainda não foram verificados.
admin-diagx-backends-run-at = Última verificação em { $date }: { $count } resultados.
admin-diagx-failover-title = Failover
admin-diagx-failover-purpose = Se um domínio pode passar para outro back-end de e-mail quando a primeira escolha não pode ser alcançada.
admin-diagx-failover-state = Failover { $state ->
    [on] ativado
   *[off] desativado
} · { $backends } back-ends de e-mail{ $enabled ->
    [no] {" "}· domínio desativado
   *[yes] {""}
 }
admin-diagx-no-domains = Nenhum domínio salvo.
admin-diagx-domain-tools = Ferramentas de domínio
admin-diagx-domain-tools-purpose = Verificações pontuais para um domínio salvo. Elas não fazem login nem enviam e-mail.
admin-diagx-domain = Domínio
admin-diagx-tool-dns = Registros DNS
admin-diagx-tool-dns-purpose = Consulta os registros MX, SPF, DMARC e de serviço (SRV) do domínio.
admin-diagx-tool-dns-fix = Adicione os registros ausentes no seu provedor de DNS. Sem MX o domínio não recebe e-mail; sem SPF e DMARC, outros servidores podem rejeitar o e-mail dele.
admin-diagx-srv-found = { $found } de { $total } registros de serviço encontrados
admin-diagx-tool-tls = TLS e handshake
admin-diagx-tool-tls-purpose = Conecta-se a cada endpoint salvo do domínio e informa a versão do TLS, a validade do certificado e o tempo gasto.
admin-diagx-tool-tls-fix = Para um endpoint inacessível, verifique host, porta e firewall. Renove um certificado que expira em até 14 dias.
admin-diagx-probe-none = O domínio não tem endpoints salvos para testar.
admin-diagx-probe-line = { $host } · acessível: { $reachable ->
    [yes] sim
   *[no] não
} · TLS { $tls } · certificado { $days ->
    [none] não informado
   *[other] expira em { $days } dias
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
 }
admin-diagx-tool-detect = Descobrir configurações
admin-diagx-tool-detect-purpose = Consulta as configurações de e-mail publicadas (autoconfig, autodiscover, SRV) para um nome de domínio. Nada é salvo.
admin-diagx-tool-detect-fix = Publique registros autoconfig ou SRV para o domínio, ou informe as configurações manualmente em Domínios.
admin-diagx-detect-name = Nome de domínio a consultar
admin-diagx-tool-health = Integridade do serviço
admin-diagx-tool-health-purpose = Sincronização de e-mail, a caixa de saída, o índice de busca, o worker de renderização e os plugins, no Painel.
admin-diagx-tool-deployment-purpose = Todas as configurações de segurança da implantação que este servidor consegue observar, com a configuração que as controla.
admin-diagx-tool-ssh = Testes de túnel SSH
admin-diagx-tool-ssh-purpose = Testa os túneis SSH usados para alcançar back-ends em outra rede.
admin-diagx-tool-geo = Geografia de login
admin-diagx-tool-geo-purpose = De onde vêm os logins e o banco de dados de países por trás da política de geografia.
admin-diagx-catalogue-purpose = O que cada conjunto de diagnóstico abrange e os limites que cada execução respeita.
admin-diagx-suite-unsupported = Indisponível: { $reason }
admin-diagx-suite-server = Verificações do servidor
admin-diagx-suite-backends = Back-ends salvos
admin-diagx-run-title = { $suite ->
    [backends] Back-ends salvos
   *[server] Verificações do servidor
}, { $date }
admin-diagx-changes = Mudanças
admin-diagx-changes-first = Nenhuma execução anterior deste tipo para comparar.
admin-diagx-changes-none = Nada mudou desde a execução de { $date }.
admin-diagx-changes-since = Mudou desde a execução de { $date }:
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = Relatório de solução de problemas
admin-diagx-report-purpose = Um relatório JSON para o suporte: flags de runtime, os resultados mais recentes e a execução que você escolher. Senhas, tokens, cookies, chaves, configuração bruta e e-mails nunca são incluídos.
admin-diagx-report-run = Execução a incluir
admin-diagx-report-preview = Prévia
admin-crumb-server = Servidor
admin-crumb-connectivity = Conectividade
admin-crumb-posture = Postura de segurança
admin-crumb-tools = Ferramentas
admin-crumb-history = Histórico
admin-crumb-report = Relatório
admin-diagx-table-actions = Ações
admin-diagx-check-pdfjs-viewer-assets = Arquivos do visualizador de PDF
admin-diagx-check-pdfjs-viewer-assets-purpose = Os arquivos do visualizador de PDF estão todos presentes, com o tipo correto e correspondem a esta versão do servidor.
admin-diagx-check-pdfjs-viewer-assets-fix = Reimplante o build web, ou remova a pasta pdfjs de MW_WEB_DIR para usar a cópia incorporada ao servidor. Se isto falhar em vez de apenas avisar, recompile a imagem do servidor.
