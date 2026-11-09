# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = Relatório de status de entrega
dsn-unverified = Este relatório descreve a tentativa de entrega do remetente. O que ele afirma não é verificado de forma independente. Entrega não significa que a mensagem foi lida.
dsn-reporting-mta = Servidor que reporta:
dsn-original-recipient = Destinatário original:
dsn-remote-mta = Servidor remoto:
dsn-last-attempt = Última tentativa:
dsn-retry-until = O remetente planeja tentar novamente até:
dsn-action-failed = Falha reportada
dsn-action-delayed = Atraso reportado
dsn-action-delivered = Entrega reportada
dsn-action-relayed = Repasse reportado
dsn-action-expanded = Expansão de lista reportada

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = Use o diagnóstico do servidor ao pedir ajuda ao seu provedor de e-mail.

dsn-advice-1 = Verifique os endereços do remetente e do destinatário. Confirme o endereço atual do destinatário, se necessário.

dsn-advice-2 = Entre em contato com o destinatário de outra forma se o problema da caixa de correio persistir.

dsn-advice-3 = Compartilhe este relatório com o administrador do sistema de e-mail receptor se o problema persistir.

dsn-advice-4 = Peça ao seu administrador de e-mail para verificar a conectividade, DNS, e as rotas de encaminhamento se o problema persistir.

dsn-advice-5 = Compartilhe o diagnóstico com seu provedor de e-mail para que eles possam verificar a compatibilidade e os limites do SMTP.

dsn-advice-6 = Verifique o formato da mensagem, os endereços internacionais e os anexos de acordo com os requisitos do destinatário.

dsn-advice-7 = Compartilhe o diagnóstico com seu administrador de e-mail para identificar a regra exata de segurança ou autenticação.

dsn-advice-success = Verifique a ação de entrega relatada acima para saber se a mensagem foi entregue, retransmitida ou expandida para uma lista.

dsn-category-0 = O servidor forneceu um código detalhado não reconhecido ou específico do provedor. Seu diagnóstico abaixo pode explicar a causa exata.

dsn-category-1 = Este é um código relacionado ao endereço. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-2 = Este é um código da caixa postal do destinatário. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-3 = Este é um código do sistema de recebimento de correio. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-4 = Este é um código de rede ou de roteamento. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-5 = Este é um código de protocolo de correio. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-6 = Este é um código de conteúdo ou formato de mensagem. Os detalhes precisos são específicos do fornecedor ou não são reconhecidos.

dsn-category-7 = Este é um código de segurança ou política. O detalhe exato é específico do fornecedor ou não é reconhecido; por si só, não estabelece que a mensagem seja spam.

dsn-class-2 = Sucesso: o servidor reporta uma etapa de entrega positiva. Isso não prova que a mensagem foi lida.

dsn-class-4 = Falha temporária: uma tentativa posterior pode ser bem-sucedida. Um relatório de atraso ainda pode ser repetido pelo remetente; um relatório de falha significa que a tentativa terminou.

dsn-class-5 = Falha permanente: é improvável que reenviar a mesma mensagem inalterada funcione. Corrija a causa antes de enviar novamente.

dsn-meaning-title = O que este código significa

dsn-next-step = O que fazer

dsn-provider-barracuda = Barracuda

dsn-provider-detected = Interpretação baseada no diagnóstico do servidor ou no nome do host:

dsn-provider-exchange = Microsoft Exchange/Microsoft 365

dsn-provider-fortimail = FortiMail

dsn-provider-gmail = Gmail/Google Workspace

dsn-provider-help = Documentação do provedor

dsn-provider-mimecast = mimecast

dsn-provider-proofpoint = Ponto de prova

dsn-provider-relay = SMTP relé

dsn-provider-yahoo = Yahoo/AOL

dsn-server-diagnostic = Diagnóstico do servidor

dsn-status-0-0 = O servidor não deu um motivo mais específico.

dsn-status-1-0 = Há um problema com um endereço de e-mail.

dsn-status-1-1 = A caixa de correio do destinatário não existe. Confirme o endereço com o destinatário.

dsn-status-1-10 = O domínio do destinatário declara explicitamente que não aceita e-mail (nulo MX). Use outro endereço.

dsn-status-1-2 = O domínio do destinatário não pode receber este email. Verifique a parte depois de @.

dsn-status-1-3 = O endereço do destinatário tem um formato inválido. Verifique se há erros de digitação ou espaços extras.

dsn-status-1-4 = O servidor não consegue identificar um destinatário exclusivo para este endereço.

dsn-status-1-5 = O endereço do destinatário foi aceito como válido.

dsn-status-1-6 = Esta caixa de correio foi movida e nenhum endereço de encaminhamento está disponível. Peça o novo endereço.

dsn-status-1-7 = O endereço do remetente tem um formato inválido. Verifique o endereço de envio da conta.

dsn-status-1-8 = O domínio do remetente é inválido ou não pode receber mensagens de retorno.

dsn-status-1-9 = A mensagem foi entregue a um sistema que não pode fornecer mais relatórios de entrega.

dsn-status-2-0 = A caixa de correio do destinatário tem um problema não especificado.

dsn-status-2-1 = A caixa de correio do destinatário está desativada ou não aceita mensagens. Entre em contato com o destinatário de outra maneira.

dsn-status-2-2 = A caixa de correio do destinatário está cheia. O destinatário precisa liberar espaço ou aumentar sua cota.

dsn-status-2-3 = A mensagem excede o limite de tamanho deste destinatário. Reduza os anexos ou use um link de compartilhamento.

dsn-status-2-4 = O servidor não conseguiu resolver os membros de uma lista de discussão. Entre em contato com o proprietário da lista.

dsn-status-3-0 = O sistema de recebimento de correio relatou um problema não especificado.

dsn-status-3-1 = O servidor de e-mail de recebimento ficou sem espaço de armazenamento. Seu administrador precisa liberar espaço.

dsn-status-3-2 = O servidor receptor não aceita mensagens, por exemplo, durante manutenção ou sobrecarga.

dsn-status-3-3 = O sistema receptor não suporta um recurso exigido por esta mensagem.

dsn-status-3-4 = A mensagem excede o limite de tamanho do servidor receptor. Reduza os anexos ou use um link de compartilhamento.

dsn-status-3-5 = Um problema de configuração do servidor de correio impede a entrega. Peça ao administrador de e-mail para investigar.

dsn-status-3-6 = O servidor aceitou a mensagem, mas alterou a prioridade de entrega solicitada.

dsn-status-4-0 = Um problema de rede ou de roteamento de correio foi relatado sem maiores detalhes.

dsn-status-4-1 = O servidor receptor não respondeu à tentativa de conexão.

dsn-status-4-2 = A conexão foi estabelecida, mas interrompida antes do término da transferência do correio.

dsn-status-4-3 = Falha na pesquisa de DNS ou diretório. Um administrador de e-mail pode precisar verificar a resolução de nomes.

dsn-status-4-4 = O servidor não conseguiu encontrar uma rota para o sistema de correio do destinatário.

dsn-status-4-5 = O congestionamento do tráfego de correio impediu a entrega.

dsn-status-4-6 = Um loop de encaminhamento foi detectado. Peça ao administrador para verificar as regras de encaminhamento e roteamento.

dsn-status-4-7 = O prazo de entrega expirou. Verifique o diagnóstico do problema subjacente antes de reenviar.

dsn-status-5-0 = Os servidores de correio não conseguiram concluir a troca do protocolo de entrega.

dsn-status-5-1 = Um servidor de e-mail rejeitou um comando não suportado ou um comando enviado na ordem errada.

dsn-status-5-2 = Um servidor de e-mail não conseguia entender a sintaxe de um comando.

dsn-status-5-3 = A mensagem tem muitos destinatários. Envie para grupos menores.

dsn-status-5-4 = Um servidor de email rejeitou os parâmetros de um comando.

dsn-status-5-5 = Os servidores usam versões incompatíveis do protocolo de correio.

dsn-status-5-6 = A resposta de autenticação foi muito longa para o servidor. Peça ao administrador de e-mail para verificar as configurações de autenticação.

dsn-status-6-0 = O servidor relatou um problema não especificado com o conteúdo ou formato da mensagem.

dsn-status-6-1 = O sistema receptor não suporta o tipo de conteúdo desta mensagem.

dsn-status-6-10 = Este código obsoleto significa que uma resposta internacional (UTF-8) é necessária, mas não é suportada, como acontece com o código x.6.8.

dsn-status-6-2 = A entrega requer uma conversão de conteúdo que a política do remetente ou do servidor proíbe.

dsn-status-6-3 = A entrega requer uma conversão de conteúdo que o servidor não pode realizar.

dsn-status-6-4 = Uma conversão de conteúdo perdeu algumas informações. Verifique a mensagem original e o diagnóstico do servidor.

dsn-status-6-5 = O servidor falhou ao converter o conteúdo da mensagem.

dsn-status-6-6 = O servidor não conseguiu recuperar o conteúdo da mensagem de outro sistema.

dsn-status-6-7 = Um endereço de e-mail internacional não é compatível com este remetente ou destinatário. Peça um endereço alternativo.

dsn-status-6-8 = O servidor precisa de uma resposta internacional (UTF-8) que o sistema remetente não pode aceitar.

dsn-status-6-9 = A mensagem contém cabeçalhos internacionais (UTF-8) que não podem ser entregues a pelo menos um destinatário.

dsn-status-7-0 = Uma regra de segurança ou política de correio impediu a entrega normal. Verifique o diagnóstico do servidor para obter a regra exata.

dsn-status-7-1 = O servidor recusou permissão para entrega, possivelmente devido a restrições de filtragem ou retransmissão. Peça ao administrador de e-mail para verificar o diagnóstico.

dsn-status-7-10 = O servidor requer uma conexão criptografada ou proteção mais forte antes da autenticação.

dsn-status-7-11 = Este método de autenticação requer uma conexão criptografada SMTP (TLS).

dsn-status-7-12 = A conta precisa de um método de autenticação ou migração de senha. Entre em contato com o provedor da conta.

dsn-status-7-13 = A conta de envio está desativada. Entre em contato com o administrador para restaurar o acesso.

dsn-status-7-14 = O servidor precisa de uma relação de confiança configurada para recuperar a mensagem. Entre em contato com seu administrador.

dsn-status-7-15 = A prioridade da mensagem está abaixo do mínimo atual do servidor receptor.

dsn-status-7-16 = A mensagem é muito grande para a prioridade solicitada. Reduza seu tamanho ou pergunte ao administrador.

dsn-status-7-17 = A caixa de correio do destinatário mudou de propriedade desde a data exigida. Confirme o destinatário antes de enviar novamente.

dsn-status-7-18 = O domínio do destinatário mudou de propriedade desde a data exigida. Confirme o destino antes de enviar novamente.

dsn-status-7-19 = O servidor não conseguiu verificar a propriedade contínua do endereço do destinatário. Pergunte ao administrador sobre a verificação de validade do destinatário.

dsn-status-7-2 = O remetente não tem permissão para postar nesta lista de discussão. Entre em contato com o proprietário da lista.

dsn-status-7-20 = Nenhuma assinatura DKIM passou na verificação. Peça ao administrador de e-mail do remetente para verificar a assinatura e DNS.

dsn-status-7-21 = Uma assinatura DKIM foi aprovada, mas a política do servidor receptor não a aceitou.

dsn-status-7-22 = Nenhuma assinatura DKIM aceita corresponde ao endereço De. Peça ao administrador de e-mail para verificar o alinhamento do domínio de assinatura.

dsn-status-7-23 = SPF falhou: o servidor remetente não foi autorizado pela política SPF do domínio remetente.

dsn-status-7-24 = O servidor encontrou um erro ao verificar SPF. Peça ao administrador de e-mail para verificar os registros SPF e DNS.

dsn-status-7-25 = O servidor de envio falhou na verificação reversa DNS do receptor. Seu administrador precisa verificar os registros DNS do endereço IP.

dsn-status-7-26 = Várias verificações de autenticação de mensagens falharam. O diagnóstico pode identificar problemas de política SPF, DKIM ou DMARC.

dsn-status-7-27 = O domínio remetente declara que não aceita e-mail de retorno (nulo MX), que o destinatário rejeita.

dsn-status-7-28 = O sistema receptor detectou uma possível enxurrada de mensagens abusivas.

dsn-status-7-29 = A verificação ARC falhou, então o receptor não pôde aceitar a cadeia de autenticação gravada da mensagem.

dsn-status-7-3 = A entrega requer uma conversão de formato de segurança que o servidor não pode executar.

dsn-status-7-30 = A mensagem requer REQUIRETLS, mas o próximo servidor de e-mail não oferece suporte. Peça ao administrador para verificar o suporte de entrega segura.

dsn-status-7-4 = O sistema de entrega não oferece suporte a um recurso de segurança necessário.

dsn-status-7-5 = O servidor não pôde validar ou descriptografar a mensagem, por exemplo, porque uma chave estava faltando ou era inválida.

dsn-status-7-6 = O servidor não suporta um algoritmo criptográfico exigido pela mensagem.

dsn-status-7-7 = Uma verificação de integridade indica que a mensagem foi alterada ou danificada.

dsn-status-7-8 = O servidor de envio rejeitou as credenciais de login. Verifique as configurações de autenticação da conta.

dsn-status-7-9 = O método de autenticação é mais fraco do que o servidor permite. É necessário um método mais forte.

dsn-vendor-advice-account = Peça ao administrador remetente para investigar possíveis abusos na conta, protegê-la e solicitar a remoção da restrição.

dsn-vendor-advice-admin = Compartilhe esse diagnóstico com o administrador de email da organização para revisar a configuração ou política correspondente.

dsn-vendor-advice-auth = Verifique o método de autenticação e autorização SMTP suportado pela conta. Conclua qualquer login necessário na página oficial da conta do provedor.

dsn-vendor-advice-authentication = Peça ao administrador do remetente para verificar SPF, DKIM, DMARC e alinhamento do domínio usando o diagnóstico exato.

dsn-vendor-advice-connector = Peça ao administrador para verificar a autorização de retransmissão, a correspondência do conector IP/certificado, os domínios aceitos e a rota de correio real.

dsn-vendor-advice-content = Revise os logs de diagnóstico e de gateway com o administrador do destinatário para identificar o conteúdo bloqueado e a política aplicável.

dsn-vendor-advice-dns = Peça ao administrador de e-mail remetente ou provedor IP para corrigir DNS, reverter DNS e a identidade do servidor SMTP.

dsn-vendor-advice-domain = Peça ao administrador do Microsoft 365 para registrar e verificar o domínio de envio e revisar os limites de envio do locatário.

dsn-vendor-advice-format = Peça ao aplicativo remetente ou ao administrador de e-mail para corrigir a geração da mensagem e, em seguida, crie uma nova mensagem.

dsn-vendor-advice-rate = Reduza o volume de envio e aguarde a eliminação da cota ou restrição. Verifique a ação relatada antes de assumir que esta tentativa ainda está na fila.

dsn-vendor-advice-recipient-admin = Entre em contato com o destinatário ou proprietário do grupo de outra forma e peça ao administrador para revisar a permissão ou política relevante.

dsn-vendor-advice-reputation = Peça ao administrador de envio para investigar o tráfego e a reputação indesejados e, em seguida, siga a revisão oficial ou o processo de exclusão do provedor.

dsn-vendor-advice-route = Peça ao administrador de e-mail para verificar o destino, as regras de encaminhamento, o serviço de e-mail e o caminho do firewall/rede.

dsn-vendor-advice-tls = Peça aos administradores de correio para repararem TLS, certificados ou registros de segurança DNS para que a rota segura necessária possa ser bem-sucedida.

dsn-vendor-barracuda-reputation = O serviço de reputação da Barracuda lista ou bloqueia o endereço de envio IP.

dsn-vendor-certificate-chain = O certificado TLS remoto não pôde ser validado em relação à confiança necessária ou à política MTA-STS.

dsn-vendor-certificate-expired = O servidor de e-mail de destino apresentou um certificado TLS expirado.

dsn-vendor-certificate-host = O certificado TLS de destino não corresponde ao nome de host do servidor de correio necessário.

dsn-vendor-connection-refused = O servidor de destino recusou a conexão de rede. Seu serviço de e-mail, firewall ou rota de rede precisam ser verificados.

dsn-vendor-content-security = O gateway rejeitou o conteúdo da mensagem ou um anexo de acordo com suas regras de filtragem de segurança. Consulte seu diagnóstico ou registros para o item específico.

dsn-vendor-credentials = O serviço SMTP rejeitou as credenciais de login fornecidas.

dsn-vendor-dane = O certificado TLS ou a configuração TLSA do destino não satisfez a validação DANE.

dsn-vendor-dkim = O receptor não aceitou a configuração de autenticação ou assinatura DKIM da mensagem.

dsn-vendor-dmarc = A política DMARC do destinatário não foi atendida. Verifique a autenticação SPF ou DKIM e o alinhamento com o domínio De visível.

dsn-vendor-dnssec = Os registros DNSSEC do domínio de destino não puderam ser validados.

dsn-vendor-exchange-auth = O aplicativo ou dispositivo não foi autenticado corretamente para envio do Microsoft 365 SMTP.

dsn-vendor-exchange-connector = A Microsoft não pôde autorizar a retransmissão por meio do conector de locatário configurado. Seu certificado, envio IP ou domínio aceito podem não corresponder mais.

dsn-vendor-exchange-domain-blocked = A Microsoft restringiu este domínio de envio após detectar atividades suspeitas de spam.

dsn-vendor-exchange-forward-relay = Um sistema downstream rejeitou mensagens encaminhadas ou retransmitidas. A autenticação pode ter sido afetada pelo encaminhamento ou por um gateway que não seja da Microsoft.

dsn-vendor-exchange-forwarding = A política do Microsoft 365 bloqueia o encaminhamento automático para destinatários externos. O administrador da organização deve revisar a política de encaminhamento.

dsn-vendor-exchange-generic = Exchange relatou uma rejeição geral de entrega. O diagnóstico identifica o problema real, que pode envolver formatação de mensagens ou política de anexos.

dsn-vendor-exchange-group-allow = O remetente não está na lista de remetentes permitidos deste grupo do Exchange.

dsn-vendor-exchange-group-internal = Este grupo de distribuição do Exchange rejeita remetentes fora de sua organização, a menos que seu proprietário ou administrador permita.

dsn-vendor-exchange-internal-only = Este destinatário do Exchange aceita remetentes autenticados de sua própria organização; esta mensagem foi tratada como externa ou não autenticada.

dsn-vendor-exchange-ipv6 = O destino não aceita esta entrega acima de IPv6. O administrador remetente precisa verificar a rota ou usar um transporte aceito.

dsn-vendor-exchange-journaling = A organização está tentando registrar mensagens locais no Microsoft 365 sem a configuração de arquivo necessária.

dsn-vendor-exchange-onmicrosoft-limit = A organização atingiu o limite de destinatários externos para seu domínio de envio onmicrosoft.com. O envio regular deve usar um domínio personalizado verificado.

dsn-vendor-exchange-public-folder = Esta pasta pública do Exchange não aceita este remetente externo ou não autenticado.

dsn-vendor-exchange-receive-rate = Este destinatário do Exchange atingiu o limite de recebimento por hora para todos os remetentes.

dsn-vendor-exchange-recipient = O Exchange não conseguiu resolver este destinatário em seu diretório. Verifique o endereço e remova qualquer entrada de preenchimento automático desatualizada antes de criar uma nova mensagem.

dsn-vendor-exchange-recipient-access = O Exchange rejeitou o acesso a este destinatário ou domínio de destino. Verifique o endereço do destinatário e a configuração do domínio aceito pela organização.

dsn-vendor-exchange-recipient-rate = O Exchange recebeu muitas mensagens deste remetente para este destinatário dentro do período de hora em hora.

dsn-vendor-exchange-sender-blocked = A Microsoft restringiu esta conta de envio após detectar suspeita de spam ou abuso. Isso pode exigir que um administrador proteja e desbloqueie a conta.

dsn-vendor-exchange-temporary-restriction = A Microsoft está restringindo temporariamente este envio IP ou tráfego enquanto avalia atividades suspeitas.

dsn-vendor-exchange-tenant-blocked = A Microsoft restringiu o envio para esta organização porque seu tráfego foi considerado suspeito.

dsn-vendor-exchange-tenant-blocklist = Um endereço ou domínio de destinatário é bloqueado pela lista de permissões/bloqueios de locatários da organização remetente. Um destino bloqueado pode rejeitar a mensagem inteira.

dsn-vendor-exchange-tenant-limit = A organização do Microsoft 365 atingiu seu limite diário contínuo para destinatários externos.

dsn-vendor-exchange-unregistered = A Microsoft rejeitou o envio de um domínio que não está devidamente registrado e verificado no locatário.

dsn-vendor-expired = O serviço de envio desistiu após o término do prazo de entrega. Verifique o diagnóstico da falha de conexão ou roteamento que causou o atraso.

dsn-vendor-fortimail-relay = O FortiMail negou esta entrega sob sua política de controle de acesso ou retransmissão. O administrador do gateway precisa verificar a regra de recebimento correspondente.

dsn-vendor-gmail-app-password = O Google exige uma senha de aplicativo ou um método de autorização de conta compatível para esse login SMTP.

dsn-vendor-gmail-auth-method = O Google rejeitou um método de autenticação não compatível, como o método XOAUTH retirado.

dsn-vendor-gmail-authentication = O Google não aceitou a autenticação do remetente. O diagnóstico distingue SPF/DKIM ausente, falha grave SPF e rejeição DMARC.

dsn-vendor-gmail-browser-login = O Google exige um login na conta ou uma verificação de segurança no navegador antes de permitir esse login SMTP.

dsn-vendor-gmail-custom-policy = A regra de e-mail personalizada de um administrador do Google Workspace rejeitou a mensagem. Isso não significa necessariamente que o filtro de spam do Google o rejeitou.

dsn-vendor-gmail-dmarc-record = O Google exige um registro DMARC com uma política declarada para este domínio de envio.

dsn-vendor-gmail-rate = O Google aplicou um limite de envio ou retransmissão ou detectou uma taxa de mensagens incomum. O diagnóstico identifica a conta, IP ou domínio afetado.

dsn-vendor-gmail-relay = O Google Workspace não autorizou o envio de IP, domínio ou conta para seu serviço de retransmissão SMTP.

dsn-vendor-gmail-reputation = A filtragem do Google rejeitou a mensagem porque considerou o e-mail não solicitado ou a reputação de envio IP/domínio muito baixa.

dsn-vendor-gmail-spf-record = O Google considera a configuração SPF do domínio do remetente inválida ou suspeita.

dsn-vendor-greylist = O gateway adiou temporariamente esta entrega usando a lista cinza. Um servidor de envio configurado corretamente normalmente tenta novamente mais tarde.

dsn-vendor-headers = O serviço de recebimento rejeitou cabeçalhos de mensagens inválidos ou ausentes. O aplicativo de envio precisa gerar mensagens formatadas corretamente.

dsn-vendor-helo = O servidor de envio usou uma identidade HELO/EHLO inaceitável. Sua saudação deve identificar corretamente o servidor remetente.

dsn-vendor-ip-block = O serviço de recebimento rejeitou o envio IP devido à sua reputação ou a uma entrada na lista de bloqueio.

dsn-vendor-mimecast-block = Mimecast rejeitou o remetente ou domínio sob uma política administrativa ou de remetente bloqueado.

dsn-vendor-mimecast-content = Uma política de exame de conteúdo do Mimecast rejeitou a mensagem.

dsn-vendor-mimecast-spoof = A política anti-spoofing do Mimecast rejeitou a identidade do remetente.

dsn-vendor-mta-sts-host = O servidor de e-mail de destino não é permitido pela política MTA-STS do domínio.

dsn-vendor-proofpoint-block = A Proofpoint bloqueou o envio IP ou domínio após detectar spam ou atividade maliciosa.

dsn-vendor-proofpoint-throttle = O Proofpoint adiou temporariamente este envio IP porque seu padrão de envio foi considerado incomum.

dsn-vendor-ptr = O envio IP não possui reverso aceitável DNS, ou seu nome PTR não resolve de volta para o mesmo IP.

dsn-vendor-recipient-ip-block = A organização destinatária bloqueia explicitamente o endereço de envio IP. Seu administrador controla este bloco local.

dsn-vendor-relay = O servidor de correio não retransmitirá esta mensagem ao destino. Verifique a autenticação SMTP, IPs de envio confiáveis, conectores e domínios aceitos.

dsn-vendor-routing-loop = A mensagem entrou em um loop de encaminhamento ou roteamento e excedeu o número permitido de saltos.

dsn-vendor-sending-limit = A conta remetente excedeu uma cota de mensagens ou destinatários. Reenvios imediatos repetidos não removerão o limite.

dsn-vendor-spf = O destinatário não aceitou a autenticação SPF para o domínio de envio IP e do remetente do envelope.

dsn-vendor-starttls = A conexão criptografada SMTP necessária não pôde ser estabelecida ou não foi suportada pelo próximo servidor.

dsn-vendor-yahoo-deferred = O Yahoo adiou temporariamente esta mensagem, geralmente ao avaliar o volume, a reputação ou as reclamações do destinatário. Verifique o diagnóstico para a causa declarada.

dsn-vendor-yahoo-policy = O Yahoo rejeitou a mensagem de acordo com sua política de correio. Autenticação, alinhamento DMARC ou conteúdo da mensagem podem estar envolvidos; o diagnóstico fornece o motivo específico.
