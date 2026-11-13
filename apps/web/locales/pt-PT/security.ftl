# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: security. Source: ../en/security.ftl
#
# HONESTY: the zero-access disclosure copy below is a deliberate, narrow, factual
# statement. Do NOT soften it into marketing wording when translating.

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Detalhes de segurança da mensagem
security-section-auth = Autenticação
security-section-received = Percurso de entrega
security-section-signature = Assinatura
security-section-attachments = Anexos
security-section-warnings = Avisos
security-section-controls = Controlos do remetente
security-received-empty = Não há cadeia Received disponível.
security-attachments-empty = Sem anexos.
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
security-chain-trusted = Cadeia fidedigna
security-chain-untrusted = Cadeia não fidedigna
security-chain-expired = Cadeia expirada
security-chain-unknown = Cadeia desconhecida
security-revocation-good = Não revogada
security-revocation-revoked = Chave revogada
security-revocation-unknown = Revogação desconhecida
security-key-changed = A chave do signatário mudou desde a última vez
security-fact-signer-key = Chave do signatário
security-fact-algorithm = Algoritmo
security-fact-key-created = Chave criada
security-fact-key-expires = Chave expira
security-fact-chain = Cadeia
security-fact-revocation = Revogação
security-fact-key-change = Mudança de chave

# -- Attachment risk --------------------------------------------------------
security-attach-none = Nenhum risco conhecido
security-attach-macro = Contém macros
security-attach-executable = Ficheiro executável
security-attach-encrypted-archive = Arquivo encriptado
security-attach-double-extension = Extensão de ficheiro dupla
security-attach-mismatch = tipo não corresponde (declarado { $declared }, detetado { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = O endereço Reply-To é diferente do remetente
security-anomaly-envelopeFromDivergence = O remetente do envelope é diferente do endereço From
security-anomaly-messageIdDomainAnomaly = O domínio do Message-ID não corresponde ao remetente
security-anomaly-dateSkew = A data de envio parece desfasada
security-anomaly-punycodeSender = O remetente utiliza punycode (possível domínio semelhante)

# -- Sender controls --------------------------------------------------------
security-control-block = Bloquear remetente
security-control-silence = Silenciar remetente
security-control-ignore-conversation = Ignorar conversa
security-control-report-phishing = Denunciar phishing
security-control-report-junk = Denunciar como indesejado
security-control-done-block = Remetente bloqueado
security-control-done-silence = Remetente silenciado
security-control-done-ignore-conversation = Conversa ignorada
security-control-done-report-phishing = Denunciado como phishing
security-control-done-report-junk = Denunciado como indesejado

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Armazenamento de acesso zero
security-za-on = Ativado
security-za-off = Desativado
security-za-protects = O acesso zero encripta o correio, as notas e os dados PIM que guarda, para que o servidor de alojamento mantenha apenas texto cifrado que não consegue ler. Protege o conteúdo dos seus dados guardados contra um operador curioso ou uma violação do anfitrião de armazenamento: quem ler a base de dados, os ficheiros no disco ou uma cópia de segurança roubada vê texto cifrado XChaCha20-Poly1305 e não consegue recuperar corpos de mensagens, assuntos, anexos, texto de notas nem o índice de pesquisa.

security-za-server-sees-title = O que o servidor continua a ver
security-za-sees-1 = Blobs de texto cifrado — as próprias linhas encriptadas.
security-za-sees-2 = IDs de linha opacos — os identificadores utilizados para guardar e obter linhas.
security-za-sees-3 = Tamanhos — o comprimento de cada texto cifrado (tamanho aproximado da mensagem/do anexo).
security-za-sees-4 = Carimbos de data/hora — quando as linhas são escritas e atualizadas.
security-za-sees-5 = Metadados de encaminhamento do envelope necessários para o proxy de IMAP/SMTP — o servidor continua a ligar-se ao seu fornecedor de correio a montante em seu nome, pelo que os metadados de ligação e o envelope de encaminhamento necessários para enviar e receber correio passam por ele.

security-za-active-server-caveat = O acesso zero protege os dados EM REPOUSO contra um anfitrião curioso ou comprometido. Um servidor ativo totalmente malicioso que faça de proxy do seu tráfego IMAP/SMTP em tempo real é um adversário mais forte, e o acesso zero NÃO protege contra ele: um servidor assim está na ligação em tempo real ao seu fornecedor de correio e pode observar ou adulterar o correio à medida que passa, independentemente da forma como a cópia guardada está encriptada. Escolha o acesso zero quando a sua preocupação for «Não quero que quem gere (ou viola) o anfitrião de armazenamento leia o meu correio guardado.» Não é uma proteção contra um operador que subverta o percurso do correio em tempo real.

security-za-no-search-claim = Não existe aqui encriptação pesquisável do lado do servidor, nem é feita qualquer afirmação nesse sentido. A pesquisa é executada inteiramente no cliente, sobre conteúdo que este desencriptou localmente; o servidor nunca detém uma forma pesquisável do seu texto simples.

security-za-recovery-tradeoff = As suas chaves são derivadas e mantidas apenas nos seus dispositivos — o servidor nunca recebe uma chave em texto simples. Se perder a frase-passe (ou a chave de acesso) E todos os dispositivos emparelhados E a frase de recuperação, os dados encriptados não podem ser recuperados. Guarde a frase de recuperação offline e trate-a como equivalente ao segredo mestre da sua conta.

security-za-setup-key = Configurar a sua chave
security-za-key-source = Origem da chave
security-za-passphrase = Frase-passe
security-za-passkey = Chave de acesso (sem palavra-passe)
security-za-passkey-unavailable = as chaves de acesso não estão disponíveis neste navegador
security-za-passphrase-label = Frase-passe de acesso zero
security-za-enable = Ativar acesso zero
security-za-recovery-title = Frase de recuperação
security-za-recovery-heading = Frase de recuperação — guarde-a offline agora
security-za-recovery-note = Esta é a única cópia. Quem a tiver pode ler os seus dados; sem ela (e sem um dispositivo emparelhado), os seus dados não podem ser recuperados.
security-za-disable-section = Desativar acesso zero
security-za-disable-heading = Desligar o acesso zero
security-za-disable-note = Os novos dados voltarão a ser guardados sem encriptação. Os dados encriptados existentes só continuam legíveis enquanto conseguir derivar a sua chave.
security-za-disable-btn = Desativar acesso zero
security-za-err-passphrase-len = utilize uma frase-passe com pelo menos 8 carateres
security-za-err-enable = não foi possível ativar o acesso zero
security-za-err-disable = não foi possível desativar o acesso zero

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Emparelhamento de dispositivos
security-pair-heading = Emparelhar um dispositivo
security-pair-intro = Transfira as suas chaves para outro dispositivo sem as enviar através do servidor — este retransmite apenas um envelope selado opaco. Compare as seis palavras em ambos os ecrãs antes de confiar no emparelhamento.
security-pair-role = Função no emparelhamento
security-pair-new-role = Este é o novo dispositivo
security-pair-existing-role = Emparelhar outro dispositivo a partir daqui
security-pair-show-qr = Mostrar QR de emparelhamento
security-pair-scan = Leia isto no seu dispositivo existente
security-pair-qr-label = Código QR de emparelhamento do dispositivo
security-pair-qr-default = Código QR de emparelhamento
security-pair-copy-code = Ou copie este código de emparelhamento
security-pair-paste-envelope = Cole o envelope selado do outro dispositivo
security-pair-envelope-placeholder = envelope…
security-pair-envelope-label = Envelope selado
security-pair-complete = Concluir emparelhamento
security-pair-code-from-new = Código de emparelhamento do novo dispositivo
security-pair-code-placeholder = código de emparelhamento…
security-pair-code-label = Código de emparelhamento
security-pair-seal = Selar as minhas chaves para esse dispositivo
security-pair-envelope-out = Envelope selado — cole-o no novo dispositivo
security-pair-compare = Compare estas palavras em ambos os dispositivos
security-pair-sas-label = Cadeia de autenticação curta
security-pair-match = As palavras coincidem
security-pair-abort = São diferentes — cancelar
security-pair-confirmed = Emparelhamento confirmado. Este canal está autenticado.
security-pair-err-start = não foi possível iniciar o emparelhamento
security-pair-err-complete = não foi possível concluir o emparelhamento
security-pair-err-unlock = desbloqueie primeiro o acesso zero para emparelhar outro dispositivo
security-pair-err-seal = não foi possível selar a chave raiz

# Fresh confirmation for sensitive changes.
security-stepup-title = Confirme a sua identidade
security-stepup-body = Confirme a sua identidade para continuar esta ação. A confirmação é válida durante cinco minutos nesta sessão.
security-stepup-password = Palavra-passe
security-stepup-code = Código do autenticador
security-stepup-confirm = Confirmar e continuar
security-stepup-passkey = Utilizar uma chave de acesso
security-stepup-cancel = Cancelar
security-stepup-failed = A confirmação falhou. Tente novamente.
security-stepup-unavailable = Esta sessão não tem nenhum método de confirmação disponível.
security-center-title = Centro de segurança
security-center-history = Histórico de inícios de sessão
security-center-result = Resultado
security-center-all = Todos os resultados
security-center-success = Com êxito
security-center-failure = Falhados
security-center-blocked = Bloqueados
security-center-refresh = Atualizar
security-center-export = Exportar histórico
security-center-devices = Dispositivos memorizados
security-center-remember-help = Se o seu administrador o permitir, memorize este dispositivo depois de verificar a sua identidade. As alterações sensíveis continuam a exigir uma nova verificação.
security-center-label = Nome do dispositivo
security-center-remember = Memorizar este dispositivo
security-center-revoke = Revogar
security-center-activity = Atividade da conta
security-center-error = Não foi possível carregar ou alterar as informações de segurança. O seu administrador pode ter desativado esta ação.
security-center-first-seen = Visto pela primeira vez: { $when }
security-center-passkey-mode = No modo motor, as chaves de acesso permitem iniciar sessão sem palavra-passe. No modo proxy, verificam um segundo fator após a palavra-passe do correio.
security-stepup-recovery = Código de recuperação (com a sua palavra-passe, em vez de um código do autenticador)

security-center-signin = Início de sessão e chaves de acesso
security-center-sessions = Sessões e acesso
security-center-encryption = Encriptação

signin-geo-title = Política de país no início de sessão
signin-geo-help = Opcionalmente recusa novos inícios de sessão na sua conta a partir de países que ainda não usou, ou de qualquer país fora da sua lista permitida. Está desligada por omissão e nunca afeta sessões já iniciadas. IPs e países permitidos passam sempre esta verificação; a palavra-passe e o segundo fator continuam a ser exigidos.
signin-geo-accuracy = A deteção do país usa uma base de dados de geolocalização de IP que pode estar errada ou desatualizada. VPN, operadoras móveis, proxies e viagens podem bloquear um início de sessão legítimo ou deixar passar um estrangeiro. Adicione IPs ou países de confiança à lista de permissões.
signin-geo-locked = Um administrador controla esta definição para a sua conta.
signin-geo-mode = Política
signin-geo-mode-off = Desligada
signin-geo-mode-block-unusual-countries = Bloquear países de onde nunca iniciei sessão
signin-geo-mode-block-foreign-countries = Bloquear países fora da minha lista permitida
signin-geo-countries = Países permitidos (códigos de duas letras, separados por vírgulas)
signin-geo-ips = Endereços IP ou redes permitidos (um por linha)
signin-geo-current = O seu endereço atual é { $ip } (país detetado: { $country }).
signin-geo-no-context = Não foi possível determinar o seu endereço atual.
signin-geo-unknown-country = desconhecido
signin-geo-add-ip = Adicionar o meu IP atual
signin-geo-add-country = Adicionar o meu país atual
signin-geo-save = Guardar
signin-geo-saved = Guardado.
signin-geo-invalid = Verifique os códigos de país e os endereços IP.
signin-geo-self-lockout = Isto bloquearia o início de sessão a partir do seu endereço ou país atual. Adicione primeiro o seu IP ou país atual à lista de permissões.
signin-geo-error = Não foi possível guardar a política de país no início de sessão.

session-alerts-title = Avisos de segurança das sessões
session-alerts-help = Avisos sobre utilização invulgar das suas sessões iniciadas. Cada aviso mostra o que o servidor viu e o que fez.
session-alerts-none = Sem avisos de segurança das sessões.
session-alerts-banner = { $count ->
    [one] Tem 1 aviso de segurança de sessão por ler.
   *[other] Tem { $count } avisos de segurança de sessão por ler.
}
session-alerts-banner-review = Rever
session-alerts-kind-countryChange = Uma sessão foi usada a partir de outro país ({ $from } para { $to }).
session-alerts-kind-impossibleTravel = Uma sessão apareceu noutro país ({ $from } para { $to }) mais depressa do que uma viagem permite.
session-alerts-kind-concurrentUse = Uma sessão foi usada ao mesmo tempo a partir de países ou redes diferentes ({ $from } e { $to }).
session-alerts-kind-fingerprintMismatch = Uma sessão foi usada a partir de um navegador ou dispositivo que não corresponde ao que iniciou sessão.
session-alerts-kind-rotationReuse = Um identificador de sessão antigo foi apresentado de novo depois de a sessão ter sido renovada.
session-alerts-action-alerted = Ação: foi alertado.
session-alerts-action-step-up = Ação: a sessão teve de confirmar a sua palavra-passe e o segundo fator.
session-alerts-action-revoked = Ação: a sessão foi terminada.
session-alerts-countries = Países: { $from } para { $to }
session-alerts-unknown-country = desconhecido
session-alerts-session = Sessão { $id }
session-alerts-caveat = A deteção do país pode ser imprecisa; VPN e redes móveis podem causar alterações falsas.
session-alerts-not-me = Não fui eu
session-alerts-dismiss = Dispensar
session-alerts-dismiss-all = Dispensar tudo
session-alerts-revoked = As suas outras sessões foram terminadas. Altere agora a palavra-passe na secção de palavra-passe de Início de sessão e chaves de acesso.
session-alerts-error = Não foi possível atualizar os avisos de segurança das sessões.
session-alerts-prefs = Preferências de alertas
session-alerts-country = Avisar-me quando uma sessão for usada a partir de um novo país
session-alerts-country-help = Se o administrador não permitir silenciar, o servidor continua a alertar mesmo com esta opção desligada.
session-alerts-channels = Onde receber os avisos
session-alerts-channel-app = Na aplicação
session-alerts-channel-email = Na minha caixa de correio
session-alerts-channel-both = Na aplicação e na minha caixa de correio
session-alerts-sensitivity = Sensibilidade
session-alerts-sensitivity-low = Baixa: apenas localização, sem terminar sessões à força
session-alerts-sensitivity-normal = Normal
session-alerts-sensitivity-high = Alta: verificação de viagem mais rigorosa, compara também redes
session-alerts-reset = Usar as predefinições do administrador
session-alerts-pref-error = Não foi possível guardar a preferência.

signin-geo-history-title = Países vistos
signin-geo-history-help = Os países de onde iniciou sessão com êxito, guardados durante { $days } dias. O modo «países invulgares» usa esta lista. Só são guardados códigos de país, datas e contagens, nunca endereços.
signin-geo-history-empty = Ainda não há países registados.
signin-geo-history-entry = { $country }: primeiro { $first }, último { $last }, { $count } inícios de sessão
signin-geo-history-forget = Esquecer os países vistos

# -- Explanations: headline is the security-anomaly-* / security-auth-* key --
security-severity-info = Informativo
security-severity-caution = Atenção
security-severity-suspicious = Suspeito
security-explain-toggle = O que significa?
security-explain-label-checked = O que foi verificado
security-explain-label-legit = Causas inofensivas comuns
security-explain-label-risk = O que pode indicar
security-explain-label-action = Ação sugerida
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = O domínio a seguir ao @ no cabeçalho Message-ID foi comparado com o domínio do endereço From. São diferentes.
security-explain-messageIdDomainAnomaly-legit = Listas de correio, plataformas de marketing e newsletters, serviços de reencaminhamento, gateways empresariais e servidores de webmail geram muitas vezes o Message-ID com o nome do seu próprio servidor.
security-explain-messageIdDomainAnomaly-risk = Um remetente falsificado, ou uma mensagem criada por software que não é o sistema de correio do remetente. Isoladamente, é um sinal fraco.
security-explain-messageIdDomainAnomaly-action = Veja os resultados de SPF, DKIM e DMARC acima. Se passaram para o domínio do remetente, é provavelmente rotina. Se falharam ou não existem, evite ligações e anexos e confirme com o remetente por outro meio.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = O endereço Reply-To foi comparado com o endereço From. As respostas seguiriam para um endereço diferente.
security-explain-replyToMismatch-legit = Listas de correio, serviços de apoio, assistentes que escrevem em nome de alguém e formulários de contacto definem um Reply-To diferente de propósito.
security-explain-replyToMismatch-risk = Respostas desviadas para um endereço controlado por um atacante, um padrão comum em fraudes de falsificação de identidade e de faturas.
security-explain-replyToMismatch-action = Confirme o endereço Reply-To antes de responder. Se a mensagem pedir dinheiro, credenciais ou alteração de dados, confirme por um canal em que já confia.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = O remetente do envelope (Return-Path), para onde vão as mensagens devolvidas, foi comparado com o endereço From. São diferentes.
security-explain-envelopeFromDivergence-legit = Serviços de envio em massa, sistemas de tickets e listas de correio usam o seu próprio endereço de devolução e mostram o endereço do seu contacto em From.
security-explain-envelopeFromDivergence-risk = Falsificação do remetente, quando o SPF ou o DMARC também falham. Isoladamente, é comum e normalmente inofensivo.
security-explain-envelopeFromDivergence-action = Verifique se o DMARC passou com alinhamento. Se sim, não é preciso fazer nada.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = O cabeçalho Date foi comparado com a hora a que a mensagem foi recebida. Estão muito afastados.
security-explain-dateSkew-legit = Um computador do remetente com o relógio errado, uma mensagem retida numa fila ou reenviada durante muito tempo, ou um envio agendado.
security-explain-dateSkew-risk = Uma mensagem repetida ou com data antiga. É pouco comum e raramente prejudicial por si só.
security-explain-dateSkew-action = Tenha cuidado com pedidos urgentes na mensagem e confirme-os se forem importantes.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = O domínio do remetente contém punycode (uma etiqueta xn--), a forma como os caracteres não latinos são escritos em nomes de domínio.
security-explain-punycodeSender-legit = Domínios genuínos em línguas como árabe, chinês, cirílico ou português com acentos são codificados assim.
security-explain-punycodeSender-risk = Um domínio semelhante que mistura caracteres de outros alfabetos para se parecer com uma marca ou pessoa conhecida.
security-explain-punycodeSender-action = Compare o domínio do remetente caráter a caráter com o que espera. Não clique em ligações nem abra anexos até confirmar o remetente por outro meio, e denuncie se não for genuíno.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = O servidor recetor verificou este método e o resultado não correspondeu ao que o domínio do remetente publica.
security-explain-auth-fail-legit = O reencaminhamento e as listas de correio podem quebrar o SPF ou o DKIM, e um serviço de envio mal configurado pode falhar em ambos.
security-explain-auth-fail-risk = A mensagem pode não vir de quem diz, ou ter sido alterada pelo caminho.
security-explain-auth-fail-action = Não confie no remetente com base nesta evidência. Evite ligações e anexos e verifique por outro canal.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = A verificação não pôde ser concluída porque o registo publicado pelo domínio é inválido ou demasiado complexo.
security-explain-auth-permerror-legit = Um erro nos registos DNS do remetente.
security-explain-auth-permerror-risk = Nada sobre a mensagem em si, mas o domínio do remetente não pode ser verificado.
security-explain-auth-permerror-action = Trate como um resultado ausente e consulte os outros métodos.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = Não foi encontrado nenhum resultado para este método nos cabeçalhos da mensagem.
security-explain-auth-none-legit = O domínio do remetente não publica este método, ou o servidor recetor não o regista.
security-explain-auth-none-risk = O domínio não oferece forma de verificar o remetente, pelo que a falsificação não pode ser excluída.
security-explain-auth-none-action = Baseie-se nos outros métodos e em saber se esperava esta mensagem.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = O domínio publicou um registo para este método, mas não fez nenhuma afirmação sobre esta mensagem.
security-explain-auth-neutral-legit = Um domínio que ainda está a definir a sua política.
security-explain-auth-neutral-risk = Pouco. O remetente não pôde ser confirmado nem rejeitado.
security-explain-auth-neutral-action = Baseie-se nos outros métodos e em saber se esperava esta mensagem.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = A verificação não pôde terminar devido a um problema temporário, normalmente uma falha na consulta DNS.
security-explain-auth-temperror-legit = Uma falha curta no DNS do remetente ou do recetor.
security-explain-auth-temperror-risk = Nada sobre a mensagem em si.
security-explain-auth-temperror-action = Normalmente não é preciso fazer nada. Volte a verificar mais tarde se a mensagem for importante.

breach-notice-heading = Aviso sobre uma violação de dados pessoais
breach-notice-reference = Referência { $reference }
breach-notice-ack = Confirmar leitura

session-evidence-ip = Endereço IP
session-evidence-location = Localização
session-evidence-device = Tipo de dispositivo
session-evidence-recognized = Reconhecimento do dispositivo
session-evidence-browser = Navegador
session-evidence-engine = Motor do navegador
session-evidence-os = Sistema operativo
session-evidence-client = Tipo de cliente
session-evidence-version = Versão do cliente
session-evidence-authentication = Autenticação
session-evidence-fingerprint = Impressão digital do dispositivo
session-evidence-agent = Agente do utilizador
session-evidence-unavailable = Não disponível
session-evidence-known = Dispositivo reconhecido
session-evidence-new = Dispositivo não reconhecido
session-evidence-technical = Detalhes técnicos
session-evidence-legacy = Os detalhes do dispositivo não foram registados neste aviso antigo.
session-evidence-help = Os detalhes são comunicados pelo cliente. O reconhecimento é informativo e não prova que o dispositivo é seguro. A impressão digital é uma referência específica da conta, não um identificador de hardware.
session-evidence-device-desktop = Computador
session-evidence-device-phone = Telemóvel
session-evidence-device-tablet = Tablet
session-evidence-device-unknown = Dispositivo desconhecido
session-evidence-client-browser = Navegador Web
session-alerts-kind-countryUnavailable = Não foi possível determinar o país de um pedido da sessão.
session-alerts-kind-unknown = Reveja a atividade invulgar da sessão.

notification-center-help = Reveja a atividade importante e as atualizações. Dispensar e arquivar organizam esta lista; não resolvem problemas de segurança.
notification-center-active = Ativas
notification-center-archived = Arquivadas
notification-center-dismissed = Dispensadas
notification-center-type = Tipo
notification-center-type-all = Todos os tipos
notification-center-type-security = Segurança
notification-center-type-session = Avisos de sessão
notification-center-type-login = Inícios de sessão
notification-center-type-update = Atualizações
notification-center-login = Início de sessão bem-sucedido
notification-center-security = Atividade de segurança da conta
notification-center-newDevice = Início de sessão num dispositivo não reconhecido
notification-center-newCountry = Início de sessão num país novo
notification-center-twoFactorChanged = Autenticação de dois fatores alterada
notification-center-delegateAdded = Foi concedido acesso à caixa de correio
notification-center-forwardingAdded = Foi adicionada uma regra de reencaminhamento
notification-center-apiKeyCreated = Foi criada uma chave de API
notification-center-error = Não foi possível carregar ou atualizar as notificações. Tente atualizar.
notification-center-empty = Não há notificações nesta vista.
notification-center-open = Abrir ação relacionada
notification-center-dismiss = Dispensar
notification-center-archive = Arquivar
notification-center-restore = Restaurar
