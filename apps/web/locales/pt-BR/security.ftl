# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Detalhes de segurança da mensagem
security-section-auth = Autenticação
security-section-received = Caminho de entrega
security-section-signature = Assinatura
security-section-attachments = Anexos
security-section-warnings = Avisos
security-section-controls = Controles do remetente
security-received-empty = Nenhuma cadeia Received disponível.
security-attachments-empty = Nenhum anexo.
security-hop-unknown = desconhecido
security-unknown = desconhecido

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = aprovado
security-auth-fail = reprovado
security-auth-none = ausente
security-auth-neutral = neutro
security-auth-temperror = erro temporário
security-auth-permerror = erro permanente

# Expert alignment detail field labels + values.
security-detail-domain = domínio
security-detail-selector = seletor
security-detail-policy = política
security-detail-alignment = alinhamento
security-detail-chain-length = comprimento da cadeia
security-aligned = alinhado
security-not-aligned = não alinhado

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Assinatura verificada
security-sig-unverified-key = Assinada — chave do signatário não verificada
security-sig-invalid = A assinatura é inválida
security-sig-none = Não assinada
security-chain-trusted = Cadeia confiável
security-chain-untrusted = Cadeia não confiável
security-chain-expired = Cadeia expirada
security-chain-unknown = Cadeia desconhecida
security-revocation-good = Não revogada
security-revocation-revoked = Chave revogada
security-revocation-unknown = Revogação desconhecida
security-key-changed = A chave do signatário mudou desde a última vez
security-fact-signer-key = Chave do signatário
security-fact-algorithm = Algoritmo
security-fact-key-created = Chave criada em
security-fact-key-expires = Chave expira em
security-fact-chain = Cadeia
security-fact-revocation = Revogação
security-fact-key-change = Mudança de chave

# -- Attachment risk --------------------------------------------------------
security-attach-none = Nenhum risco conhecido
security-attach-macro = Contém macros
security-attach-executable = Arquivo executável
security-attach-encrypted-archive = Arquivo compactado criptografado
security-attach-double-extension = Extensão de arquivo dupla
security-attach-mismatch = tipo divergente (declarado { $declared }, detectado { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = O endereço Reply-To é diferente do remetente
security-anomaly-envelopeFromDivergence = O remetente do envelope é diferente do endereço From
security-anomaly-messageIdDomainAnomaly = O domínio do Message-ID não corresponde ao remetente
security-anomaly-dateSkew = A data de envio parece incorreta
security-anomaly-punycodeSender = O remetente usa punycode (possível domínio parecido)

# -- Sender controls --------------------------------------------------------
security-control-block = Bloquear remetente
security-control-silence = Silenciar remetente
security-control-ignore-conversation = Ignorar conversa
security-control-report-phishing = Denunciar phishing
security-control-report-junk = Denunciar spam
security-control-done-block = Remetente bloqueado
security-control-done-silence = Remetente silenciado
security-control-done-ignore-conversation = Conversa ignorada
security-control-done-report-phishing = Denunciado como phishing
security-control-done-report-junk = Denunciado como spam

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Armazenamento de acesso zero
security-za-on = Ativado
security-za-off = Desativado
security-za-protects = O acesso zero criptografa suas mensagens, notas e dados PIM armazenados, de modo que o servidor de hospedagem guarda apenas texto cifrado que não consegue ler. Ele protege o conteúdo dos seus dados armazenados contra um operador curioso ou uma invasão do host de armazenamento: quem ler o banco de dados, os arquivos em disco ou um backup roubado vê texto cifrado XChaCha20-Poly1305 e não consegue recuperar corpos de mensagens, assuntos, anexos, texto de notas nem o índice de pesquisa.

security-za-server-sees-title = O que o servidor ainda vê
security-za-sees-1 = Blobs de texto cifrado — as próprias linhas criptografadas.
security-za-sees-2 = IDs de linha opacos — os identificadores usados para armazenar e buscar linhas.
security-za-sees-3 = Tamanhos — o comprimento de cada texto cifrado (tamanho aproximado de mensagens/anexos).
security-za-sees-4 = Data e hora — quando as linhas são gravadas e atualizadas.
security-za-sees-5 = Metadados de roteamento do envelope necessários para intermediar IMAP/SMTP — o servidor ainda se conecta ao seu provedor de e-mail upstream em seu nome, então os metadados de conexão e o envelope de roteamento necessários para enviar e receber e-mail passam por ele.

security-za-active-server-caveat = O acesso zero protege os dados EM REPOUSO contra um host curioso ou invadido. Um servidor ativo totalmente malicioso, que intermedeia seu tráfego IMAP/SMTP em tempo real, é um adversário mais forte, e o acesso zero NÃO protege contra ele: esse servidor fica na conexão ativa com seu provedor de e-mail e poderia observar ou adulterar as mensagens enquanto elas passam, independentemente de como a cópia armazenada é criptografada. Escolha o acesso zero quando sua preocupação for “não quero que quem administra (ou invade) o host de armazenamento leia minhas mensagens armazenadas”. Ele não é uma defesa contra um operador que subverte o caminho ativo do e-mail.

security-za-no-search-claim = Não há criptografia pesquisável no servidor aqui, e nenhuma afirmação desse tipo é feita. A pesquisa roda inteiramente no cliente, sobre o conteúdo que ele descriptografou localmente; o servidor nunca guarda uma forma pesquisável do seu texto simples.

security-za-recovery-tradeoff = Suas chaves são derivadas e guardadas apenas nos seus dispositivos — o servidor nunca recebe uma chave em texto simples. Se você perder sua frase secreta (ou chave de acesso) E todos os dispositivos pareados E sua frase de recuperação, os dados criptografados não poderão ser recuperados. Guarde a frase de recuperação off-line e trate-a como equivalente ao segredo mestre da sua conta.

security-za-setup-key = Configurar sua chave
security-za-key-source = Origem da chave
security-za-passphrase = Frase secreta
security-za-passkey = Chave de acesso (sem senha)
security-za-passkey-unavailable = chaves de acesso não estão disponíveis neste navegador
security-za-passphrase-label = Frase secreta de acesso zero
security-za-enable = Ativar acesso zero
security-za-recovery-title = Frase de recuperação
security-za-recovery-heading = Frase de recuperação — guarde-a off-line agora
security-za-recovery-note = Esta é a única cópia. Qualquer pessoa que a tenha pode ler seus dados; sem ela (e sem um dispositivo pareado) seus dados não podem ser recuperados.
security-za-disable-section = Desativar acesso zero
security-za-disable-heading = Desligar o acesso zero
security-za-disable-note = Os novos dados voltarão a ser armazenados sem criptografia. Os dados criptografados existentes continuam legíveis apenas enquanto você ainda conseguir derivar sua chave.
security-za-disable-btn = Desativar acesso zero
security-za-err-passphrase-len = use uma frase secreta de pelo menos 8 caracteres
security-za-err-enable = não foi possível ativar o acesso zero
security-za-err-disable = não foi possível desativar o acesso zero

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Pareamento de dispositivos
security-pair-heading = Parear um dispositivo
security-pair-intro = Transfira suas chaves para outro dispositivo sem enviá-las pelo servidor — ele apenas retransmite um envelope lacrado opaco. Compare as seis palavras nas duas telas antes de confiar no pareamento.
security-pair-role = Função no pareamento
security-pair-new-role = Este é o novo dispositivo
security-pair-existing-role = Parear outro dispositivo a partir daqui
security-pair-show-qr = Mostrar QR de pareamento
security-pair-scan = Escaneie isto no seu dispositivo existente
security-pair-qr-label = Código QR de pareamento de dispositivo
security-pair-qr-default = Código QR de pareamento
security-pair-copy-code = Ou copie este código de pareamento
security-pair-paste-envelope = Cole o envelope lacrado do outro dispositivo
security-pair-envelope-placeholder = envelope…
security-pair-envelope-label = Envelope lacrado
security-pair-complete = Concluir pareamento
security-pair-code-from-new = Código de pareamento do novo dispositivo
security-pair-code-placeholder = código de pareamento…
security-pair-code-label = Código de pareamento
security-pair-seal = Lacrar minhas chaves para esse dispositivo
security-pair-envelope-out = Envelope lacrado — cole isto no novo dispositivo
security-pair-compare = Compare estas palavras nos dois dispositivos
security-pair-sas-label = Sequência curta de autenticação
security-pair-match = As palavras coincidem
security-pair-abort = São diferentes — cancelar
security-pair-confirmed = Pareamento confirmado. Este canal está autenticado.
security-pair-err-start = não foi possível iniciar o pareamento
security-pair-err-complete = não foi possível concluir o pareamento
security-pair-err-unlock = desbloqueie o acesso zero primeiro para parear outro dispositivo
security-pair-err-seal = não foi possível lacrar a chave raiz

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = Confirme que é você
security-stepup-body = Confirme sua identidade para continuar esta ação. A confirmação vale por cinco minutos nesta sessão.
security-stepup-password = Senha
security-stepup-code = Código do autenticador
security-stepup-confirm = Confirmar e continuar
security-stepup-passkey = Usar uma passkey
security-stepup-cancel = Cancelar
security-stepup-failed = A confirmação falhou. Tente novamente.
security-stepup-unavailable = Esta sessão não tem nenhum método de confirmação disponível.
security-center-title = Central de segurança
security-center-history = Histórico de logins
security-center-result = Resultado
security-center-all = Todos os resultados
security-center-success = Bem-sucedidos
security-center-failure = Com falha
security-center-blocked = Bloqueados
security-center-refresh = Atualizar
security-center-export = Exportar histórico
security-center-devices = Dispositivos lembrados
security-center-remember-help = Se o seu administrador permitir, lembre este dispositivo depois de verificar sua identidade. Alterações sensíveis ainda exigem uma nova verificação.
security-center-label = Nome do dispositivo
security-center-remember = Lembrar este dispositivo
security-center-revoke = Revogar
security-center-activity = Atividade da conta
security-center-error = Não foi possível carregar ou alterar as informações de segurança. Seu administrador pode ter desativado esta ação.
security-center-first-seen = Visto pela primeira vez: { $when }
security-center-passkey-mode = No modo engine, as passkeys podem fazer seu login sem senha. No modo proxy, elas verificam um segundo fator depois da senha do seu e-mail.
security-stepup-recovery = Código de recuperação (com sua senha, no lugar de um código do autenticador)
security-center-signin = Login e passkeys
security-center-sessions = Sessões e acesso
security-center-encryption = Criptografia
signin-geo-title = Política de país para login
signin-geo-help = Opcionalmente, recuse novos logins na sua conta vindos de países que você ainda não usou ou de qualquer país fora da sua lista permitida. Vem desativada por padrão e nunca afeta sessões que já estão conectadas. IPs e países permitidos sempre passam por esta verificação; a senha e o segundo fator continuam sendo exigidos.
signin-geo-accuracy = A detecção de país usa um banco de dados de geolocalização de IP que pode estar errado ou desatualizado. VPNs, operadoras móveis, proxies e viagens podem bloquear um login legítimo ou deixar passar um login do exterior. Adicione endereços IP ou países confiáveis à lista de permissões.
signin-geo-locked = Um administrador controla esta configuração para a sua conta.
signin-geo-mode = Política
signin-geo-mode-off = Desativada
signin-geo-mode-block-unusual-countries = Bloquear países dos quais eu ainda não fiz login
signin-geo-mode-block-foreign-countries = Bloquear países fora da minha lista permitida
signin-geo-countries = Países permitidos (códigos de duas letras, separados por vírgula)
signin-geo-ips = Endereços IP ou redes permitidos (um por linha)
signin-geo-current = Seu endereço atual é { $ip } (país detectado: { $country }).
signin-geo-no-context = Não foi possível determinar seu endereço atual.
signin-geo-unknown-country = desconhecido
signin-geo-add-ip = Adicionar meu IP atual
signin-geo-add-country = Adicionar meu país atual
signin-geo-save = Salvar
signin-geo-saved = Salvo.
signin-geo-invalid = Verifique os códigos de país e os endereços IP.
signin-geo-self-lockout = Isso bloquearia o login a partir do seu endereço ou país atual. Adicione primeiro seu endereço IP ou país atual à lista de permissões.
signin-geo-error = Não foi possível salvar a política de país para login.
session-alerts-title = Avisos de segurança das sessões
session-alerts-help = Alertas sobre uso incomum das suas sessões conectadas. Cada aviso mostra o que o servidor viu e o que ele fez.
session-alerts-none = Nenhum aviso de segurança das sessões.
session-alerts-banner = { $count ->
    [one] Você tem 1 aviso de segurança de sessão não lido.
   *[other] Você tem { $count } avisos de segurança de sessão não lidos.
}
session-alerts-banner-review = Revisar
session-alerts-kind-countryChange = Uma sessão foi usada a partir de um país diferente ({ $from } para { $to }).
session-alerts-kind-impossibleTravel = Uma sessão apareceu em outro país ({ $from } para { $to }) mais rápido do que uma viagem permitiria.
session-alerts-kind-concurrentUse = Uma sessão foi usada ao mesmo tempo a partir de países ou redes diferentes ({ $from } e { $to }).
session-alerts-kind-fingerprintMismatch = Uma sessão foi usada a partir de um navegador ou dispositivo que não corresponde ao que fez o login.
session-alerts-kind-rotationReuse = Um identificador de sessão antigo foi apresentado novamente depois que a sessão havia sido renovada.
session-alerts-action-alerted = Ação: você foi alertado.
session-alerts-action-step-up = Ação: foi solicitado à sessão que confirmasse sua senha e o segundo fator.
session-alerts-action-revoked = Ação: a sessão foi encerrada.
session-alerts-countries = Países: { $from } para { $to }
session-alerts-unknown-country = desconhecido
session-alerts-session = Sessão { $id }
session-alerts-caveat = A detecção de país pode ser imprecisa; VPNs e redes móveis podem causar mudanças falsas.
session-alerts-not-me = Não fui eu
session-alerts-dismiss = Dispensar
session-alerts-dismiss-all = Dispensar tudo
session-alerts-revoked = Suas outras sessões foram encerradas. Altere sua senha agora na seção de senha de Login e passkeys.
session-alerts-error = Não foi possível atualizar os avisos de segurança das sessões.
session-alerts-prefs = Preferências de alertas
session-alerts-country = Avisar-me quando uma sessão for usada a partir de um novo país
session-alerts-country-help = Se o administrador não permitir silenciar, o servidor continua alertando mesmo com esta opção desativada.
session-alerts-channels = Onde receber os avisos
session-alerts-channel-app = No aplicativo
session-alerts-channel-email = Na minha caixa de correio
session-alerts-channel-both = No aplicativo e na minha caixa de correio
session-alerts-sensitivity = Sensibilidade
session-alerts-sensitivity-low = Baixa: somente indícios de localização, sem encerramento forçado
session-alerts-sensitivity-normal = Normal
session-alerts-sensitivity-high = Alta: verificação de viagem mais rigorosa, também compara redes
session-alerts-reset = Usar os padrões do administrador
session-alerts-pref-error = Não foi possível salvar a preferência.
signin-geo-history-title = Países vistos
signin-geo-history-help = Os países dos quais você fez login com sucesso, mantidos por { $days } dias. O modo "Países incomuns" usa esta lista. Somente códigos de país, datas e contagens são armazenados, nunca endereços.
signin-geo-history-empty = Nenhum país registrado ainda.
signin-geo-history-entry = { $country }: primeiro { $first }, último { $last }, { $count } logins
signin-geo-history-forget = Esquecer os países vistos
security-severity-info = Informativo
security-severity-caution = Atenção
security-severity-suspicious = Suspeito
security-explain-toggle = O que isso significa?
security-explain-label-checked = O que foi verificado
security-explain-label-legit = Causas comuns e inofensivas
security-explain-label-risk = O que pode indicar
security-explain-label-action = Ação sugerida
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = O domínio após o @ no cabeçalho Message-ID foi comparado com o domínio do endereço De. Eles são diferentes.
security-explain-messageIdDomainAnomaly-legit = Listas de e-mail, plataformas de marketing e de newsletters, serviços de encaminhamento, gateways corporativos e provedores de webmail costumam gerar o Message-ID com o nome do próprio servidor.
security-explain-messageIdDomainAnomaly-risk = Um remetente forjado ou uma mensagem criada por um software que não é o sistema de e-mail do próprio remetente. Isoladamente, é um sinal fraco.
security-explain-messageIdDomainAnomaly-action = Veja os resultados de SPF, DKIM e DMARC acima. Se passaram para o domínio do remetente, provavelmente é algo rotineiro. Se falharam ou estão ausentes, evite links e anexos e confirme com o remetente por outro meio.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = O endereço de Responder para foi comparado com o endereço De. As respostas iriam para um endereço diferente.
security-explain-replyToMismatch-legit = Listas de e-mail, centrais de suporte, assistentes que escrevem em nome de alguém e formulários de contato definem um Responder para diferente de propósito.
security-explain-replyToMismatch-risk = As respostas sendo redirecionadas para um endereço controlado pelo atacante, um padrão comum em falsificação de identidade e fraude de faturas.
security-explain-replyToMismatch-action = Verifique o endereço de Responder para antes de responder. Se a mensagem pedir dinheiro, credenciais ou uma alteração de dados, confirme por um canal em que você já confia.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = O remetente do envelope (Return-Path), para onde vão as devoluções, foi comparado com o endereço De. Eles são diferentes.
security-explain-envelopeFromDivergence-legit = Serviços de envio em massa, sistemas de chamados e listas de e-mail usam seu próprio endereço de devolução enquanto mostram o endereço do seu contato em De.
security-explain-envelopeFromDivergence-risk = Falsificação de remetente, quando o SPF ou o DMARC também falha. Sozinho, é comum e geralmente inofensivo.
security-explain-envelopeFromDivergence-action = Verifique se o DMARC passou com alinhamento. Se passou, nenhuma ação é necessária.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = O cabeçalho Date foi comparado com o horário em que a mensagem foi recebida. Há uma grande diferença entre eles.
security-explain-dateSkew-legit = Um computador do remetente com o relógio errado, uma mensagem retida em uma fila ou reenviada por muito tempo, ou um envio agendado.
security-explain-dateSkew-risk = Uma mensagem reproduzida ou com data retroativa. Isso é incomum e raramente é prejudicial por si só.
security-explain-dateSkew-action = Trate com cuidado os pedidos urgentes da mensagem e confirme-os se forem importantes.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = O domínio do remetente contém punycode (um rótulo xn--), que é como os caracteres não latinos são escritos em nomes de domínio.
security-explain-punycodeSender-legit = Domínios legítimos em idiomas como árabe, chinês, cirílico ou português com acentos são codificados dessa forma.
security-explain-punycodeSender-risk = Um domínio parecido que mistura caracteres de outros alfabetos para imitar uma marca ou pessoa conhecida.
security-explain-punycodeSender-action = Compare o domínio do remetente caractere por caractere com o que você espera. Não clique em links nem abra anexos até confirmar o remetente por outro meio, e denuncie se não for legítimo.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = O servidor receptor verificou este método e o resultado não correspondeu ao que o domínio do remetente publica.
security-explain-auth-fail-legit = Encaminhamentos e listas de e-mail podem quebrar o SPF ou o DKIM, e um serviço de envio mal configurado pode falhar em qualquer um deles.
security-explain-auth-fail-risk = A mensagem pode não vir de quem diz ser ou ter sido alterada no caminho.
security-explain-auth-fail-action = Não confie no remetente com base nesta evidência. Evite links e anexos e verifique por outro canal.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = A verificação não pôde ser concluída porque o registro publicado do domínio é inválido ou complexo demais.
security-explain-auth-permerror-legit = Um erro nos registros DNS do remetente.
security-explain-auth-permerror-risk = Nada sobre a mensagem em si, mas o domínio do remetente não pode ser verificado.
security-explain-auth-permerror-action = Trate como um resultado ausente e verifique os outros métodos.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = Nenhum resultado para este método foi encontrado nos cabeçalhos da mensagem.
security-explain-auth-none-legit = O domínio do remetente não publica este método ou o servidor receptor não o registra.
security-explain-auth-none-risk = O domínio não oferece meio de verificar o remetente, por isso a falsificação não pode ser descartada.
security-explain-auth-none-action = Baseie-se nos outros métodos e em saber se você esperava esta mensagem.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = O domínio publicou um registro para este método, mas não fez nenhuma afirmação sobre esta mensagem.
security-explain-auth-neutral-legit = Um domínio que ainda está definindo sua política.
security-explain-auth-neutral-risk = Pouco. O remetente não pôde ser confirmado nem rejeitado.
security-explain-auth-neutral-action = Baseie-se nos outros métodos e em saber se você esperava esta mensagem.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = A verificação não pôde terminar por causa de um problema temporário, normalmente uma falha de consulta DNS.
security-explain-auth-temperror-legit = Uma breve indisponibilidade do DNS do remetente ou do destinatário.
security-explain-auth-temperror-risk = Nada sobre a mensagem em si.
security-explain-auth-temperror-action = Normalmente nenhuma ação é necessária. Verifique novamente mais tarde se a mensagem for importante.
breach-notice-heading = Aviso sobre uma violação de dados pessoais
breach-notice-reference = Referência { $reference }
breach-notice-ack = Confirmar ciência

session-evidence-ip = Endereço IP
session-evidence-location = Localização
session-evidence-device = Tipo de dispositivo
session-evidence-recognized = Reconhecimento do dispositivo
session-evidence-browser = Navegador
session-evidence-engine = Motor do navegador
session-evidence-os = Sistema operacional
session-evidence-client = Tipo de cliente
session-evidence-version = Versão do cliente
session-evidence-authentication = Autenticação
session-evidence-fingerprint = Impressão digital do dispositivo
session-evidence-agent = Agente do usuário
session-evidence-unavailable = Indisponível
session-evidence-known = Dispositivo reconhecido
session-evidence-new = Dispositivo não reconhecido
session-evidence-technical = Detalhes técnicos
session-evidence-legacy = Os detalhes do dispositivo não foram registrados neste aviso antigo.
session-evidence-help = Os detalhes são informados pelo cliente. O reconhecimento é informativo e não comprova a segurança do dispositivo. A impressão digital é uma referência específica da conta, não um identificador de hardware.
session-evidence-device-desktop = Computador
session-evidence-device-phone = Celular
session-evidence-device-tablet = Tablet
session-evidence-device-unknown = Dispositivo desconhecido
session-evidence-client-browser = Navegador Web
session-alerts-kind-countryUnavailable = Não foi possível determinar o país de uma solicitação da sessão.
session-alerts-kind-unknown = Revise a atividade incomum da sessão.

notification-center-help = Revise a atividade importante e as atualizações. Dispensar e arquivar organizam esta lista; não resolvem problemas de segurança.
notification-center-active = Ativas
notification-center-archived = Arquivadas
notification-center-dismissed = Dispensadas
notification-center-type = Tipo
notification-center-type-all = Todos os tipos
notification-center-type-security = Segurança
notification-center-type-session = Avisos de sessão
notification-center-type-login = Logins
notification-center-type-update = Atualizações
notification-center-login = Login bem-sucedido
notification-center-security = Atividade de segurança da conta
notification-center-newDevice = Login em um dispositivo não reconhecido
notification-center-newCountry = Login em um novo país
notification-center-twoFactorChanged = Autenticação de dois fatores alterada
notification-center-delegateAdded = Foi concedido acesso à caixa de correio
notification-center-forwardingAdded = Foi adicionada uma regra de encaminhamento
notification-center-apiKeyCreated = Foi criada uma chave de API
notification-center-error = Não foi possível carregar ou atualizar as notificações. Tente atualizar.
notification-center-empty = Não há notificações nesta visualização.
notification-center-open = Abrir ação relacionada
notification-center-dismiss = Dispensar
notification-center-archive = Arquivar
notification-center-restore = Restaurar
