# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Acima
nextcloud-loading = A carregar…
nextcloud-list-error = Não foi possível listar esta pasta.
nextcloud-empty = Esta pasta está vazia.
nextcloud-file-list = Ficheiros do Nextcloud
nextcloud-open-folder = Abrir a pasta { $name }
nextcloud-select-file = Selecionar { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Anexar do Nextcloud
nextcloud-attach-action = { $count ->
    [0] Anexar
    [one] Anexar { $count } ficheiro
   *[other] Anexar { $count } ficheiros
  }
nextcloud-error-select-file = selecione pelo menos um ficheiro para anexar
nextcloud-error-attach-failed = não foi possível anexar os ficheiros selecionados

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Guardar no Nextcloud
nextcloud-save-title = Guardar «{ $name }» no Nextcloud
nextcloud-save-action = Guardar aqui ({ $dir })
nextcloud-saved = Guardado em { $path }
nextcloud-error-save-failed = não foi possível guardar no Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Criar ligação de partilha
nextcloud-share-title = Ligação de partilha
nextcloud-share-intro = Crie uma ligação pública para { $path } em vez de anexar o ficheiro.
nextcloud-protect-password = Proteger com palavra-passe
nextcloud-password-label = Palavra-passe
nextcloud-share-password = Palavra-passe da partilha
nextcloud-set-expiry = Definir uma data de expiração
nextcloud-expires-on = Expira em
nextcloud-expiry-date = Data de expiração
nextcloud-create-link = Criar ligação
nextcloud-password-protected = Protegida por palavra-passe
nextcloud-no-password = Sem palavra-passe
nextcloud-expires = expira em { $date }
nextcloud-no-expiry = sem expiração
nextcloud-error-need-password = introduza uma palavra-passe ou desative a proteção por palavra-passe
nextcloud-error-need-expiry = escolha uma data de expiração ou desative a expiração
nextcloud-error-share-failed = não foi possível criar a ligação de partilha
# Anexo grande (t39): um ficheiro acima do limite de anexos é carregado e depois partilhado.
nextcloud-share-upload-intro = { $path } é maior do que o limite de anexos. Carregue-o para a pasta “Mail attachments” do seu Nextcloud e insira uma ligação de partilha na mensagem.
nextcloud-upload-and-link = Carregar e inserir ligação
nextcloud-share-not-now = Agora não
nextcloud-upload-progress = A carregar: { $percent }%
nextcloud-upload-progress-label = Progresso do carregamento
nextcloud-share-inserted = Ligação de partilha de { $name } inserida na mensagem.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = A ligação funciona até { $date }.
nextcloud-share-body-password = A ligação está protegida por palavra-passe.

# Account-owned integration configuration
nextcloud-account-title = Contas Nextcloud
nextcloud-account-help = Associe a sua conta a uma instância aprovada pelo seu administrador. As credenciais são guardadas encriptadas apenas para esta conta de correio.
nextcloud-account-load-error = Não foi possível carregar as associações ao Nextcloud. Recarregue para tentar novamente.
nextcloud-account-unlink-error = Não foi possível remover a associação. Recarregue antes de tentar novamente.
nextcloud-account-reload = Recarregar
nextcloud-account-no-instances = Não há instâncias Nextcloud ativadas disponíveis. Peça ao seu administrador que configure uma.
nextcloud-account-ready = Associada e pronta
nextcloud-account-stale = A configuração da instância foi alterada. Verifique novamente a associação.
nextcloud-account-unlinked = Não associada
nextcloud-account-relink = Verificar novamente a associação
nextcloud-account-link = Associar conta
nextcloud-account-unlink = Desassociar conta
nextcloud-account-password-help = Nas definições de segurança pessoais do Nextcloud, crie uma palavra-passe de aplicação e copie aqui o respetivo nome de utilizador e palavra-passe. A verificação faz um único pedido limitado; as credenciais rejeitadas não são tentadas novamente.
nextcloud-account-username = Nome de utilizador da palavra-passe de aplicação do Nextcloud
nextcloud-account-password = Palavra-passe de aplicação do Nextcloud
nextcloud-account-verify = Verificar e associar
nextcloud-account-verify-error = A verificação ou a gravação falhou. Verifique a instância, o nome de utilizador da palavra-passe de aplicação e a palavra-passe. O campo da palavra-passe foi limpo; uma associação existente que funcione não foi substituída.
nextcloud-admin-title = Instâncias Nextcloud
nextcloud-admin-help = Aprove instâncias HTTPS que os utilizadores de correio possam associar individualmente. Nenhuma palavra-passe de serviço partilhada é dada às contas de correio. As operações com ficheiros estão limitadas a 8 MiB. Os utilizadores do motor nativo detetam e ligam explicitamente as coleções de calendários, tarefas e livros de endereços depois da associação.
nextcloud-admin-add = Adicionar instância Nextcloud
nextcloud-admin-edit = Editar instância Nextcloud
nextcloud-admin-enabled = Ativada
nextcloud-admin-disabled = Desativada
nextcloud-admin-missing = Esta instância já não existe. Volte à lista de instâncias.
nextcloud-admin-private-help = Os endereços privados exigem uma permissão de saída explícita para o Nextcloud. As rotas de proxy configuradas continuam a prevalecer; um pedido recusado pelo proxy nunca recorre a uma ligação direta.
nextcloud-admin-revalidation = Guardar qualquer alteração a uma instância obriga os utilizadores a verificar novamente as suas associações. Desativar uma instância interrompe imediatamente novas operações.
nextcloud-admin-authority = Autoridade da instância
nextcloud-admin-name = Nome a apresentar
nextcloud-admin-name-help = Um nome curto mostrado aos utilizadores quando escolhem uma instância.
nextcloud-admin-url = URL HTTPS da instalação
nextcloud-admin-url-help = Use a raiz da instalação, por exemplo https://cloud.example.org/ ou https://example.org/nextcloud/. O certificado tem de ser fidedigno e corresponder ao nome de anfitrião. Credenciais, parâmetros de query e fragmentos são recusados.
nextcloud-admin-enable = Permitir que os utilizadores associem esta instância

nextcloud-admin-ca = Autoridades de certificação adicionais (PEM)
nextcloud-admin-ca-help = Certificados de AC privadas opcionais para este servidor Nextcloud. A confiança pública continua disponível e a verificação do nome de anfitrião continua obrigatória. As alterações obrigam os utilizadores a verificar novamente as suas associações; a confiança nos certificados do proxy é separada.

nextcloud-instance-select = Instância Nextcloud
nextcloud-instance-placeholder = Escolha uma instância associada
nextcloud-instances-empty = Não há associações Nextcloud verificadas. Adicione ou verifique uma associação em «Definições do utilizador → Nextcloud».
nextcloud-instances-error = Não foi possível carregar as instâncias associadas. Volte a abrir o seletor para tentar novamente.

nextcloud-save-warning = O seu administrador exige confirmação antes de este anexo poder sair da caixa de correio.
nextcloud-save-acknowledge = Compreendo o aviso sobre o anexo e pretendo guardar este ficheiro na instância Nextcloud selecionada.
nextcloud-save-attachment = Guardar { $name } no Nextcloud
nextcloud-save-close = Fechar o destino Nextcloud
nextcloud-save-options = Guardar anexos no Nextcloud
nextcloud-pim-title = Contactos e calendários
nextcloud-pim-help = Detete as coleções desta conta Nextcloud verificada e ligue explicitamente as que pretende. Sincronize para obter os dados atuais; as edições ligadas usam a coleção remota e falham se a respetiva associação ou permissões forem revogadas.
nextcloud-pim-category = Categoria da coleção
nextcloud-pim-calendar = Calendários
nextcloud-pim-contacts = Livros de endereços
nextcloud-pim-tasks = Listas de tarefas
nextcloud-pim-discover = Detetar coleções
nextcloud-pim-discovered = Coleções disponíveis
nextcloud-pim-empty = Não foi devolvida nenhuma coleção correspondente.
nextcloud-pim-connect = Ligar
nextcloud-pim-connected = Coleções ligadas
nextcloud-pim-sync = Sincronizar agora
nextcloud-pim-sync-needed = Coleção ligada. Sincronize agora para carregar o seu conteúdo.
nextcloud-pim-synced = Sincronização concluída.
nextcloud-pim-error = A operação falhou. Verifique a associação da conta e as permissões e tente novamente de forma explícita. Os dados em cache existentes não foram descartados.
nextcloud-pim-native-required = A sincronização de contactos e calendários exige o motor de correio nativo. O acesso aos ficheiros continua disponível através da associação da sua conta.
nextcloud-pim-disconnect = Desligar
nextcloud-pim-disconnect-help = Desligar preserva os dados em cache e interrompe as escritas remotas. As contas associadas novamente exigem uma nova deteção e ligação explícita.
nextcloud-pim-ready = Pronta
nextcloud-pim-disconnected = Desligada ou associação alterada
nextcloud-pim-unnamed = Coleção sem nome
nextcloud-pim-back = Voltar às contas Nextcloud
nextcloud-pim-create-title = Nova coleção no Nextcloud
nextcloud-pim-create-help = Cria um calendário ou uma lista de tarefas vazios nesta conta Nextcloud e liga-os aqui.
nextcloud-pim-create-name = Nome da nova coleção
nextcloud-pim-create = Criar no Nextcloud
nextcloud-pim-created = { $name } foi criado no Nextcloud e ligado. Sincronize agora para o carregar.
nextcloud-pim-interval = Intervalo de atualização automática (segundos)
nextcloud-pim-interval-help = Zero significa apenas atualização manual. Os intervalos positivos têm um mínimo de 60 segundos. As contas associadas são atualizadas em segundo plano; as falhas aumentam o intervalo entre tentativas e as credenciais rejeitadas interrompem novos pedidos até voltar a associar a conta.
nextcloud-pim-interval-reset = Usar a predefinição do administrador
nextcloud-admin-empty = Ainda não há instâncias aprovadas. Adicione uma instância para a disponibilizar para associação de contas.
nextcloud-admin-status = Estado
nextcloud-admin-open = Abrir { $name }
