# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Administração > Conformidade > Cifragem: o que está cifrado em repouso e como foi determinado.
# Descreve a configuração e o conteúdo observado dos ficheiros. Não certifica conformidade com qualquer lei ou norma.

admin-encryption-title = Cifragem em repouso
admin-encryption-intro = Onde este servidor guarda dados em disco, se os bytes aí guardados estão cifrados e como cada resposta foi obtida. Uma área só é "cifrada" quando o conteúdo do ficheiro foi lido e é texto cifrado. Os estados indicados descrevem a configuração deste servidor; por si só não tornam uma instalação conforme com qualquer lei ou norma.
admin-encryption-docs = As origens da chave, os comandos de migração, a rotação, a recuperação e o que significa perder a chave estão em docs/admin/encryption.md.
admin-encryption-loading = A carregar o estado da cifragem…
admin-encryption-summary = Resumo
admin-encryption-tile-database = Base de dados
admin-encryption-tile-mode = Modo
admin-encryption-tile-key = Origem da chave
admin-encryption-tile-rotation = Rotação prevista
admin-encryption-mode-off = Desligado
admin-encryption-mode-optional = Opcional
admin-encryption-mode-required = Obrigatório
admin-encryption-key-none = Não configurada
admin-encryption-no-due = Sem data prevista

admin-encryption-findings = Pontos a rever
admin-encryption-warning-databasePlaintext = O ficheiro da base de dados é legível como SQLite simples. As credenciais, os corpos das mensagens e os ficheiros enviados estão selados no seu interior; o resto não.
admin-encryption-warning-modeNotRequired = A base de dados está cifrada mas o modo é opcional, pelo que um ficheiro simples colocado no lugar seria aberto sem aviso. Defina MW_DB_ENCRYPTION=required.
admin-encryption-warning-searchIndexPlaintext = O índice de pesquisa em disco guarda o texto das mensagens em claro. Mude-o para memória (MW_SEARCH_DIR=memory) ou coloque a pasta num volume cifrado.
admin-encryption-warning-plaintextRollbackCopy = Uma cópia simples da base de dados de uma migração anterior continua junto ao ficheiro da base de dados. Apague-a depois de verificar o resultado.
admin-encryption-warning-rotationOverdue = A chave da base de dados ultrapassou o intervalo de rotação.
admin-encryption-warning-keyBesideDatabase = O ficheiro da chave está na mesma pasta que a base de dados. Quem consiga copiar a base de dados consegue copiar a chave.
admin-encryption-warning-postgresNoTls = A ligação ao Postgres não usa TLS.

admin-encryption-stores = Áreas
admin-encryption-stores-note = Uma linha por local onde se guardam dados. "Só colunas seladas" significa que a área pode ser lida no seu conjunto mas as colunas sensíveis no interior estão seladas com a chave do servidor.
admin-encryption-col-store = Área
admin-encryption-col-state = Estado
admin-encryption-col-evidence = Como foi determinado
admin-encryption-store-database = Base de dados
admin-encryption-store-searchIndex = Índice de pesquisa
admin-encryption-store-uploads = Ficheiros enviados
admin-encryption-store-backups = Cópias de segurança da base de dados
admin-encryption-store-logs = Ficheiros de registo
admin-encryption-store-exports = Exportações de dados
admin-encryption-store-archive = Arquivo de retenção
admin-encryption-store-secrets = Segredos do servidor
admin-encryption-state-encrypted = Cifrado
admin-encryption-state-sealed-columns-only = Só colunas seladas
admin-encryption-state-plaintext = Texto simples
admin-encryption-state-memory-only = Só em memória
admin-encryption-state-not-stored = Não guardado
admin-encryption-state-unknown = Desconhecido
admin-encryption-counts = Cifrado: { $encrypted }. Só colunas seladas: { $sealed_columns_only }. Texto simples: { $plaintext }. Só em memória: { $memory_only }. Não guardado: { $not_stored }. Desconhecido: { $unknown }.

admin-encryption-evidence-db-header = O ficheiro da base de dados foi lido: os primeiros 16 bytes não são o cabeçalho SQLite. { $cipher }. O registo de escrita antecipada usa a mesma chave.
admin-encryption-evidence-db-plain-header = O ficheiro da base de dados começa com o cabeçalho SQLite, pelo que é legível sem chave.
admin-encryption-evidence-db-empty = O ficheiro da base de dados está vazio ou não existe.
admin-encryption-evidence-db-memory = A base de dados está em memória; nada é escrito em disco.
admin-encryption-evidence-db-postgres = O disco do servidor não pode ser observado a partir desta aplicação. A secção Postgres abaixo indica o que a ligação revela. A cifragem do disco ou do volume é da responsabilidade do operador.
admin-encryption-evidence-error = Não foi possível ler: { $error }
admin-encryption-evidence-search-disk = A pasta do índice contém { $files } ficheiros com texto de mensagens (remetente, destinatários, assunto, corpo, texto de anexos). Não estão cifrados.
admin-encryption-evidence-search-memory = O índice é mantido em memória e reconstruído a partir da base de dados no arranque.
admin-encryption-evidence-uploads = Cada objeto é selado com a chave do servidor antes de ser escrito. Foram amostrados { $sampled } ficheiros: { $plainLooking } pareciam um formato simples conhecido.
admin-encryption-evidence-uploads-off = Não há pasta de envios configurada, por isso nada é guardado.
admin-encryption-evidence-backups-leftover = { $leftovers } cópia(s) de reversão de uma migração anterior estão junto à base de dados e pelo menos uma é texto simples.
admin-encryption-evidence-backups-unseen = O servidor não cria cópias de segurança, por isso o seu estado não pode ser observado. `mailwoman db backup` escreve cópias cifradas; as cópias do deploy.sh podem ser cifradas com age ou gpg.
admin-encryption-evidence-logs-file = Os ficheiros de registo em { $directory } são escritos como texto com endereços, endereços de clientes e tokens ocultados. Não estão cifrados.
admin-encryption-evidence-logs-stdout = Os registos vão apenas para a saída padrão; quem os recolhe decide como são guardados.
admin-encryption-evidence-exports = As exportações de dados ficam na base de dados. Ficam cobertas quando a base de dados está cifrada; caso contrário, só se a selagem das exportações estiver ativa (selagem ativa: { $dsrExportSealed }).
admin-encryption-evidence-archive = Os ficheiros do arquivo de retenção são comprimidos e têm um selo de integridade (HMAC), que deteta alterações mas não oculta o conteúdo. Mantenha a pasta do arquivo num volume cifrado.
admin-encryption-evidence-secrets = A chave do servidor está guardada como: { $serverKey }. É uma chave diferente da chave da base de dados. Os ficheiros de chave são ficheiros simples protegidos apenas por permissões.

admin-encryption-key = Chave da base de dados
admin-encryption-key-source = Origem da chave
admin-encryption-key-fingerprint = Impressão da chave
admin-encryption-key-age = Idade da chave
admin-encryption-age-unknown = Desconhecida para esta origem de chave
admin-encryption-age-days = { $days } dias
admin-encryption-key-rotation = Rotação
admin-encryption-rotation-every = A cada { $days } dias
admin-encryption-overdue = Em atraso

admin-encryption-actions = Verificações e comandos
admin-encryption-actions-note = A verificação lê o ficheiro da base de dados e testa a chave. O ensaio exporta uma cópia dos dados atuais com uma chave descartável, compara-a tabela a tabela e apaga-a. A alteração da chave ou a conversão do ficheiro faz-se com os comandos abaixo, com o servidor parado.
admin-encryption-verify = Verificar cifragem
admin-encryption-verifying = A verificar…
admin-encryption-rehearse = Ensaiar uma migração
admin-encryption-rehearsing = A executar…
admin-encryption-refresh = Atualizar
admin-encryption-verify-passed = Verificado: o ficheiro da base de dados está cifrado e a chave funciona.
admin-encryption-verify-failed = Não verificado.
admin-encryption-pass = Passou
admin-encryption-fail = Falhou
admin-encryption-check-header-not-sqlite = O ficheiro não começa com o cabeçalho SQLite
admin-encryption-check-wrong-key-rejected = Uma chave aleatória errada é recusada
admin-encryption-check-right-key-opens = A chave configurada abre o ficheiro
admin-encryption-check-row-round-trip = Uma linha conhecida é escrita, lida numa nova ligação e removida
admin-encryption-check-marker-not-in-file = O texto dessa linha não aparece nos bytes do ficheiro
admin-encryption-check-key-configured = Existe uma chave da base de dados configurada
admin-encryption-check-sqlite-file = A base de dados é um ficheiro SQLite
admin-encryption-rehearse-ok = A cópia coincidiu com a origem: { $tables } tabelas, { $rows } linhas, { $millis } ms. Nada foi alterado.
admin-encryption-rehearse-failed = O ensaio falhou: { $error }
admin-encryption-runbook = Comandos
admin-encryption-runbook-note = Execute-os no servidor com o servidor parado. A chave é lida de MW_DB_KEY, MW_DB_KEY_FILE ou MW_DB_KEY_COMMAND; uma chave nova, dos equivalentes MW_DB_NEW_*.

admin-encryption-postgres = Postgres
admin-encryption-postgres-note = O que esta ligação consegue ver. Nada disto prova que o disco do servidor está cifrado, e a aplicação não o cifra.
admin-encryption-pg-connection = Esta ligação
admin-encryption-pg-tls-on = TLS ({ $version })
admin-encryption-pg-tls-off = Sem TLS
admin-encryption-pg-unknown = Não foi possível ler
admin-encryption-pg-server-ssl = Definição ssl do servidor
admin-encryption-pg-checksums = data_checksums (deteção de corrupção, não cifragem)
admin-encryption-pg-pgcrypto = pgcrypto
admin-encryption-pg-installed = Instalada
admin-encryption-pg-available = Disponível, não instalada
admin-encryption-pg-absent = Não disponível
admin-encryption-pg-tde = Indicadores de cifragem transparente de dados
admin-encryption-pg-none-seen = Nenhum encontrado

admin-encryption-columns = Colunas seladas
admin-encryption-columns-note = Colunas que a aplicação sela com a chave do servidor antes de escrever, e algumas que não sela. A lista não é exaustiva: uma coluna ausente não significa nem que está selada nem que está em claro.
admin-encryption-col-table = Tabela
admin-encryption-col-column = Coluna
admin-encryption-col-holds = Contém
admin-encryption-col-sealed = Selada
admin-encryption-sealed = Selada
admin-encryption-not-sealed = Não selada
