# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Acima
nextcloud-loading = Carregando…
nextcloud-list-error = Não foi possível listar esta pasta.
nextcloud-empty = Esta pasta está vazia.
nextcloud-file-list = Arquivos do Nextcloud
nextcloud-open-folder = Abrir a pasta { $name }
nextcloud-select-file = Selecionar { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Anexar do Nextcloud
nextcloud-attach-action = { $count ->
    [0] Anexar
    [one] Anexar { $count } arquivo
   *[other] Anexar { $count } arquivos
  }
nextcloud-error-select-file = selecione pelo menos um arquivo para anexar
nextcloud-error-attach-failed = não foi possível anexar os arquivos selecionados

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Salvar no Nextcloud
nextcloud-save-title = Salvar “{ $name }” no Nextcloud
nextcloud-save-action = Salvar aqui ({ $dir })
nextcloud-saved = Salvo em { $path }
nextcloud-error-save-failed = não foi possível salvar no Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Criar link de compartilhamento
nextcloud-share-title = Link de compartilhamento
nextcloud-share-intro = Crie um link público para { $path } em vez de anexar o arquivo.
nextcloud-protect-password = Proteger com senha
nextcloud-password-label = Senha
nextcloud-share-password = Senha do compartilhamento
nextcloud-set-expiry = Definir data de expiração
nextcloud-expires-on = Expira em
nextcloud-expiry-date = Data de expiração
nextcloud-create-link = Criar link
nextcloud-password-protected = Protegido por senha
nextcloud-no-password = Sem senha
nextcloud-expires = expira em { $date }
nextcloud-no-expiry = sem expiração
nextcloud-error-need-password = digite uma senha ou desative a proteção por senha
nextcloud-error-need-expiry = escolha uma data de expiração ou desative a expiração
nextcloud-error-share-failed = não foi possível criar o link de compartilhamento

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } é maior que o limite de anexos. Envie-o para a pasta “Mail attachments” do seu Nextcloud e insira um link de compartilhamento na mensagem.
nextcloud-upload-and-link = Enviar e inserir link
nextcloud-share-not-now = Agora não
nextcloud-upload-progress = Enviando: { $percent }%
nextcloud-upload-progress-label = Progresso do envio
nextcloud-share-inserted = Link de compartilhamento de { $name } inserido na mensagem.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = O link funciona até { $date }.
nextcloud-share-body-password = O link é protegido por senha.
nextcloud-account-title = Contas do Nextcloud
nextcloud-account-help = Vincule sua própria conta a uma instância aprovada pelo administrador. As credenciais são armazenadas criptografadas, somente para esta conta de e-mail.
nextcloud-account-load-error = Não foi possível carregar os vínculos do Nextcloud. Recarregue para tentar novamente.
nextcloud-account-unlink-error = Não foi possível remover o vínculo. Recarregue antes de tentar novamente.
nextcloud-account-reload = Recarregar
nextcloud-account-no-instances = Nenhuma instância do Nextcloud ativada está disponível. Peça ao administrador para configurar uma.
nextcloud-account-ready = Vinculada e pronta
nextcloud-account-stale = A configuração da instância mudou. Verifique o vínculo novamente.
nextcloud-account-unlinked = Não vinculada
nextcloud-account-relink = Verificar vínculo novamente
nextcloud-account-link = Vincular conta
nextcloud-account-unlink = Desvincular conta
nextcloud-account-password-help = Nas configurações de segurança pessoais do Nextcloud, crie uma senha de aplicativo e copie o nome de usuário e a senha dela aqui. A verificação faz uma única solicitação limitada; credenciais recusadas não são tentadas de novo.
nextcloud-account-username = Nome de usuário da senha de aplicativo do Nextcloud
nextcloud-account-password = Senha de aplicativo do Nextcloud
nextcloud-account-verify = Verificar e vincular
nextcloud-account-verify-error = A verificação ou o salvamento falhou. Confira a instância, o nome de usuário e a senha de aplicativo. O campo de senha foi limpo; um vínculo existente que funcionava não foi substituído.
nextcloud-admin-title = Instâncias do Nextcloud
nextcloud-admin-help = Aprove instâncias HTTPS que os usuários de e-mail podem vincular individualmente. Nenhuma senha de serviço compartilhada é dada às contas de e-mail. As operações com arquivos são limitadas a 8 MiB. Os usuários do mecanismo nativo descobrem e conectam explicitamente as coleções de agendas, tarefas e catálogos de endereços depois de vincular.
nextcloud-admin-add = Adicionar instância do Nextcloud
nextcloud-admin-edit = Editar instância do Nextcloud
nextcloud-admin-enabled = Ativada
nextcloud-admin-disabled = Desativada
nextcloud-admin-missing = Esta instância não existe mais. Volte para a lista de instâncias.
nextcloud-admin-private-help = Endereços privados exigem uma permissão de egress do Nextcloud explícita. As rotas de proxy configuradas continuam prevalecendo; uma solicitação recusada pelo proxy nunca recorre a uma conexão direta.
nextcloud-admin-revalidation = Salvar qualquer alteração em uma instância exige que os usuários verifiquem seus vínculos novamente. Desativar uma instância interrompe imediatamente as novas operações.
nextcloud-admin-authority = Autoridade da instância
nextcloud-admin-name = Nome de exibição
nextcloud-admin-name-help = Um nome curto mostrado aos usuários ao escolher uma instância.
nextcloud-admin-url = URL de instalação HTTPS
nextcloud-admin-url-help = Use a raiz da instalação, como https://cloud.example.org/ ou https://example.org/nextcloud/. O certificado deve ser confiável e corresponder ao nome do host. Credenciais, parâmetros de consulta e fragmentos são recusados.
nextcloud-admin-enable = Permitir que os usuários vinculem esta instância
nextcloud-admin-ca = Autoridades certificadoras adicionais (PEM)
nextcloud-admin-ca-help = Certificados de CA privadas opcionais para este servidor Nextcloud. A confiança pública continua disponível e a verificação do nome do host continua obrigatória. As alterações exigem que os usuários verifiquem seus vínculos novamente; a confiança nos certificados do proxy é separada.
nextcloud-instance-select = Instância do Nextcloud
nextcloud-instance-placeholder = Escolha uma instância vinculada
nextcloud-instances-empty = Nenhum vínculo do Nextcloud verificado. Adicione ou verifique um vínculo em Configurações do usuário → Nextcloud.
nextcloud-instances-error = Não foi possível carregar as instâncias vinculadas. Reabra o seletor para tentar novamente.
nextcloud-save-warning = O administrador exige confirmação antes que este anexo possa sair da caixa de correio.
nextcloud-save-acknowledge = Entendo o aviso sobre o anexo e quero salvar este arquivo na instância do Nextcloud selecionada.
nextcloud-save-attachment = Salvar { $name } no Nextcloud
nextcloud-save-close = Fechar destino do Nextcloud
nextcloud-save-options = Salvar anexos no Nextcloud
nextcloud-pim-title = Contatos e agendas
nextcloud-pim-help = Descubra as coleções desta conta verificada do Nextcloud e conecte explicitamente as que você quiser. Sincronize para obter os dados atuais; as edições nas coleções conectadas usam a coleção remota e falham se o vínculo ou as permissões forem revogados.
nextcloud-pim-category = Categoria da coleção
nextcloud-pim-calendar = Agendas
nextcloud-pim-contacts = Catálogos de endereços
nextcloud-pim-tasks = Listas de tarefas
nextcloud-pim-discover = Descobrir coleções
nextcloud-pim-discovered = Coleções disponíveis
nextcloud-pim-empty = Nenhuma coleção correspondente foi retornada.
nextcloud-pim-connect = Conectar
nextcloud-pim-connected = Coleções conectadas
nextcloud-pim-sync = Sincronizar agora
nextcloud-pim-sync-needed = Coleção conectada. Sincronize agora para carregar o conteúdo.
nextcloud-pim-synced = Sincronização concluída.
nextcloud-pim-error = A operação falhou. Confira o vínculo da conta e as permissões e tente novamente explicitamente. Os dados em cache existentes não foram descartados.
nextcloud-pim-native-required = A sincronização de contatos e agenda exige o mecanismo de e-mail nativo. O acesso a arquivos continua disponível pelo vínculo da conta.
nextcloud-pim-disconnect = Desconectar
nextcloud-pim-disconnect-help = Desconectar preserva os dados em cache e interrompe as gravações remotas. Contas vinculadas novamente exigem nova descoberta e conexão explícita.
nextcloud-pim-ready = Pronta
nextcloud-pim-disconnected = Desconectada ou vínculo alterado
nextcloud-pim-unnamed = Coleção sem nome
nextcloud-pim-back = Voltar para as contas do Nextcloud
nextcloud-pim-create-title = Nova coleção no Nextcloud
nextcloud-pim-create-help = Cria uma agenda ou lista de tarefas vazia nesta conta do Nextcloud e a conecta aqui.
nextcloud-pim-create-name = Nome da nova coleção
nextcloud-pim-create = Criar no Nextcloud
nextcloud-pim-created = { $name } foi criada no Nextcloud e conectada. Sincronize agora para carregá-la.
nextcloud-pim-interval = Intervalo de atualização automática (segundos)
nextcloud-pim-interval-help = Zero usa somente a atualização manual. Os intervalos positivos têm mínimo de 60 segundos. As contas conectadas são atualizadas em segundo plano; as falhas aumentam o intervalo e credenciais recusadas interrompem novas solicitações até você vincular novamente.
nextcloud-pim-interval-reset = Usar o padrão do administrador
nextcloud-admin-empty = Nenhuma instância aprovada ainda. Adicione uma instância para torná-la disponível para vincular contas.
nextcloud-admin-status = Status
nextcloud-admin-open = Abrir { $name }
