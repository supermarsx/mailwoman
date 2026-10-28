# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-policy. Source: ../en/admin-policy.ftl

## Security policy

admin-policy-title = Política de segurança
admin-policy-intro = O nível mínimo de TLS, o custo do hash de palavras-passe, o nível mínimo de leitura e as regras de prevenção de perda de dados desta implementação. Todos os processos do servidor voltam a ler a política em 30 segundos.
admin-policy-unavailable = este servidor não oferece uma política de segurança.
admin-policy-history = Histórico
admin-policy-off = Desativado

admin-policy-group-connections = Ligações
admin-policy-min-tls = Versão mínima de TLS
admin-policy-min-tls-help = Aplica-se ao serviço HTTPS e às novas ligações IMAP, SMTP, POP3, ManageSieve e de proxy; as ligações abertas mantêm a sua versão.
admin-policy-min-tls-12 = TLS 1.2 e posteriores
admin-policy-min-tls-13 = Apenas TLS 1.3

admin-policy-group-sign-in = Início de sessão
admin-policy-group-sign-in-help = O custo do hash aplica-se aos novos hashes de palavras-passe e de códigos de recuperação; os hashes existentes mantêm o seu custo até a palavra-passe ser alterada.
admin-policy-require-2fa = Exigir um segundo fator em todas as contas
admin-policy-require-2fa-help = As contas sem segundo fator são convidadas a configurá-lo no próximo início de sessão; as regras por domínio estão na página da autenticação de dois fatores.
admin-policy-argon-memory = Memória do Argon2, em KiB
admin-policy-argon-memory-help = Memória usada em cada hash de palavra-passe, de { $min } a { $max } KiB; mais memória torna a adivinhação mais lenta e cada início de sessão mais pesado.
admin-policy-argon-single-pass = Com uma única passagem, a memória tem de ser de pelo menos { $min } KiB.
admin-policy-argon-passes = Passagens do Argon2
admin-policy-argon-passes-help = Quantas vezes o hash percorre a sua memória, de 1 a { $max }.
admin-policy-argon-parallelism = Paralelismo do Argon2
admin-policy-argon-parallelism-help = Threads usadas em cada hash, de 1 a { $max }; não altera quantos inícios de sessão decorrem em simultâneo.

admin-policy-group-reading = Leitura
admin-policy-group-reading-help = Os clientes Mailwoman leem estas opções através da política pública; os outros clientes de correio não as aplicam.
admin-policy-floor = Nível mínimo de leitura
admin-policy-floor-help = A forma menos restritiva de mostrar uma mensagem; cada pessoa pode escolher uma vista mais restrita, mas não uma menos restrita.
admin-policy-floor-none = Sem nível mínimo
admin-policy-floor-full-sanitized = HTML limpo com multimédia
admin-policy-floor-sanitized-no-media = HTML limpo sem multimédia remota
admin-policy-floor-plain-text = Apenas texto simples
admin-policy-capture = Proteção contra captura de ecrã
admin-policy-capture-help = Pede às aplicações Mailwoman que bloqueiem capturas e gravações de ecrã do correio; os navegadores não o podem impor.
admin-policy-capture-off = Desativado
admin-policy-capture-protect = Proteger

admin-policy-group-dlp = Prevenção de perda de dados
admin-policy-group-dlp-help = Regras verificadas no correio de saída enviado através deste servidor no modo de motor, além das regras em MW_DLP_RULES.
admin-policy-dlp-rules = Regras guardadas
admin-policy-dlp-rules-help = Cada regra indica o que procurar e o que acontece quando uma mensagem corresponde; uma regra de MW_DLP_RULES com o mesmo id prevalece.
admin-policy-dlp-json = Regras guardadas (texto)
admin-policy-dlp-json-help = Não foi possível ler as regras guardadas como regras, pelo que são mostradas em JSON; corrija o texto ou substitua-o por [] para recomeçar.
admin-policy-dlp-json-not-list = O texto tem de ser uma lista JSON de regras.
admin-policy-dlp-json-not-json = O texto não é JSON válido.
admin-policy-dlp-env-heading = Regras de MW_DLP_RULES
admin-policy-dlp-env-note = Definidas onde o servidor arranca; não podem ser alteradas aqui.
admin-policy-dlp-none = Não há regras guardadas.
admin-policy-dlp-add = Adicionar uma regra
admin-policy-dlp-on = Ativada
admin-policy-dlp-off = Desativada
admin-policy-dlp-rule-untitled = Regra { $n }
admin-policy-dlp-shadowed = Uma regra de MW_DLP_RULES usa o id { $id }, pelo que esta regra guardada não é aplicada.
admin-policy-dlp-name = Nome
admin-policy-dlp-action = Quando uma mensagem corresponde
admin-policy-dlp-action-warn = Avisar o remetente
admin-policy-dlp-action-block = Bloquear a mensagem
admin-policy-dlp-action-require-encryption = Exigir encriptação
admin-policy-dlp-action-notify-admin = Enviar e notificar um administrador
admin-policy-dlp-priority = Prioridade
admin-policy-dlp-priority-help = Os números mais altos são verificados primeiro.
admin-policy-dlp-enabled = Regra ativada
admin-policy-dlp-message = Mensagem para o remetente
admin-policy-dlp-message-help = Mostrada ao remetente quando a regra corresponde; deixe em branco para usar a frase padrão.
admin-policy-dlp-detectors = Procurar
admin-policy-dlp-detector-pan = Números de cartões de pagamento
admin-policy-dlp-detector-iban = Números de contas bancárias (IBAN)
admin-policy-dlp-detector-national-id = Números de identificação nacionais
admin-policy-dlp-detector-ssn = Números da segurança social dos EUA
admin-policy-dlp-detector-custom-regex = Um padrão próprio
admin-policy-dlp-pattern = Padrão
admin-policy-dlp-pattern-help = Uma expressão regular comparada com o texto da mensagem.
admin-policy-dlp-domains = Domínios dos destinatários
admin-policy-dlp-domains-none = Qualquer destinatário.
admin-policy-dlp-domains-placeholder = example.org
admin-policy-dlp-domain-mode = A regra aplica-se a
admin-policy-dlp-domain-mode-in = Destinatários nestes domínios
admin-policy-dlp-domain-mode-not-in = Destinatários fora destes domínios
admin-policy-dlp-types = Tipos de anexo
admin-policy-dlp-types-none = Qualquer tipo de anexo.
admin-policy-dlp-types-placeholder = pdf
admin-policy-dlp-size = Anexo maior, em bytes
admin-policy-dlp-size-help = Uma mensagem com um anexo maior corresponde; deixe em branco para não ter limite de tamanho.
admin-policy-dlp-kept = Esta regra também tem dicionários ou uma classificação, que são mantidos tal como guardados.
admin-policy-dlp-remove = Remover { $name }
admin-policy-dlp-err-name = A regra { $n } precisa de um nome.
admin-policy-dlp-err-id = A regra { $n } usa o id { $id }, que outra regra já tem.
admin-policy-dlp-err-priority = Regra { $n }: a prioridade tem de ser um número inteiro.
admin-policy-dlp-err-pattern-missing = A regra { $n } procura um padrão próprio, mas o padrão está vazio.
admin-policy-dlp-err-pattern = Regra { $n }: o padrão não é uma expressão regular válida.
admin-policy-dlp-err-size = Regra { $n }: o anexo maior tem de ser um número inteiro de bytes.
admin-policy-dlp-err-condition = A regra { $n } precisa de pelo menos uma coisa a procurar.
admin-policy-dlp-err-domain = { $domain } não é um nome de domínio.

## OAuth clients (dynamic client registration)

admin-oauth-title = Clientes OAuth
admin-oauth-intro = Se as aplicações se podem registar como clientes OAuth, para que endereços podem reencaminhar as pessoas e o que podem fazer.
admin-oauth-unavailable = este servidor não oferece definições de registo de clientes.
admin-oauth-off = Desativado
admin-oauth-group-registration = Registo automático
admin-oauth-enabled = Permitir que as aplicações se registem
admin-oauth-enabled-help = Abre /oauth/register às aplicações (RFC 7591); os clientes adicionados por outros meios não são afetados.
admin-oauth-enabled-needs-hosts = Adicione primeiro pelo menos um anfitrião de redirecionamento: com a lista vazia, todos os registos são recusados.
admin-oauth-hosts = Anfitriões de redirecionamento permitidos
admin-oauth-hosts-help = Uma aplicação só pode registar endereços de redirecionamento nestes anfitriões ou nos seus subdomínios; example.org também permite app.example.org.
admin-oauth-hosts-none = Nenhum anfitrião: todos os registos são recusados.
admin-oauth-hosts-placeholder = example.org
admin-oauth-hosts-invalid = { $host } não é um nome de anfitrião.
admin-oauth-token = Exigir o token de registo partilhado
admin-oauth-token-help = O registo passa a exigir o token bearer definido em MW_DCR_INITIAL_ACCESS_TOKEN onde o servidor arranca; sem essa variável, todos os registos são recusados.
admin-oauth-group-scope = O que recebe uma aplicação registada
admin-oauth-group-scope-help = Todas as aplicações registadas automaticamente recebem exatamente este âmbito; o que uma aplicação pede nunca o alarga.
admin-oauth-scope = Âmbito concedido
admin-oauth-scope-help = As permissões e as partes da conta que uma aplicação registada automaticamente pode usar depois de uma pessoa a aprovar.
admin-oauth-scope-nothing = Nada
admin-oauth-scope-permissions = Permissões
admin-oauth-scope-surfaces = Partes da conta
admin-oauth-scope-read = Ler
admin-oauth-scope-send = Enviar
admin-oauth-scope-delete = Eliminar
admin-oauth-scope-mail = Correio
admin-oauth-scope-pim = Calendário, contactos, tarefas e notas
admin-oauth-scope-rate = Pedidos por minuto
admin-oauth-scope-rate-help = Deixe em branco para não ter limite.
admin-oauth-scope-needs-surface = Escolha também o correio ou o calendário e os contactos: uma permissão sem uma parte da conta não concede nada.
admin-oauth-scope-rate-invalid = O limite tem de ser um número inteiro igual ou superior a 1.

admin-policy-dlp-detector-pt-nif = Número de identificação fiscal português (NIF, dígito de controlo)
admin-policy-dlp-detector-pt-niss = Número de Segurança Social português (NISS, dígito de controlo)
admin-policy-dlp-detector-pt-citizen-card = Cartão de Cidadão / identificação civil portuguesa (dígito de controlo)
admin-policy-dlp-detector-credentials = Palavras-chave de palavras-passe e credenciais (inglês / português)
admin-policy-dlp-keywords = Palavras-chave e frases personalizadas
admin-policy-dlp-keywords-placeholder = p. ex. segredo interno
admin-policy-dlp-keywords-none = Sem palavras-chave personalizadas.
admin-policy-dlp-keywords-help = Palavras ou frases literais, sem distinção entre maiúsculas e minúsculas. Corresponde a palavras inteiras; sem expressões regulares. O detetor de credenciais inclui os termos password, senha, palavra-passe, API key e private-key. As verificações de identificadores validam o formato, não se um número foi emitido.

admin-policy-dlp-enforcement-note = Os envios nativos são verificados novamente imediatamente antes da expedição. Qualquer regra ativada desativa os envios através de proxy a montante, porque não é possível garantir o seu conteúdo de saída exato. Uma regra «Exigir encriptação» correspondente recusa o envio; os conteúdos encriptados não são desencriptados nem certificados por esta política.
