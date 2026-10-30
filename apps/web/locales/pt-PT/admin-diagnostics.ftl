# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl (t105).

## Separadores
admin-diagx-tabs = Páginas de diagnóstico
admin-diagx-tab-overview = Visão geral
admin-diagx-tab-server = Servidor
admin-diagx-tab-connectivity = Conectividade
admin-diagx-tab-backends = Back-ends de correio
admin-diagx-tab-posture = Postura de segurança
admin-diagx-tab-tools = Ferramentas
admin-diagx-tab-history = Histórico
admin-diagx-tab-report = Relatório
admin-diagx-tab-server-intro = Verificações das partes do próprio servidor: a base de dados, as suas migrações e a chave do servidor. São executadas com as verificações do servidor.
admin-diagx-tab-connectivity-intro = Se este servidor está acessível pelo seu nome público, se consegue chegar aos back-ends guardados e se mantém os endereços privados fora de alcance. As ferramentas de domínio abaixo consultam registos DNS, testam o TLS e descobrem definições para um domínio.
admin-diagx-tab-backends-intro = Uma ligação e um handshake de segurança com cada endpoint de correio, submissão, regras e DAV guardado. Não são enviadas credenciais nem é entregue correio.
admin-diagx-tab-posture-intro = Definições que protegem os inícios de sessão e as sessões, e como o fail2ban veria o endereço de um pedido.
admin-diagx-tab-tools-intro = Outros locais para verificar o serviço, e os limites que se aplicam às execuções de diagnóstico.
admin-diagx-tab-history-intro = Execuções guardadas, da mais recente para a mais antiga. Abra uma execução para ver os resultados e o que mudou desde a anterior.

## Estados
admin-diagx-status-ok = OK
admin-diagx-status-warn = Aviso
admin-diagx-status-fail = Falhou
admin-diagx-status-skipped = Ignorada
admin-diagx-status-not-run = Não executada
admin-diagx-status-running = Em execução

## Cartões
admin-diagx-run = Executar
admin-diagx-run-again = Executar de novo
admin-diagx-running = Em execução…
admin-diagx-run-server-note = Executa de novo as verificações do servidor. Todos os cartões dessa execução são atualizados.
admin-diagx-run-backends-note = Verifica de novo todos os back-ends guardados. Todos os cartões de back-end são atualizados.
admin-diagx-how-to-fix = Como corrigir
admin-diagx-details = Detalhes
admin-diagx-copy-details = Copiar detalhes
admin-diagx-copy-value = Copiar { $label }
admin-diagx-copied = Copiado.
admin-diagx-copy-failed = O navegador não permitiu copiar. Selecione o texto e copie-o manualmente.
admin-diagx-not-run-yet = Ainda não executada.
admin-diagx-not-reported = A última execução do servidor não indicou esta verificação; não se aplica a esta implementação.
admin-diagx-posture-met = Cumprido
admin-diagx-posture-not-met = Não cumprido
admin-diagx-loading = A carregar…
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

## Visão geral
admin-diagx-overview-intro = O resultado mais recente de cada verificação, por grupo. Executar tudo corre as verificações do servidor e depois verifica cada back-end guardado.
admin-diagx-run-all = Executar todas as verificações
admin-diagx-run-all-step = Passo { $step } de { $steps }: { $suite ->
    [backends] back-ends guardados
   *[server] verificações do servidor
}.
admin-diagx-run-all-done = Todas as verificações terminaram. Os resultados ficam guardados no Histórico.
admin-diagx-backends-progress = { $completed } de { $total } back-ends verificados.
admin-diagx-counts = { $ok } OK · { $warn } avisos · { $fail } falhadas · { $notRun } não executadas
admin-diagx-attention = Requer atenção
admin-diagx-attention-none = Nenhuma verificação indicou um aviso ou uma falha na execução mais recente.
admin-diagx-never-run = Ainda não foi executada nenhuma verificação. Execute todas as verificações para obter um primeiro resultado.
admin-diagx-last-server = Últimas verificações do servidor
admin-diagx-last-backends = Últimas verificações de back-ends
admin-diagx-never = Nunca
admin-diagx-open-health = Estado do serviço

## Verificações do servidor
admin-diagx-check-database-connectivity = Ligação à base de dados
admin-diagx-check-database-connectivity-purpose = A base de dados responde a uma consulta simples.
admin-diagx-check-database-connectivity-fix = Verifique se a base de dados está em execução e se MW_DB_PATH ou MW_DATABASE_URL apontam para ela. O registo do servidor indica o erro de ligação.
admin-diagx-check-database-write = Escritas na base de dados
admin-diagx-check-database-write-purpose = É possível escrever uma linha temporária, que depois é revertida.
admin-diagx-check-database-write-fix = Verifique o espaço livre em disco e se o ficheiro ou o utilizador da base de dados pode escrever. Um volume só de leitura ou um disco cheio impedem as escritas.
admin-diagx-check-database-migrations = Migrações da base de dados
admin-diagx-check-database-migrations-purpose = A base de dados tem todas as migrações que esta versão do servidor espera.
admin-diagx-check-database-migrations-fix = Reinicie o servidor para aplicar as migrações pendentes e procure no registo uma migração que tenha falhado. Não execute um servidor mais antigo sobre uma base de dados mais recente.
admin-diagx-check-server-key-roundtrip = Chave do servidor
admin-diagx-check-server-key-roundtrip-purpose = A chave do servidor sela e abre um valor de teste.
admin-diagx-check-server-key-roundtrip-fix = Verifique se MW_SERVER_KEY (ou o ficheiro da chave) é a chave com que esta base de dados foi criada. Os segredos guardados não abrem com outra chave.
admin-diagx-check-other-purpose = Uma verificação indicada por este servidor.
admin-diagx-check-other-fix = Abra os Detalhes para ver o resultado em bruto e consulte o registo do servidor à hora da execução.

## Verificações de conectividade
admin-diagx-check-network-public-dns = Nome público
admin-diagx-check-network-public-dns-purpose = O nome de anfitrião em MW_PUBLIC_URL é resolvido a partir deste servidor.
admin-diagx-check-network-public-dns-fix = Verifique o registo DNS do nome público, ou defina MW_PUBLIC_URL com o endereço que as pessoas usam para chegar a este servidor.
admin-diagx-check-network-public-port = Porta pública
admin-diagx-check-network-public-port-purpose = A porta do URL público aceita uma ligação a partir deste servidor.
admin-diagx-check-network-public-port-fix = Verifique a firewall, o proxy inverso e o reencaminhamento de portas. Alguns routers não conseguem ligar ao seu próprio endereço público (NAT hairpinning); teste também a partir de fora da sua rede.
admin-diagx-check-network-backend-ports = Portas dos back-ends
admin-diagx-check-network-backend-ports-purpose = O anfitrião e a porta de cada back-end guardado aceitam uma ligação direta.
admin-diagx-check-network-backend-ports-fix = Verifique se os anfitriões dos back-ends estão ativos e se uma firewall permite que este servidor chegue até eles. Back-ends de correio mostra qual o endpoint que falha o handshake.
admin-diagx-check-egress-private-policy = Proteção de endereços privados
admin-diagx-check-egress-private-policy-purpose = Por predefinição, as ligações de saída para endereços de loopback e privados são recusadas.
admin-diagx-check-egress-private-policy-fix = Reveja as definições de saída em Privacidade. Permita endereços privados apenas para os back-ends identificados que precisem deles.

## Verificações de postura de segurança
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Os pedidos chegam a este servidor por HTTPS.
admin-diagx-check-security-https-fix = Sirva a consola por HTTPS, ou defina MW_PUBLIC_URL com https e MW_TRUSTED_PROXIES quando um proxy inverso termina o TLS.
admin-diagx-check-security-secure-cookies = Cookies seguros
admin-diagx-check-security-secure-cookies-purpose = Os cookies de sessão só são enviados por HTTPS.
admin-diagx-check-security-secure-cookies-fix = Defina MW_COOKIE_SECURE como true quando o servidor for acedido por HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Os navegadores são instruídos a usar HTTPS neste site.
admin-diagx-check-security-hsts-fix = Ative Strict-Transport-Security nas definições de segurança, ou envie-o a partir do proxy inverso.
admin-diagx-check-security-csrf = Proteção CSRF
admin-diagx-check-security-csrf-purpose = Os pedidos que alteram dados têm de trazer o token CSRF.
admin-diagx-check-security-csrf-fix = Defina MW_CSRF_STRICT como true.
admin-diagx-check-security-admin-factors = Segundo fator dos administradores
admin-diagx-check-security-admin-factors-purpose = Todos os administradores têm um segundo fator de início de sessão.
admin-diagx-check-security-admin-factors-fix = Peça a cada administrador sem segundo fator que adicione uma chave de acesso ou uma aplicação de autenticação em As minhas preferências, ou torne-o obrigatório no Centro de segurança.
admin-diagx-check-security-signin-pacing = Espera após falha de início de sessão
admin-diagx-check-security-signin-pacing-purpose = Uma falha de início de sessão é seguida de uma breve espera.
admin-diagx-check-security-signin-pacing-fix = Ative o ritmo de início de sessão na política de segurança.
admin-diagx-check-security-credential-locks = Bloqueio de credenciais
admin-diagx-check-security-credential-locks-purpose = Falhas repetidas de início de sessão bloqueiam a credencial durante algum tempo.
admin-diagx-check-security-credential-locks-fix = Ative os bloqueios automáticos no Centro de segurança.
admin-diagx-check-security-geography = Dados de geografia dos inícios de sessão
admin-diagx-check-security-geography-purpose = A política de geografia dos inícios de sessão tem os dados de países de que precisa.
admin-diagx-check-security-geography-fix = Instale ou atualize a base de dados de países, ou desative a política de geografia.
admin-diagx-check-security-render-isolation = Apresentação isolada de mensagens
admin-diagx-check-security-render-isolation-purpose = As mensagens são apresentadas no processo separado e isolado.
admin-diagx-check-security-render-isolation-fix = Verifique se MW_RENDER_BIN aponta para o programa mw-render e se este arranca. O estado do serviço no Painel mostra o estado do processo.
admin-diagx-check-security-deployment = Definições da implementação
admin-diagx-check-security-deployment-purpose = É possível ler as definições de segurança da implementação.
admin-diagx-check-security-deployment-fix = Verifique a ligação à base de dados e depois abra a avaliação de segurança da implementação.
admin-diagx-check-security-plaintext-backends = Palavras-passe em ligações não cifradas
admin-diagx-check-security-plaintext-backends-purpose = Nenhum back-end guardado aceita palavras-passe sem TLS.
admin-diagx-check-security-plaintext-backends-fix = Edite as ligações do domínio e escolha TLS ou STARTTLS. Permita ligações simples apenas numa rede privada que controle.
admin-diagx-check-fail2ban-client-address = Endereço do cliente para o fail2ban
admin-diagx-check-fail2ban-client-address-purpose = O endereço que uma linha de registo do fail2ban indicaria é o endereço real do cliente.
admin-diagx-check-fail2ban-client-address-fix = Atrás de um proxy inverso, defina MW_TRUSTED_PROXIES com o endereço do proxy; caso contrário, o fail2ban bloqueia o proxy em vez do cliente. Um endereço de loopback é esperado quando abre a consola no próprio servidor.

## Back-ends de correio
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = Endpoint { $protocol }: ligação e handshake de segurança, sem iniciar sessão.
admin-diagx-backend-other-purpose = Uma nota da execução de back-ends.
admin-diagx-backend-fix = Verifique o nome de anfitrião, a porta e o modo TLS guardados para este back-end, se o DNS resolve e se o certificado é válido para o nome de anfitrião. Depois execute de novo.
admin-diagx-backend-config-fix = Abra as ligações do domínio e guarde-as de novo; não foi possível ler a lista de back-ends guardada.
admin-diagx-backend-results = Verificações de back-ends
admin-diagx-backends-not-run = Os back-ends guardados ainda não foram verificados.
admin-diagx-backends-run-at = Última verificação a { $date }: { $count } resultados.
admin-diagx-failover-title = Comutação por falha
admin-diagx-failover-purpose = Se um domínio pode mudar para outro back-end de correio quando a primeira escolha não está acessível.
admin-diagx-failover-state = Comutação por falha { $state ->
    [on] ativa
   *[off] inativa
} · { $backends } back-ends de correio{ $enabled ->
    [no] {" "}· domínio desativado
   *[yes] {""}
}
admin-diagx-no-domains = Não existem domínios guardados.

## Ferramentas de conectividade
admin-diagx-domain-tools = Ferramentas de domínio
admin-diagx-domain-tools-purpose = Verificações pontuais para um domínio guardado. Não iniciam sessão nem enviam correio.
admin-diagx-domain = Domínio
admin-diagx-tool-dns = Registos DNS
admin-diagx-tool-dns-purpose = Consulta os registos MX, SPF, DMARC e de serviço (SRV) do domínio.
admin-diagx-tool-dns-fix = Adicione os registos em falta no seu fornecedor de DNS. Sem MX o domínio não recebe correio; sem SPF e DMARC outros servidores podem rejeitar o seu correio.
admin-diagx-srv-found = { $found } de { $total } registos de serviço encontrados
admin-diagx-tool-tls = TLS e handshake
admin-diagx-tool-tls-purpose = Liga a cada endpoint guardado do domínio e indica a versão TLS, a validade do certificado e o tempo gasto.
admin-diagx-tool-tls-fix = Para um endpoint inacessível, verifique o anfitrião, a porta e a firewall. Renove um certificado que expire nos próximos 14 dias.
admin-diagx-probe-none = O domínio não tem endpoints guardados para testar.
admin-diagx-probe-line = { $host } · acessível: { $reachable ->
    [yes] sim
   *[no] não
} · TLS { $tls } · certificado { $days ->
    [none] não indicado
   *[other] expira dentro de { $days } dias
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = Descobrir definições
admin-diagx-tool-detect-purpose = Consulta as definições de correio publicadas (autoconfig, autodiscover, SRV) de um nome de domínio. Nada é guardado.
admin-diagx-tool-detect-fix = Publique registos autoconfig ou SRV para o domínio, ou introduza as definições manualmente em Domínios.
admin-diagx-detect-name = Nome de domínio a consultar

## Ferramentas
admin-diagx-tool-health = Estado do serviço
admin-diagx-tool-health-purpose = Sincronização de correio, caixa de saída, índice de pesquisa, processo de apresentação e plugins, no Painel.
admin-diagx-tool-deployment-purpose = Todas as definições de segurança da implementação que este servidor consegue observar, com a definição que as controla.
admin-diagx-tool-ssh = Testes de túneis SSH
admin-diagx-tool-ssh-purpose = Teste os túneis SSH usados para chegar a back-ends noutra rede.
admin-diagx-tool-geo = Geografia dos inícios de sessão
admin-diagx-tool-geo-purpose = De onde vêm os inícios de sessão, e a base de dados de países por trás da política de geografia.
admin-diagx-catalogue-purpose = O que cada conjunto de diagnóstico cobre, e os limites que cada execução respeita.
admin-diagx-suite-unsupported = Não disponível: { $reason }
admin-diagx-suite-server = Verificações do servidor
admin-diagx-suite-backends = Back-ends guardados

## Histórico
admin-diagx-run-title = { $suite ->
    [backends] Back-ends guardados
   *[server] Verificações do servidor
}, { $date }
admin-diagx-changes = Alterações
admin-diagx-changes-first = Não existe uma execução anterior deste tipo para comparar.
admin-diagx-changes-none = Nada mudou desde a execução de { $date }.
admin-diagx-changes-since = Alterado desde a execução de { $date }:
admin-diagx-change = { $before } → { $after }

## Relatório
admin-diagx-report-title = Relatório de resolução de problemas
admin-diagx-report-purpose = Um relatório JSON para o suporte: indicadores de execução, os resultados mais recentes e a execução que escolher. Palavras-passe, tokens, cookies, chaves, configuração em bruto e correio nunca são incluídos.
admin-diagx-report-run = Execução a incluir
admin-diagx-report-preview = Pré-visualizar

## Palavras do trilho de navegação para os separadores
admin-crumb-server = Servidor
admin-crumb-connectivity = Conectividade
admin-crumb-posture = Postura de segurança
admin-crumb-tools = Ferramentas
admin-crumb-history = Histórico
admin-crumb-report = Relatório
admin-diagx-table-actions = Ações
