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
