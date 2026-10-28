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
