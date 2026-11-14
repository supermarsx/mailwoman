# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Permitir o envio
admin-settings-help-mail-allow-send = Permite que os usuários enviem mensagens. Quando desativado, o editor desabilita Enviar e seu atalho e o servidor recusa os envios; os rascunhos ainda podem ser salvos.
admin-settings-key-mail-allow-reply = Permitir responder
admin-settings-help-mail-allow-reply = Permite que os usuários respondam e respondam a todos pelo leitor e com atalhos. Quando desativado, essas ações não são oferecidas.
admin-settings-key-mail-allow-forward = Permitir encaminhar
admin-settings-help-mail-allow-forward = Permite que os usuários encaminhem mensagens no corpo ou como anexo. Quando desativado, essas ações não são oferecidas.
admin-settings-key-mail-allow-download = Permitir downloads
admin-settings-help-mail-allow-download = Permite que os usuários baixem mensagens e anexos. Quando desativado, o aplicativo web não oferece download e o servidor recusa a exportação de mensagens.
admin-settings-key-mail-allow-print = Permitir imprimir
admin-settings-help-mail-allow-print = Permite que os usuários imprimam uma mensagem pelo leitor. Quando desativado, a ação Imprimir não é oferecida.
admin-settings-key-mail-show-spam = Mostrar a pasta Spam
admin-settings-help-mail-show-spam = Mostra a pasta Spam ou Junk e as mensagens marcadas como spam. Quando desativado, elas ficam ocultas da lista de pastas e dos resultados de pesquisa; nada é excluído.
admin-settings-key-mail-show-auth-status = Mostrar os resultados de autenticação do remetente
admin-settings-help-mail-show-auth-status = Mostra no leitor os resultados SPF, DKIM e DMARC do cabeçalho Authentication-Results.
admin-settings-key-mail-close-on-delete = Fechar uma mensagem ao excluí-la
admin-settings-help-mail-close-on-delete = Fecha a mensagem, a guia ou a janela separada abertas quando essa mensagem é excluída ou movida para a Lixeira. Quando desativado, o leitor mantém o comportamento anterior.
admin-settings-key-links-trusted-senders = Remetentes de links confiáveis
admin-settings-help-links-trusted-senders = Caixas de correio de remetentes exatos cujos links de mensagens revisados podem ser abertos sem consentimento por mensagem. Um endereço por entrada.
admin-settings-key-appearance-font-size = Tamanho da fonte da interface
admin-settings-help-appearance-font-size = Tamanho da fonte da interface em pixels, de 12 a 24.
admin-settings-key-appearance-font-weight = Peso da fonte da interface
admin-settings-help-appearance-font-weight = Peso base do texto da interface: 300, 400, 500 ou 600. O texto enfatizado mantém sua hierarquia.
admin-settings-key-appearance-font-color = Cor do texto da interface
admin-settings-help-appearance-font-color = Cor do texto da interface como #RRGGBB. Deixe vazio para seguir o tema.
admin-settings-key-appearance-low-contrast = Suavizar o contraste do texto
admin-settings-help-appearance-low-contrast = Suaviza o contraste do texto da interface.
admin-settings-key-mail-message-list-order = Ordem da lista de mensagens
admin-settings-help-mail-message-list-order = Ordem da lista de mensagens: mais recentes, mais antigas, remetente ou assunto.
admin-settings-key-mail-message-list-style = Estilo da lista de mensagens
admin-settings-help-mail-message-list-style = Mostrar a lista de mensagens como cartões ou como linhas com títulos de coluna.
admin-settings-key-mail-message-list-preview = Prévias da lista de mensagens
admin-settings-help-mail-message-list-preview = Mostrar trechos do corpo na lista de mensagens.
admin-settings-key-mail-after-delete = Após excluir uma mensagem
admin-settings-help-mail-after-delete = Qual mensagem o leitor mostra depois que a mensagem aberta é excluída: a próxima, a anterior ou nenhuma.
admin-settings-key-reader-attachments-at-end = Anexos após o corpo
admin-settings-help-reader-attachments-at-end = Mostrar os anexos depois do corpo da mensagem em vez de antes.
admin-settings-key-notifications-ask-permission = Oferecer permissão de notificações
admin-settings-help-notifications-ask-permission = Oferecer o pedido de permissão de notificações do navegador aos usuários em dispositivos que não permitiram nem bloquearam notificações.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Configurações
admin-settings-description = Valores para o servidor inteiro, por domínio e por usuário. Um valor bloqueado não pode ser alterado por usuários nem por escopos mais restritos.
admin-settings-list-caption = Configurações
admin-settings-search = Pesquisar configurações
admin-settings-empty = Nenhuma configuração corresponde à pesquisa.
admin-settings-unavailable = Este servidor ainda não oferece o modelo de configurações.
admin-settings-col-setting = Configuração
admin-settings-col-value = Valor
admin-settings-col-source = Origem
admin-settings-col-overrides = Valores mais restritos
admin-settings-open = Abrir
admin-settings-overrides-none = Nenhum
admin-settings-overrides = { $domains } domínios, { $users } usuários
admin-settings-not-found = Não existe configuração chamada { $key }.
admin-settings-value-on = Ativado
admin-settings-value-off = Desativado
admin-settings-value-empty = Não definido
admin-settings-value-list = { $count ->
    [one] { $count } item
   *[other] { $count } itens
 }
admin-settings-detail-key = Chave: { $key }
admin-settings-global = Para o servidor inteiro
admin-settings-global-help = Este valor se aplica a todos os usuários, a menos que um domínio ou um usuário tenha um valor mais restrito, ou que o usuário escolha o próprio valor onde isso for permitido.
admin-settings-value-label = Valor
admin-settings-json-hint = JSON. O servidor verifica o valor e indica a parte que recusa.
admin-settings-json-invalid = Este JSON não é válido.
admin-settings-user-override = Os usuários podem alterar
admin-settings-impact-user = { $count ->
    [one] { $count } escolha própria de usuário é ignorada
   *[other] { $count } escolhas próprias de usuários são ignoradas
 }
admin-settings-impact-admin-user = { $count ->
    [one] { $count } valor por usuário é ignorado
   *[other] { $count } valores por usuário são ignorados
 }
admin-settings-impact-profile = { $count ->
    [one] { $count } valor de perfil de recursos é ignorado
   *[other] { $count } valores de perfil de recursos são ignorados
 }
admin-settings-impact-group = { $count ->
    [one] { $count } valor de grupo é ignorado
   *[other] { $count } valores de grupo são ignorados
 }
admin-settings-domains = Valores por domínio
admin-settings-domains-help = Um valor de domínio se aplica aos usuários desse domínio em vez do valor do servidor inteiro.
admin-settings-domains-caption = Valores por domínio de { $setting }
admin-settings-domains-empty = Nenhum domínio tem valor próprio.
admin-settings-domain = Domínio
admin-settings-domain-value = Valor para este domínio
admin-settings-domain-locked = Bloqueado para os usuários do domínio
admin-settings-domain-add = Adicionar valor de domínio
admin-settings-domain-save = Salvar valor do domínio
admin-settings-domain-remove = Remover
admin-settings-domain-remove-title = Remover o valor de { $domain }?
admin-settings-domain-remove-body = Os usuários de { $domain } voltam a receber o valor do servidor inteiro.
admin-settings-domain-required = Informe um nome de domínio.
admin-settings-locked-yes = Bloqueado
admin-settings-locked-no = Não bloqueado
admin-settings-users = Valores por usuário
admin-settings-users-help = Um valor definido por um administrador para um usuário. Para limites, o usuário é identificado pelo endereço.
admin-settings-users-caption = Valores por usuário de { $setting }
admin-settings-users-empty = Nenhum usuário tem valor definido por um administrador.
admin-settings-user = Usuário
admin-settings-user-hint = ID da conta ou, para limites, o endereço.
admin-settings-user-value = Valor para este usuário
admin-settings-user-add = Adicionar valor por usuário
admin-settings-user-save = Salvar valor do usuário
admin-settings-user-required = Informe um usuário.
admin-settings-user-remove-title = Remover o valor de { $user }?
admin-settings-user-remove-body = O usuário passa a receber o valor do domínio ou o valor do servidor inteiro.
admin-settings-source-default = Padrão integrado
admin-settings-source-envFallback = Variável de ambiente (sem valor armazenado)
admin-settings-source-envPin = Variável de ambiente
admin-settings-source-global = Armazenado para o servidor inteiro
admin-settings-cat-interface = Interface
admin-settings-cat-appearance = Aparência
admin-settings-cat-mail = E-mail
admin-settings-cat-folders = Layout das pastas
admin-settings-cat-notifications = Notificações
admin-settings-cat-offline = Uso offline
admin-settings-cat-assistant = Assistente de IA
admin-settings-cat-upstream = Servidor de e-mail
admin-settings-cat-signin = Login
admin-settings-cat-admin = Console de administração
admin-settings-cat-passwords = Senhas
admin-settings-cat-limits = Limites e uso
admin-settings-cat-plugins = Plugins
admin-settings-cat-features = Permissões de recursos
admin-settings-cat-integrations = Permissões de integração
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Permite arquivos, compartilhamento e operações de calendário, contatos e tarefas do Nextcloud vinculados à conta.
admin-settings-key-cap-integration-google = Ponte do Google
admin-settings-help-cap-integration-google = Permite operações da ponte do Google, incluindo obtenção e renovação de tokens.
admin-settings-key-cap-integration-microsoft = Ponte da Microsoft
admin-settings-help-cap-integration-microsoft = Permite operações da ponte da Microsoft, incluindo obtenção e renovação de tokens.
admin-settings-key-cap-integration-family = Integração individual
admin-settings-help-cap-integration-family = Permissão para o identificador de uma integração individual; a cobertura em execução está listada no catálogo de recursos.
admin-settings-key-cap-ui-plugin-family = Plugin de interface individual
admin-settings-help-cap-ui-plugin-family = Permissão para um plugin de interface. Use a chave de política exata do catálogo de integrações.
admin-settings-key-cap-ai-assistant = Assistente de IA
admin-settings-help-cap-ai-assistant = Permite a configuração e a chamada do assistente.
admin-settings-key-cap-plugins = Plugins
admin-settings-help-cap-plugins = Permite operações de plugins de interface e de motor.
admin-settings-key-cap-pim-calendar = Calendário
admin-settings-help-cap-pim-calendar = Permite métodos de calendário e assinaturas.
admin-settings-key-cap-pim-contacts = Contatos
admin-settings-help-cap-pim-contacts = Permite métodos de contatos e de catálogo de endereços.
admin-settings-key-cap-pim-tasks = Tarefas
admin-settings-help-cap-pim-tasks = Permite métodos de tarefas.
admin-settings-key-cap-pim-notes = Notas
admin-settings-help-cap-pim-notes = Permite métodos de notas.
admin-settings-key-cap-export = Exportação
admin-settings-help-cap-export = Permite exportações de e-mail e PIM.
admin-settings-key-cap-import = Importação
admin-settings-help-cap-import = Permite importações de e-mail e PIM.
admin-settings-key-cap-api-keys = Chaves de API
admin-settings-help-cap-api-keys = Política de chaves de API. Consulte o catálogo de recursos para a cobertura de aplicação.
admin-settings-key-cap-oauth-apps = Aplicativos OAuth
admin-settings-help-cap-oauth-apps = Política de aplicativos OAuth. Consulte o catálogo de recursos para a cobertura de aplicação.
admin-settings-key-cap-rules = Regras de e-mail
admin-settings-help-cap-rules = Permite métodos gerenciados de regras de e-mail e de resposta automática.
admin-settings-key-cap-rules-forward = Encaminhamento por regras
admin-settings-help-cap-rules-forward = Política de encaminhamento. Consulte o catálogo de recursos para a cobertura de aplicação.
admin-settings-key-cap-remote-images = Imagens remotas
admin-settings-help-cap-remote-images = Capacidade de imagens remotas. As configurações de privacidade existentes continuam a regular as solicitações; verifique a cobertura do recurso.
admin-settings-key-cap-user-background = Plano de fundo personalizado
admin-settings-help-cap-user-background = Capacidade de plano de fundo personalizado, desativada por padrão. Verifique a cobertura do recurso.
admin-settings-key-cap-user-css = CSS personalizado
admin-settings-help-cap-user-css = Capacidade de CSS personalizado, desativada por padrão. Verifique a cobertura do recurso.
admin-settings-key-limit-attachment-bytes = Limite de tamanho de anexo
admin-settings-help-limit-attachment-bytes = Política de anexos em bytes, padrão 50000000. A aplicação em tempo de execução ainda está pendente.
admin-settings-key-limit-send-per-hour = Limite de envio por hora
admin-settings-help-limit-send-per-hour = Política de envio por hora; zero significa ilimitado. A aplicação em tempo de execução ainda está pendente.
admin-settings-key-limit-recipients-per-message = Destinatários por mensagem
admin-settings-help-limit-recipients-per-message = Política de número de destinatários; zero significa ilimitado. A aplicação em tempo de execução ainda está pendente.
admin-settings-cat-import = Importação
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Atualizações ao vivo
admin-settings-cat-privacy = Privacidade
admin-settings-cat-observability = Observabilidade
admin-settings-cat-egress = Conexões de saída
admin-settings-cat-updates = Atualizações
admin-settings-cat-backups = Backups
admin-settings-cat-about = Sobre
admin-settings-cat-config = Configuração
admin-settings-cat-jobs = Tarefas agendadas
admin-settings-cat-maintenance = Manutenção
admin-settings-cat-health = Verificações de integridade
admin-settings-cat-diagnostics = Diagnóstico
admin-settings-cat-users = Usuários
admin-settings-cat-server-text = Texto do servidor
admin-settings-key-ui-language = Idioma
admin-settings-help-ui-language = O idioma do cliente de e-mail para os usuários que não escolheram um. "Igual ao navegador" segue os idiomas de cada navegador.
admin-settings-key-ui-notify-new-version = Aviso de nova versão
admin-settings-help-ui-notify-new-version = Ativado (padrão) mostra um aviso com "Recarregar agora" e "Depois" aos usuários conectados quando uma versão mais nova do aplicativo web foi implantada depois que a página deles carregou. Desativado nunca mostra; os usuários recebem o código novo na próxima recarga.
admin-settings-key-ui-undo-history-size = Tamanho do histórico de desfazer e refazer
admin-settings-help-ui-undo-history-size = Quantas ações recentes de e-mail (arquivar, excluir, mover, marcar como lida, sinalizar, marcar com marcador e semelhantes) os botões Desfazer e Refazer lembram, de 0 a 50. 0 desativa os botões e o atalho Ctrl+Z. O padrão é 50. Os usuários podem escolher o próprio tamanho, nunca acima do máximo abaixo, a menos que um administrador bloqueie.
admin-settings-key-folders-reorder-enabled = Arrastar e soltar pastas
admin-settings-help-folders-reorder-enabled = Permite reorganizar as pastas arrastando-as na barra lateral e em Configurações. Desativado oculta as alças de arrastar e os comandos Mover. Os usuários podem desativar para si mesmos, a menos que isto esteja bloqueado.
admin-settings-key-folders-reorder-allow-nesting = Aninhamento de pastas
admin-settings-help-folders-reorder-allow-nesting = Permite soltar uma pasta sobre outra para aninhá-la, ou de volta ao nível superior. Desativado mantém cada pasta sob o pai atual e permite reordenar apenas entre irmãs.
admin-settings-key-folders-reorder-allow-server-move = Mover pastas no servidor
admin-settings-help-folders-reorder-allow-server-move = Permite que aninhar ou desaninhar altere o pai real da pasta no servidor de e-mail, valendo em todo lugar. Desativado mantém essas movimentações apenas como organização neste aplicativo.
admin-settings-key-folders-reorder-allow-system-folders = Mover pastas do sistema
admin-settings-help-folders-reorder-allow-system-folders = Permite arrastar Caixa de entrada, Rascunhos, Enviados, Spam, Lixeira e Arquivo para reordená-las. Desativado as mantém fixas no topo. Elas só são reordenadas neste aplicativo, nunca renomeadas nem movidas no servidor.
admin-settings-key-folders-reorder-max-depth = Profundidade de aninhamento de pastas
admin-settings-help-folders-reorder-max-depth = O nível mais profundo de aninhamento que um arraste pode criar, de 1 a 16. Uma movimentação que colocaria uma pasta ou uma de suas subpastas mais fundo é recusada.
admin-settings-key-folders-order-locked = Fixar a ordem das pastas
admin-settings-help-folders-order-locked = Fixa a ordem das pastas para todos no escopo. Arrastar e os comandos Mover ficam desativados, e os usuários veem uma dica de bloqueio. Fixações e a ordem definida pelo administrador permanecem.
admin-settings-key-folders-pinned-admin = Pastas fixadas à força
admin-settings-help-folders-pinned-admin = Pastas exibidas em um grupo Fixadas no topo da barra lateral de todos os usuários. Uma entrada por linha, como role:inbox ou um caminho de pasta como Shared/Support. Os usuários não podem desafixá-las nem movê-las. Uma entrada sem pasta correspondente é ignorada.
admin-settings-key-folders-order-default = Ordem padrão das pastas
admin-settings-help-folders-order-default = A ordem inicial das pastas para usuários que não as organizaram, da primeira à última. Uma entrada por linha, como role:<role> ou um caminho de pasta. As pastas não listadas vêm depois, na ordem do servidor. A ordem própria do usuário substitui esta.
admin-settings-key-folders-new-folder-placement = Posição de novas pastas
admin-settings-help-folders-new-folder-placement = Onde fica uma pasta que aparece no servidor entre as irmãs: no fim ou em ordem alfabética.
admin-settings-key-ui-undo-history-max = Máximo do histórico de desfazer e refazer
admin-settings-help-ui-undo-history-max = O máximo de ações que qualquer usuário pode manter no histórico de Desfazer e Refazer, de 0 a 50. O tamanho próprio do usuário é reduzido a este valor. 0 desativa o histórico para todos. O histórico fica na aba do navegador e é apagado ao sair.
admin-settings-key-ui-bar-mail-top-bar = Barra de ferramentas de e-mail
admin-settings-help-ui-bar-mail-top-bar = "pinned" mantém a barra de ferramentas de e-mail com os controles de layout sempre visível. "auto" a oculta atrás de uma alça fina que a abre quando o usuário aponta, toca ou navega até ela.
admin-settings-key-ui-bar-workspace-tabs = Abas do espaço de trabalho
admin-settings-help-ui-bar-workspace-tabs = "pinned" mantém a faixa de abas do espaço de trabalho sempre visível. "auto" mostra só uma alça fina com o nome da aba ativa até o usuário abri-la.
admin-settings-key-ui-bar-mail-ribbon = Faixa de opções de e-mail
admin-settings-help-ui-bar-mail-ribbon = Para o layout de faixa de opções. "pinned" mantém a faixa aberta. "auto" mostra só os nomes das abas; uma aba abre a faixa sobre a lista de mensagens.
admin-settings-key-ui-bar-composer-ribbon = Faixa de opções do editor
admin-settings-help-ui-bar-composer-ribbon = Para todos os editores de mensagem. "pinned" mantém a faixa do editor aberta. "auto" mostra só os nomes das abas; abrir uma aba empurra o corpo da mensagem para baixo em vez de cobri-lo.
admin-settings-key-appearance-theme = Tema
admin-settings-help-appearance-theme = O tema de cores que os usuários veem até escolherem o próprio. Bloqueá-lo prende todos os usuários a este tema.
admin-settings-key-appearance-accent = Cor de destaque
admin-settings-help-appearance-accent = A cor de destaque do tema. Vazio significa a cor de destaque do próprio tema.
admin-settings-key-appearance-badge-color = Cor do indicador de não lidas
admin-settings-help-appearance-badge-color = A cor dos indicadores de contagem de não lidas. Default mantém a cor do tema; Accent segue a cor de destaque; accent-lighter e accent-darker usam o destaque 20% mais claro ou mais escuro, com texto de contagem preto ou branco. Um bloqueio prevalece sobre a escolha da conta.
admin-settings-key-appearance-density = Densidade
admin-settings-help-appearance-density = Quanto espaço listas e controles ocupam: compact, cozy ou relaxed.
admin-settings-key-appearance-font = Fonte da interface
admin-settings-key-packages-allowed = Rastreamento de encomendas permitido
admin-settings-help-packages-allowed = Permite o rastreamento de encomendas para as contas afetadas. O provedor também deve estar ativado em Rastreamento de encomendas, e cada usuário precisa aderir. Desativar esta configuração impede imediatamente novas chamadas ao provedor.
admin-settings-help-appearance-font = Escolha uma fonte incluída para contas sem preferência de fonte, ou bloqueie a escolha para todas as contas afetadas. As fontes são carregadas deste servidor; nenhuma solicitação ao Google Fonts é feita. Default usa a fonte do tema.
admin-settings-key-appearance-brand-name = Nome do produto
admin-settings-help-appearance-brand-name = O nome exibido na página de login e na aba do navegador.
admin-settings-key-login-notice = Aviso de login
admin-settings-help-login-notice = Um texto curto exibido na página de login, por exemplo um contato de suporte. Deixe vazio para nenhum.
admin-settings-key-mail-read-receipts = Confirmações de leitura
admin-settings-key-receipts-dsn-allowed = Solicitações de status de entrega
admin-settings-help-receipts-dsn-allowed = Permite que as contas solicitem notificações de status de entrega SMTP. Isso não solicita nem comprova que uma mensagem foi lida.
admin-settings-help-mail-read-receipts = O que acontece quando uma mensagem aberta pede confirmação de leitura: perguntar ao usuário, nunca enviar ou sempre enviar.
admin-settings-key-notifications-enabled = Notificações de novos e-mails
admin-settings-help-notifications-enabled = Se os usuários recebem notificações de novos e-mails. Não altera as regras de notificação deles.
admin-settings-key-offline-retention-days = Idade do cache offline
admin-settings-help-offline-retention-days = Por quantos dias um dispositivo guarda mensagens para uso offline. 0 significa sem limite de idade.
admin-settings-key-offline-budget-mb = Tamanho do cache offline
admin-settings-help-offline-budget-mb = Quantos megabytes um dispositivo pode usar para e-mail offline. 0 significa sem cache offline.
admin-settings-key-assistant-enabled = Assistente de IA
admin-settings-help-assistant-enabled = Se os usuários veem o assistente de IA. A chave de implantação em Assistente de IA continua valendo acima desta.
admin-settings-key-assist-profile = Perfil do provedor de IA
admin-settings-help-assist-profile = O ID do perfil de provedor que atende estes usuários (Assistente de IA > Perfis de provedor). Vazio usa o endpoint da implantação. Um ID de perfil que não existe desativa o assistente de IA para estes usuários.
admin-settings-key-upstream-default = Servidor de e-mail padrão
admin-settings-help-upstream-default = O servidor de e-mail em que um endereço entra quando nenhum domínio indica um. A tabela de domínios prevalece sobre este.
admin-settings-key-upstream-proxy-timeout-secs = Tempo limite do servidor de e-mail
admin-settings-help-upstream-proxy-timeout-secs = Quanto tempo uma solicitação ao servidor de e-mail pode levar, em segundos.
admin-settings-key-signin-new-users = Novos usuários
admin-settings-help-signin-new-users = Quem pode criar uma conta no primeiro login: qualquer pessoa (open), apenas endereços adicionados antecipadamente (preprovisioned), esses mais os usuários importados (provisioned) ou ninguém novo (closed). Contas existentes nunca são afetadas. Alterado em Política de login.
admin-settings-key-signin-new-users-mail = Login por senha: novos usuários
admin-settings-help-signin-new-users-mail = Regra para um novo usuário de caixa de e-mail que entra com senha (IMAP, POP3, Exchange). Inherit usa Novos usuários.
admin-settings-key-signin-new-users-sso = Login único: novos usuários
admin-settings-help-signin-new-users-sso = Regra para um novo usuário que entra por login único ou por um cabeçalho confiável. Inherit usa Novos usuários.
admin-settings-key-signin-new-users-linked = Caixas vinculadas: novos usuários
admin-settings-help-signin-new-users-linked = Regra para vincular uma caixa de e-mail que ainda não tem conta aqui, avaliada pelo domínio da própria caixa vinculada. Inherit usa Novos usuários.
admin-settings-key-signin-new-users-linked-imap = Caixas IMAP vinculadas: novos usuários
admin-settings-help-signin-new-users-linked-imap = Regra apenas para caixas IMAP vinculadas. Inherit usa Caixas vinculadas.
admin-settings-key-signin-new-users-linked-pop3 = Caixas POP3 vinculadas: novos usuários
admin-settings-help-signin-new-users-linked-pop3 = Regra apenas para caixas POP3 vinculadas. Inherit usa Caixas vinculadas.
admin-settings-key-signin-new-users-linked-ews = Caixas Exchange vinculadas: novos usuários
admin-settings-help-signin-new-users-linked-ews = Regra apenas para caixas Exchange (EWS) vinculadas. Inherit usa Caixas vinculadas.
admin-settings-key-signin-preprovision-ttl-days = Validade de usuário pré-adicionado (dias)
admin-settings-help-signin-preprovision-ttl-days = Por quanto tempo um usuário pendente adicionado antecipadamente continua válido quando nenhuma outra validade é informada.
admin-settings-key-signin-preprovision-max-ttl-days = Maior validade de usuário pré-adicionado (dias)
admin-settings-help-signin-preprovision-max-ttl-days = A maior validade que um administrador pode dar a um usuário pendente adicionado antecipadamente.
admin-settings-key-signin-preprovision-domain-wide = Permitir entradas pré-adicionadas para o domínio inteiro
admin-settings-help-signin-preprovision-domain-wide = Permite entradas que admitem qualquer pessoa de um domínio inteiro, até um limite de usos. Desativado por padrão.
admin-settings-key-signin-preprovision-retention-days = Manter usuários pendentes expirados (dias)
admin-settings-help-signin-preprovision-retention-days = Por quanto tempo um usuário pendente expirado é mantido antes de ser removido.
admin-settings-key-signin-domain-mode = Domínios de login
admin-settings-help-signin-domain-mode = Se qualquer domínio pode fazer login ou apenas os domínios listados. Alterado em Política de login.
admin-settings-key-signin-refusal-detail = Mensagens de recusa
admin-settings-help-signin-refusal-detail = Se um login recusado diz o motivo (conta desativada, endereço não permitido) ou dá a mesma resposta de uma senha incorreta. Alterado em Política de login.
admin-settings-key-signin-ldap-bind-precheck = Verificação LDAP antes do login
admin-settings-help-signin-ldap-bind-precheck = Verifica a senha no diretório antes do servidor de e-mail. Alterado em Integrações, após um teste de bind.
admin-settings-key-twofa-pending-ttl-secs = Limite de tempo do segundo fator
admin-settings-help-twofa-pending-ttl-secs = Quanto tempo o usuário tem para informar o segundo fator depois da senha, em segundos.
admin-settings-key-twofa-max-attempts = Tentativas do segundo fator
admin-settings-help-twofa-max-attempts = Quantos códigos incorretos do segundo fator são permitidos antes de o login recomeçar.
admin-settings-key-admin-session-idle-secs = Tempo limite de inatividade do console
admin-settings-help-admin-session-idle-secs = Quanto tempo uma sessão do console de administração dura sem uma solicitação, em segundos.
admin-settings-key-admin-session-absolute-secs = Limite da sessão do console
admin-settings-help-admin-session-absolute-secs = O máximo que uma sessão do console de administração dura, em segundos, ativa ou não.
admin-settings-key-admin-session-rotate-secs = Rotação do ID de sessão do console
admin-settings-help-admin-session-rotate-secs = Com que frequência, em segundos, o console de administração troca o ID de sessão durante o uso (0 = apenas após uma confirmação de reautenticação). A rotação nunca estende os limites de inatividade ou absoluto.
admin-settings-key-passwd-policy-min-length = Comprimento mínimo da senha
admin-settings-help-passwd-policy-min-length = O menor número de caracteres que uma nova senha pode ter.
admin-settings-key-passwd-policy-require-upper = Exigir letra maiúscula
admin-settings-help-passwd-policy-require-upper = Uma nova senha deve conter uma letra maiúscula.
admin-settings-key-passwd-policy-require-lower = Exigir letra minúscula
admin-settings-help-passwd-policy-require-lower = Uma nova senha deve conter uma letra minúscula.
admin-settings-key-passwd-policy-require-digit = Exigir um dígito
admin-settings-help-passwd-policy-require-digit = Uma nova senha deve conter um dígito.
admin-settings-key-passwd-policy-require-symbol = Exigir um símbolo
admin-settings-help-passwd-policy-require-symbol = Uma nova senha deve conter um caractere que não seja letra nem dígito.
admin-settings-key-limit-storage-bytes = Limite de armazenamento
admin-settings-help-limit-storage-bytes = O máximo de bytes que um usuário pode armazenar neste servidor. 0 significa sem limite.
admin-settings-key-limit-send-per-day = Mensagens enviadas por dia
admin-settings-help-limit-send-per-day = O máximo de mensagens que um usuário pode enviar em um dia. 0 significa sem limite.
admin-settings-key-limit-ai-calls-per-day = Chamadas ao assistente de IA por dia
admin-settings-help-limit-ai-calls-per-day = O máximo de solicitações ao assistente de IA que um usuário pode fazer em um dia. 0 significa sem limite.
admin-settings-key-quota-warn-thresholds = Níveis de aviso de uso
admin-settings-help-quota-warn-thresholds = As porcentagens de um limite em que os usuários são avisados, da menor para a maior.
admin-settings-key-quota-warn-enabled = Avisos de uso
admin-settings-help-quota-warn-enabled = Se os usuários são avisados ao se aproximarem de um limite.
admin-settings-key-usage-retention-days = Histórico de uso
admin-settings-help-usage-retention-days = Por quantos dias as contagens diárias de uso são mantidas.
admin-settings-key-usage-meter-api = Contar uso da API
admin-settings-help-usage-meter-api = Se as solicitações de API são contadas nos números de uso.
admin-settings-key-usage-measure-interval-hours = Intervalo de medição de armazenamento
admin-settings-help-usage-measure-interval-hours = Com que frequência os dados armazenados de cada usuário são medidos, em horas.
admin-settings-key-assistant-rate-limit-per-min = Limite de taxa do assistente de IA
admin-settings-help-assistant-rate-limit-per-min = O máximo de solicitações ao assistente de IA por minuto para o servidor inteiro. 0 significa sem limite.
admin-settings-key-plugins-kv-max-key-bytes = Tamanho da chave de plugin
admin-settings-help-plugins-kv-max-key-bytes = A maior chave que um plugin pode armazenar, em bytes.
admin-settings-key-plugins-kv-max-value-bytes = Tamanho do valor de plugin
admin-settings-help-plugins-kv-max-value-bytes = O maior valor que um plugin pode armazenar sob uma chave, em bytes.
admin-settings-key-plugins-kv-max-total-bytes = Armazenamento de plugin
admin-settings-help-plugins-kv-max-total-bytes = O máximo de bytes que um plugin pode armazenar no total.
admin-settings-key-plugins-kv-max-keys = Chaves de plugin
admin-settings-help-plugins-kv-max-keys = O máximo de chaves que um plugin pode armazenar.
admin-settings-key-import-max-messages = Número de mensagens da importação
admin-settings-help-import-max-messages = O máximo de mensagens que uma importação pode trazer.
admin-settings-key-import-max-message-bytes = Tamanho de mensagem da importação
admin-settings-help-import-max-message-bytes = A maior mensagem que uma importação aceita, em bytes.
admin-settings-key-webhooks-max-attempts = Tentativas de webhook
admin-settings-help-webhooks-max-attempts = Quantas vezes uma entrega de webhook é tentada antes de ser abandonada.
admin-settings-key-webhooks-timeout-secs = Tempo limite de webhook
admin-settings-help-webhooks-timeout-secs = Quanto tempo uma entrega de webhook pode levar, em segundos.
admin-settings-key-outbox-max-attempts = Tentativas de envio
admin-settings-help-outbox-max-attempts = Quantas vezes uma mensagem é entregue ao servidor de e-mail antes de ser informada como não enviada.
admin-settings-key-realtime-recheck-secs = Nova verificação de atualização ao vivo
admin-settings-help-realtime-recheck-secs = Com que frequência uma conexão de atualização ao vivo aberta verifica se a sessão ainda é válida, em segundos.
admin-settings-key-privacy-watermark-enabled = Marca d'água do leitor
admin-settings-help-privacy-watermark-enabled = Mostra uma marca d'água com o endereço do leitor sobre as mensagens abertas.
admin-settings-key-privacy-watermark-opacity = Intensidade da marca d'água
admin-settings-help-privacy-watermark-opacity = Quão visível é a marca d'água, de 0,02 (discreta) a 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Histórico de rastreamento de abertura
admin-settings-help-privacy-open-tracking-retention-days = Por quantos dias os registros de rastreamento de leitura são mantidos.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Tokens de rastreamento não enviados
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Quantas horas um token de rastreamento espera que a mensagem seja enviada antes de ser excluído.
admin-settings-key-masked-email-domain = Domínio de endereço mascarado

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-user-override-help = Quando ativado, os usuários podem escolher o próprio valor em Configurações. Quando desativado, vale para eles o valor definido aqui ou para o domínio.
admin-settings-lock-help = Quando bloqueado, os valores de domínio e as escolhas dos usuários são ignorados e este valor vale para todos.
admin-settings-save = Salvar
admin-settings-saved = Salvo. Passa a valer em até 30 segundos em todos os processos do servidor.
admin-settings-saved-restart = Salvo. Passa a valer após uma reinicialização.
admin-settings-dedicated = Esta configuração é alterada em { $section }, que verifica o efeito antes.
admin-settings-dedicated-link = Abrir { $section }
admin-settings-pinned = Definido pela variável de ambiente { $name }. O valor armazenado aqui vale quando a variável não está definida.
admin-settings-lock-title = Bloquear { $setting }?
admin-settings-lock-body = Bloquear ignora os valores mais restritos listados abaixo. Eles são mantidos e voltam a valer quando você desbloquear.
admin-settings-lock-confirm = Bloquear
admin-settings-impact-domain = { $count ->
    [one] { $count } valor de domínio é ignorado
   *[other] { $count } valores de domínio são ignorados
 }
admin-settings-help-masked-email-domain = O domínio sob o qual os endereços mascarados são criados. Deixe vazio para desativar os endereços mascarados.
admin-settings-key-abuse-report-address = Endereço de denúncias de abuso
admin-settings-help-abuse-report-address = Para onde são enviadas as denúncias de spam e abuso.
admin-settings-key-dlp-admin-address = Avisos de perda de dados
admin-settings-help-dlp-admin-address = Para onde são enviados os avisos de prevenção contra perda de dados.
admin-settings-key-observability-error-forward-url = Endereço de encaminhamento de erros
admin-settings-help-observability-error-forward-url = Um endereço https para o qual os relatórios de erro do cliente são encaminhados. Deixe vazio para nenhum.
admin-settings-key-egress-probe-url = Endereço de teste de saída
admin-settings-help-egress-probe-url = O endereço https usado para testar um proxy de saída.
admin-settings-key-push-vapid-contact = Contato de push
admin-settings-help-push-vapid-contact = O contato (mailto: ou https:) informado aos serviços de push em cada notificação.
admin-settings-key-push-quiet-hours = Horário silencioso de push
admin-settings-help-push-quiet-hours = Horas (UTC) em que nenhuma notificação push é enviada. Deixe vazio para nenhuma.
admin-settings-key-updates-check = Verificar atualizações
admin-settings-help-updates-check = Se o servidor verifica uma vez por dia se há uma versão mais nova e a mostra no painel.
admin-settings-key-updates-feed-url = Feed de versões
admin-settings-help-updates-feed-url = O endereço https que a verificação de atualizações lê.
admin-settings-key-updates-apply = Aplicar atualizações
admin-settings-help-updates-apply = Se uma atualização pode ser iniciada pelo console por meio do agente de atualização do host.
admin-settings-key-backup-retention-count = Backups mantidos
admin-settings-help-backup-retention-count = Quantos backups do banco de dados são mantidos.
admin-settings-key-about-security-contact = Contato de segurança
admin-settings-help-about-security-contact = O contato em security.txt (mailto: ou https:).
admin-settings-key-about-security-policy-url = Política de segurança
admin-settings-help-about-security-policy-url = O endereço da sua política de segurança, listado em security.txt.
admin-settings-key-about-project-links = Links de Sobre
admin-settings-help-about-project-links = Links exibidos na página Sobre, cada um com um rótulo e um endereço https.
admin-settings-key-config-snapshots-kept = Instantâneos de configuração
admin-settings-help-config-snapshots-kept = Quantos instantâneos de configuração são mantidos.
admin-settings-key-jobs-history-days = Histórico de tarefas
admin-settings-help-jobs-history-days = Quantos dias de execuções de tarefas são mantidos.
admin-settings-key-maintenance-announce-minutes-default = Antecedência do aviso de manutenção
admin-settings-help-maintenance-announce-minutes-default = Quantos minutos antes de uma janela de manutenção os usuários são avisados.
admin-settings-key-maintenance-default-message = Mensagem de manutenção
admin-settings-help-maintenance-default-message = O texto que os usuários veem durante a manutenção, por idioma.
admin-settings-key-health-disk-free-min-percent = Mínimo de espaço livre em disco
admin-settings-help-health-disk-free-min-percent = A verificação de prontidão falha quando o espaço livre em disco fica abaixo desta porcentagem.
admin-settings-key-observability-otlp-metrics = Enviar métricas por OTLP
admin-settings-help-observability-otlp-metrics = Envia as métricas do servidor ao coletor OTLP definido na inicialização.
admin-settings-key-metrics-drilldown-top-n = Linhas de detalhe das métricas
admin-settings-help-metrics-drilldown-top-n = Quantas linhas um detalhamento de métricas mostra.
admin-settings-key-metrics-rollup-days = Histórico de métricas
admin-settings-help-metrics-rollup-days = Quantos dias de resumos horários de métricas são mantidos.
admin-settings-key-observability-client-timing-enabled = Tempos de carregamento do navegador
admin-settings-help-observability-client-timing-enabled = Coleta tempos de carregamento de página de navegadores conectados como histogramas agregados anônimos. Nada identifica uma pessoa, sessão ou página. Desative para interromper a coleta.
admin-settings-key-observability-client-timing-sample-rate-percent = Taxa de amostragem dos tempos de carregamento
admin-settings-help-observability-client-timing-sample-rate-percent = Porcentagem das sessões de navegador que informam tempos de carregamento (1 a 100).
admin-settings-key-observability-client-timing-min-samples = Mínimo de amostras de tempo de carregamento
admin-settings-help-observability-client-timing-min-samples = A mediana e o percentil 95 dos tempos de carregamento ficam ocultos até que um período tenha pelo menos este número de amostras.
admin-settings-key-observability-client-timing-folder-loads = Tempos de carregamento de pastas
admin-settings-help-observability-client-timing-folder-loads = Mede quanto tempo uma pasta leva para mostrar a lista de mensagens, separando o que veio do cache do navegador do que veio da rede. Ativado por padrão. Os usuários podem desativar para si, a menos que esta configuração esteja bloqueada. Exige que os tempos de carregamento do navegador estejam ativados.
admin-settings-key-observability-client-timing-message-loads = Tempos de carregamento de mensagens
admin-settings-help-observability-client-timing-message-loads = Mede quanto tempo uma mensagem leva para mostrar os cabeçalhos e o corpo sanitizado, separando o que estava em cache ou foi pré-carregado do que veio da rede. Ativado por padrão. Os usuários podem desativar para si, a menos que esta configuração esteja bloqueada. Exige que os tempos de carregamento do navegador estejam ativados.
admin-settings-key-diagnostics-retention-days = Histórico de diagnósticos
admin-settings-help-diagnostics-retention-days = Quantos dias de execuções de diagnóstico são mantidos.
admin-settings-key-data-export-link-hours = Prazo de download da exportação de dados
admin-settings-help-data-export-link-hours = Por quantas horas uma exportação de dados pode ser baixada antes de ser excluída.
admin-settings-key-dsr-enabled = Solicitações de dados
admin-settings-help-dsr-enabled = Se as pessoas podem fazer solicitações de dados (acesso, download, correção, exclusão, restrição, oposição) em Configurações.
admin-settings-key-dsr-type-access = Aceitar solicitações de acesso
admin-settings-help-dsr-type-access = Se o tipo de solicitação de acesso é oferecido aos usuários.
admin-settings-key-dsr-type-portability = Aceitar solicitações de download (cópia portátil)
admin-settings-help-dsr-type-portability = Se o tipo de solicitação de download (cópia portátil) é oferecido aos usuários.
admin-settings-key-dsr-type-rectification = Aceitar solicitações de correção
admin-settings-help-dsr-type-rectification = Se o tipo de solicitação de correção é oferecido aos usuários.
admin-settings-key-dsr-type-erasure = Aceitar solicitações de exclusão
admin-settings-help-dsr-type-erasure = Se o tipo de solicitação de exclusão é oferecido aos usuários.
admin-settings-key-dsr-type-restriction = Aceitar solicitações de restrição
admin-settings-help-dsr-type-restriction = Se o tipo de solicitação de restrição é oferecido aos usuários.
admin-settings-key-dsr-type-objection = Aceitar solicitações de oposição
admin-settings-help-dsr-type-objection = Se o tipo de solicitação de oposição é oferecido aos usuários.
admin-settings-key-dsr-type-other = Aceitar outras solicitações
admin-settings-help-dsr-type-other = Se o tipo de solicitação genérica é oferecido aos usuários.
admin-settings-key-dsr-approval-export = Aprovação de solicitações de acesso e download
admin-settings-help-dsr-approval-export = auto inicia a exportação assim que a solicitação é confirmada. admin espera que um administrador a aprove.
admin-settings-key-dsr-approval-erasure = Aprovação de solicitações de exclusão
admin-settings-help-dsr-approval-erasure = auto agenda a exclusão assim que a solicitação é confirmada. admin espera que um administrador a aprove.
admin-settings-key-dsr-approval-other = Tratamento de outras solicitações
admin-settings-help-dsr-approval-other = auto marca a solicitação como em andamento assim que confirmada. admin espera que um administrador a aprove.
admin-settings-key-dsr-reauth-required = Nova autenticação para solicitações
admin-settings-help-dsr-reauth-required = Exige uma senha ou segundo fator recente para fazer uma solicitação de exportação ou exclusão e para criar um link de download.
admin-settings-key-dsr-confirmation-required = Mensagem de confirmação
admin-settings-help-dsr-confirmation-required = Envia uma mensagem de confirmação com um código para a caixa de entrada do solicitante; a solicitação só prossegue depois que o código é informado.
admin-settings-key-dsr-identity-verification-required = Etapa de verificação de identidade
admin-settings-help-dsr-identity-verification-required = Um administrador deve registrar que a identidade do solicitante foi verificada antes de a solicitação ser aprovada.
admin-settings-key-dsr-erasure-grace-days = Período de carência da exclusão
admin-settings-help-dsr-erasure-grace-days = Dias entre a aprovação e a exclusão. O solicitante pode cancelar nesse período. 0 executa a exclusão na próxima passagem do worker.
admin-settings-key-dsr-sla-days = Dias para responder a uma solicitação
admin-settings-help-dsr-sla-days = O número de dias entre o registro e o prazo exibido a usuários e administradores.
admin-settings-key-dsr-sla-max-extension-days = Maior prorrogação
admin-settings-help-dsr-sla-max-extension-days = O máximo de dias que um administrador pode acrescentar ao prazo de uma solicitação, com um motivo registrado.
admin-settings-key-dsr-sla-reminder-days = Sinalizações de prazo
admin-settings-help-dsr-sla-reminder-days = Dias antes do prazo em que uma solicitação é sinalizada na fila como próxima do vencimento.
admin-settings-key-dsr-export-mail-allowed = E-mails nas exportações
admin-settings-help-dsr-export-mail-allowed = Permite que o solicitante inclua mensagens em uma exportação. Só são incluídas as mensagens mantidas no cache deste servidor; o e-mail que fica no servidor de e-mail de origem não é.
admin-settings-key-dsr-export-mail-format = Formato de e-mail nas exportações
admin-settings-help-dsr-export-mail-format = mbox grava um arquivo por pasta; eml grava um arquivo por mensagem.
admin-settings-key-dsr-export-include-settings = Exportação: configurações
admin-settings-help-dsr-export-include-settings = Inclui as configurações em uma exportação.
admin-settings-key-dsr-export-include-identities = Exportação: identidades
admin-settings-help-dsr-export-include-identities = Inclui as identidades em uma exportação.
admin-settings-key-dsr-export-include-contacts = Exportação: contatos
admin-settings-help-dsr-export-include-contacts = Inclui os contatos em uma exportação.
admin-settings-key-dsr-export-include-calendars = Exportação: calendários
admin-settings-help-dsr-export-include-calendars = Inclui os calendários em uma exportação.
admin-settings-key-dsr-export-include-notes = Exportação: notas
admin-settings-help-dsr-export-include-notes = Inclui as notas em uma exportação.
admin-settings-key-dsr-export-include-rules = Exportação: filtros e regras
admin-settings-help-dsr-export-include-rules = Inclui filtros e regras em uma exportação.
admin-settings-key-dsr-export-include-sessions = Exportação: sessões, dispositivos e histórico de login
admin-settings-help-dsr-export-include-sessions = Inclui sessões, dispositivos e histórico de login em uma exportação.
admin-settings-key-dsr-export-include-consent = Exportação: registros de consentimento
admin-settings-help-dsr-export-include-consent = Inclui os registros de consentimento em uma exportação.
admin-settings-key-dsr-export-include-audit = Exportação: entradas de auditoria sobre a pessoa
admin-settings-help-dsr-export-include-audit = Inclui em uma exportação as entradas de auditoria sobre a pessoa.
admin-settings-key-dsr-export-max-megabytes = Limite de tamanho da exportação
admin-settings-help-dsr-export-max-megabytes = Maior exportação em megabytes. As seções que ultrapassariam o limite ficam de fora e são listadas no manifesto.
admin-settings-key-dsr-export-format = Formatos de exportação
admin-settings-help-dsr-export-format = json grava arquivos JSON. jsonCsv também grava arquivos CSV para dados em lista.
admin-settings-key-dsr-export-encrypt = Selar exportações em repouso
admin-settings-help-dsr-export-encrypt = Armazena as exportações criadas criptografadas com a chave do servidor até serem baixadas ou expirarem.
admin-settings-key-dsr-download-link-minutes = Validade do link de download
admin-settings-help-dsr-download-link-minutes = Minutos em que um link de download funciona após ser criado. A exportação em si é mantida durante o prazo de download da exportação de dados.
admin-settings-key-dsr-download-single-use = Link de download de uso único
admin-settings-help-dsr-download-single-use = Um link de download deixa de funcionar depois de usado uma vez.
admin-settings-key-dsr-retention-completed-days = Retenção de solicitações concluídas
admin-settings-help-dsr-retention-completed-days = Dias em que uma solicitação concluída e seu histórico são mantidos antes de serem excluídos.
admin-settings-key-dsr-notify-admins = Registrar novas solicitações para administradores
admin-settings-help-dsr-notify-admins = Registra cada nova solicitação como um evento de segurança no log de auditoria, onde administradores e destinos de alerta a veem.
admin-settings-key-dsr-notify-requester = Mensagens de andamento ao solicitante
admin-settings-help-dsr-notify-requester = Coloca uma mensagem na caixa de entrada do solicitante quando uma solicitação muda de estado.
admin-settings-key-dsr-contact-name = Nome do contato de proteção de dados
admin-settings-help-dsr-contact-name = Nome exibido aos usuários na página de solicitações de dados.
admin-settings-key-dsr-contact-email = Endereço do contato de proteção de dados
admin-settings-help-dsr-contact-email = Endereço de e-mail exibido aos usuários na página de solicitações de dados.
admin-settings-key-dsr-contact-url = Página do contato de proteção de dados
admin-settings-help-dsr-contact-url = Link exibido aos usuários na página de solicitações de dados.
admin-settings-key-dsr-on-behalf-allowed = Solicitações em nome de um usuário
admin-settings-help-dsr-on-behalf-allowed = Permite que um administrador faça uma solicitação por um usuário, por exemplo uma recebida por correio.
admin-settings-key-dsr-rate-limit-per-day = Solicitações por pessoa por dia
admin-settings-help-dsr-rate-limit-per-day = O máximo de solicitações que uma pessoa pode fazer em 24 horas.
admin-settings-key-dsr-legal-hold-subjects = Retenções legais
admin-settings-help-dsr-legal-hold-subjects = Endereços, ou entradas @domínio, cuja exclusão é bloqueada. Uma solicitação de exclusão para eles fica na fila como bloqueada até a entrada ser removida.
admin-settings-key-scrub-enabled = Limpeza de pessoa
admin-settings-help-scrub-enabled = Permite a ação Limpar pessoa e a limpeza que pode seguir uma exclusão. Desativado, nenhuma é executada.
admin-settings-key-scrub-pseudonym-algorithm = Tamanho do pseudônimo
admin-settings-help-scrub-pseudonym-algorithm = Como é feito o pseudônimo com chave que substitui um identificador: HMAC-SHA256 sob a geração de chave atual, cortado em 16 ou 32 caracteres hexadecimais.
admin-settings-key-scrub-include-logs = Limpar arquivos de log
admin-settings-help-scrub-include-logs = Uma limpeza também reescreve o identificador nos arquivos de log rotacionados, inclusive os compactados.
admin-settings-key-scrub-auto-run-on-erasure = Limpar após a exclusão
admin-settings-help-scrub-auto-run-on-erasure = Uma exclusão concluída também substitui os identificadores da pessoa no histórico de auditoria e nos logs pelo pseudônimo. Desativado por padrão.
admin-settings-key-scrub-key-retention-days = Retenção da chave de pseudônimo
admin-settings-help-scrub-key-retention-days = Dias em que uma chave de pseudônimo é mantida depois de rotacionada. 0 a mantém. Após a exclusão, os pseudônimos antigos não podem mais ser recalculados e as lápides feitas com a chave não podem mais ser reaplicadas.
admin-settings-key-backup-propagate-erasure = Reaplicar exclusões na restauração
admin-settings-help-backup-propagate-erasure = Uma restauração reaplica as lápides de exclusão mais recentes que o backup, de modo que as pessoas excluídas desde então sejam removidas de novo.
admin-settings-key-backup-tombstone-retention-days = Retenção de lápides de exclusão
admin-settings-help-backup-tombstone-retention-days = Dias em que uma lápide de exclusão é mantida. 0 as mantém para sempre.
admin-settings-key-dsr-text-intro = Introdução das solicitações de dados
admin-settings-help-dsr-text-intro = Texto exibido acima do formulário de solicitação. Um valor por idioma.
admin-settings-key-dsr-text-confirmation = Texto da mensagem de confirmação
admin-settings-help-dsr-text-confirmation = Texto acrescentado à mensagem de confirmação. Um valor por idioma.
admin-settings-key-dsr-text-completion = Texto da mensagem de conclusão
admin-settings-help-dsr-text-completion = Texto acrescentado à mensagem enviada quando uma solicitação é concluída. Um valor por idioma.
admin-settings-key-server-text-family = Texto do servidor
admin-settings-help-server-text-family = Texto que o servidor escreve em e-mails e notificações, por idioma e domínio.
admin-settings-cat-sieve = Filtros de e-mail
admin-settings-key-sieve-max-script-bytes = Tamanho máximo do script de filtro
admin-settings-help-sieve-max-script-bytes = Máximo de bytes em um único script de filtro Sieve.
admin-settings-key-sieve-max-scripts = Máximo de scripts de filtro
admin-settings-help-sieve-max-scripts = Número máximo de scripts salvos por conta.
admin-settings-key-sieve-max-redirects = Máximo de redirecionamentos
admin-settings-help-sieve-max-redirects = Máximo de comandos de redirecionamento em um script.
admin-settings-key-sieve-vacation-allowed = Ações de filtro de férias
admin-settings-help-sieve-vacation-allowed = Permite que os scripts enviem respostas de férias quando o backend as aceita.
admin-settings-key-sieve-allowed-extensions = Extensões de filtro permitidas
admin-settings-help-sieve-allowed-extensions = Uma lista opcional de nomes de extensões Sieve em minúsculas. O backend também deve aceitar cada extensão.
admin-settings-key-cap-sieve-raw-edit = Editar o código-fonte do script de filtro
admin-settings-help-cap-sieve-raw-edit = Permite edições diretas de scripts Sieve para as contas selecionadas.
admin-settings-key-cap-sieve-forward-external = Encaminhar para fora do domínio da conta
admin-settings-help-cap-sieve-forward-external = Permite encaminhamento externo nos scripts. Salvar esses scripts também exige verificação recente.
admin-settings-key-session-max-concurrent = Sessões simultâneas
admin-settings-help-session-max-concurrent = Máximo de sessões por conta. As sessões mais antigas são revogadas quando um novo login excede este limite.
admin-settings-key-session-idle-secs = Tempo limite de inatividade da sessão
admin-settings-help-session-idle-secs = Encerra sessões inativas após este número de segundos, até o limite da implantação.
admin-settings-key-session-absolute-secs = Duração da sessão
admin-settings-help-session-absolute-secs = Encerra as sessões após este número de segundos, até o limite da implantação.
admin-settings-key-twofa-remember-days = Validade do dispositivo lembrado
admin-settings-help-twofa-remember-days = Dias até que um dispositivo lembrado precise verificar o segundo fator de novo. Zero desativa este recurso.
admin-settings-key-twofa-passkeys-enabled = Permitir passkeys
admin-settings-help-twofa-passkeys-enabled = Permite passkeys como fator de autenticação.
admin-settings-key-twofa-required = Exigir autenticação em dois fatores
admin-settings-help-twofa-required = Exige um segundo fator cadastrado antes de emitir uma sessão.
admin-settings-key-twofa-passkey-attachment = Tipo de autenticador da passkey
admin-settings-help-twofa-passkey-attachment = any (padrão): todos os autenticadores. platform: passkeys armazenadas no dispositivo (Windows Hello, Touch ID, bloqueio de tela do Android). cross-platform: chaves de segurança e celulares. O navegador informa isto; não é assinado pelo autenticador.
admin-settings-key-twofa-passkey-user-verification = Verificação do usuário na passkey
admin-settings-help-twofa-passkey-user-verification = required: a passkey deve verificar um PIN ou biometria ao ser adicionada e em cada login; uma cerimônia sem isso é recusada. preferred (padrão) e discouraged só mudam o que é pedido ao navegador.
admin-settings-key-twofa-passkey-resident-key = Passkeys detectáveis
admin-settings-help-twofa-passkey-resident-key = required: a credencial deve ser armazenada no autenticador. discouraged (padrão) ou preferred. No modo engine, as passkeys são sempre detectáveis, porque também permitem entrar sem senha.
admin-settings-key-twofa-passkey-legacy = Passkeys adicionadas antes da política atual
admin-settings-help-twofa-passkey-legacy = allow (padrão): uma passkey mais antiga que não atende à política continua funcionando e o usuário é solicitado a substituí-la. refuse: ela é recusada no login com uma mensagem; o usuário entra com outro fator ou um código de recuperação.
admin-settings-key-twofa-last-factor-removal = Remoção do último segundo fator
admin-settings-help-twofa-last-factor-removal = Quando remover uma passkey ou um aplicativo autenticador deixaria a conta sem nenhum. allow (padrão): é removido e o login exige apenas a senha. block: o usuário deve primeiro adicionar outro método de login (códigos de recuperação não contam). Onde a autenticação em dois fatores é obrigatória, o último é sempre mantido.
admin-settings-key-privacy-remote-content = Conteúdo remoto
admin-settings-help-privacy-remote-content = Bloquear imagens, perguntar antes, usar o proxy ou carregar diretamente quando permitido.
admin-settings-key-privacy-trackers = Bloquear rastreadores
admin-settings-help-privacy-trackers = Bloqueia imagens de rastreamento de e-mail conhecidas.
admin-settings-key-privacy-clean-links = Limpar links
admin-settings-help-privacy-clean-links = Remove parâmetros de rastreamento dos links das mensagens.
admin-settings-key-privacy-max-bytes = Limite de imagem remota
admin-settings-help-privacy-max-bytes = Máximo de bytes obtidos para uma imagem remota.
admin-settings-key-privacy-cache-bytes = Cache de imagens remotas
admin-settings-help-privacy-cache-bytes = Máximo de bytes retidos no cache de imagens remotas.
admin-settings-key-privacy-rate-per-min = Taxa de solicitações de imagens remotas
admin-settings-help-privacy-rate-per-min = Máximo de buscas de imagens remotas por conta por minuto.
admin-settings-privacy-fetch-title = Busca de imagens remotas
admin-settings-privacy-fetch-help = Limites de tempo, redirecionamentos e downloads paralelos de imagens que o servidor busca para o leitor. Uma imagem acima de um limite mostra um marcador de posição.
admin-settings-key-privacy-fetch-connect-timeout-ms = Tempo limite de conexão de imagem remota
admin-settings-help-privacy-fetch-connect-timeout-ms = Milissegundos permitidos para abrir a conexão com o host da imagem, incluindo o handshake TLS. Padrão 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Tempo limite de resposta de imagem remota
admin-settings-help-privacy-fetch-headers-timeout-ms = Milissegundos permitidos desde o início de uma solicitação até o host da imagem responder com os cabeçalhos. Padrão 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Tempo limite de travamento de imagem remota
admin-settings-help-privacy-fetch-idle-timeout-ms = Maior pausa, em milissegundos, entre duas partes do download de uma imagem antes de ele ser interrompido. Padrão 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Tempo limite total de imagem remota
admin-settings-help-privacy-fetch-total-timeout-ms = Milissegundos permitidos para buscar uma imagem, incluindo redirecionamentos. A espera na fila tem um orçamento separado, para que uma imagem em espera não consuma seu tempo de rede antes de começar. Padrão 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Tempo limite de fila de imagem remota
admin-settings-help-privacy-fetch-queue-timeout-ms = Máximo de milissegundos que as imagens podem esperar por capacidade além do orçamento de busca. A fila limitada comporta grandes picos autenticados enquanto os limites de busca e decodificação ativas continuam aplicados. Padrão 60000.
admin-settings-key-privacy-max-redirects = Redirecionamentos de imagem remota
admin-settings-help-privacy-max-redirects = Redirecionamentos seguidos para uma imagem. Cada um é verificado contra as mesmas regras de endereço da primeira solicitação. Padrão 3.
admin-settings-key-privacy-max-concurrent-per-message = Downloads de imagem remota por mensagem
admin-settings-help-privacy-max-concurrent-per-message = Imagens de uma mensagem buscadas ao mesmo tempo. As outras esperam por uma vaga livre. Intervalo 1–128; padrão 30.
admin-settings-key-privacy-max-concurrent-per-account = Downloads de imagem remota por conta
admin-settings-help-privacy-max-concurrent-per-account = Imagens buscadas ao mesmo tempo para uma conta em todas as suas mensagens abertas. Intervalo 1–256; padrão 128.
admin-settings-key-privacy-allow-direct = Permitir imagens diretas
admin-settings-key-cap-remote-content-direct = Permissão de imagem direta
admin-settings-help-cap-remote-content-direct = Permite que esta conta selecione imagens diretas quando o servidor permitir.
admin-settings-help-privacy-allow-direct = Permite o carregamento direto de imagens para contas com a capacidade necessária.
admin-settings-key-logs-auth-retention-days = Retenção do histórico de autenticação
admin-settings-help-logs-auth-retention-days = Dias de retenção dos metadados de autenticação, independentemente da identidade durável da conta.
admin-settings-key-logs-auth-max-entries = Limite de entradas do histórico de autenticação
admin-settings-help-logs-auth-max-entries = Máximo de entradas retidas. Falhas repetidas equivalentes em um minuto compartilham uma entrada.
admin-settings-key-reader-color-mode = Cores da mensagem
admin-settings-cat-reader = Exibição de mensagens
admin-settings-key-mail-row-actions-mode = Ações rápidas da lista de mensagens
admin-settings-help-mail-row-actions-mode = Como as ações rápidas de cada linha de mensagem aparecem: open (sempre disponíveis, reveladas ao passar o mouse ou focar), hover (três controles e um botão Mais), click (um botão que as abre) ou off (removidas das linhas; o menu de contexto e a barra de ferramentas mantêm todas as ações). Hover se comporta como click em telas de toque. Os usuários podem escolher a própria opção, a menos que um administrador bloqueie.
admin-settings-key-mail-quick-reply = Resposta rápida

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-quick-reply = Oferece uma aba de resposta compacta com os destinatários já preenchidos. Desativado por padrão; os usuários podem ativar, a menos que um administrador bloqueie. As proteções normais de envio continuam valendo.
admin-settings-key-mail-request-read-receipt = Solicitar confirmação de leitura por padrão
admin-settings-help-mail-request-read-receipt = Inicia as mensagens enviadas compatíveis com a solicitação de confirmação selecionada. É um padrão de redação, não uma garantia de que o destinatário enviará a confirmação.
admin-settings-help-reader-color-mode = Automático segue os temas escuros. Escolha a renderização original, adaptativa ou invertida dentro do quadro isolado da mensagem.
admin-settings-key-reader-font-size = Tamanho da fonte da mensagem
admin-settings-help-reader-font-size = Tamanho de leitura padrão em pixels, de 12 a 24.
admin-settings-key-reader-line-height = Altura da linha da mensagem
admin-settings-help-reader-line-height = Espaçamento entre linhas de leitura de 1,2 a 2,2.
admin-settings-key-reader-content-width = Largura de leitura da mensagem
admin-settings-help-reader-content-width = Conteúdo da mensagem confortável, largo ou em largura total do painel.
admin-settings-key-reader-loading = Aparência do carregamento da mensagem
admin-settings-help-reader-loading = Linhas de espaço reservado, um indicador de texto ou nenhum indicador visual. Os anúncios de acessibilidade continuam disponíveis.
admin-settings-key-reader-loading-delay-ms = Atraso do carregamento da mensagem
admin-settings-help-reader-loading-delay-ms = Atrasa apenas o indicador de carregamento, entre 0 e 1000 milissegundos. A entrega da mensagem nunca é atrasada.
admin-settings-key-reader-zoom = Zoom da mensagem
admin-settings-help-reader-zoom = Zoom padrão do corpo da mensagem em porcentagem, de 50 a 200; o leitor usa a etapa de zoom mais próxima. Apenas a mensagem é ampliada, e a impressão usa 100%.
admin-settings-key-reader-attachments-collapsed = Barra de anexos recolhida
admin-settings-help-reader-attachments-collapsed = Se a barra de anexos do leitor começa recolhida, mostrando apenas a contagem e o tamanho total. A última abertura ou fechamento do usuário é salva aqui. Desativado por padrão: os anexos são exibidos.
admin-settings-key-login-show-domain-hint = Mostrar o domínio preferido no login
admin-settings-help-login-show-domain-hint = Mostra qual domínio aceita um nome de usuário sem sufixo de e-mail. Oculto por padrão. Isso só altera a dica; entrar com um nome de usuário continua funcionando.
admin-settings-key-signin-retry-cooldown-secs = Intervalo entre tentativas de login
admin-settings-help-signin-retry-cooldown-secs = Mínimo de segundos entre tentativas de login com senha de uma mesma rede de cliente. Padrão 3; 0 desativa o intervalo. Os bloqueios existentes por falhas continuam valendo.
admin-settings-key-mail-quote-style = Estilo do cabeçalho de resposta e encaminhamento
admin-settings-key-mail-external-recipient-warning = Aviso de destinatário externo
admin-settings-help-mail-external-recipient-warning = Desativado, avisar ou exigir confirmação no editor. Abrange Para, Cc e Cco. É uma salvaguarda do editor, não uma restrição de entrega do servidor.
admin-settings-key-mail-trusted-recipient-domains = Domínios de destinatários confiáveis
admin-settings-help-mail-trusted-recipient-domains = Até 100 domínios de destinatários exatos. O domínio da identidade de envio selecionada também é confiável. Entradas inválidas nunca concedem confiança. Bloqueie esta configuração para impedir alterações dos usuários.
admin-settings-help-mail-quote-style = Use respostas citadas clássicas ou cabeçalhos de mensagem no estilo do Outlook. Isto define o padrão do usuário; bloqueie para impor o estilo. Os rascunhos existentes não mudam.
admin-settings-key-signin-anti-bot-mode = Proteção contra bots no login
admin-settings-help-signin-anti-bot-mode = Verificação de prova de trabalho hospedada localmente: off, mailbox, admin ou all. Nenhum serviço externo de verificação é contatado.
admin-settings-key-signin-anti-bot-difficulty = Esforço da verificação de login
admin-settings-help-signin-anti-bot-difficulty = Contador máximo do solucionador, 100–20000. Valores maiores exigem mais trabalho do navegador; padrão 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Validade da verificação de login
admin-settings-help-signin-anti-bot-expiry-secs = Segundos em que uma verificação de uso único permanece válida, 60–600; padrão 180. É necessária uma nova verificação após cada tentativa de credencial.
admin-settings-key-attachments-preview-allowed = Permitir pré-visualização de anexos
admin-settings-help-attachments-preview-allowed = Desative para impedir a renderização de pré-visualizações de anexos; os downloads originais têm controle próprio.
admin-settings-key-attachments-download-allowed = Permitir a entrega do anexo original
admin-settings-help-attachments-download-allowed = Desativar também impede pré-visualizações que exigem os bytes originais. As pré-visualizações do navegador não são proteção contra cópia.
admin-settings-key-attachments-max-bytes = Máximo de bytes do anexo
admin-settings-help-attachments-max-bytes = Zero significa ilimitado. Aplica-se aos bytes originais entregues, incluindo solicitações de intervalo.
admin-settings-key-attachments-allowed-extensions = Extensões de nome de arquivo permitidas
admin-settings-help-attachments-allowed-extensions = Matriz JSON de extensões em minúsculas sem ponto, como ["pdf","tar.gz"]. Vazio permite qualquer extensão; as regras de bloqueio prevalecem.
admin-settings-key-attachments-blocked-extensions = Extensões de nome de arquivo bloqueadas
admin-settings-help-attachments-blocked-extensions = Matriz JSON de extensões em minúsculas. Avaliada com os nomes de arquivo MIME armazenados, nunca com o nome na URL de download.
admin-settings-key-attachments-warn-extensions = Avisar para extensões de nome de arquivo
admin-settings-help-attachments-warn-extensions = Matriz JSON de extensões que exigem confirmação explícita antes da entrega do original.
admin-settings-key-attachments-allowed-mime-types = Tipos MIME permitidos
admin-settings-help-attachments-allowed-mime-types = Matriz JSON de tipos em minúsculas ou curingas de nível superior, como ["image/*","application/pdf"]. Vazio permite qualquer tipo.
admin-settings-key-attachments-blocked-mime-types = Tipos MIME bloqueados
admin-settings-help-attachments-blocked-mime-types = Matriz JSON de tipos em minúsculas ou curingas de nível superior. As regras de bloqueio têm precedência.
admin-settings-key-attachments-warn-mime-types = Avisar para tipos MIME
admin-settings-help-attachments-warn-mime-types = Matriz JSON de tipos em minúsculas ou curingas de nível superior que exigem confirmação. Os metadados MIME não são detecção de malware.
admin-settings-cat-shortcuts = Atalhos de teclado
admin-settings-cat-time = Fusos horários
admin-settings-key-time-display-zone = Fuso horário de exibição
admin-settings-help-time-display-zone = Fuso horário IANA como Europe/Lisbon, ou local para seguir o dispositivo atual. Os carimbos de data e hora das mensagens continuam armazenados como seus instantes originais.
admin-settings-key-time-comparison-zone = Fuso horário de comparação
admin-settings-help-time-comparison-zone = Segundo fuso horário IANA opcional para os auxiliares de mensagem e calendário. Deixe vazio para ocultar a comparação; nenhuma localização é inferida de um endereço de e-mail.
admin-settings-key-time-show-helpers = Mostrar auxiliares de fuso horário
admin-settings-help-time-show-helpers = Mostra comparações entre o horário local e o fuso selecionado para os carimbos de data e hora das mensagens e eventos do calendário. Os bloqueios de administrador podem impor esta escolha.
admin-settings-key-shortcuts-enabled = Ativar atalhos de teclado
admin-settings-help-shortcuts-enabled = Permite atalhos do aplicativo. Os bloqueios podem desativá-los globalmente ou por domínio. Campos de digitação e caixas de diálogo modais mantêm o tratamento normal do teclado.
admin-settings-key-shortcuts-compose = Atalho de escrever
admin-settings-key-shortcuts-archive = Atalho de arquivar
admin-settings-key-shortcuts-reply = Atalho de responder
admin-settings-key-shortcuts-reply-all = Atalho de responder a todos
admin-settings-key-shortcuts-forward = Atalho de encaminhar
admin-settings-key-shortcuts-next = Atalho de próxima mensagem
admin-settings-key-shortcuts-previous = Atalho de mensagem anterior
admin-settings-key-shortcuts-search = Atalho de focar a pesquisa
admin-settings-key-shortcuts-refresh = Atalho de atualizar mensagens
admin-settings-key-shortcuts-settings = Atalho de abrir configurações
admin-settings-key-shortcuts-trash = Atalho de mover para a lixeira
admin-settings-key-shortcuts-mark-read = Atalho de marcar como lida
admin-settings-key-shortcuts-mark-unread = Atalho de marcar como não lida
admin-settings-key-shortcuts-spam = Atalho de marcar como spam
admin-settings-key-shortcuts-zoom-in = Atalho de ampliar a mensagem
admin-settings-key-shortcuts-zoom-out = Atalho de reduzir a mensagem
admin-settings-key-shortcuts-zoom-reset = Atalho de redefinir o zoom da mensagem
admin-settings-key-shortcuts-open-in-tab = Atalho de abrir a mensagem na própria aba
admin-settings-key-notifications-push-allowed = Notificações push em segundo plano permitidas
admin-settings-help-notifications-push-allowed = Permite que as notificações ativadas desta conta se registrem e recebam push em segundo plano. A permissão do navegador ainda é necessária.
admin-settings-key-offline-persistent-storage-allowed = Solicitações de armazenamento persistente do navegador permitidas
admin-settings-help-offline-persistent-storage-allowed = Permite uma solicitação explícita para proteger os dados em cache locais contra a remoção automática pelo navegador. O navegador pode recusar; isto não concede a permissão automaticamente.
admin-settings-key-security-signin-mailbox-failure-limit = Limite de credenciais incorretas da caixa de e-mail
admin-settings-help-security-signin-mailbox-failure-limit = Credenciais rejeitadas de um mesmo IP de origem dentro da janela de contagem acionam um bloqueio temporário de login na caixa de e-mail. Falhas de rede e de configuração não contam.
admin-settings-key-security-signin-admin-failure-limit = Limite de credenciais incorretas de administrador
admin-settings-help-security-signin-admin-failure-limit = Credenciais de administrador rejeitadas de um mesmo IP de origem acionam um bloqueio temporário de login de administrador separado. A sessão curta de administrador e os requisitos de prova recente continuam inalterados.
admin-settings-key-security-signin-failure-window-secs = Janela de contagem de credenciais incorretas
admin-settings-help-security-signin-failure-window-secs = Janela móvel em segundos para rejeições de credenciais e MFA. Um login concluído com sucesso zera essa origem e superfície.
admin-settings-key-security-signin-lock-secs = Duração do bloqueio temporário de login
admin-settings-help-security-signin-lock-secs = Duração do bloqueio em segundos. Tentativas repetidas durante o bloqueio não estendem sua expiração original. Os bloqueios globais manuais de IP são separados.
admin-settings-key-security-signin-geo-scope = Escopo da política de país no login
admin-settings-help-security-signin-geo-scope = Desativado por padrão. Aplica-se apenas a novos logins de caixa de e-mail, de administrador ou de todos. As sessões existentes não são revogadas. Verifique o diagnóstico da origem atual antes de ativar; um banco de dados GeoIP local indisponível recusa o login.
admin-settings-key-security-signin-geo-mode = Lista de países permitidos ou negados
admin-settings-help-security-signin-geo-mode = Allow permite apenas os países listados; deny recusa os países listados. Endereços desconhecidos usam a própria política explícita.
admin-settings-key-security-signin-geo-countries = Códigos de país do login
admin-settings-help-security-signin-geo-countries = Matriz JSON de códigos de país de duas letras em maiúsculas, por exemplo ["PT", "GB"]. Usa apenas o banco de dados local de países (Segurança > Banco de dados de países, ou MW_GEOIP_DB); os endereços nunca são enviados a um serviço de localização.
admin-settings-key-security-signin-geo-unknown = Política para endereço de origem não mapeado
admin-settings-help-security-signin-geo-unknown = Permite ou nega endereços ausentes de um banco de dados local legível, incluindo endereços privados. Bancos de dados ausentes ou ilegíveis sempre recusam logins sob política ativa, mesmo quando endereços desconhecidos são permitidos.
admin-settings-key-contacts-avatar-mode = Fotos de contatos e avatares gerados
admin-settings-help-contacts-avatar-mode = Gera iniciais localmente, oculta os avatares ou usa fotos incorporadas dos próprios catálogos de endereços de cada conta. Nenhum URL de foto remota nem serviço externo de avatar é contatado. A política de usuário e de domínio pode ser mantida por um administrador.
admin-settings-cat-contacts = Fotos de contatos
admin-settings-key-reader-action-style = Faixa de ações da mensagem
admin-settings-help-reader-action-style = Ícones compactos são o padrão. Escolha rótulos para ícones com texto visível; ambos mantêm os nomes das ações e dicas temáticas. Aceita bloqueios de política globais, de domínio e de usuário.
admin-settings-key-nextcloud-sync-interval-secs = Intervalo de atualização automática do Nextcloud
admin-settings-help-nextcloud-sync-interval-secs = Segundos entre atualizações de coleções em segundo plano para contas conectadas do motor nativo. Zero significa apenas manual; valores positivos têm mínimo de sessenta segundos. As falhas aumentam o intervalo. Nenhum login em segundo plano é tentado.
admin-settings-key-mail-folder-role-inbox = Mapeamento da pasta Caixa de entrada
admin-settings-key-mail-folder-role-drafts = Mapeamento da pasta Rascunhos
admin-settings-key-mail-folder-role-sent = Mapeamento da pasta Enviados
admin-settings-key-mail-folder-role-trash = Mapeamento da pasta Lixeira
admin-settings-key-mail-folder-role-junk = Mapeamento da pasta Spam
admin-settings-key-mail-folder-role-archive = Mapeamento da pasta Arquivo
admin-settings-key-mail-request-delivery-receipt = Solicitar notificações de entrega por padrão
admin-settings-help-mail-request-delivery-receipt = Seleciona notificações de entrega para novas mensagens quando a conta aceita DSN SMTP. O bloqueio controla este padrão salvo; receipts.dsnAllowed define separadamente se as solicitações são permitidas. A entrega não prova a leitura.
admin-settings-key-security-signin-geo-enforce-sessions = Aplicar a política de país às sessões existentes
admin-settings-help-security-signin-geo-enforce-sessions = Desativado por padrão. Aplica o escopo de país de login configurado a solicitações autenticadas, acesso bearer nativo e conteúdo assinado do leitor. Clientes bloqueados ainda podem sair. Os fluxos já abertos são verificados ao reconectar; isto não encerra as conexões existentes.
admin-settings-key-security-protection-rate-mode = Orçamentos agregados de solicitações
admin-settings-help-security-protection-rate-mode = Desativado por padrão. Observe registra o esgotamento do orçamento sem recusar; enforce retorna um 429 temporário. Os limites de concorrência de protocolo existentes continuam independentes.
admin-settings-key-security-protection-adaptive-mode = Proteção adaptativa de solicitações
admin-settings-help-security-protection-adaptive-mode = Desativado por padrão. Observe registra anomalias tipadas explicáveis e ações propostas; enforce aplica limitações ou bloqueios temporários limitados. É uma política determinística de contadores, não um classificador de IA.
admin-settings-key-security-protection-client-requests = Solicitações do cliente por janela
admin-settings-help-security-protection-client-requests = Solicitações por janela de um endereço sem login, e todas as solicitações de login. Uma solicitação com sessão válida de caixa de e-mail ou de administrador usa em vez disso uma cota própria, maior. Zero desativa este orçamento.
admin-settings-key-security-protection-account-requests = Solicitações da conta por janela
admin-settings-help-security-protection-account-requests = Solicitações por janela de uma caixa de e-mail conectada, em suas sessões, dispositivos e endereços. Mantida acima da cota anônima para dar mais folga ao uso autenticado. Zero desativa este orçamento.
admin-settings-key-rate-limit-diagnostics-burst = Execuções de diagnóstico simultâneas
admin-settings-help-rate-limit-diagnostics-burst = Execuções de diagnóstico e verificações de backend que uma sessão de administrador pode iniciar em sequência. Iniciar uma verificação que já está em andamento retorna essa verificação em vez de contar de novo.
admin-settings-key-rate-limit-diagnostics-per-minute = Execuções de diagnóstico por minuto
admin-settings-help-rate-limit-diagnostics-per-minute = Execuções de diagnóstico e verificações de backend por minuto que uma sessão de administrador recupera.
admin-settings-key-security-protection-admin-requests = Solicitações de administrador por janela
admin-settings-help-security-protection-admin-requests = Solicitações por janela de um administrador conectado. Mantida acima da cota anônima. Zero desativa este orçamento.
admin-settings-key-security-protection-rate-window-secs = Janela do orçamento de solicitações (segundos)
admin-settings-help-security-protection-rate-window-secs = Intervalo fixo do orçamento de solicitações. O descarte de carga por capacidade e as respostas comuns de não encontrado não são evidência adaptativa.
admin-settings-key-security-protection-adaptive-clients = Observar clientes para a proteção adaptativa
admin-settings-key-security-protection-adaptive-accounts = Observar contas para a proteção adaptativa
admin-settings-key-security-protection-adaptive-admins = Observar administradores para a proteção adaptativa
admin-settings-key-security-protection-short-window-secs = Janela curta de anomalias (segundos)
admin-settings-help-security-protection-short-window-secs = Janela móvel de observação. As janelas longas aceitam de horas a trinta dias; o arredondamento de buckets pode reter no máximo mais um sessenta e quatro avos de janela.
admin-settings-key-security-protection-short-minimum-samples = Mínimo de amostras da razão curta
admin-settings-help-security-protection-short-minimum-samples = Mínimo de solicitações normais mais anômalas observadas antes de a razão acionar. O limite absoluto de anomalias é independente deste mínimo.
admin-settings-key-security-protection-short-absolute-anomalies = Orçamento absoluto de anomalias curto
admin-settings-help-security-protection-short-absolute-anomalies = Aciona quando esta quantidade de anomalias selecionadas se acumula, mesmo que as solicitações normais diluam a razão. Zero desativa este gatilho.
admin-settings-key-security-protection-short-ratio-percent = Razão de anomalias curta (porcentagem)
admin-settings-help-security-protection-short-ratio-percent = Aciona quando as anomalias selecionadas atingem esta porcentagem após a contagem mínima de amostras. Basta o gatilho absoluto ou o de razão. Zero desativa este gatilho.
admin-settings-key-security-protection-long-window-secs = Janela longa de anomalias (segundos)
admin-settings-help-security-protection-long-window-secs = Janela móvel de observação. As janelas longas aceitam de horas a trinta dias; o arredondamento de buckets pode reter no máximo mais um sessenta e quatro avos de janela.
admin-settings-key-security-protection-long-minimum-samples = Mínimo de amostras da razão longa
admin-settings-help-security-protection-long-minimum-samples = Mínimo de solicitações normais mais anômalas observadas antes de a razão acionar. O limite absoluto de anomalias é independente deste mínimo.
admin-settings-key-security-protection-long-absolute-anomalies = Orçamento absoluto de anomalias longo
admin-settings-help-security-protection-long-absolute-anomalies = Aciona quando esta quantidade de anomalias selecionadas se acumula, mesmo que as solicitações normais diluam a razão. Zero desativa este gatilho.
admin-settings-key-security-protection-long-ratio-percent = Razão de anomalias longa (porcentagem)
admin-settings-help-security-protection-long-ratio-percent = Aciona quando as anomalias selecionadas atingem esta porcentagem após a contagem mínima de amostras. Basta o gatilho absoluto ou o de razão. Zero desativa este gatilho.
admin-settings-key-security-protection-normal-cap-per-bucket = Teto da linha de base normal por bucket
admin-settings-help-security-protection-normal-cap-per-bucket = Limita a linha de base normal aprendida em cada bucket da janela móvel. Orçamentos absolutos de anomalias independentes impedem que um enchimento ilimitado de solicitações válidas oculte o abuso.
admin-settings-key-security-protection-signals = Sinais de evidência adaptativa
admin-settings-help-security-protection-signals = Selecione observações autoritativas de malformado, grande demais, credencial rejeitada, permissão negada, prazo do corpo esgotado ou, por opção, acerto em rota de administrador inexistente. A sondagem de rotas de administrador exclui respostas de entidade não encontrada e arquivos estáticos; códigos de status genéricos, recusas por capacidade e cadeias de user-agent não estabelecem abuso.
admin-settings-key-security-protection-throttle-secs = Limitação temporária inicial (segundos)
admin-settings-help-security-protection-throttle-secs = A primeira passagem do limite inicia um prazo fixo de limitação. As solicitações durante esse prazo não o estendem.
admin-settings-key-security-protection-block-secs = Bloqueio temporário escalonado (segundos)
admin-settings-help-security-protection-block-secs = Uma passagem posterior do limite antes da recuperação pode escalar para este bloqueio limitado. Nenhum bloqueio permanente de conta é criado.
admin-settings-key-security-protection-decay-secs = Intervalo de recuperação tranquila (segundos)
admin-settings-help-security-protection-decay-secs = Após este intervalo sem anomalias e a expiração da ação atual, a escalada volta ao estágio inicial. A evidência móvel expira conforme suas janelas configuradas.
admin-settings-key-security-protection-exempt-clients = Endereços e redes de clientes isentos
admin-settings-help-security-protection-exempt-clients = Endereços IP exatos ou redes CIDR isentam apenas a dimensão de cliente. Os limites de conta verificada continuam valendo. Use com critério em redes NAT compartilhadas; endereços encaminhados só são confiáveis por meio de proxies configurados.
admin-settings-key-security-protection-exempt-accounts = IDs de contas verificadas isentas
admin-settings-help-security-protection-exempt-accounts = IDs exatos de contas de caixa de e-mail verificadas pelo servidor isentam apenas a dimensão de conta, nunca a de cliente de origem. Endereços de e-mail e parâmetros de conta fornecidos pelo chamador não são evidência de identidade.
admin-settings-key-security-protection-exempt-admins = IDs de administradores isentos
admin-settings-help-security-protection-exempt-admins = IDs exatos de administradores autenticados isentam apenas a dimensão de administrador. Os bloqueios existentes por falha de login e os prazos rígidos da sessão de administrador continuam inalterados.
admin-settings-key-security-protection-storage-failure = Política de falha de armazenamento da imposição
admin-settings-help-security-protection-storage-failure = Closed recusa o trabalho protegido quando um orçamento de imposição ativado não pode ser verificado. Open o permite temporariamente, com um aviso operacional limitado. Falhas apenas em modo de observação nunca impõem uma ação.
admin-settings-key-security-admin-probe-enabled = Proteção contra sondagem do admin
admin-settings-help-security-admin-probe-enabled = Conta as solicitações que procuram o console de administração por rede de cliente (endereço IPv4, IPv6 /64), desacelera uma origem que insiste e depois a bloqueia. Erros de digitação e favoritos antigos ficam dentro da tolerância. Alterado em Segurança > Proteção adaptativa.
admin-settings-key-security-admin-probe-grace-allowance = Tolerância (erros por janela)
admin-settings-help-security-admin-probe-grace-allowance = Erros ignorados por janela. A pontuação diminui continuamente nesta quantidade de pontos por janela.
admin-settings-key-security-admin-probe-window-secs = Janela de tolerância (segundos)
admin-settings-help-security-admin-probe-window-secs = A janela à qual a tolerância se aplica.
admin-settings-key-security-admin-probe-block-after = Erros além da tolerância antes de um bloqueio
admin-settings-help-security-admin-probe-block-after = Erros acima da tolerância que iniciam um bloqueio. Os erros intermediários são respondidos após o atraso de tarpit.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Atraso de tarpit (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Atraso adicionado a cada erro contado acima da tolerância. 0 desativa o tarpit. Outras solicitações nunca são atrasadas.
admin-settings-key-security-admin-probe-block-secs = Primeiro bloqueio (segundos)
admin-settings-help-security-admin-probe-block-secs = Duração do primeiro bloqueio. Uma origem bloqueada recebe a mesma resposta para qualquer caminho.
admin-settings-key-security-admin-probe-max-block-secs = Maior bloqueio (segundos)
admin-settings-help-security-admin-probe-max-block-secs = Um bloqueio repetido dobra a duração anterior, até este limite.
admin-settings-key-security-admin-probe-repeat-window-secs = Janela de repetição (segundos)
admin-settings-help-security-admin-probe-repeat-window-secs = Um bloqueio que começa dentro deste tempo após o término do anterior conta como repetição.
admin-settings-key-security-admin-probe-block-status = Status da resposta de bloqueio
admin-settings-help-security-admin-probe-block-status = Status HTTP enviado a uma origem bloqueada, com Retry-After: 429 ou 403.
admin-settings-key-security-admin-probe-signals = Erros contados
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (um 404 na profundidade de caminho do console), admin-signin-failed e admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Caminhos de sondagem
admin-settings-help-security-admin-probe-probe-paths = Caminhos que os scanners pedem, como /wp-admin ou /.env. Um caminho que contenha um deles como segmentos inteiros conta como erro. O caminho do próprio console nunca conta.
admin-settings-key-security-admin-probe-allowlist = Redes nunca bloqueadas
admin-settings-help-security-admin-probe-allowlist = Endereços IP ou redes CIDR (escritório, VPN) que nunca são contados nem bloqueados.
admin-settings-key-security-admin-probe-audit-ip = Endereço de origem no log de auditoria
admin-settings-help-security-admin-probe-audit-ip = masked mostra IPv4 /24 e IPv6 /48; full mostra o endereço. A linha do fail2ban sempre traz o endereço completo.
admin-settings-key-security-admin-probe-fail2ban-log = Linha de log do fail2ban ao bloquear
admin-settings-help-security-admin-probe-fail2ban-log = Grava uma linha mailwoman[admin-probe] com rhost=<address> quando uma origem é bloqueada.
admin-settings-key-appearance-custom-css-enabled = Permitir CSS de aparência pessoal
admin-settings-help-appearance-custom-css-enabled = Permite estilos visuais restritos no espaço de trabalho do proprietário. Desativado por padrão; os bloqueios de aparência impostos na caixa de e-mail suspendem esses estilos.
admin-settings-key-appearance-custom-js-enabled = Permitir scripts de aparência pessoal
admin-settings-help-appearance-custom-js-enabled = Permite scripts isolados que só podem solicitar alterações de aparência. Desativado por padrão; os scripts não podem acessar e-mails, dados da conta nem o DOM do aplicativo.
admin-settings-key-mail-modifier-selection = Seleção de mensagens com teclas modificadoras
admin-settings-help-shortcuts-compose = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-archive = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-reply = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-reply-all = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-forward = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-next = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-previous = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-search = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-refresh = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-settings = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-trash = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-mark-read = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-mark-unread = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-spam = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-zoom-in = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-zoom-out = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-zoom-reset = Use inherit para o predefinido selecionado, disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-shortcuts-open-in-tab = Use inherit para o predefinido selecionado (Shift+Enter), disabled para remover esta ação, ou um acorde como Ctrl+Shift+K. Acordes conflitantes ou inválidos não são executados.
admin-settings-help-mail-folder-role-inbox = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-mail-folder-role-drafts = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-mail-folder-role-sent = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-mail-folder-role-trash = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-mail-folder-role-junk = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-mail-folder-role-archive = Null usa a descoberta automática. Use um objeto de caminho exato para padrões compartilhados ou um objeto de ID de caixa de e-mail própria para uma conta. Destinos ausentes ou conflitantes desativam esta função; os bloqueios de administrador se aplicam.
admin-settings-help-security-protection-adaptive-clients = Ativa esta dimensão de sujeito quando o modo adaptativo está ativo. Contas e administradores só são contados depois que sua identidade é verificada.
admin-settings-help-security-protection-adaptive-accounts = Ativa esta dimensão de sujeito quando o modo adaptativo está ativo. Contas e administradores só são contados depois que sua identidade é verificada.
admin-settings-help-security-protection-adaptive-admins = Ativa esta dimensão de sujeito quando o modo adaptativo está ativo. Contas e administradores só são contados depois que sua identidade é verificada.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-mail-modifier-selection = Permite a seleção de intervalos com Ctrl, Command e Shift. Bloqueie este valor para impô-lo; os controles de seleção visíveis continuam disponíveis.
admin-settings-key-mail-drag-to-folder = Arrastar mensagens para pastas
admin-settings-help-mail-drag-to-folder = Permite arrastar mensagens vinculadas à conta para pastas reais. Isso não concede permissão de gravação na caixa de e-mail; o servidor ainda autoriza cada movimentação.
admin-settings-key-admin-allow-last-superadmin-loss = Permitir perder o último Super admin
admin-settings-help-admin-allow-last-superadmin-loss = Desativado por padrão. Ativar isto permite que alterações posteriores removam o último acesso de Super admin. Um Super admin completo deve confirmar a consequência com autenticação recente.
admin-settings-last-superadmin-title = Permitir a perda de todo o acesso de Super admin?
admin-settings-last-superadmin-body = Alterações posteriores em contas podem deixar ninguém em condições de administrar esta implantação. A recuperação exigiria administração local da implantação. Digite ALLOW LAST SUPERADMIN LOSS para ativar esta exceção.
admin-settings-last-superadmin-confirm = Permitir a perda de acesso
admin-settings-key-mail-folder-counts = Contagens das pastas
admin-settings-help-mail-folder-counts = Escolha ocultas, não lidas, não lidas apenas enquanto houver e-mail não lido, total ou ambas as contagens ao lado das pastas. Defina padrões globais, de domínio ou de usuário e bloqueie a escolha quando necessário. Aplica-se a todas as caixas de e-mail vinculadas de uma conta.
admin-settings-key-mail-folder-count-scope = Pastas com contagens
admin-settings-help-mail-folder-count-scope = Todas as pastas, ou apenas as principais (Caixa de entrada, Rascunhos, Enviados, Arquivo, Spam, Lixeira, Caixa de saída e outras pastas de uso especial). Com apenas as principais, pastas personalizadas e suas subpastas não mostram contagem. Aplica-se a todas as caixas de e-mail vinculadas de uma conta.
admin-settings-key-mail-compact-folder-badges = Indicadores secundários compactos nas pastas
admin-settings-help-mail-compact-folder-badges = Mostra indicadores menores fora da Caixa de entrada. Os usuários podem escolher o próprio tamanho, a menos que esta configuração esteja bloqueada.
admin-settings-key-mail-focused-inbox = Caixa de entrada com foco
admin-settings-help-mail-focused-inbox = Divide a Caixa de entrada nas abas Com foco e Outros; newsletters e remetentes automáticos vão para Outros. Desativado por padrão. Os usuários podem ativar ou desativar em Configurações > Pastas e barra lateral, a menos que esta configuração esteja bloqueada.
admin-settings-key-mail-account-unread-badge = Contagem de não lidas ao lado do nome da caixa de e-mail
admin-settings-help-mail-account-unread-badge = Mostra a contagem de não lidas da Caixa de entrada ao lado do endereço ou apelido conectado na barra lateral. Desativado por padrão. As contagens das outras caixas de e-mail no menu de caixas são sempre exibidas. Aplica-se a todas as caixas de e-mail vinculadas de uma conta.
admin-settings-key-privacy-remote-image-downloads = Downloads manuais de imagens remotas
admin-settings-key-privacy-remote-images-toast = Notificação de carregamento de imagens
admin-settings-help-privacy-remote-images-toast = Mostra uma notificação no leitor enquanto as imagens remotas de uma mensagem carregam pelo proxy de privacidade, e informa quando foram carregadas ou quantas não puderam ser. Ativado por padrão. Só altera o que o leitor mostra; quais imagens carregam continua sendo decidido pela política de conteúdo remoto.
admin-settings-help-privacy-remote-image-downloads = Permite o download explícito de uma imagem referenciada pela mensagem do usuário por meio do proxy de privacidade protegido. Desativado por padrão; isso nunca abre o URL externo diretamente.
admin-settings-key-mail-resume-tabs = Retomar abas do espaço de trabalho
admin-settings-help-mail-resume-tabs = Restaura as referências de abas da conta após recarregar. Desativar isto limpa o registro local do espaço de trabalho sem excluir rascunhos.
admin-settings-key-metrics-default-range-seconds = Intervalo padrão do histórico de solicitações
admin-settings-help-metrics-default-range-seconds = Segundos exibidos quando uma solicitação de histórico não especifica um intervalo. Limitado pela retenção e pelo intervalo máximo.
admin-settings-key-metrics-max-range-seconds = Intervalo máximo do histórico de solicitações
admin-settings-help-metrics-max-range-seconds = Máximo de segundos por consulta de histórico. A retenção continua sendo um limite adicional.
admin-settings-key-contacts-favorites-first = Favoritos primeiro nas sugestões
admin-settings-help-contacts-favorites-first = Lista os contatos favoritos antes das outras correspondências nas sugestões de destinatários. Só a ordem muda; nenhuma sugestão é ocultada. As substituições por usuário e domínio podem ser bloqueadas.
admin-settings-key-contacts-sender-icons = Ícones de remetentes incluídos
admin-settings-help-contacts-sender-icons = Mostra ícones incluídos localmente para domínios de remetentes reconhecidos, com iniciais geradas para os demais. As fotos de contatos têm prioridade. Os ícones não verificam a identidade de um remetente. Nenhum serviço externo de logotipos é contatado; as substituições por usuário e domínio podem ser bloqueadas.
admin-settings-key-mail-brand-avatars = Logotipos de marcas ao lado de remetentes verificados
admin-settings-help-mail-brand-avatars = Mostra um logotipo de marca incluído localmente ao lado do remetente no leitor, somente quando a mensagem passou por DKIM e por uma verificação DMARC alinhada para o domínio da própria marca. Mensagens que falham ou não têm essas verificações mantêm o avatar normal. Nenhum logotipo é buscado do remetente nem de um serviço de logotipos.
admin-settings-key-mail-draft-save-mode = Modo de salvamento de rascunhos da conta
admin-settings-help-mail-draft-save-mode = Salvar após uma pausa na digitação, periodicamente ou apenas sob demanda. Salvar exige uma pasta Rascunhos de origem compatível; gravações incertas nunca são repetidas automaticamente.
admin-settings-key-mail-draft-debounce-ms = Pausa de digitação do rascunho (milissegundos)
admin-settings-help-mail-draft-debounce-ms = Aguarda de 500 a 10000 milissegundos após a edição antes de salvar no modo em tempo real.
admin-settings-key-mail-draft-interval-secs = Intervalo de salvamento do rascunho (segundos)
admin-settings-help-mail-draft-interval-secs = Salva rascunhos editados a cada 5 a 3600 segundos no modo periódico. Rascunhos sem alterações não são gravados de novo.
admin-settings-key-login-remember-signins = Lembrar endereços de login bem-sucedidos
admin-settings-help-login-remember-signins = Mantém até dez nomes de usuário ou endereços de e-mail usados com sucesso neste dispositivo para a lista suspensa de login. Nunca salva senhas nem sessões. Desativar remove os endereços salvos deste servidor na próxima vez que um dispositivo carregar a página de login ou concluir um login; não controla o preenchimento automático do próprio navegador.
admin-settings-key-login-show-remembered-hint = Mostrar explicação dos endereços salvos
admin-settings-help-login-show-remembered-hint = Mostra texto explicativo na lista suspensa de logins anteriores. Oculto por padrão. Só se aplica quando lembrar logins bem-sucedidos está ativado.
admin-settings-key-mail-show-from = Mostrar o campo De do editor
admin-settings-help-mail-show-from = Mostra De por padrão nos novos editores. Os usuários podem exibi-lo para uma mensagem; apenas endereços autorizados podem enviar.
admin-settings-key-accounts-linking-enabled = Permitir contas de caixas de e-mail vinculadas
admin-settings-help-accounts-linking-enabled = Desativado por padrão. A desativação global bloqueia todo acesso vinculado; as duas contas também devem atender à política de domínio e de conta. A vinculação exige autenticação real da conta filha e prova recente da principal.
admin-settings-key-accounts-use-linked = Usar caixas de e-mail vinculadas
admin-settings-help-accounts-use-linked = Opção de desativação do usuário para o acesso vinculado. Não pode substituir uma desativação administrativa. Os dados da caixa de e-mail são preservados quando o acesso é desativado.
admin-settings-key-accounts-max-linked = Máximo de caixas de e-mail vinculadas
admin-settings-help-accounts-max-linked = Máximo de caixas filhas verificadas por identidade principal, de 1 a 16. Padrão 5.
admin-settings-key-accounts-store-linked-credentials = Manter o acesso às caixas de e-mail vinculadas
admin-settings-help-accounts-store-linked-credentials = Mantém a senha de cada caixa de e-mail vinculada selada no servidor para que o acesso se renove sozinho, entre sessões e reinicializações. Nunca é mantida para contas de acesso zero nem para caixas com verificação em duas etapas. Desativar apaga as senhas armazenadas; os vínculos passam a se renovar entrando de novo. Ativado por padrão.
admin-settings-key-privacy-strip-mailer-headers = Remover cabeçalhos de programa de e-mail das mensagens enviadas
admin-settings-help-privacy-strip-mailer-headers = Remove X-Mailer, User-Agent e X-MimeOLE de nível superior dos e-mails SMTP e EWS nativos. Mensagens anexadas e cargas de relatório permanecem intactas. Os envios pelo proxy JMAP de origem não são cobertos.
admin-settings-key-images-trusted-hosts = Hosts de imagens confiáveis
admin-settings-help-images-trusted-hosts = Até 100 hosts de imagens, e seus subdomínios, cujas imagens remotas carregam sem uma autorização por mensagem quando o conteúdo remoto está em perguntar. Nunca substitui o modo de bloqueio; um host bloqueado continua bloqueado.
admin-settings-key-images-blocked-hosts = Hosts de imagens bloqueados
admin-settings-help-images-blocked-hosts = Até 100 hosts de imagens, e seus subdomínios, cujas imagens remotas nunca carregam em nenhum modo. Prevalece sobre os hosts confiáveis e sobre qualquer autorização.
admin-settings-key-links-trusted-domains = Domínios de links confiáveis
admin-settings-help-links-trusted-domains = Até 100 domínios exatos cujos links abrem a partir de Revisar links sem a etapa de destino. Sem curingas, esquemas, portas nem endereços IP. Links HTTP simples são sempre revisados. Bloqueie esta configuração para impedir que os usuários alterem a lista.
admin-settings-key-links-trusted-subdomains = Confiar em subdomínios de domínios de links confiáveis
admin-settings-help-links-trusted-subdomains = Desativado: apenas o domínio exato é confiável. Ativado: nomes terminados em .<domain> também são confiáveis. A correspondência é por rótulos inteiros, então example.com nunca confia em evilexample.com.
admin-settings-key-cap-compose-rich-text = Escrita em texto formatado
admin-settings-help-cap-compose-rich-text = Se os usuários podem escrever mensagens formatadas (HTML). Desativado envia apenas texto simples.
admin-settings-key-cap-compose-attachments = Anexos em novas mensagens
admin-settings-help-cap-compose-attachments = Se os usuários podem anexar arquivos às mensagens que escrevem. Imagens embutidas no corpo não contam.
admin-settings-key-cap-compose-share-links = Links de compartilhamento de arquivos grandes
admin-settings-help-cap-compose-share-links = Se os usuários podem enviar um arquivo grande como link de compartilhamento do Nextcloud em vez de anexo.
admin-settings-key-cap-compose-scheduled-send = Envio agendado
admin-settings-help-cap-compose-scheduled-send = Se os usuários podem agendar o envio de uma mensagem para mais tarde.
admin-settings-key-cap-compose-read-receipts = Solicitar confirmações de leitura
admin-settings-help-cap-compose-read-receipts = Se os usuários podem pedir confirmação de leitura aos destinatários.
admin-settings-key-cap-signatures = Assinaturas
admin-settings-help-cap-signatures = Se os usuários podem criar e editar as próprias assinaturas.
admin-settings-key-cap-identities = Identidades de envio
admin-settings-help-cap-identities = Se os usuários podem adicionar e editar as identidades com as quais enviam.
admin-settings-key-cap-keys = Chaves OpenPGP e S/MIME
admin-settings-help-cap-keys = Se os usuários têm o módulo Chaves e podem assinar e criptografar mensagens.
admin-settings-key-cap-links-trusted = Editar domínios de links confiáveis
admin-settings-help-cap-links-trusted = Se os usuários podem alterar a própria lista de domínios de links confiáveis. A lista do administrador continua valendo.
admin-settings-key-cap-zoom = Zoom do leitor
admin-settings-help-cap-zoom = Se os usuários podem ampliar o leitor de mensagens.
admin-settings-key-cap-mail-focused-inbox = Caixa de entrada com foco
admin-settings-help-cap-mail-focused-inbox = Se os usuários podem ativar as abas Com foco e Outros da caixa de entrada.
admin-settings-key-cap-views-events = Visualização de eventos
admin-settings-help-cap-views-events = Se os usuários têm a visualização Eventos do e-mail de calendário.
admin-settings-key-cap-views-attachments = Visualização de anexos
admin-settings-help-cap-views-attachments = Se os usuários têm a visualização Anexos. As mensagens continuam acessíveis pela pesquisa.
admin-settings-key-cap-search-operators = Operadores de pesquisa
admin-settings-help-cap-search-operators = Se os usuários podem pesquisar com operadores como from:, subject: ou has:attachment.
admin-settings-key-cap-saved-searches = Pesquisas salvas
admin-settings-help-cap-saved-searches = Se os usuários podem salvar pesquisas.
admin-settings-key-cap-vacation = Resposta automática de férias
admin-settings-help-cap-vacation = Se os usuários podem definir uma resposta automática de férias.
admin-settings-key-cap-labels = Marcadores
admin-settings-help-cap-labels = Se os usuários podem colocar marcadores nas mensagens.
admin-settings-key-cap-snooze = Adiar
admin-settings-help-cap-snooze = Se os usuários podem adiar mensagens. Encerrar um adiamento continua possível.
admin-settings-key-cap-follow-up = Lembretes de acompanhamento
admin-settings-help-cap-follow-up = Se os usuários podem definir lembretes de acompanhamento em mensagens. Remover um continua possível.
admin-settings-key-cap-masked = Endereços mascarados
admin-settings-help-cap-masked = Se os usuários podem criar e gerenciar endereços mascarados.
admin-settings-key-cap-directory = Pesquisa no diretório
admin-settings-help-cap-directory = Se os usuários podem pesquisar no diretório da organização.
admin-settings-key-cap-mcp = Acesso MCP
admin-settings-help-cap-mcp = Se as chaves e tokens do usuário podem usar o endpoint MCP.
admin-settings-key-cap-appearance-custom = Personalização da aparência
admin-settings-help-cap-appearance-custom = Se os usuários podem alterar tema, cor de destaque, cores personalizadas e de indicadores, densidade e fonte. Desativado mantém a aparência do administrador.
admin-settings-key-cap-appearance-custom-code = CSS e scripts personalizados
admin-settings-help-cap-appearance-custom-code = Se os usuários podem salvar o próprio CSS e scripts de aparência. As chaves de implantação em Aparência continuam valendo acima desta.
admin-settings-key-cap-avatars = Avatares de caixas de e-mail
admin-settings-help-cap-avatars = Se os usuários podem alterar os avatares das caixas de e-mail.
admin-settings-key-cap-shortcuts = Atalhos de teclado personalizados
admin-settings-help-cap-shortcuts = Se os usuários podem alterar os atalhos de teclado. Os atalhos padrão continuam funcionando.
admin-settings-key-cap-analytics = Análise de produto
admin-settings-help-cap-analytics = Se os usuários são consultados sobre o consentimento de análise e podem enviar eventos de uso.
admin-settings-key-cap-apps-native = Aplicativos de desktop e móveis
admin-settings-help-cap-apps-native = Se os usuários podem usar os aplicativos de desktop e móveis.
admin-settings-key-access-protocols = Protocolos de acesso ao e-mail
admin-settings-help-access-protocols = Com quais protocolos de servidor de e-mail estes usuários podem entrar: imap, pop3, ews. Os logins por um servidor JMAP seguem esse servidor.
admin-settings-key-limit-linked-mailboxes = Caixas de e-mail vinculadas por usuário
admin-settings-help-limit-linked-mailboxes = Quantas caixas de e-mail um usuário pode vincular, abaixo do máximo do servidor. 0 significa que vale apenas o máximo do servidor.
admin-settings-key-login-show-account-help-notice = Mostrar aviso de ajuda da conta no login
admin-settings-help-login-show-account-help-notice = Mostra uma explicação curta após um login malsucedido. Oculto por padrão; o aviso não revela se uma conta existe ou está desativada.
admin-settings-key-appearance-leetspeak = Estilo de texto leetspeak
admin-settings-help-appearance-leetspeak = Exibe os rótulos da interface e o texto somente leitura das mensagens em leetspeak. A mensagem original e o texto editável permanecem inalterados. Os usuários podem escolher texto normal, a menos que esta configuração esteja bloqueada.
admin-settings-key-mail-cache-enabled = Ativar cache da sessão
admin-settings-key-mail-cache-mailboxes = Armazenar em cache as listas de mensagens das pastas
admin-settings-key-mail-cache-messages = Armazenar em cache as mensagens abertas
admin-settings-key-mail-cache-fresh-seconds = Intervalo de atualidade (segundos)
admin-settings-key-mail-cache-max-age-seconds = Idade máxima em cache (segundos)
admin-settings-key-mail-cache-max-entries = Máximo de entradas em cache
admin-settings-key-mail-cache-max-bytes = Orçamento de memória (bytes)
admin-settings-key-mail-cache-prefetch-enabled = Ativar pré-carregamento especulativo
admin-settings-key-mail-cache-hover-enabled = Pré-carregar ao apontar para uma pasta
admin-settings-key-mail-cache-focus-enabled = Pré-carregar quando uma pasta recebe o foco do teclado
admin-settings-key-mail-cache-dwell-ms = Atraso de ponteiro e foco (milissegundos)
admin-settings-key-mail-cache-idle-enabled = Pré-carregar pastas quando ocioso
admin-settings-key-mail-cache-idle-delay-ms = Atraso de ociosidade (milissegundos)
admin-settings-key-mail-cache-max-concurrency = Máximo de pré-carregamentos simultâneos
admin-settings-key-mail-cache-max-queued = Máximo de pré-carregamentos na fila
admin-settings-key-mail-cache-preload-messages = Permitir corpos de mensagens especulativos
admin-settings-key-mail-cache-excluded-folders = Pastas excluídas do pré-carregamento
admin-settings-key-mail-cache-refresh-on-open = Atualizar o conteúdo em cache ao abrir
admin-settings-help-mail-cache-refresh-on-open = Atualiza o conteúdo em cache ao abrir
admin-settings-key-mail-templates-enabled = Modelos de resposta
admin-settings-help-mail-templates-enabled = Ativa modelos de resposta em texto simples, pessoais e compartilhados, no editor. Os usuários podem desativar, a menos que esta configuração esteja bloqueada.
admin-settings-key-mail-templates-allow-personal = Permitir modelos pessoais
admin-settings-help-mail-templates-allow-personal = Permite que os usuários criem os próprios modelos de resposta. Os modelos compartilhados são gerenciados separadamente pelos administradores.
admin-settings-key-mail-templates-shared = Modelos de resposta compartilhados
admin-settings-help-mail-templates-shared = Gerencie respostas reutilizáveis em texto simples para este escopo. Os modelos nunca definem destinatários nem enviam mensagens.
admin-settings-key-mail-templates-personal = Modelos de resposta pessoais
admin-settings-help-mail-templates-personal = Respostas pessoais reutilizáveis em texto simples. Criar ou editar exige que os modelos de resposta e os modelos pessoais estejam ativados.
admin-settings-key-mail-compose-window = Janela do editor
admin-settings-help-mail-compose-window = Abre novos rascunhos em uma aba do espaço de trabalho ou em uma janela separada. Pop-ups bloqueados recorrem a uma aba.
admin-settings-key-mail-reply-copy-to-original = Manter as respostas na pasta original
admin-settings-help-mail-reply-copy-to-original = Salva uma cópia da resposta enviada na pasta da mensagem original, onde houver suporte.
admin-settings-key-mail-compose-check-invalid-recipients = Verificar destinatários inválidos antes de enviar
admin-settings-help-mail-compose-check-invalid-recipients = O que acontece quando uma mensagem é enviada a um endereço inválido: permitir, avisar, avisar e lembrar a escolha ou recusar.
admin-settings-key-mail-compose-check-missing-attachment = Verificar anexo ausente antes de enviar
admin-settings-help-mail-compose-check-missing-attachment = O que acontece quando o texto menciona um anexo mas nenhum está anexado: permitir, avisar, avisar e lembrar a escolha ou recusar.
admin-settings-key-mail-compose-check-empty-subject = Verificar assunto vazio antes de enviar
admin-settings-help-mail-compose-check-empty-subject = O que acontece quando uma mensagem não tem assunto: permitir, avisar, avisar e lembrar a escolha ou recusar.
admin-settings-key-mail-compose-auto-chip-recipients = Concluir automaticamente os destinatários digitados
admin-settings-help-mail-compose-auto-chip-recipients = Transforma um endereço digitado em um chip de destinatário quando um espaço, vírgula ou ponto e vírgula o segue.
admin-settings-key-mail-compose-header-collapse = Cabeçalho do editor
admin-settings-help-mail-compose-header-collapse = Como se comportam as linhas Para, Cc, Cco e Assunto do editor: expandidas, recolhidas em um resumo de uma linha ou auto (recolhidas enquanto se escreve o corpo, abertas quando um campo recebe o foco). Os usuários podem recolher ou expandir por conta própria, a menos que isto esteja bloqueado.
admin-settings-key-mail-compose-recipient-separator = Separador de destinatários
admin-settings-help-mail-compose-recipient-separator = O separador inserido após um destinatário concluído: vírgula ou ponto e vírgula.
admin-settings-key-mail-reply-from-mode = Identidade usada em respostas e encaminhamentos
admin-settings-help-mail-reply-from-mode = De qual identidade uma resposta ou encaminhamento é enviado: aquela para a qual o original foi endereçado, a padrão, um seletor ou a atualmente selecionada.
admin-settings-key-mail-forward-mode = Formato padrão de encaminhamento
admin-settings-help-mail-forward-mode = Encaminhar o texto no corpo ou anexar a mensagem original como arquivo .eml.
admin-settings-key-mail-show-go-to-top = Mostrar o botão Ir para o topo
admin-settings-help-mail-show-go-to-top = Mostra na lista de mensagens um botão Ir para o topo que rola de volta à primeira mensagem. Desativado por padrão.
admin-settings-key-mail-show-create-folder-button = Mostrar o botão Criar pasta
admin-settings-help-mail-show-create-folder-button = Mostra na barra lateral um botão Criar pasta apenas com ícone ao lado de Escrever. Desativado por padrão.
admin-settings-key-security-protection-user-agent-mode = Modo de observação de user-agent
admin-settings-help-security-protection-user-agent-mode = Enforce por padrão: cabeçalhos de bots reconhecidos selecionados são bloqueados imediatamente quando o bloqueio de bots na primeira solicitação está ativado. Observe registra as decisões sem recusar; off desativa esta dimensão. Declarações de cabeçalho não são identidades verificadas.
admin-settings-key-security-protection-user-agent-block-bots = Bloquear bots reconhecidos na primeira solicitação
admin-settings-help-security-protection-user-agent-block-bots = Verdadeiro por padrão. Cabeçalhos selecionados de bots de busca, IA, automação e personalizados acionam um bloqueio temporário HTTP 429 na primeira solicitação no modo Enforce. Observe apenas registra; falso restaura os orçamentos de solicitações anteriores. Cabeçalhos ausentes ou incomuns, escopo e exceções mantêm o comportamento existente.
admin-settings-key-security-protection-user-agent-categories = Categorias de user-agent
admin-settings-key-security-protection-user-agent-weak-requests = Limite de solicitações de sinal fraco
admin-settings-key-security-protection-user-agent-crawler-requests = Limite de solicitações de rastreadores
admin-settings-key-security-protection-user-agent-window-secs = Janela de observação de user-agent (segundos)
admin-settings-key-security-protection-user-agent-max-action = Resposta máxima de user-agent
admin-settings-help-security-protection-user-agent-max-action = Escalada máxima para evidência fraca ou de solicitações repetidas, padrão limitação. Este limite não restringe os bloqueios na primeira solicitação de cabeçalhos de bots reconhecidos selecionados.
admin-settings-key-security-protection-user-agent-scope = Escopo da política de user-agent
admin-settings-key-security-protection-user-agent-adaptive = Observações adaptativas de user-agent
admin-settings-key-security-protection-user-agent-patterns = Padrões de user-agent
admin-settings-key-security-protection-user-agent-exceptions = Exceções de user-agent
admin-settings-key-mail-reader-unsafe-html-mode = HTML de mensagem potencialmente inseguro
admin-settings-help-mail-reader-unsafe-html-mode = Ask oferece por padrão um botão de HTML sanitizado por mensagem. Strict mostra apenas texto seguro; sanitized mostra automaticamente o HTML limpo. Conteúdo ativo e recursos remotos automáticos continuam bloqueados. Bloqueie para impor esta política.
admin-settings-key-reader-attachments-density = Densidade dos anexos
admin-settings-help-reader-attachments-density = Botões de arquivo compactos por padrão, ou pré-visualizações confortáveis. Controla a apresentação quando os anexos de uma mensagem estão expandidos.
admin-settings-key-cap-compose-open-tracking = Permitir rastreamento de abertura de mensagens
admin-settings-help-cap-compose-open-tracking = Se esta conta pode registrar aberturas de mensagens. Exige um URL público configurado; contas desativadas não podem criar nem vincular tokens de rastreamento.
admin-settings-key-mail-open-tracking = Rastrear aberturas por padrão
admin-settings-help-mail-open-tracking = Escolha padrão de rastreamento por mensagem para mensagens formatadas elegíveis. Os usuários podem substituí-la, a menos que bloqueada; desativar a capacidade de rastreamento sempre prevalece.
admin-settings-key-cap-reading-original-html = Permitir renderização do HTML original
admin-settings-help-cap-reading-original-html = Se os usuários podem optar pela visualização do HTML original em mensagens individuais. O renderizador sanitizado padrão e a segurança do quadro isolado continuam valendo.
admin-settings-key-mail-reader-allow-original-html = Oferecer visualização do HTML original
admin-settings-help-mail-reader-allow-original-html = Mostra a opção por mensagem de usar o HTML original quando a capacidade de leitura permite. Desativado por padrão; não altera o renderizador de mensagens padrão.
admin-settings-key-privacy-tracker-override-allowed = Oferecer consentimento de rastreadores por mensagem
admin-settings-help-privacy-tracker-override-allowed = Mostra a ação Permitir rastreadores na barra de privacidade da mensagem. Disponível por padrão, mas nunca ativa sem uma autorização explícita. Uma política de bloqueio de rastreadores bloqueada ainda proíbe a substituição.
admin-settings-key-privacy-max-concurrent-global = Buscas de imagens paralelas globais
admin-settings-help-privacy-max-concurrent-global = Máximo de solicitações de rede de imagens ativas neste processo do servidor. Intervalo 1–256; padrão 128. Os limites de mensagem e de conta continuam valendo.
admin-settings-key-privacy-max-concurrent-pipeline = Pipelines de processamento de imagens simultâneos
admin-settings-help-privacy-max-concurrent-pipeline = Máximo de imagens admitidas juntas em busca, decodificação e publicação no cache. Intervalo 1–256; padrão 128. Os tetos de rede e de decodificador são aplicados separadamente.
admin-settings-key-privacy-max-concurrent-decodes = Decodificadores de imagem paralelos
admin-settings-help-privacy-max-concurrent-decodes = Máximo de decodificações de imagem em execução ao mesmo tempo. Intervalo 1–16; padrão 10. Valores maiores usam mais CPU e memória.
admin-settings-key-privacy-max-queued-requests = Solicitações de imagem em espera
admin-settings-help-privacy-max-queued-requests = Máximo de solicitações esperando capacidade de rede ou processamento de imagens. Intervalo 16–16384; padrão 2048. O estouro ou a expiração da fila continuam podendo ser repetidos.
admin-settings-key-privacy-rate-burst = Margem de pico de solicitações de imagem
admin-settings-help-privacy-rate-burst = Tokens de busca por conta iniciais e máximos, reabastecidos à taxa de solicitações de imagem configurada. Intervalo 1–12000; padrão 1024. Sucessos em cache não consomem tokens.
admin-settings-key-privacy-image-rate-mode = Dimensionamento da taxa de solicitações de imagem
admin-settings-help-privacy-image-rate-mode = Automatic dimensiona o pico e o reabastecimento por conta a partir da capacidade das réplicas, da carga da fila e do número de imagens da mensagem, dentro dos pisos e tetos abaixo. Manual usa exatamente as configurações de margem de pico e taxa de solicitações. Padrão automatic. As solicitações acima da margem esperam por um token até o prazo da fila de busca; autenticação e autorizações são verificadas antes.
admin-settings-help-mail-cache-enabled = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-mailboxes = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-messages = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-fresh-seconds = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-max-age-seconds = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-max-entries = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-max-bytes = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-prefetch-enabled = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-hover-enabled = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-focus-enabled = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-dwell-ms = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-idle-enabled = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-idle-delay-ms = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-max-concurrency = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-max-queued = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-preload-messages = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-mail-cache-excluded-folders = Mantém as pastas e mensagens visitadas recentemente na memória desta aba. O conteúdo em cache é atualizado em segundo plano. Fechar a aba ou sair o apaga. O pré-carregamento nunca marca e-mails como lidos nem carrega imagens remotas.
admin-settings-help-security-protection-user-agent-categories = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-weak-requests = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-crawler-requests = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-window-secs = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-scope = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-adaptive = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-patterns = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.
admin-settings-help-security-protection-user-agent-exceptions = Configura esta parte da observação opcional de user-agent. O texto de user-agent fornecido pelo cliente é um sinal fraco, não prova de identidade ou de abuso.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-privacy-image-rate-burst-floor = Piso da margem de rajada automática
admin-settings-help-privacy-image-rate-burst-floor = Menor margem de rajada por conta no modo de taxa automático. Intervalo 1–12000; padrão 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Teto da margem de rajada automática
admin-settings-help-privacy-image-rate-burst-ceiling = Maior margem de rajada por conta no modo de taxa automático. Intervalo 1–12000; padrão 2048.
admin-settings-key-privacy-image-rate-refill-floor = Piso de reposição automática
admin-settings-help-privacy-image-rate-refill-floor = Menor reposição por conta, em tokens de busca por minuto, no modo de taxa automático. Intervalo 10–60000; padrão 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Teto de reposição automática
admin-settings-help-privacy-image-rate-refill-ceiling = Maior reposição por conta, em tokens de busca por minuto, no modo de taxa automático. Intervalo 10–60000; padrão 6000.
admin-settings-key-privacy-image-cache-mode = Dimensionamento do cache de imagens
admin-settings-help-privacy-image-cache-mode = O modo automático dimensiona o cache pela memória que sobra após as reservas do decodificador e do pipeline e aumenta a duração enquanto a taxa de acertos é alta e há memória livre. O modo manual usa exatamente as configurações de tamanho do cache, número de entradas e duração. Padrão: automático.
admin-settings-key-privacy-image-cache-bytes-floor = Piso do tamanho automático do cache
admin-settings-help-privacy-image-cache-bytes-floor = Menor cache de imagens, em bytes, no modo automático. Intervalo 0–1073741824; padrão 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Teto do tamanho automático do cache
admin-settings-help-privacy-image-cache-bytes-ceiling = Maior cache de imagens, em bytes, no modo automático. Intervalo 0–1073741824; padrão 268435456.
admin-settings-key-privacy-cache-entries = Limite de entradas do cache de imagens
admin-settings-help-privacy-cache-entries = Máximo de imagens em cache; é a contagem fixa no modo manual e o teto no modo automático. Intervalo 16–65536; padrão 4096.
admin-settings-key-privacy-cache-entry-bytes = Maior imagem em cache
admin-settings-help-privacy-cache-entry-bytes = Maior imagem recodificada mantida no cache, em bytes. Imagens maiores são entregues, mas não mantidas. Intervalo 65536–16777216; padrão 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Piso da duração automática do cache
admin-settings-help-privacy-image-cache-ttl-floor-secs = Menor duração do cache, em segundos, no modo automático, usada com taxa de acertos baixa ou sob pressão de memória. Intervalo 0–86400; padrão 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Teto da duração automática do cache
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Maior duração do cache, em segundos, no modo automático, usada com taxa de acertos alta e memória livre. Intervalo 0–86400; padrão 21600.
admin-settings-key-privacy-cache-ttl-secs = Duração do cache de imagens bem-sucedidas
admin-settings-help-privacy-cache-ttl-secs = Segundos em que as imagens decodificadas continuam reutilizáveis neste processo. Intervalo 0–86400; padrão 3600. Zero desativa a reutilização de imagens bem-sucedidas; a autorização é sempre verificada antes.
admin-settings-key-privacy-image-latency-tracking = Acompanhamento de velocidade do proxy de imagens
admin-settings-help-privacy-image-latency-tracking = Registra quanto tempo cada imagem via proxy leva (mediana, p95, p99 e o tempo de espera, busca e decodificação) como contagens em intervalos de tempo fixos na memória. Nenhuma URL, host ou conta é armazenado. Ativado por padrão; o histórico é zerado ao reiniciar e vale por processo do servidor. Desativar interrompe novos registros; a seção Velocidade continua mostrando o que foi registrado.
admin-settings-key-privacy-negative-cache-ttl-secs = Duração do cache de imagens com falha
admin-settings-help-privacy-negative-cache-ttl-secs = Segundos em que falhas elegíveis do servidor de origem são lembradas para evitar solicitações repetidas. Intervalo 0–300; padrão 10. Zero desativa a reutilização de falhas. Uma nova tentativa explícita limpa as falhas afetadas.
admin-settings-key-privacy-image-retry-attempts = Novas tentativas automáticas de imagem
admin-settings-help-privacy-image-retry-attempts = Máximo de novas tentativas para falhas temporárias de imagem em um carregamento visível de mensagem. Intervalo 0–8; padrão 3. Zero desativa as tentativas automáticas, mas mantém a manual.
admin-settings-key-privacy-image-retry-delay-ms = Atraso das novas tentativas automáticas de imagem
admin-settings-help-privacy-image-retry-delay-ms = Milissegundos entre as tentativas automáticas, depois que o quadro atual se estabiliza. Intervalo 500–60000; padrão 5000. Recusas permanentes não são repetidas automaticamente.
admin-settings-key-privacy-image-decode-timeout-ms = Tempo limite de decodificação de imagem
admin-settings-help-privacy-image-decode-timeout-ms = Milissegundos de relógio permitidos para uma decodificação de imagem em sandbox após a busca. Intervalo 1000–60000; padrão 10000. Aumente para imagens grandes; os tempos limite de rede são independentes.
admin-settings-key-privacy-image-concurrency-mode = Perfil de concorrência de imagens
admin-settings-help-privacy-image-concurrency-mode = O modo automático se adapta à capacidade de CPU, à memória do contêiner e à quantidade de imagens aprovadas de cada mensagem. Poucos recursos, Equilibrado e Alto desempenho também se adaptam, com limites de segurança diferentes. O modo manual usa diretamente os limites numéricos. Aplicar um perfil altera apenas esta configuração.
admin-settings-cat-image-proxy = Proxy de imagens
admin-settings-key-privacy-image-ram-max-mi-b = Máximo da camada de RAM
admin-settings-help-privacy-image-ram-max-mi-b = Limite superior, em MiB, da camada de imagens em memória, aplicado sobre o tamanho de cache automático ou manual. Zero não impõe limite. Mantenha pequeno e deixe a camada de disco guardar a maior parte. Intervalo 0–1048576; padrão 0.
admin-settings-key-privacy-image-disk-enabled = Camada de disco de imagens
admin-settings-help-privacy-image-disk-enabled = Mantém em disco, atrás da camada de RAM, as imagens já recodificadas. Desativada por padrão; quando o diretório não existe, está cheio ou é somente leitura, o cache continua silenciosamente apenas na RAM. Só a saída do jail é armazenada, com nomes de arquivo em hash.
admin-settings-key-privacy-image-disk-path = Diretório de disco de imagens
admin-settings-help-privacy-image-disk-path = Diretório absoluto usado pela camada de disco. Vazio usa uma pasta image-cache ao lado do arquivo do banco de dados. Monte um volume gravável ali quando o sistema de arquivos raiz for somente leitura. Caminhos relativos e .. são recusados e deixam a camada desativada.
admin-settings-key-privacy-image-disk-mode = Dimensionamento do disco de imagens
admin-settings-help-privacy-image-disk-mode = O modo automático usa um décimo do espaço que este cache poderia usar (espaço livre mais o que ele já ocupa), entre o piso e o teto e nunca mais da metade desse espaço. O modo manual usa exatamente o tamanho máximo abaixo. Padrão: automático.
admin-settings-key-privacy-image-disk-max-mi-b = Tamanho do disco de imagens (manual)
admin-settings-help-privacy-image-disk-max-mi-b = Tamanho da camada de disco em MiB no modo manual; a leitura mostra o tamanho efetivo nos dois modos. Intervalo 1–1048576; padrão 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Piso do tamanho automático do disco
admin-settings-help-privacy-image-disk-floor-mi-b = Menor camada de disco em MiB no modo automático; também usado enquanto o espaço livre é desconhecido. Intervalo 0–1048576; padrão 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Teto do tamanho automático do disco
admin-settings-help-privacy-image-disk-ceiling-mi-b = Maior camada de disco em MiB no modo automático. Intervalo 0–1048576; padrão 4096.
admin-settings-key-privacy-image-disk-entries = Limite de entradas em disco
admin-settings-help-privacy-image-disk-entries = Máximo de imagens mantidas em disco. Cada entrada também custa uma quantidade fixa de RAM no índice, que conta contra a camada de RAM. Intervalo 16–1048576; padrão 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Maior imagem em disco
admin-settings-help-privacy-image-disk-entry-bytes = Maior imagem recodificada gravada em disco, em bytes. Imagens acima do limite de entrada da RAM, mas dentro deste, vão direto para o disco. Intervalo 65536–67108864; padrão 8388608.
admin-settings-key-privacy-image-disk-min-hits = Acertos antes de gravar em disco
admin-settings-help-privacy-image-disk-min-hits = Acertos de que uma entrada da RAM precisa para que sua remoção a grave em disco em vez de descartá-la. Zero grava toda entrada removida cuja pontuação passe o limite de rebaixamento. Intervalo 0–100; padrão 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Fila de gravação em disco
admin-settings-help-privacy-image-disk-write-queue-mi-b = Dados de imagem, em MiB, que podem aguardar o gravador de disco em segundo plano. Contam contra a camada de RAM; acima disso, as imagens removidas são descartadas em vez de enfileiradas. Intervalo 1–256; padrão 8.
admin-settings-key-privacy-image-ttl-step-secs = Incremento de duração por acerto
admin-settings-help-privacy-image-ttl-step-secs = Segundos somados à duração restante de uma imagem a cada acerto, multiplicados por 1 mais o logaritmo da contagem de acertos com decaimento, de modo que imagens muito usadas durem mais. Zero mantém todas as imagens na duração base. Intervalo 0–86400; padrão 600.
admin-settings-key-privacy-image-ttl-max-secs = Maior duração estendida
admin-settings-help-privacy-image-ttl-max-secs = Maior duração restante, em segundos, que os acertos podem acumular. Intervalo 60–2592000; padrão 86400.
admin-settings-key-privacy-image-max-age-secs = Idade máxima da imagem
admin-settings-help-privacy-image-max-age-secs = Idade, em segundos, após a qual uma imagem é buscada de novo, independentemente dos acertos. O caminho de busca não guarda ETag nem Last-Modified do servidor de origem, portanto não há revalidação condicional; a imagem é simplesmente buscada outra vez. Intervalo 3600–7776000; padrão 604800.
admin-settings-key-privacy-image-score-half-life-secs = Meia-vida da pontuação
admin-settings-help-privacy-image-score-half-life-secs = Segundos para que a contagem de acertos e a recência de uma imagem caiam pela metade na pontuação de prioridade que decide RAM, disco e remoção. Intervalo 60–604800; padrão 3600.
admin-settings-key-privacy-image-score-demote-milli = Pontuação de rebaixamento
admin-settings-help-privacy-image-score-demote-milli = Pontuação, em milésimos, abaixo da qual uma imagem da RAM é rebaixada para o disco ou descartada. O limite de promoção é este valor mais a histerese. Intervalo 0–10000; padrão 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Histerese de promoção
admin-settings-help-privacy-image-score-hysteresis-percent = Porcentagem pela qual o limite de promoção fica acima do limite de rebaixamento. Uma pontuação entre os dois mantém a imagem onde está. Intervalo 0–500; padrão 50.
admin-settings-key-privacy-image-context-open-percent = Peso da mensagem aberta
admin-settings-help-privacy-image-context-open-percent = Peso na pontuação, em porcentagem, para imagens solicitadas por uma mensagem. As imagens de uma mensagem passam a contar como Recentes após dez minutos de inatividade. Intervalo 100–1000; padrão 200.
admin-settings-key-privacy-image-context-recent-percent = Peso da mensagem recente
admin-settings-help-privacy-image-context-recent-percent = Peso na pontuação, em porcentagem, para imagens usadas recentemente. Solicitações que não indicam nenhuma mensagem pesam 100. Intervalo 100–1000; padrão 150.
admin-settings-key-privacy-image-account-share-percent = Fatia de RAM por conta
admin-settings-help-privacy-image-account-share-percent = Maior fatia da camada de RAM que as inserções de uma conta podem ocupar enquanto outras contas usam o cache; acima dela, a conta remove primeiro suas próprias imagens de menor pontuação. Uma única conta pode usar tudo. Intervalo 1–100; padrão 50.
admin-settings-key-privacy-image-sweep-secs = Intervalo de limpeza do cache
admin-settings-help-privacy-image-sweep-secs = Segundos entre as varreduras em segundo plano que expiram imagens, repontuam um lote limitado, rebaixam imagens frias da RAM sob pressão e verificam de novo o espaço livre em disco. Intervalo 5–3600; padrão 30.
admin-settings-key-privacy-image-snapshot-secs = Intervalo do instantâneo do índice
admin-settings-help-privacy-image-snapshot-secs = Segundos entre instantâneos atômicos do índice de disco, que preservam contagens de acertos e durações estendidas entre reinicializações sem uma gravação por acerto. Zero desativa os instantâneos; após reiniciar, o cabeçalho de cada arquivo é lido. Intervalo 0–86400; padrão 300.
admin-settings-key-privacy-image-sweep-batch = Tamanho do lote de varredura
admin-settings-help-privacy-image-sweep-batch = Entradas que uma varredura de manutenção pode expirar, repontuar ou remover. Limita o trabalho feito sob o bloqueio do cache. Intervalo 16–8192; padrão 256.
admin-image-proxy-title = Leitura do cache de imagens
admin-image-proxy-unavailable = A leitura do cache de imagens não está disponível.
admin-image-proxy-unknown = desconhecido
admin-image-proxy-ram = Camada de RAM: { $used } usados de { $budget } (configurado { $configured }), { $entries } imagens, { $hot } quentes, { $cold } frias, taxa de acertos { $hit }.
admin-image-proxy-disk-off = Camada de disco: desativada.
admin-image-proxy-disk = Camada de disco: { $bytes } usados de { $budget } efetivos ({ $mode }; piso { $floor }, teto { $ceiling }, manual { $manual }), { $entries } imagens, { $hot } quentes, { $cold } frias, taxa de acertos { $hit }.
admin-image-proxy-disk-inactive = Camada de disco: ativada, mas inativa. { $reason }
admin-image-proxy-moves = Promoções por minuto: { $perMin }. Total de promoções { $promotions }, rebaixamentos { $demotions }, descartes { $drops }. Entradas do índice: { $index }.
admin-image-proxy-refresh = Atualizar leitura
admin-settings-key-security-trust-enabled = Níveis de confiança
admin-settings-help-security-trust-enabled = Atribui um nível a cada solicitação: autenticada, origem conhecida (um login bem-sucedido recente desse endereço) ou desconhecida. Só mudam as cotas de solicitações anônimas; credenciais, segundos fatores e bloqueios nunca são relaxados.
admin-settings-key-security-trust-known-ttl-days = Janela de origem conhecida (dias)
admin-settings-help-security-trust-known-ttl-days = Por quanto tempo um login bem-sucedido mantém seu endereço como conhecido. Tentativas com falha nunca contam.
admin-settings-key-security-trust-known-capacity = Origens conhecidas mantidas
admin-settings-help-security-trust-known-capacity = Máximo de origens conhecidas mantidas na memória (cada uma com um hash com sal, um país e uma contagem de contas). A mais antiga é descartada primeiro.
admin-settings-key-security-trust-persist = Salvar origens conhecidas entre reinicializações
admin-settings-help-security-trust-persist = Salva a tabela de origens conhecidas (hashes com chave, país, hora da última vez visto, contagem de contas) no repositório de configurações e a carrega na inicialização. Entradas mais antigas que a janela são descartadas.
admin-settings-key-security-trust-persist-interval-secs = Intervalo de salvamento das origens conhecidas (segundos)
admin-settings-help-security-trust-persist-interval-secs = Segundos entre salvamentos, de 30 a 3600. O salvamento só ocorre quando a tabela mudou, e mais uma vez no desligamento ordenado.
admin-settings-key-security-trust-unknown-percent = Cota de origem desconhecida (%)
admin-settings-help-security-trust-unknown-percent = Cota de solicitações anônimas de uma origem desconhecida, como porcentagem dos limites configurados. 100 mantém a base; menos é mais rígido.
admin-settings-key-security-trust-known-percent = Cota de origem conhecida (%)
admin-settings-help-security-trust-known-percent = Cota de solicitações anônimas de uma origem conhecida. 200 dobra os orçamentos de solicitações e reduz à metade o intervalo de espera do login e a dificuldade do desafio. Nunca ignora credenciais, segundos fatores nem bloqueios.
admin-settings-key-security-trust-ua-missing = Bots: user-agent vazio
admin-settings-help-security-trust-ua-missing = Ação para solicitações anônimas com user-agent vazio: permitir, limitar ou bloquear. Sessões autenticadas nunca são afetadas.
admin-settings-key-security-trust-ua-uncommon = Bots: user-agent não reconhecido
admin-settings-help-security-trust-ua-uncommon = Ação para solicitações anônimas com user-agent não reconhecido: permitir, limitar ou bloquear.
admin-settings-key-security-trust-ua-automation = Bots: scripts e automação
admin-settings-help-security-trust-ua-automation = Ação para solicitações anônimas que se declaram script, biblioteca HTTP, navegador headless ou bot genérico: permitir, limitar ou bloquear.
admin-settings-key-security-trust-ua-search = Bots: rastreadores de busca
admin-settings-help-security-trust-ua-search = Ação para solicitações anônimas que se declaram rastreador de mecanismo de busca: permitir, limitar ou bloquear.
admin-settings-key-security-trust-ua-ai-training = Bots: rastreadores de treinamento de IA
admin-settings-help-security-trust-ua-ai-training = Ação para solicitações anônimas que se declaram GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended ou Applebot-Extended: permitir, limitar ou bloquear. Uma declaração é um cabeçalho, não uma identidade verificada.
admin-settings-key-security-trust-ua-ai-search = Bots: rastreadores de busca de IA
admin-settings-help-security-trust-ua-ai-search = Ação para solicitações anônimas que se declaram OAI-SearchBot, PerplexityBot ou Claude-SearchBot: permitir, limitar ou bloquear.
admin-settings-key-security-trust-ua-ai-user = Bots: buscadores de IA a pedido do usuário
admin-settings-help-security-trust-ua-ai-user = Ação para solicitações anônimas que se declaram ChatGPT-User ou Claude-User: permitir, limitar ou bloquear.
admin-settings-key-security-trust-ua-custom = Bots: padrões personalizados
admin-settings-help-security-trust-ua-custom = Ação para solicitações anônimas que correspondem aos padrões personalizados de user-agent: permitir, limitar ou bloquear.
admin-settings-key-security-trust-admin-geo-enabled = Bloquear a descoberta do admin vinda de países desconhecidos
admin-settings-help-security-trust-admin-geo-enabled = Bloqueia uma origem que sonda o console de administração a partir de um país sem login bem-sucedido recente. Requer um banco de dados de países legível e a proteção contra sondagem do admin. MW_ADMIN_GEO_ENABLED=false a desativa para recuperação.
admin-settings-key-security-trust-admin-geo-countries = Países sempre conhecidos (bloqueio do admin)
admin-settings-help-security-trust-admin-geo-countries = Códigos de país ISO em maiúsculas sempre tratados como conhecidos, além dos países de logins bem-sucedidos recentes.
admin-settings-key-security-trust-admin-geo-lookup-failure = Origem sem país
admin-settings-help-security-trust-admin-geo-lookup-failure = allow não bloqueia e registra o caso; unknown trata uma origem sem país detectado como um país desconhecido.
admin-settings-key-security-trust-admin-geo-threshold = Sondagens antes de um bloqueio
admin-settings-help-security-trust-admin-geo-threshold = Sondagens de descoberta do admin vindas de um país desconhecido, dentro da janela, que iniciam um bloqueio.
admin-settings-key-security-trust-admin-geo-window-secs = Janela de sondagem (segundos)
admin-settings-help-security-trust-admin-geo-window-secs = Janela do limite de sondagens.
admin-settings-key-security-trust-admin-geo-block-on-first = Bloquear na primeira sondagem
admin-settings-help-security-trust-admin-geo-block-on-first = Bloqueia na primeira sondagem vinda de um país desconhecido. Aumenta a chance de bloquear um administrador em viagem.
admin-settings-key-security-trust-admin-geo-block-secs = Primeiro bloqueio (segundos)
admin-settings-help-security-trust-admin-geo-block-secs = Duração do primeiro bloqueio.
admin-settings-key-security-trust-admin-geo-second-block-secs = Segundo bloqueio (segundos)
admin-settings-help-security-trust-admin-geo-second-block-secs = Duração do segundo bloqueio da mesma origem dentro do período de reincidência.
admin-settings-key-security-trust-admin-geo-max-block-secs = Bloqueio de reincidência (segundos)
admin-settings-help-security-trust-admin-geo-max-block-secs = Duração do terceiro bloqueio e de todos os seguintes dentro do período de reincidência.
admin-settings-key-security-trust-admin-geo-repeat-days = Período de reincidência (dias)
admin-settings-help-security-trust-admin-geo-repeat-days = Um bloqueio dentro desta quantidade de dias após o anterior conta como reincidência e usa a próxima duração, mais longa.
admin-settings-key-security-signin-account-geo-mode = Política de país de login por conta
admin-settings-help-security-signin-account-geo-mode = Desativada por padrão. Bloqueia logins de países que a conta ainda não usou ou de qualquer país fora da sua lista permitida. IPs e países da lista de permissão sempre passam. Vale apenas para novos logins no webmail, nunca para sessões existentes nem clientes de protocolo. A detecção de país usa um banco de geolocalização por IP que pode estar errado ou desatualizado; VPNs, operadoras móveis, proxies e viagens podem bloquear um login legítimo ou deixar passar um estrangeiro. Adicione IPs ou países confiáveis à lista de permissão.
admin-settings-key-security-signin-account-geo-allow-countries = Países permitidos da conta
admin-settings-help-security-signin-account-geo-allow-countries = Códigos de país ISO em maiúsculas sempre permitidos para a conta. A detecção de país usa um banco de geolocalização por IP que pode estar errado ou desatualizado; VPNs, operadoras móveis, proxies e viagens podem bloquear um login legítimo ou deixar passar um estrangeiro. Adicione IPs ou países confiáveis à lista de permissão.
admin-settings-key-security-signin-account-geo-allow-ips = IPs permitidos da conta
admin-settings-help-security-signin-account-geo-allow-ips = Endereços IP ou redes CIDR sempre permitidos para a conta. Um único endereço IPv6 equivale ao seu /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Países permitidos para todas as contas
admin-settings-help-security-signin-account-geo-global-allow-countries = Códigos de país ISO em maiúsculas permitidos para todas as contas, somados à lista própria de cada conta.
admin-settings-key-security-signin-account-geo-global-allow-ips = IPs permitidos para todas as contas
admin-settings-help-security-signin-account-geo-global-allow-ips = Endereços IP ou redes CIDR (um escritório ou VPN) permitidos para todas as contas, somados à lista própria de cada conta.
admin-settings-key-security-signin-account-geo-lookup-failure = Origem sem país (política da conta)
admin-settings-help-security-signin-account-geo-lookup-failure = Quando a origem não tem país detectado: permitir o login e registrá-lo, ou negá-lo.
admin-settings-key-security-signin-account-geo-history-days = Histórico de países da conta (dias)
admin-settings-help-security-signin-account-geo-history-days = Quantos dias um país permanece no histórico de logins da própria conta para a política de países incomuns. É mantido separado do registro de autenticação, então reduzir o registro não o reduz.
admin-settings-key-security-geo-no-database = Sem banco de países: comportamento padrão
admin-settings-help-security-geo-no-database = O que os recursos de país fazem quando nenhum banco está carregado: permitir, tratar a origem como país desconhecido ou bloquear (o bloqueio vale apenas para logins). A configuração própria de um recurso também se aplica e vale a mais rígida.
admin-settings-key-security-geo-not-found = Endereço fora do banco: comportamento padrão
admin-settings-help-security-geo-not-found = O que os recursos de país fazem quando um banco está carregado, mas não tem entrada para o endereço. Endereços privados, de loopback e link-local são redes locais e nunca são afetados.
admin-settings-key-security-signin-geo-no-database = Política de login da instância sem banco
admin-settings-help-security-signin-geo-no-database = Quando a política de país de login da instância está ativa e nenhum banco está carregado: bloquear logins (falha fechada) ou permiti-los sem verificação. Redes locais são sempre permitidas.
admin-settings-key-security-geo-session-lookup-failure = Alertas de sessão sem país
admin-settings-help-security-geo-session-lookup-failure = Quando o país de uma solicitação autenticada não pode ser determinado: ignorar, ou emitir um alerta de sessão por sessão por dia. Nunca bloqueia nem revoga uma sessão.
admin-settings-key-security-protocol-framing-mode = Proteção de protocolo: delimitação de solicitações
admin-settings-help-security-protocol-framing-mode = Classe de request smuggling: Content-Length junto com Transfer-Encoding, Transfer-Encoding diferente de exatamente chunked (ou em HTTP/1.0 e HTTP/2), Content-Length não numérico ou conflitante. Aplicar responde 400 e fecha a conexão; detectar apenas conta. Uma isenção por prefixo de rota nunca vale para esta classe.
admin-settings-key-security-protocol-host-mode = Proteção de protocolo: sintaxe do Host
admin-settings-help-security-protocol-host-mode = Sintaxe do cabeçalho Host: ausente em HTTP/1.1, vazio, repetido ou com userinfo, espaços ou caracteres de caminho. Literais IPv6, portas e ponto final são aceitos. Aplicar responde 400 e fecha a conexão.
admin-settings-key-security-protocol-host-pinning-mode = Proteção de protocolo: o Host deve corresponder ao nome público
admin-settings-help-security-protocol-host-pinning-mode = Host que não corresponde ao host de MW_PUBLIC_URL nem a um dos allowedHosts. Não faz nada enquanto nenhum deles nomear um host. Detectar por padrão, porque proxies e verificações de integridade podem enviar um nome interno.
admin-settings-key-security-protocol-path-control-mode = Proteção de protocolo: caracteres de controle no caminho
admin-settings-help-security-protocol-path-control-mode = NUL, caracteres de controle ou CR/LF codificados (%00, %0d, %0a) no caminho da solicitação. Aplicar responde 400. Nunca isento por prefixo de rota.
admin-settings-key-security-protocol-target-mode = Proteção de protocolo: artifícios no alvo da solicitação
admin-settings-help-security-protocol-target-mode = Codificação percentual dupla ou excessiva de pontos e barras, pontos codificados, segmentos de ponto, segmentos de caminho vazios (//), barras invertidas, bytes brutos não ASCII e alvo em forma absoluta que nomeia outra autoridade além do Host.
admin-settings-key-security-protocol-path-length-mode = Proteção de protocolo: tamanho do caminho e da consulta
admin-settings-help-security-protocol-path-length-mode = Caminho ou consulta maior que maxPathBytes ou maxQueryBytes. Aplicar responde 414.
admin-settings-key-security-protocol-method-mode = Proteção de protocolo: métodos HTTP
admin-settings-help-security-protocol-method-mode = Método fora de allowedMethods (por padrão TRACE, TRACK, CONNECT e métodos desconhecidos ou em minúsculas). Aplicar responde 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Proteção de protocolo: cabeçalhos de valor único repetidos
admin-settings-help-security-protocol-duplicate-headers-mode = Um cabeçalho que deveria aparecer uma vez aparecendo mais de uma (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, cabeçalhos condicionais e de intervalo), um Content-Length repetido com valores iguais ou cabeçalhos Cookie repetidos em HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Proteção de protocolo: cabeçalhos de encaminhamento de pares não confiáveis
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* ou X-Real-IP enviados por um par fora de MW_TRUSTED_PROXIES. Os cabeçalhos nunca são considerados confiáveis, de qualquer forma; isto registra a tentativa. Só conta para a proteção adaptativa quando há proxies confiáveis configurados.
admin-settings-key-security-protocol-header-hygiene-mode = Proteção de protocolo: higiene de cabeçalhos
admin-settings-help-security-protocol-header-hygiene-mode = Quantidade de cabeçalhos, um único valor ou o cabeçalho Cookie acima dos limites configurados; bytes de cabeçalho não ASCII; Upgrade ou Connection: upgrade fora da rota WebSocket; Connection nomeando cabeçalhos como Content-Length ou Host. Aplicar responde 431 para achados de tamanho e quantidade e 400 para os demais.
admin-settings-key-security-protocol-body-mode = Proteção de protocolo: delimitação do corpo da solicitação
admin-settings-help-security-protocol-body-mode = GET ou HEAD com corpo, Content-Length acima de maxBodyBytes (413), Expect diferente de 100-continue (417) ou sem corpo.
admin-settings-key-security-protocol-max-path-bytes = Proteção de protocolo: maior caminho (bytes)
admin-settings-help-security-protocol-max-path-bytes = Maior caminho de solicitação antes que a classe de tamanho do caminho o reporte.
admin-settings-key-security-protocol-max-query-bytes = Proteção de protocolo: maior consulta (bytes)
admin-settings-help-security-protocol-max-query-bytes = Maior string de consulta antes que a classe de tamanho do caminho a reporte.
admin-settings-key-security-protocol-max-header-count = Proteção de protocolo: máximo de cabeçalhos
admin-settings-help-security-protocol-max-header-count = Máximo de campos de cabeçalho em uma solicitação antes que a classe de higiene de cabeçalhos o reporte. O limite rígido do próprio listener (MW_HTTP_MAX_HEADERS) continua valendo.
admin-settings-key-security-protocol-max-header-value-bytes = Proteção de protocolo: maior valor de cabeçalho (bytes)
admin-settings-help-security-protocol-max-header-value-bytes = Maior valor de um único cabeçalho antes que a classe de higiene de cabeçalhos o reporte.
admin-settings-key-security-protocol-max-cookie-bytes = Proteção de protocolo: maior cabeçalho Cookie (bytes)
admin-settings-help-security-protocol-max-cookie-bytes = Tamanho combinado dos cabeçalhos Cookie antes que a classe de higiene de cabeçalhos o reporte.
admin-settings-key-security-protocol-max-body-bytes = Proteção de protocolo: maior corpo declarado (bytes)
admin-settings-help-security-protocol-max-body-bytes = Maior Content-Length que a classe de corpo aceita sem reportar. Mantenha acima do maior upload ou importação que uma rota aceita; uma isenção por prefixo de rota pode aumentá-lo para uma rota.
admin-settings-key-security-protocol-allowed-methods = Proteção de protocolo: métodos permitidos
admin-settings-help-security-protocol-allowed-methods = Métodos que a classe de métodos aceita. O padrão lista os verbos HTTP comuns e os verbos WebDAV, CalDAV e CardDAV; o servidor não monta rotas DAV de entrada hoje, então eles resultam em 405 no roteador.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protocol-allowed-hosts = Proteção de protocolo: hosts extras aceitos
admin-settings-help-security-protocol-allowed-hosts = Nomes de host aceitos pela classe de fixação de host além do host de MW_PUBLIC_URL, por exemplo um nome interno usado por um proxy ou verificação de integridade. Portas e ponto final são ignorados.
admin-settings-key-security-protocol-exempt-prefixes = Proteção de protocolo: prefixos de rota isentos
admin-settings-help-security-protocol-exempt-prefixes = Prefixos de rota (cada um começando com / e com mais de um caractere) nos quais as classes heurísticas não são executadas, para um cliente legítimo que precise disso, por exemplo uma rota de upload grande. Corresponde a segmentos inteiros do caminho. Achados de delimitação e de controle de caminho nunca são isentos, e / é ignorado.
admin-settings-key-security-health-mode = Acesso ao endpoint de integridade
admin-settings-help-security-health-mode = Quem pode acessar /healthz e /readyz. internal-only: endereços de cliente de loopback, privados, link-local e unique-local. allowlist: esses mais as redes abaixo. open: qualquer pessoa. Qualquer outro recebe o mesmo 404 de um caminho desconhecido. Uma solicitação com o token MW_HEALTH_TOKEN é sempre admitida. O endereço do cliente é o resolvido pelo proxy confiável: atrás de um proxy reverso sem MW_FORWARDED_MODE e MW_TRUSTED_PROXIES, toda solicitação parece vir do endereço privado do proxy, e assim a internet inteira é admitida.
admin-settings-key-security-health-networks = Redes do endpoint de integridade
admin-settings-help-security-health-networks = Endereços IP ou redes CIDR que podem acessar /healthz e /readyz no modo allowlist, além das faixas internas, por exemplo um monitor de disponibilidade externo.
admin-settings-key-security-spray-enabled = Detecção de password spray
admin-settings-help-security-spray-enabled = Conta os nomes de usuário distintos com que uma origem falha. Apenas um hash com chave de cada nome de usuário é mantido, na memória.
admin-settings-key-security-spray-window-secs = Janela de spray (segundos)
admin-settings-help-security-spray-window-secs = Janela em que os logins com falha de uma origem são contados por nome de usuário distinto. Padrão 900.
admin-settings-key-security-spray-challenge-at = Spray: limite de desafio
admin-settings-help-security-spray-challenge-at = Nomes de usuário distintos com falha de uma origem, dentro da janela, que impõem o desafio de prova de trabalho a essa origem. Logins bem-sucedidos nunca contam, então um endereço compartilhado de escritório ou campus não é afetado. Padrão 10.
admin-settings-key-security-spray-pace-at = Spray: limite de ritmo
admin-settings-help-security-spray-pace-at = Nomes de usuário distintos com falha que acrescentam espera extra entre as tentativas da origem. Padrão 15.
admin-settings-key-security-spray-lock-at = Spray: limite de bloqueio
admin-settings-help-security-spray-lock-at = Nomes de usuário distintos com falha que bloqueiam a origem pela duração do bloqueio, por meio do bloqueio de login existente. Padrão 25.
admin-settings-key-security-spray-pace-secs = Spray: espera de ritmo (segundos)
admin-settings-help-security-spray-pace-secs = Segundos entre tentativas de uma origem no limite de ritmo ou acima dele. Padrão 10.
admin-settings-key-security-spray-lock-secs = Spray: duração do bloqueio (segundos)
admin-settings-help-security-spray-lock-secs = Duração do bloqueio temporário. Um administrador que desbloqueia a origem também zera sua contagem. Padrão 900.
admin-settings-key-security-spray-known-multiplier = Spray: multiplicador de origem conhecida
admin-settings-help-security-spray-known-multiplier = Multiplicador dos três limites para uma origem conhecida, com um login bem-sucedido recente. Padrão 3.
admin-settings-key-security-spray-capacity = Spray: origens monitoradas
admin-settings-help-security-spray-capacity = Máximo de origens monitoradas na memória; as menos recentemente ativas são descartadas primeiro. Padrão 20000.
admin-settings-key-security-spike-enabled = Detecção de picos em toda a instância
admin-settings-help-security-spike-enabled = Aprende uma linha de base de logins com falha e entra em modo reforçado quando a instância inteira está muito acima dela. Nunca bloqueia a instância.
admin-settings-key-security-spike-multiplier-percent = Pico: multiplicador (%)
admin-settings-help-security-spike-multiplier-percent = Um sinal conta quando está pelo menos nesta porcentagem da sua linha de base aprendida (400 = quatro vezes) e acima do seu piso absoluto.
admin-settings-key-security-spike-min-failures-per-min = Pico: mínimo de falhas por minuto
admin-settings-help-security-spike-min-failures-per-min = Piso: logins com falha por minuto abaixo do qual o sinal de taxa de falhas nunca conta. Padrão 20.
admin-settings-key-security-spike-min-sources = Pico: mínimo de origens com falha
admin-settings-help-security-spike-min-sources = Piso: origens distintas com falha em um intervalo de cinco minutos abaixo do qual esse sinal nunca conta. Padrão 10.
admin-settings-key-security-spike-min-usernames = Pico: mínimo de nomes de usuário com falha
admin-settings-help-security-spike-min-usernames = Piso: nomes de usuário distintos com falha em um intervalo de cinco minutos abaixo do qual esse sinal nunca conta. Padrão 30.
admin-settings-key-security-spike-signals-required = Pico: sinais necessários
admin-settings-help-security-spike-signals-required = Sinais que precisam estar juntos acima da linha de base e do piso. A assinatura de spray distribuído conta como dois. Padrão 2.
admin-settings-key-security-spike-warmup-hours = Pico: aquecimento (horas)
admin-settings-help-security-spike-warmup-hours = Horas de tráfego observado antes que o modo reforçado possa começar. Até lá, a linha de base apenas aprende. Padrão 48.
admin-settings-key-security-spike-adapt-hours = Pico: adaptação da linha de base (horas)
admin-settings-help-security-spike-adapt-hours = Constante de tempo da linha de base. Valores maiores absorvem o crescimento mais devagar. Um pico nunca atualiza a linha de base. Padrão 168.
admin-settings-key-security-spike-duration-secs = Pico: duração do modo reforçado (segundos)
admin-settings-help-security-spike-duration-secs = Quanto tempo o modo reforçado dura após a última evidência de um pico; é estendido enquanto os sinais permanecem acima da metade do nível de disparo. Padrão 1800.
admin-settings-key-security-spike-force-challenge = Pico: forçar desafio
admin-settings-help-security-spike-force-challenge = No modo reforçado, exige o desafio de prova de trabalho para logins anônimos de origens desconhecidas. Usuários autenticados e origens conhecidas não são afetados.
admin-settings-key-security-spike-pace-factor-percent = Pico: espera de login (%)
admin-settings-help-security-spike-pace-factor-percent = No modo reforçado, a espera de login para origens desconhecidas como porcentagem da espera habitual. Padrão 300.
admin-settings-key-security-session-binding-enabled = Vinculação de sessão
admin-settings-help-security-session-binding-enabled = Vincula cada sessão do navegador a hashes com chave dos atributos do seu cliente. Clientes nativos com token bearer, chaves de API e clientes de protocolo são isentos.
admin-settings-key-security-session-binding-attributes = Vinculação de sessão: atributos
admin-settings-help-security-session-binding-attributes = Atributos vinculados: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Network vem desativado por padrão porque as redes móveis mudam. Nomes desconhecidos são ignorados.
admin-settings-key-security-session-binding-strictness = Vinculação de sessão: rigor
admin-settings-help-security-session-binding-strictness = Tolerante aceita atualizações do navegador e a mudança de um único atributo leve; rigoroso sinaliza qualquer diferença leve.
admin-settings-key-security-session-binding-webmail-action = Vinculação de sessão: ação no webmail
admin-settings-help-security-session-binding-webmail-action = O que uma divergência na sessão do webmail faz: off, detect (apenas registrar), step-up (pedir de novo a senha e o segundo fator) ou revogar a sessão. Padrão step-up.
admin-settings-key-security-session-binding-admin-action = Vinculação de sessão: ação no console de administração
admin-settings-help-security-session-binding-admin-action = O que uma divergência na sessão do console de administração faz: off, detect, step-up ou revogar. Padrão revogar.
admin-settings-key-security-theft-enabled = Detecção de roubo de sessão
admin-settings-help-security-theft-enabled = Detecta a reutilização de um id de sessão rotacionado, viagem impossível e uso simultâneo de países diferentes.
admin-settings-key-security-theft-rotation-grace-secs = Roubo: tolerância de rotação (segundos)
admin-settings-help-security-theft-rotation-grace-secs = Depois que uma sessão é rotacionada, o id antigo pode ser apresentado de novo por este tempo (uma solicitação em andamento) antes de contar como roubo. Padrão 30.
admin-settings-key-security-theft-min-travel-secs = Roubo: tempo mínimo de viagem (segundos)
admin-settings-help-security-theft-min-travel-secs = Uma mudança de país mais cedo que isto após a solicitação anterior é tratada como viagem impossível. A detecção de país pode ser imprecisa; VPNs e redes móveis podem causar mudanças falsas. Padrão 3600.
admin-settings-key-security-theft-concurrent-use = Roubo: uso simultâneo
admin-settings-help-security-theft-concurrent-use = Sinaliza uma sessão usada alternadamente de países (ou redes) diferentes em cinco minutos: off, country ou network. A detecção de país pode ser imprecisa; VPNs e redes móveis podem causar mudanças falsas.
admin-settings-key-security-theft-webmail-action = Roubo: resposta no webmail
admin-settings-help-security-theft-webmail-action = Resposta a viagem impossível ou uso simultâneo em uma sessão de webmail: alert, step-up ou revogar. A reutilização de uma sessão rotacionada sempre revoga. Padrão step-up.
admin-settings-key-security-theft-admin-action = Roubo: resposta no console de administração
admin-settings-help-security-theft-admin-action = Resposta a viagem impossível ou uso simultâneo em uma sessão do console de administração: alert, step-up ou revogar. Padrão revogar.
admin-settings-key-security-theft-retention-days = Roubo: retenção de registros (dias)
admin-settings-help-security-theft-retention-days = Por quanto tempo os marcadores de sessão rotacionada e os registros de segurança por sessão são mantidos; defina pelo menos igual à maior duração de sessão. Padrão 15.
admin-settings-key-security-alerts-country-change = Alerta de mudança de país
admin-settings-help-security-alerts-country-change = Avisa quando uma sessão autenticada é usada de um país diferente daquele em que foi iniciada. A detecção de país pode ser imprecisa; VPNs e redes móveis podem causar mudanças falsas. Os usuários podem alterar isto, a menos que esteja bloqueado; o usuário que o desativa só é respeitado se o silenciamento for permitido.
admin-settings-key-security-alerts-country-change-enabled = Alerta de mudança de país: chave global
admin-settings-help-security-alerts-country-change-enabled = Desativado, desliga o alerta de mudança de país para todos. A detecção de país pode ser imprecisa; VPNs e redes móveis podem causar mudanças falsas.
admin-settings-key-security-alerts-country-change-cooldown-secs = Alerta de mudança de país: intervalo mínimo (segundos)
admin-settings-help-security-alerts-country-change-cooldown-secs = Tempo mínimo entre alertas de mudança de país de uma mesma sessão. Cada novo país alerta uma vez por sessão. Padrão 300.
admin-settings-key-security-alerts-country-change-user-mute = Alerta de mudança de país: permitir que usuários silenciem
admin-settings-help-security-alerts-country-change-user-mute = Se um usuário pode desativar o próprio alerta de mudança de país. Desativado, o servidor continua alertando mesmo que a configuração do usuário esteja desativada.
admin-settings-key-security-alerts-channels = Canais de alerta de segurança
admin-settings-help-security-alerts-channels = Como os avisos de segurança da sessão chegam ao usuário: no aplicativo, como mensagem na própria caixa de correio ou ambos. Os usuários podem alterar isto, a menos que esteja bloqueado.
admin-settings-key-security-alerts-email-enabled = Alertas de segurança por e-mail
admin-settings-help-security-alerts-email-enabled = Permite que os avisos de segurança da sessão sejam entregues na própria caixa de correio do usuário. Nenhuma mensagem é enviada por SMTP; sem uma sessão de entrega, o aviso fica no aplicativo e no registro de auditoria.
admin-settings-key-security-alerts-sensitivity = Sensibilidade dos alertas de segurança
admin-settings-help-security-alerts-sensitivity = Baixa só avisa sobre achados de localização e nunca eleva a verificação nem revoga; normal segue as ações do administrador; alta dobra o intervalo de viagem e também verifica redes simultâneas. Os usuários podem alterar isto, a menos que esteja bloqueado.
admin-settings-key-security-alerts-notice-retention-days = Avisos de segurança: retenção (dias)
admin-settings-help-security-alerts-notice-retention-days = Por quanto tempo os avisos de segurança no aplicativo são mantidos por conta (no máximo 20 por conta). Padrão 30.
admin-settings-inline-effective = Em vigor:
admin-settings-inline-reset = Descartar alterações
admin-settings-inline-domains = Valores por domínio
admin-attachments-tab-lists = Listas de permissão e bloqueio
admin-attachments-tab-limits = Limites
admin-attachments-tab-delivery = Download e visualização
admin-attachments-tab-warnings = Avisos
admin-attachments-tab-test = Testar um arquivo
admin-attachments-tab-denied = Você não tem acesso a esta aba.
admin-attachments-group-lists = Listas de permissão e bloqueio
admin-attachments-group-limits = Limite de tamanho
admin-attachments-group-delivery = Download e visualização
admin-attachments-group-warnings = Avisos
admin-attachments-add-extension = Extensão, por exemplo pdf ou .tar.gz
admin-attachments-add-mime = Tipo MIME, por exemplo image/png ou image/*
admin-attachments-add = Adicionar
admin-attachments-remove = Remover { $rule }
admin-attachments-filter = Filtrar a lista
admin-attachments-list-label = Regras desta lista
admin-attachments-count = { $count } de { $max } regras
admin-attachments-empty-list = Nenhuma regra nesta lista.
admin-attachments-none-match = Nenhuma regra corresponde ao filtro.
admin-attachments-bulk = Colar várias
admin-attachments-bulk-add = Adicionar todas
admin-attachments-import = Importar um arquivo de texto
admin-attachments-export = Exportar como texto
admin-attachments-err-extension = Não é uma extensão. Use letras, dígitos, hifens e sublinhados, até 32 caracteres; pontos podem separar partes, como em tar.gz.
admin-attachments-err-mime = Não é um tipo MIME. Use tipo/subtipo, ou tipo/* para todos os subtipos.
admin-attachments-err-long = Essa regra é longa demais.
admin-attachments-invalid = Não adicionada, não é uma regra válida: { $entries }
admin-attachments-duplicates = Já está na lista: { $rules }
admin-attachments-overflow = Uma lista comporta no máximo { $max } regras. As demais não foram adicionadas.
admin-attachments-conflict-blockedOverAllowed = { $rule } se sobrepõe a { $other } na outra lista. Regras de bloqueio prevalecem, então ela é bloqueada.
admin-attachments-conflict-blockedOverWarn = { $rule } se sobrepõe a { $other } na outra lista. Um arquivo bloqueado é recusado antes de qualquer aviso, então o aviso nunca aparece.
admin-attachments-conflict-allowedAndWarn = { $rule } se sobrepõe a { $other } na outra lista. O arquivo é permitido e mostra um aviso.
admin-attachments-bytes-unit = Unidade
admin-attachments-unlimited = Sem limite de tamanho
admin-attachments-bytes-equals = { $size } ({ $bytes } bytes)
admin-attachments-list-allowedExtensions = extensões permitidas
admin-attachments-list-allowedMimeTypes = tipos MIME permitidos
admin-attachments-list-blockedExtensions = extensões bloqueadas
admin-attachments-list-blockedMimeTypes = tipos MIME bloqueados

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-list-warnExtensions = extensões com aviso
admin-attachments-list-warnMimeTypes = tipos MIME com aviso
admin-attachments-sim-title = Testar um arquivo
admin-attachments-sim-help = Pergunta ao servidor como a política salva trata um arquivo com este nome, tipo e tamanho. Nada é alterado ou armazenado. Mensagens e anexos nunca são lidos.
admin-attachments-sim-filename = Nome do arquivo
admin-attachments-sim-mime = Tipo MIME
admin-attachments-sim-size = Tamanho
admin-attachments-sim-run = Testar
admin-attachments-sim-invalid = Informe um nome de arquivo e um tamanho igual ou maior que zero.
admin-attachments-sim-allowed = Permitido
admin-attachments-sim-warn = Permitido com aviso
admin-attachments-sim-blocked = Não entregue
admin-attachments-sim-preview-yes = A visualização é permitida.
admin-attachments-sim-preview-no = A visualização não é permitida.
admin-attachments-sim-checked = Tamanho verificado: { $size }.
admin-attachments-sim-matched = Regras correspondentes em: { $lists }.
admin-attachments-sim-matched-none = Nenhuma regra de permissão, bloqueio ou aviso correspondeu.
admin-attachments-reason-downloadsDisabled = Os downloads estão desativados.
admin-attachments-reason-attachmentTooLarge = O arquivo é maior que o limite de tamanho.
admin-attachments-reason-attachmentBlocked = Uma regra de bloqueio corresponde.
admin-attachments-reason-attachmentNotAllowed = Existe uma lista de permissão e nada nela corresponde.
admin-attachments-reason-attachmentContainerRestricted = Mensagens anexadas e arquivos TNEF são recusados enquanto houver qualquer restrição ativa, porque seu conteúdo não é listado.
admin-settings-key-mail-reply-images = Imagens em respostas e encaminhamentos
admin-settings-help-mail-reply-images = Se uma resposta ou encaminhamento inclui as imagens da mensagem original: perguntar (padrão), sempre permitir, permitir apenas para domínios de remetente confiáveis e perguntar nos demais casos, ou sempre negar. Bloqueie esta configuração para impô-la. As imagens só são lidas quando incluídas.
admin-settings-key-mail-reply-image-trusted-domains = Domínios de remetente confiáveis para imagens
admin-settings-help-mail-reply-image-trusted-domains = Até 200 domínios de remetente exatos cujas imagens são incluídas sem perguntar quando a política é domínios confiáveis. Os usuários acrescentam à própria lista a partir do aviso da citação, a menos que esta configuração esteja bloqueada.
admin-settings-key-mail-send-recall-delay-seconds = Janela de desfazer envio (padrão)
admin-settings-help-mail-send-recall-delay-seconds = Quantos segundos uma mensagem enviada espera na Caixa de saída, onde o remetente pode recolhê-la, antes de ser entregue. 0 envia imediatamente. Padrão 10, até 1800 (30 minutos). Os usuários escolhem o próprio valor até o limite abaixo, a menos que você bloqueie isto.
admin-settings-key-mail-send-recall-delay-max-seconds = Janela de desfazer envio (limite)
admin-settings-help-mail-send-recall-delay-max-seconds = A maior janela de desfazer envio que um usuário pode escolher, em segundos. O servidor reduz um pedido maior a este valor. Padrão e máximo 1800.
admin-settings-key-mail-send-allow-recall = Permitir recolher mensagens enviadas
admin-settings-help-mail-send-allow-recall = Desativado, as mensagens saem imediatamente e não podem ser recolhidas da Caixa de saída. Envios agendados não são afetados; ainda podem ser cancelados até a hora marcada.
admin-settings-key-mail-send-max-delayed-seconds = Horizonte de agendamento
admin-settings-help-mail-send-max-delayed-seconds = Com quanta antecedência uma mensagem pode ser agendada, em segundos. Padrão 7776000 (90 dias), máximo 31536000 (1 ano). 0 desativa o envio agendado.
admin-settings-key-mail-send-max-queued = Mensagens na fila por caixa de correio
admin-settings-help-mail-send-max-queued = Quantas mensagens uma caixa de correio pode ter aguardando na Caixa de saída (janela de desfazer ou agendadas) ao mesmo tempo. Padrão 200. Novos envios são recusados com uma mensagem clara e nunca descartados.
admin-attachments-sim-pick = Preencher a partir de um arquivo (opcional)
admin-attachments-mime-toggle = Mostrar tipos MIME comuns
admin-attachments-mime-hint = Escolha um tipo comum ou digite qualquer outro tipo/subtipo.
admin-attachments-mime-invalid = Isto não é um tipo/subtipo como text/plain. Será enviado como digitado.
admin-attachments-mime-group-documents = Documentos
admin-attachments-mime-group-images = Imagens
admin-attachments-mime-group-archives = Arquivos compactados
admin-attachments-mime-group-executables = Executáveis e scripts
admin-attachments-mime-group-media = Áudio e vídeo
admin-attachments-mime-group-email = E-mail e calendário
admin-attachments-mime-group-other = Outros
admin-settings-key-cookies-enabled = Controle de consentimento de cookies
admin-settings-help-cookies-enabled = Mostra o controle de consentimento de cookies aos visitantes. Desativado, o aplicativo se comporta como sem o controle.
admin-settings-key-cookies-mode = Modelo de consentimento
admin-settings-help-cookies-mode = optIn: nada opcional é armazenado até o visitante aceitar. optOut: os padrões das categorias valem até o visitante recusar. noticeOnly: o visitante é informado, sem escolha.
admin-settings-key-cookies-region-mode = Modelo de consentimento em regiões rígidas
admin-settings-help-cookies-region-mode = O modelo usado para visitantes das regiões rígidas. same usa lá também o modelo de consentimento acima.
admin-settings-key-cookies-strict-regions = Regiões rígidas
admin-settings-help-cookies-strict-regions = Códigos de país ISO 3166-1 que usam o modelo de consentimento das regiões rígidas. O padrão são os Estados da UE e do EEE, o Reino Unido e a Suíça.
admin-settings-key-cookies-region-header = Cabeçalho de país
admin-settings-help-cookies-region-header = Nome do cabeçalho de solicitação, definido por um proxy ou CDN, que contém o código de país do visitante.
admin-settings-key-cookies-unknown-region = País desconhecido
admin-settings-help-cookies-unknown-region = Qual modelo se aplica quando nenhum país pode ser determinado: o das regiões rígidas ou o normal.
admin-settings-key-cookies-functional-enabled = Oferecer cookies funcionais
admin-settings-help-cookies-functional-enabled = Oferece a categoria de preferências e funcionalidade (tema, idioma, memória de layout) no controle de consentimento.
admin-settings-key-cookies-functional-default = Padrão de funcionais
admin-settings-help-cookies-functional-default = Estado inicial da categoria funcional quando o modelo de consentimento permite padrões.
admin-settings-key-cookies-analytics-enabled = Oferecer cookies de análise
admin-settings-help-cookies-analytics-enabled = Oferece a categoria de análise no controle de consentimento.
admin-settings-key-cookies-analytics-default = Padrão de análise
admin-settings-help-cookies-analytics-default = Estado inicial da categoria de análise quando o modelo de consentimento permite padrões.
admin-settings-key-cookies-marketing-enabled = Oferecer cookies de marketing
admin-settings-help-cookies-marketing-enabled = Oferece a categoria de marketing no controle de consentimento.
admin-settings-key-cookies-marketing-default = Padrão de marketing
admin-settings-help-cookies-marketing-default = Estado inicial da categoria de marketing quando o modelo de consentimento permite padrões.
admin-settings-key-cookies-layout = Layout
admin-settings-help-cookies-layout = banner é uma barra na largura da página, card é um cartão flutuante, modal é uma caixa de diálogo que bloqueia a página até o visitante escolher.
admin-settings-key-cookies-position = Posição
admin-settings-help-cookies-position = Onde o controle de consentimento aparece na página.
admin-settings-key-cookies-reject-prominent = Recusar tão destacado quanto aceitar
admin-settings-help-cookies-reject-prominent = Mostra Recusar tudo com o mesmo tamanho e estilo de Aceitar tudo.
admin-settings-key-cookies-policy-url = URL da política de cookies
admin-settings-help-cookies-policy-url = Link para a sua política de cookies, exibido no controle de consentimento.
admin-settings-key-cookies-privacy-url = URL da política de privacidade
admin-settings-help-cookies-privacy-url = Link para a sua política de privacidade, exibido no controle de consentimento.
admin-settings-key-cookies-policy-version = Versão da política
admin-settings-help-cookies-policy-version = Aumente este número quando o seu uso de cookies mudar, para que os visitantes sejam consultados de novo.
admin-settings-key-cookies-lifetime-days = Validade do consentimento
admin-settings-help-cookies-lifetime-days = Dias em que uma decisão permanece válida antes que o visitante seja consultado de novo.
admin-settings-key-cookies-reprompt-on-version = Perguntar de novo em uma nova versão da política
admin-settings-help-cookies-reprompt-on-version = Pergunta de novo aos visitantes quando a versão da política é maior que a da decisão deles.
admin-settings-key-cookies-honour-gpc = Respeitar o Global Privacy Control
admin-settings-help-cookies-honour-gpc = Trata um sinal Global Privacy Control do navegador como recusa de análise e marketing.
admin-settings-key-cookies-honour-dnt = Respeitar o Do Not Track
admin-settings-help-cookies-honour-dnt = Trata um sinal Do Not Track do navegador como recusa de análise e marketing.
admin-settings-key-cookies-log-consent = Manter um registro de consentimento
admin-settings-help-cookies-log-consent = Mantém no servidor um registro de cada decisão de consentimento, para estatísticas e para os seus próprios registros.
admin-settings-key-cookies-log-retention-days = Retenção do registro de consentimento
admin-settings-help-cookies-log-retention-days = Dias em que as linhas do registro de consentimento são mantidas antes de serem excluídas.
admin-settings-key-cookies-scan-interval-hours = Intervalo de varredura do armazenamento
admin-settings-help-cookies-scan-interval-hours = Horas entre varreduras automáticas dos cookies e do armazenamento do navegador. 0 varre apenas sob demanda.
admin-settings-key-cookies-unclassified = Armazenamento não classificado
admin-settings-help-cookies-unclassified = report deixa de lado o armazenamento que nada classifica e o lista para revisão. Os outros valores o tratam como essa categoria até você classificá-lo.
admin-settings-key-cookies-text-title = Título do controle de consentimento
admin-settings-help-cookies-text-title = Título do controle de consentimento por idioma. Vazio usa o texto integrado.
admin-settings-key-cookies-text-body = Texto do controle de consentimento
admin-settings-help-cookies-text-body = Texto explicativo do controle de consentimento por idioma. Vazio usa o texto integrado.
admin-settings-key-cookies-label-necessary = Nome da categoria necessária
admin-settings-help-cookies-label-necessary = Nome da categoria necessária por idioma. Vazio usa o nome integrado.
admin-settings-key-cookies-description-necessary = Descrição da categoria necessária
admin-settings-help-cookies-description-necessary = Descrição da categoria necessária por idioma. Vazio usa a descrição integrada.
admin-settings-key-cookies-label-functional = Nome da categoria funcional
admin-settings-help-cookies-label-functional = Nome da categoria funcional por idioma. Vazio usa o nome integrado.
admin-settings-key-cookies-description-functional = Descrição da categoria funcional
admin-settings-help-cookies-description-functional = Descrição da categoria funcional por idioma. Vazio usa a descrição integrada.
admin-settings-key-cookies-label-analytics = Nome da categoria de análise
admin-settings-help-cookies-label-analytics = Nome da categoria de análise por idioma. Vazio usa o nome integrado.
admin-settings-key-cookies-description-analytics = Descrição da categoria de análise
admin-settings-help-cookies-description-analytics = Descrição da categoria de análise por idioma. Vazio usa a descrição integrada.
admin-settings-key-cookies-label-marketing = Nome da categoria de marketing
admin-settings-help-cookies-label-marketing = Nome da categoria de marketing por idioma. Vazio usa o nome integrado.
admin-settings-key-cookies-description-marketing = Descrição da categoria de marketing
admin-settings-help-cookies-description-marketing = Descrição da categoria de marketing por idioma. Vazio usa a descrição integrada.
admin-settings-key-encryption-rotation-days = Intervalo de rotação da chave do banco de dados (dias)
admin-settings-help-encryption-rotation-days = Dias após a última troca de chave em que a chave do banco de dados é indicada como vencida para rotação. 0 não define prazo. A chave em si nunca é armazenada nas configurações.
admin-settings-key-encryption-warnings-enabled = Relatar achados de criptografia
admin-settings-help-encryption-warnings-enabled = Mostra repositórios em texto simples e uma rotação de chave atrasada como achado na avaliação da implantação.
admin-settings-key-retention-sweeper-enabled = Limpeza de retenção
admin-settings-help-retention-sweeper-enabled = Executa a limpeza diária que exclui linhas mais antigas que os limites desta página. Desativada, mantém todas as linhas até ser ativada de novo.
admin-settings-key-retention-sweeper-batch-rows = Linhas por lote
admin-settings-help-retention-sweeper-batch-rows = Linhas que uma instrução de exclusão remove. Lotes menores seguram os bloqueios do banco de dados por menos tempo.
admin-settings-key-retention-sweeper-max-batches = Lotes por repositório por execução
admin-settings-help-retention-sweeper-max-batches = Máximo de instruções de exclusão que um repositório recebe em uma execução. Um repositório com mais linhas antigas continua na próxima execução.
admin-settings-key-retention-sweeper-jitter-minutes = Dispersão do horário de início (minutos)
admin-settings-help-retention-sweeper-jitter-minutes = A próxima execução começa um número aleatório de minutos, até este valor, após 24 horas, para que vários servidores não façam a limpeza juntos.
admin-settings-key-retention-sweeper-vacuum-after-rows = Recuperar espaço após (linhas)
admin-settings-help-retention-sweeper-vacuum-after-rows = Quando uma execução exclui pelo menos este número de linhas, o banco de dados é compactado (VACUUM). 0 nunca compacta após uma limpeza; a tarefa semanal de vacuum é separada.
admin-settings-key-retention-password-change-audit-days = Registros de troca de senha (dias)
admin-settings-help-retention-password-change-audit-days = Dias em que um registro de troca de senha (conta, método, resultado) é mantido. 0 os mantém para sempre.
admin-settings-key-retention-sso-login-audit-days = Registros de login com logon único (dias)
admin-settings-help-retention-sso-login-audit-days = Dias em que um registro de login com logon único (provedor, assunto com hash, resultado) é mantido. 0 os mantém para sempre.
admin-settings-key-retention-dlp-audit-days = Registros de prevenção contra perda de dados (dias)
admin-settings-help-retention-dlp-audit-days = Dias em que um registro de correspondência de regra de prevenção contra perda de dados é mantido. 0 os mantém para sempre.
admin-settings-key-retention-assist-audit-days = Registros de acesso do assistente (dias)

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-retention-assist-audit-days = Dias em que um registro de uso de uma capacidade por um assistente (ator, capacidade, resumo do escopo) é mantido. 0 os mantém para sempre.
admin-settings-key-retention-alert-events-days = Registros de alertas disparados (dias)
admin-settings-help-retention-alert-events-days = Dias em que um registro de alerta de administrador disparado é mantido. 0 os mantém para sempre.
admin-settings-key-retention-job-runs-days = Histórico de tarefas em segundo plano (dias)
admin-settings-help-retention-job-runs-days = Dias em que uma execução concluída de tarefa em segundo plano é mantida. Uma execução ainda em andamento nunca é removida. 0 mantém o histórico para sempre.
admin-settings-key-retention-outbox-days = Registros de envios concluídos e cancelados (dias)
admin-settings-help-retention-outbox-days = Dias em que o registro de envio de uma mensagem entregue ou cancelada é mantido. Uma mensagem em espera, em nova tentativa, com falha ou incerta nunca é removida. 0 os mantém para sempre.
admin-settings-key-retention-expired-credentials-grace-days = Credenciais expiradas (dias após a expiração)
admin-settings-help-retention-expired-credentials-grace-days = Dias em que um dispositivo lembrado, desafio de login, token de login, token OAuth, nonce de webhook, contador de falhas de origem ou identificador de link de imagem expirado é mantido após expirar. 0 o remove na próxima execução.
admin-settings-key-retention-ip-addresses-days = Reduzir endereços IP armazenados após (dias)
admin-settings-help-retention-ip-addresses-days = Após este número de dias, o endereço IP no histórico de autenticação e nas entradas de auditoria é truncado ou transformado em hash, e a linha é mantida. 0 mantém os endereços como registrados.
admin-settings-key-retention-ip-addresses-mode = Método de redução de IP
admin-settings-help-retention-ip-addresses-mode = Truncar mantém o prefixo IPv4 /24 ou IPv6 /48. Hash substitui o endereço por um hash curto com chave que ainda agrupa o mesmo endereço.
admin-settings-key-retention-user-enabled = Usuários podem definir regras de retenção
admin-settings-help-retention-user-enabled = Os usuários podem definir as próprias regras de retenção para pastas e e-mails. Desativado, as regras de usuário existentes deixam de ser executadas; as regras de toda a organização abaixo continuam.
admin-settings-key-retention-user-min-days = Mínimo de dias antes de uma regra de usuário excluir e-mails
admin-settings-help-retention-user-min-days = Uma regra de usuário que exclui e-mails não pode agir sobre e-mails mais recentes que isto. Um valor menor é recusado ao salvar a regra e elevado ao executá-la.
admin-settings-key-retention-user-max-days = Máximo de dias que uma regra de usuário pode esperar para excluir e-mails
admin-settings-help-retention-user-max-days = Uma regra de usuário que exclui e-mails não pode esperar mais que isto. 0 não define teto. Um valor maior é recusado ao salvar a regra e reduzido ao executá-la.
admin-settings-key-retention-user-max-rules = Máximo de regras de retenção por usuário
admin-settings-help-retention-user-max-rules = O máximo de regras de retenção que um usuário pode manter.
admin-settings-key-retention-user-allow-permanent-delete = Regras de usuário podem excluir e-mails permanentemente
admin-settings-help-retention-user-allow-permanent-delete = Desativado, uma regra de exclusão sempre move os e-mails para a Lixeira e a janela de desfazer vale antes da eliminação definitiva.
admin-settings-key-retention-user-undo-days = Janela de desfazer das regras de retenção (dias)
admin-settings-help-retention-user-undo-days = Dias em que os e-mails movidos para a Lixeira por uma regra de retenção ficam lá antes de serem eliminados. Um usuário pode escolher uma janela menor. 0 elimina na próxima execução.
admin-settings-key-retention-user-mandatory = Regras de e-mail de toda a organização
admin-settings-help-retention-user-mandatory = Regras que toda conta recebe, como entradas função-da-pasta:dias, por exemplo trash:30 ou junk:30. E-mails dessa pasta mais antigos que os dias são eliminados. Um usuário não pode alterar, pausar nem excluir e-mails dessas regras. E-mails de uma conta sob retenção legal são ignorados.
admin-settings-key-retention-user-protected-folders = Pastas que as regras de retenção nunca tocam
admin-settings-help-retention-user-protected-folders = Pastas em que nenhuma regra de usuário atua, por função (drafts, sent, inbox, archive) ou por nome da pasta. Uma pasta protegida prevalece sobre uma regra de toda a organização para a mesma pasta.
admin-settings-key-retention-user-templates = Modelos de retenção oferecidos aos usuários
admin-settings-help-retention-user-templates = Modelos que os usuários podem adotar em Configurações, por id: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Executar regras de usuário para contas restritas
admin-settings-help-retention-user-run-for-restricted = Se as regras de retenção de usuário também são executadas para contas sob restrição de tratamento. Desativado, essas contas são ignoradas.
admin-settings-key-retention-user-interval-hours = Horas entre as execuções de retenção de usuário
admin-settings-help-retention-user-interval-hours = Horas entre as execuções agendadas da tarefa de retenção de usuário para uma conta.
admin-settings-key-retention-user-batch-messages = Mensagens por chamada de retenção
admin-settings-help-retention-user-batch-messages = Mensagens que uma chamada de mover ou excluir processa. No máximo 200 são usadas.
admin-settings-key-retention-user-max-per-run = Máximo de mensagens alteradas por conta por execução
admin-settings-help-retention-user-max-per-run = Máximo de mensagens que a tarefa de retenção de usuário altera para uma conta em uma execução. O restante espera a próxima execução.
admin-settings-key-retention-user-pause-ms = Pausa entre lotes de retenção (ms)
admin-settings-help-retention-user-pause-ms = Pausa entre lotes, em milissegundos, para que uma limpeza grande não sobrecarregue o servidor de e-mail.
admin-settings-key-retention-class-short-days = Teto da classe curta (dias)
admin-settings-help-retention-class-short-days = Teto para dados operacionais: sessões, tokens, limites de taxa, histórico de tarefas e métricas. Vale quando o limite de classe está ativado.
admin-settings-key-retention-class-medium-days = Teto da classe média (dias)
admin-settings-help-retention-class-medium-days = Teto para histórico de login, histórico de proteção de solicitações e avisos de segurança. Não pode ficar abaixo da classe curta; um valor menor é elevado a ela.
admin-settings-key-retention-class-long-days = Retenção da classe longa (dias)
admin-settings-help-retention-class-long-days = Dias em que a classe longa (auditoria de segurança, ações de administradores, solicitações, consentimento, tombstones) é mantida, incluindo o arquivo. 0 a mantém para sempre e o console avisa. Não pode ficar abaixo da classe média.
admin-settings-key-retention-class-enforce = Limitar cada repositório ao teto da sua classe
admin-settings-help-retention-class-enforce = Ativado, o limite de cada repositório é restringido ao teto da sua classe e 0 (manter) conta como o teto. Desativado, valem os limites por repositório.
admin-settings-key-retention-archive-enabled = Arquivar linhas antigas da classe longa
admin-settings-help-retention-archive-enabled = Move as linhas antigas da classe longa para arquivos lacrados e compactados em vez de excluí-las.
admin-settings-key-retention-archive-directory = Diretório de arquivamento
admin-settings-help-retention-archive-directory = Diretório dos arquivos de arquivamento, absoluto ou relativo ao diretório de trabalho do servidor. Segmentos de diretório pai (..) são recusados.
admin-settings-key-retention-archive-batch-rows = Linhas por arquivo de arquivamento
admin-settings-help-retention-archive-batch-rows = Linhas gravadas em um arquivo de arquivamento.
admin-settings-key-retention-archive-max-segments = Arquivos de arquivamento por repositório por execução
admin-settings-help-retention-archive-max-segments = Arquivos de arquivamento que uma execução pode gravar para cada repositório. O restante espera a próxima execução.
admin-settings-key-retention-archive-key-generation = Geração da chave de lacre do arquivamento
admin-settings-help-retention-archive-key-generation = Geração da chave que lacra novos arquivos de arquivamento. Aumente-a para rotacionar a chave; os arquivos anteriores ainda são verificados com a própria geração.
admin-settings-key-retention-archive-verify-days = Dias entre verificações do arquivamento
admin-settings-help-retention-archive-verify-days = Dias entre as verificações de integridade dos arquivos de arquivamento.
admin-settings-key-retention-archive-legal-hold = Retenção legal do arquivamento
admin-settings-help-retention-archive-legal-hold = Mantém todos os arquivos de arquivamento, qualquer que seja o limite da classe longa.
admin-settings-key-logging-file-enabled = Gravar arquivos de log
admin-settings-help-logging-file-enabled = Grava também o log do servidor em arquivos rotativos. Desativado, o log fica só na saída padrão, que é o que um contêiner normalmente usa.
admin-settings-key-logging-file-directory = Diretório de logs
admin-settings-help-logging-file-directory = Diretório dos arquivos de log, absoluto ou relativo ao diretório de trabalho do servidor. Segmentos de diretório pai (..) são recusados.
admin-settings-key-logging-file-max-size-mb = Rotacionar por tamanho (MB)
admin-settings-help-logging-file-max-size-mb = O arquivo de log atual é fechado e um novo é iniciado quando atinge este tamanho.
admin-settings-key-logging-file-rotate-hours = Rotacionar por idade (horas)
admin-settings-help-logging-file-rotate-hours = O arquivo de log atual é rotacionado quando atinge esta idade. 0 rotaciona apenas por tamanho.
admin-settings-key-logging-file-max-age-days = Manter arquivos rotacionados (dias)
admin-settings-help-logging-file-max-age-days = Arquivos de log rotacionados mais antigos que isto são excluídos.
admin-settings-key-logging-file-max-files = Máximo de arquivos rotacionados
admin-settings-help-logging-file-max-files = Os arquivos rotacionados mais antigos além desta quantidade são excluídos.
admin-settings-key-logging-file-max-total-mb = Tamanho total máximo (MB)
admin-settings-help-logging-file-max-total-mb = Os arquivos rotacionados mais antigos são excluídos até que o diretório de logs fique abaixo deste tamanho.
admin-settings-key-logging-file-compress = Compactar arquivos rotacionados
admin-settings-help-logging-file-compress = Os arquivos de log rotacionados são compactados com gzip.
admin-settings-key-logging-level = Nível de log
admin-settings-help-logging-level = O nível dos componentes que não têm nível próprio abaixo. A variável de ambiente MW_LOG define o nível na inicialização até que esta configuração seja lida.
admin-settings-key-logging-component-levels = Níveis de log por componente
admin-settings-help-logging-component-levels = Uma entrada componente=nível por linha, por exemplo mw_imap=debug. Os níveis são error, warn, info, debug e trace.
admin-settings-key-logging-redaction-addresses = Endereços de e-mail nos logs
admin-settings-help-logging-redaction-addresses = Completo deixa os endereços como registrados. Truncado mantém a primeira letra e o domínio. Hash substitui a parte local por um hash curto. Nenhum remove o endereço.
admin-settings-key-logging-redaction-ips = Endereços IP nos logs
admin-settings-help-logging-redaction-ips = Completo deixa os endereços como registrados. Truncado mantém o prefixo IPv4 /24 ou IPv6 /48. Hash substitui o endereço por um hash curto. Nenhum o remove.
admin-settings-key-logging-redaction-tokens = Segredos nos logs
admin-settings-help-logging-redaction-tokens = Remove das linhas de log tokens bearer, chaves de API, senhas em URLs e segredos semelhantes.
admin-settings-key-retention-sessions-idle-days = Sessões sem uso (dias)
admin-settings-help-retention-sessions-idle-days = Dias em que um registro de sessão de navegador ou cliente nativo, com seu endereço e user agent, pode ficar sem uso antes de ser excluído. Os tempos limite de sessão valem primeiro; isto apenas remove linhas que sobraram. 0 as mantém.
admin-settings-key-policy-public-pages = Páginas públicas de política
admin-settings-help-policy-public-pages = Serve a política de privacidade gerada em /privacy e a política de cookies em /cookies-policy sem login.
admin-settings-key-policy-auto-version = Versões automáticas da política
admin-settings-help-policy-auto-version = Registra uma nova versão da política quando uma configuração descrita pela política muda. Desativado: as mudanças esperam a publicação.
admin-settings-key-policy-reconsent-on-change = Pedir consentimento de cookies de novo em uma nova versão da política
admin-settings-help-policy-reconsent-on-change = Aumenta a versão da política de cookies quando uma nova versão da política é registrada, para que os visitantes sejam consultados de novo (requer cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Nome do controlador
admin-settings-help-policy-controller-name = Nome da organização que decide por que e como os dados são tratados, exibido na política de privacidade.
admin-settings-key-policy-controller-address = Endereço do controlador
admin-settings-help-policy-controller-address = Endereço postal do controlador, exibido na política de privacidade.
admin-settings-key-policy-legal-bases = Bases legais
admin-settings-help-policy-legal-bases = As bases legais em que você se apoia, com suas próprias palavras, exibidas na política de privacidade.
admin-settings-key-policy-hosting-provider = Provedor de hospedagem
admin-settings-help-policy-hosting-provider = Quem hospeda os dados e onde, exibido na política de privacidade.
admin-settings-key-policy-hosting-transfers = Transferências para fora do país de hospedagem
admin-settings-help-policy-hosting-transfers = Quaisquer transferências de dados para outros países e as salvaguardas usadas, exibidas na política de privacidade.
admin-settings-key-policy-supervisory-authority = Autoridade de controle
admin-settings-help-policy-supervisory-authority = A autoridade à qual os usuários podem apresentar reclamação, exibida na política de privacidade.
admin-settings-key-dpia-suggest = Sugerir avaliações de impacto
admin-settings-help-dpia-suggest = Lista uma avaliação de impacto sugerida quando o assistente, o rastreamento de abertura ou a análise está ativado e nenhuma existe.
admin-settings-key-dpia-review-months = Intervalo de revisão da avaliação de impacto
admin-settings-help-dpia-review-months = Meses até a data de revisão definida quando uma avaliação é aprovada.
admin-settings-key-dpia-reminder-days = Janela de lembrete da avaliação de impacto
admin-settings-help-dpia-reminder-days = Dias antes da data de revisão em que uma avaliação é listada como a vencer.
admin-settings-key-restriction-enabled = Restrições de tratamento
admin-settings-help-restriction-enabled = Se um administrador ou uma solicitação aprovada pode restringir o tratamento dos dados de uma pessoa.
admin-settings-key-restriction-auto-apply-on-approval = Restringir quando uma solicitação é aprovada
admin-settings-help-restriction-auto-apply-on-approval = Aplica a restrição assim que uma solicitação de restrição é aprovada. Desativado: um administrador a aplica em Conformidade > Restrições.
admin-settings-key-restriction-mail-delivery = Novos e-mails durante a restrição

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-restriction-mail-delivery = O que o administrador escolheu para e-mails que chegam enquanto uma restrição se aplica: mantê-los, enfileirá-los ou rejeitá-los. Este servidor não recebe e-mails; o valor é registrado e exibido, e o servidor de e-mail age com base nele.
admin-settings-key-restriction-default-expiry-days = Duração padrão da restrição (dias)
admin-settings-help-restriction-default-expiry-days = Dias até que uma nova restrição termine sozinha. 0 a mantém até alguém suspendê-la.
admin-settings-key-restriction-notify-user = Avisar a pessoa
admin-settings-help-restriction-notify-user = Envia um aviso à pessoa quando uma restrição é aplicada ou suspensa.
admin-settings-key-restriction-scope-indexing = Pausar indexação de busca e embeddings
admin-settings-help-restriction-scope-indexing = Se a indexação de busca e os embeddings ficam pausados por padrão em uma nova restrição.
admin-settings-key-restriction-scope-assistant = Pausar o assistente
admin-settings-help-restriction-scope-assistant = Se o assistente fica pausado por padrão em uma nova restrição.
admin-settings-key-restriction-scope-analytics = Pausar análise e rastreamento de abertura
admin-settings-help-restriction-scope-analytics = Se a análise e o rastreamento de abertura ficam pausados por padrão em uma nova restrição.
admin-settings-key-restriction-scope-classification = Pausar regras e arquivamento automático
admin-settings-help-restriction-scope-classification = Se as regras e o arquivamento automático ficam pausados por padrão em uma nova restrição.
admin-settings-key-restriction-scope-push = Pausar notificações push
admin-settings-help-restriction-scope-push = Se as notificações push ficam pausadas por padrão em uma nova restrição.
admin-settings-key-restriction-scope-third-party = Pausar serviços conectados e plugins
admin-settings-help-restriction-scope-third-party = Se os serviços conectados e plugins ficam pausados por padrão em uma nova restrição.
admin-settings-key-consent-enabled = Consentimento por finalidade
admin-settings-help-consent-enabled = Mostra aos usuários as finalidades e as opções de consentimento e as aplica em tempo de execução.
admin-settings-key-consent-receipts-retention-days = Recibos de consentimento mantidos (dias)
admin-settings-help-consent-receipts-retention-days = Dias em que os recibos de consentimento são mantidos. 0 os mantém até a pessoa ser apagada.
admin-settings-key-consent-art22-notice = Aviso de tratamento automatizado
admin-settings-help-consent-art22-notice = Mostra aos usuários o aviso em linguagem simples sobre os recursos do assistente e da busca, com as chaves por recurso.
admin-settings-key-consent-analytics-basis = Base legal: contagens de uso
admin-settings-help-consent-analytics-basis = A base legal exibida aos usuários para as contagens de uso. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-analytics-version = Versão do texto: contagens de uso
admin-settings-help-consent-analytics-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-analytics-text = Texto próprio: contagens de uso
admin-settings-help-consent-analytics-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-analytics-tag-basis = Base legal: script do provedor de análise
admin-settings-help-consent-analytics-tag-basis = A base legal exibida aos usuários para o script do provedor de análise. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-analytics-tag-version = Versão do texto: script do provedor de análise
admin-settings-help-consent-analytics-tag-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-analytics-tag-text = Texto próprio: script do provedor de análise
admin-settings-help-consent-analytics-tag-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-assistant-basis = Base legal: assistente
admin-settings-help-consent-assistant-basis = A base legal exibida aos usuários para o assistente. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-assistant-version = Versão do texto: assistente
admin-settings-help-consent-assistant-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-assistant-text = Texto próprio: assistente
admin-settings-help-consent-assistant-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-push-basis = Base legal: notificações push
admin-settings-help-consent-push-basis = A base legal exibida aos usuários para as notificações push. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-push-version = Versão do texto: notificações push
admin-settings-help-consent-push-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-push-text = Texto próprio: notificações push
admin-settings-help-consent-push-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-open-tracking-basis = Base legal: rastreamento de abertura
admin-settings-help-consent-open-tracking-basis = A base legal exibida aos usuários para o rastreamento de abertura. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-open-tracking-version = Versão do texto: rastreamento de abertura
admin-settings-help-consent-open-tracking-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-open-tracking-text = Texto próprio: rastreamento de abertura
admin-settings-help-consent-open-tracking-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-remote-content-basis = Base legal: imagens remotas
admin-settings-help-consent-remote-content-basis = A base legal exibida aos usuários para as imagens remotas. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-remote-content-version = Versão do texto: imagens remotas
admin-settings-help-consent-remote-content-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-remote-content-text = Texto próprio: imagens remotas
admin-settings-help-consent-remote-content-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-integrations-basis = Base legal: serviços conectados
admin-settings-help-consent-integrations-basis = A base legal exibida aos usuários para os serviços conectados. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-integrations-version = Versão do texto: serviços conectados
admin-settings-help-consent-integrations-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-integrations-text = Texto próprio: serviços conectados
admin-settings-help-consent-integrations-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-consent-telemetry-basis = Base legal: medições de tempo de carregamento
admin-settings-help-consent-telemetry-basis = A base legal exibida aos usuários para as medições de tempo de carregamento. Somente o consentimento faz o recurso esperar um sim; as outras bases valem a menos que a pessoa retire o consentimento. O administrador decide qual base é a adequada.
admin-settings-key-consent-telemetry-version = Versão do texto: medições de tempo de carregamento
admin-settings-help-consent-telemetry-version = Aumente este número quando o texto ou o tratamento mudar; quem concordou com uma versão anterior é consultado de novo.
admin-settings-key-consent-telemetry-text = Texto próprio: medições de tempo de carregamento
admin-settings-help-consent-telemetry-text = O seu próprio texto para esta finalidade, por idioma. Vazio usa o texto integrado.
admin-settings-key-breach-enabled = Registro de violações
admin-settings-help-breach-enabled = Liga o registro e o relógio de notificação.
admin-settings-key-breach-notice-hours = Horas para notificar a autoridade
admin-settings-help-breach-notice-hours = Horas desde a tomada de conhecimento até a notificação à autoridade. O prazo habitual é 72.
admin-settings-key-breach-reminders-hours = Horários de lembrete
admin-settings-help-breach-reminders-hours = Horas antes do prazo que geram um lembrete, por exemplo 6, 24 e 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Repetição após o prazo
admin-settings-help-breach-reminders-overdue-repeat-hours = Horas entre lembretes depois que o prazo passou. 0 lembra uma vez.
admin-settings-key-breach-authority-name = Autoridade de controle
admin-settings-help-breach-authority-name = Nome impresso na notificação à autoridade.
admin-settings-key-breach-authority-url = Página de notificação da autoridade
admin-settings-help-breach-authority-url = Endereço da página de notificação da autoridade.
admin-settings-key-breach-authority-contact = Contato da autoridade
admin-settings-help-breach-authority-contact = Dados de contato da autoridade.
admin-settings-key-breach-dpo-name = Nome do contato de proteção de dados
admin-settings-help-breach-dpo-name = Nome informado como ponto de contato nas notificações.
admin-settings-key-breach-dpo-email = E-mail do contato de proteção de dados
admin-settings-help-breach-dpo-email = Endereço informado como ponto de contato nas notificações.
admin-settings-key-breach-dpo-phone = Telefone do contato de proteção de dados
admin-settings-help-breach-dpo-phone = Número de telefone informado nas notificações.
admin-settings-key-breach-template-subject = Assunto da mensagem
admin-settings-help-breach-template-subject = Assunto da mensagem às pessoas afetadas, por idioma.
admin-settings-key-breach-template-body = Texto da mensagem
admin-settings-help-breach-template-body = Texto da mensagem às pessoas afetadas, por idioma.
admin-settings-key-breach-notify-audit = Registrar avisos no log de auditoria
admin-settings-help-breach-notify-audit = Lembretes e detecções viram eventos de segurança.
admin-settings-key-breach-notify-webhook-urls = Endereços de webhook
admin-settings-help-breach-notify-webhook-urls = Endereços que recebem lembretes e mensagens. Trate-os como segredos; cada chamada é assinada.
admin-settings-key-breach-notify-operator-addresses = Caixas de correio dos operadores
admin-settings-help-breach-notify-operator-addresses = Endereços de caixas de correio locais que recebem os lembretes na própria caixa de entrada.
admin-settings-key-breach-subjects-in-app = Aviso na conta da pessoa
admin-settings-help-breach-subjects-in-app = Mostra a mensagem dentro da conta da pessoa.
admin-settings-key-breach-subjects-inbox = Mensagem na caixa de correio da pessoa
admin-settings-help-breach-subjects-inbox = Entrega a mensagem na própria caixa de correio da pessoa.
admin-settings-key-breach-subjects-webhook = Enviar também para webhooks
admin-settings-help-breach-subjects-webhook = Envia o texto da mensagem também para os endereços de webhook.
admin-settings-key-breach-retention-years = Retenção do registro em anos
admin-settings-help-breach-retention-years = Anos em que um incidente encerrado permanece no registro.
admin-settings-key-breach-auto-enabled = Automações
admin-settings-help-breach-auto-enabled = Abre rascunhos de incidentes a partir de detecções.
admin-settings-key-breach-auto-start-clock = Relógio do rascunho
admin-settings-help-breach-auto-start-clock = Inicia o relógio quando o rascunho é aberto, ou somente quando é confirmado.
admin-settings-key-breach-auto-assignee = Atribuir rascunhos a
admin-settings-help-breach-auto-assignee = A quem um rascunho é atribuído.
admin-settings-key-breach-auto-escalate-after-hours = Escalar após horas
admin-settings-help-breach-auto-escalate-after-hours = Horas que um rascunho espera antes de ser escalado. 0 nunca escala.
admin-settings-key-breach-auto-escalate-to = Escalar para
admin-settings-help-breach-auto-escalate-to = A quem um rascunho escalado é atribuído.
admin-settings-key-breach-auto-window-minutes = Janela de contagem em minutos
admin-settings-help-breach-auto-window-minutes = Minutos durante os quais uma regra conta eventos.
admin-settings-key-breach-auto-theft-threshold = Alertas de roubo de sessão
admin-settings-help-breach-auto-theft-threshold = Alertas na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-credential-stuffing-threshold = Spray ou pico de logins
admin-settings-help-breach-auto-credential-stuffing-threshold = Eventos na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-mass-forward-threshold = Regras de encaminhamento em massa
admin-settings-help-breach-auto-mass-forward-threshold = Eventos na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-role-escalation-threshold = Mudanças de função de administrador
admin-settings-help-breach-auto-role-escalation-threshold = Mudanças na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-export-volume-threshold = Exportações incomuns
admin-settings-help-breach-auto-export-volume-threshold = Exportações na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-audit-chain-threshold = Falhas na cadeia de auditoria
admin-settings-help-breach-auto-audit-chain-threshold = Falhas que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-backup-anomaly-threshold = Falhas de backup ou restauração
admin-settings-help-breach-auto-backup-anomaly-threshold = Falhas na janela que abrem um rascunho. 0 desativa.
admin-settings-key-breach-auto-dsr-export-threshold = Muitas exportações de solicitações de dados
admin-settings-help-breach-auto-dsr-export-threshold = Exportações na janela que abrem um rascunho. 0 desativa.
admin-settings-key-ratelimit-queue-ms = Maior espera por uma vaga livre
admin-settings-help-ratelimit-queue-ms = Maior tempo que uma solicitação espera por uma vaga livre (concorrência JMAP, decodificadores de imagem, workers de visualização) antes de ser recusada com 429 e Retry-After. Intervalo 0-30000 ms; padrão 3000. Zero recusa imediatamente.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Solicitações JMAP simultâneas por conta

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-help-ratelimit-jmap-per-account-concurrent = Solicitações JMAP que uma conta autenticada pode executar ao mesmo tempo, somando suas abas e dispositivos. Nunca é compartilhado com outra conta nem com outro usuário atrás do mesmo endereço. Intervalo 4-64; padrão 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Solicitações JMAP simultâneas, todas as contas
admin-settings-help-ratelimit-jmap-global-concurrent = Solicitações JMAP que todo o processo do servidor executa ao mesmo tempo, somando todas as contas. Uma solicitação acima do limite espera pelo tempo de fila. Intervalo 4-4096; padrão 256.
admin-settings-key-ratelimit-session-burst = Relatórios de tempo seguidos por sessão
admin-settings-help-ratelimit-session-burst = Relatórios de tempo do cliente que uma sessão pode enviar em sequência. O endpoint usa a sessão como chave, nunca o endereço. Intervalo 1-1000; padrão 60.
admin-settings-key-ratelimit-session-per-minute = Relatórios de tempo recuperados por minuto
admin-settings-help-ratelimit-session-per-minute = Relatórios de tempo do cliente por minuto que uma sessão recupera após a rajada. Intervalo 1-600; padrão 30.
admin-settings-key-ratelimit-anon-burst = Solicitações anônimas de consentimento seguidas
admin-settings-help-ratelimit-anon-burst = Solicitações que um endereço pode fazer em sequência às rotas anônimas de consentimento de cookies. Dimensionado para um escritório atrás de um único endereço. Intervalo 1-1000; padrão 60.
admin-settings-key-ratelimit-anon-per-minute = Solicitações anônimas de consentimento recuperadas por minuto
admin-settings-help-ratelimit-anon-per-minute = Solicitações por minuto que um endereço recupera nas rotas anônimas de consentimento de cookies. Intervalo 1-600; padrão 30.
admin-settings-key-ratelimit-trusted-multiplier = Cota da conta vinda de origem conhecida (%)
admin-settings-help-ratelimit-trusted-multiplier = Porcentagem da cota de solicitações da conta que uma caixa de correio autenticada recebe quando a solicitação vem de uma origem com login bem-sucedido registrado. 100 mantém a cota básica. Nunca relaxa os orçamentos anônimo, de administrador ou de user agent, nem um bloqueio ativo. Intervalo 100-1000; padrão 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Se clicar em qualquer lugar da barra de anexos do leitor, ela será expandida ou recolhida. Quando desativado, apenas o botão de seta funciona.

admin-settings-help-attachments-bar-download-all = Se a barra de anexos oferece um botão Baixar tudo que salva os anexos permitidos como um arquivo ZIP.

admin-settings-help-attachments-bar-show-download-buttons = Quando cada anexo na barra mostra seu botão de download: ao passar o mouse ou focar (padrão), sempre ou nunca. Os downloads ainda seguem a política de anexos e mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Se as imagens mostradas dentro do texto da mensagem estão incluídas em Baixar tudo ZIP. Desativado por padrão.

admin-settings-help-cap-reading-delete-sender = Permita que os usuários ativem a ação do leitor que move todas as mensagens do remetente correspondentes para a Lixeira após a confirmação. Desativado por padrão. O perfil do recurso também deve permitir isso.

admin-settings-help-compose-feature-family = Ative ou desative um controle do compositor por ID do recurso. Padrão ativado; bloqueie-o para impedir que os usuários o alterem.

admin-settings-help-compose-headers-allow-auto-submitted = Permite que os usuários definam o envio automático (RFC 3834) em uma mensagem. Desativado por padrão: diz a outros sistemas para não responderem.

admin-settings-help-compose-headers-allow-custom = Permite que os usuários adicionem seus próprios cabeçalhos (até 20, nomes verificados, cabeçalhos protegidos recusados).

admin-settings-help-compose-headers-allow-return-receipt-to = Permite que os usuários solicitem recibos com o cabeçalho Return-Receipt-To herdado. Desativado por padrão.

admin-settings-help-compose-headers-allow-threading-edit = Permite que os usuários editem respostas e referências ou iniciem um novo tópico a partir de uma resposta.

admin-settings-help-compose-headers-forced = Um par Nome=valor por linha, adicionado a cada mensagem enviada do domínio e substituindo o valor do usuário. Cabeçalhos protegidos são recusados.

admin-settings-help-compose-insert-image-resize-default = O tamanho com o qual a caixa de diálogo Imagem do arquivo começa: original, grande (1600 px) ou médio (1024 px).

admin-settings-help-compose-insert-max-inline-image-mb = Imagens maiores que isso não são colocadas no corpo da mensagem; eles ainda podem ser anexados. 1 a 50.

admin-settings-help-compose-layout-default-font = Família de fonte, tamanho e cor com que uma nova mensagem começa, como family|size|#colour. Vazio usa a fonte do tema.

admin-settings-help-compose-reply-to-allow-override = Quando desativado, a resposta da identidade é usada como está e os usuários não podem editá-la ou removê-la.

admin-settings-help-compose-reply-to-show = Se o compositor mostra a linha Responder para: somente quando uma identidade ou a mensagem original tem uma, sempre ou nunca.

admin-settings-help-compose-sent-folder-fixed-id = Função ou ID da pasta usada quando a regra é "pasta fixa".

admin-settings-help-compose-sent-folder-mode = Para onde vai a cópia de uma mensagem enviada: a pasta Enviados normal, uma pasta fixa, por domínio de destinatário, por identidade ou por regras de assunto.

admin-settings-help-compose-sent-folder-per-identity = Um par identidade=pasta por linha. Usado quando a regra é "por identidade".

admin-settings-help-compose-sent-folder-rules = Uma regra por linha, domain:example.com=pasta ou subject:text=pasta. A primeira partida vence.

admin-settings-help-compose-sent-folder-skip-copy = Nunca guarde uma cópia de uma mensagem enviada. Um administrador pode forçar a ativação ou desativação disso.

admin-settings-help-compose-styles-custom = Os estilos de compositor do próprio usuário como JSON, escritos pela galeria de estilos. Os usuários configuram; não há nada para configurar aqui.

admin-settings-help-compose-styles-default-set = O visual da galeria de Estilos do compositor: padrão, moderno, clássico ou compacto.

admin-settings-help-compose-styles-max-custom = A maioria dos estilos personalizados que um usuário pode manter no compositor. 0 desativa Criar estilo.

admin-settings-help-contacts-save-from-message-add-note = Adicione a nota "Salvo de uma mensagem em <data>" aos contatos salvos de uma mensagem. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-from-message-default-book = O ID do catálogo de endereços que recebe os contatos salvos de uma mensagem; "default" usa o catálogo de endereços padrão. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-from-message-default-scope = Quais participantes são marcados primeiro na caixa de diálogo Salvar contatos: somente o remetente, o remetente e Responder para ou todos. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-from-message-enabled = Oferta Salve os contatos desta mensagem nos menus de mensagens. Quando desligado, o item fica oculto. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-from-message-skip-automated = Deixe os endereços sem resposta, notificação e devolução fora da caixa de diálogo Salvar contatos, a menos que sejam marcados manualmente. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-sender-enabled = Ofereça o item Salvar remetente como contato com um clique nos menus de mensagens. Quando desligado, o item fica oculto. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-contacts-save-sender-use-reply-to = Salve o endereço de resposta em vez de De quando o remetente for uma lista de e-mails ou um endereço de notificação. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-dsr-admin-step-up = Ativado (padrão): um administrador que aprova uma solicitação de dados, executa um apagamento ou aprova em massa deve primeiro confirmar que são eles novamente (senha e segundo fator ou uma chave de acesso). A confirmação dura cinco minutos. Off remove a verificação extra; a decisão ainda fica registrada na trilha de solicitação.

admin-settings-help-export-default-format = O formato com o qual a caixa de diálogo de exportação começa.

admin-settings-help-export-default-scope = O escopo com o qual a caixa de diálogo de exportação começa: mensagem, conversa, seleção ou pasta.

admin-settings-help-export-dialog-enabled = Exportar na visualização de mensagens e os menus de mensagens abrem uma caixa de diálogo. Quando desativados, esses menus baixam a mensagem como .eml.

admin-settings-help-export-folder-enabled = Permitir que os usuários baixem uma pasta inteira como um ZIP de mensagens. Cada exportação é auditada.

admin-settings-help-export-folder-format = Os formatos que uma exportação de pasta oferece: um ZIP de arquivos .eml, um ZIP contendo um arquivo mbox ou ambos.

admin-settings-help-export-folder-include-subfolders = Se uma exportação de pasta inclui suas subpastas por padrão, mantidas como uma estrutura de pastas em ZIP.

admin-settings-help-export-folder-max-bytes = O máximo de bytes de mensagens que uma exportação de pasta pode conter. Uma pasta maior é recusada com o tamanho mostrado.

admin-settings-help-export-folder-max-messages = O maior número de mensagens que uma exportação de pasta pode conter.

admin-settings-help-export-formats = Os formatos que a caixa de diálogo de exportação oferece: eml, mbox, txt, md, msg, docx, csv, json, pdf. O servidor recusa um formato criado pelo servidor listado.

admin-settings-help-export-message-download-enabled = Permita que os usuários baixem uma mensagem como .eml, apenas seus cabeçalhos ou como um arquivo mbox.

admin-settings-help-folders-hide-empty = Oculte pastas vazias na barra lateral. Os usuários podem alterar o padrão ou substituí-lo em pastas individuais, a menos que essa configuração esteja bloqueada. Nenhum e-mail é excluído.

admin-settings-help-folders-pinned-hide-from-tree = As pastas fixadas são exibidas em um grupo Fixado acima da árvore de pastas e também mantêm seu lugar na árvore. Ativado oculta uma pasta fixada que não possui subpastas na árvore, portanto, é exibida apenas no grupo Fixado. Uma pasta fixada com subpastas permanece na árvore.

admin-settings-help-mail-badge-auth = Quando o cabeçalho do leitor mostra o crachá de autenticação DMARC: sempre, somente quando notável ou nunca.

admin-settings-help-mail-badge-density = Tamanho dos crachás no cabeçalho do leitor: compacto ou confortável.

admin-settings-help-mail-badge-folder = Quando o cabeçalho do leitor mostra o emblema da pasta: sempre, somente quando notável ou nunca.

admin-settings-help-mail-badge-headers = Mostre um selo no leitor para os cabeçalhos Expira, Responder por, Sensibilidade e Substitui de uma mensagem.

admin-settings-help-mail-badge-labels = Se os emblemas de cabeçalho do leitor mostram um ícone com texto ou apenas o ícone.

admin-settings-help-mail-badge-list = Quando o cabeçalho do leitor mostra o emblema da lista de discussão: sempre, somente quando notável ou nunca. Ele precisa de um cabeçalho List-Id ou List-Unsubscribe.

admin-settings-help-mail-badge-sanitized = Quando o cabeçalho do leitor mostra o crachá HTML higienizado: sempre, somente quando notável ou nunca.

admin-settings-help-mail-badge-spam = Quando o cabeçalho do leitor mostra o selo de pontuação de spam: sempre, somente quando notável ou nunca.

admin-settings-help-mail-brand-avatars-require-auth = nunca: mostra um logotipo sempre que o domínio corresponder. falha conhecida (padrão): mostre-a, a menos que o resultado DKIM ou DMARC armazenado em cache da mensagem seja uma falha; mensagens desmarcadas mantêm o logotipo. sempre: mostre somente após um passe DMARC alinhado com DKIM para o domínio próprio da marca. As substituições de usuário e domínio podem ser bloqueadas.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Segundos, o corpo deve manter o foco antes que um destinatário completo e o cabeçalho do assunto sejam dobrados em seu resumo. Escolha 0 para desativar a dobra cronometrada ou 1–60 segundos. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-mail-compose-priority-default = A prioridade com que uma nova mensagem começa no compositor.

admin-settings-help-mail-compose-priority-enabled = Mostre o seletor de prioridade Normal, Alta e Baixa no grupo de opções do compositor. Quando desativado, novos e-mails são enviados sem cabeçalhos prioritários.

admin-settings-help-mail-expiry-hide-expired = O que fazer com uma mensagem cujo prazo de validade expirou: nada, mostrá-la esmaecida ou sugerir arquivá-la.

admin-settings-help-mail-list-keep = Remetentes e listas de discussão sempre mantidas, uma por linha como sender:address ou list:List-Id. Limpe este remetente, sugestões de cancelamento de assinatura e regras de retenção ignore-as.

admin-settings-help-mail-list-page-size = Quantas mensagens a lista busca cada vez que um usuário rola até o final. Padrão 50, de 20 a 200. Os usuários escolhem seu próprio valor, a menos que você bloqueie isso.

admin-settings-help-mail-list-unsubscribe-mode = Como funciona o cancelamento de inscrição quando uma lista oferece uma solicitação de um clique: pergunte primeiro, envie imediatamente ou abra apenas o link ou rascunho da lista.

admin-settings-help-mail-message-list-group = Agrupe conversas por data de recebimento ou envio, remetente, assunto, importância ou tamanho. Os usuários podem substituir isso, a menos que estejam bloqueados.

admin-settings-help-mail-priority-marks = Qual prioridade marca a lista de mensagens, as conversas e o leitor mostram: alta e baixa, apenas alta ou nenhuma. Os usuários escolhem em Configurações > Pastas e barra lateral, a menos que estejam bloqueados.

admin-settings-help-mail-priority-marks-style = A aparência de uma marca de prioridade: um ícone, um ícone com as palavras Alta ou Baixa ou uma barra fina. O ícone e o texto mantêm o significado sem cor.

admin-settings-help-mail-priority-send-headers = Quais cabeçalhos uma mensagem de alta ou baixa prioridade recebe quando enviada: qualquer Importância, X-Priority e X-MSMail-Priority.

admin-settings-help-mail-priority-trust-sender-headers = Se uma prioridade definida pelo remetente é mostrada: sempre, somente quando a mensagem passou na verificação DMARC alinhada ou nunca. Use confiável ou nunca contra remetentes que marcam cada mensagem como alta prioridade.

admin-settings-help-mail-reply-from-header-priority = Os cabeçalhos das mensagens são lidos, nesta ordem, para encontrar o endereço para o qual a mensagem foi enviada ao escolher a identidade para resposta. Nomes: Entregue para, X-Original-Para, Envelope-To, X-Forwarded-To, Reenviado para, Para, Cc. Um cabeçalho deixado de fora não é lido.

admin-settings-help-mail-search-default-scope = Inicie novas pesquisas normais de correio na pasta atual ou em todas as pastas. Isso controla o padrão inicial, inclusive quando bloqueado; os usuários ainda podem escolher um escopo diferente para uma pesquisa específica. Consultas salvas e com escopo explícito são preservadas.

admin-settings-help-metrics-dimensions-enabled = Armazene as estatísticas salvas divididas por tipo de cliente (web) e classe de dispositivo (desktop, celular). Desative para armazenar uma linha combinada por métrica e minuto.

admin-settings-help-metrics-persist-enabled = Salve as estatísticas de desempenho (velocidade do proxy de imagem, tempos de carregamento do navegador, limite de taxa e contagens de proteção de solicitação) no banco de dados a cada intervalo de liberação e quando o servidor parar, para que os gráficos mantenham seu histórico após uma reinicialização. Nada identifica uma pessoa, endereço ou mensagem. Desligue para mantê-los apenas na memória.

admin-settings-help-metrics-persist-flush-seconds = A frequência com que as estatísticas de desempenho são gravadas no banco de dados (5 a 3.600 segundos). Um servidor que é eliminado sem aviso perde no máximo esse valor.

admin-settings-help-observability-client-capture-console = Relate também chamadas console.error, no máximo cinco por minuto por página. Desativado por padrão porque as bibliotecas registram as condições esperadas.

admin-settings-help-observability-client-capture-csp = Relate violações da Política de Segurança de Conteúdo que o navegador detecta, apenas com a diretiva bloqueada e a origem. Ativado por padrão.

admin-settings-help-observability-client-enabled = Colete relatórios de erros do aplicativo Web (limites de erros, erros não detectados, partes com falha, erros de trabalho, violações CSP) com rastreamentos de pilha limpos, mostrados em Observabilidade > Erros do cliente. Ativado por padrão. Os relatórios não contêm conteúdo de mensagens, endereços, tokens, nomes de anexos ou consultas URL. Desligue para interromper a coleta; os relatórios existentes expiram com sua retenção.

admin-settings-help-observability-client-forward = Envie também cada relatório limpo para o coletor de erros configurado com MW_ERROR_FORWARD_URL. Desativado por padrão. Sem um sink configurado nada é enviado.

admin-settings-help-observability-client-include-account-hash = Adicione um hash unidirecional codificado da conta aos relatórios enviados de uma sessão conectada, para que os relatórios de uma conta possam ser diferenciados sem nomeá-la. Desativado por padrão. O hash muda quando a chave de privacidade gira.

admin-settings-help-observability-client-max-per-session = A maioria dos relatórios de erros que um carregamento de página pode enviar (1 a 1.000). As repetições do mesmo erro em 60 segundos contam uma vez. Padrão 20.

admin-settings-help-observability-client-sample-rate = Fração de relatórios de erro que um navegador envia, de 0 a 1. Padrão 1 (todos). Cada carregamento de página ainda para no limite por sessão.

admin-settings-help-observability-client-source-maps = Resolva as posições do pacote configurável em rastreamentos de pilha para arquivos de origem quando a compilação enviou mapas de origem ocultos (criados com MW_SOURCEMAP=hidden). Os mapas são lidos apenas pelo servidor e nunca exibidos aos navegadores. Ativado por padrão; sem mapas, os quadros brutos são mostrados.

admin-settings-help-privacy-error-reports = Se um navegador envia um relatório de erro quando parte do aplicativo falha. Ativado por padrão. Escopo global, domínio e usuário: defina o padrão aqui ou bloqueie-o; as pessoas podem desligá-lo em Configurações> Privacidade, a menos que esteja bloqueado.

admin-settings-help-privacy-image-cache-max-stale-secs = Segundos após sua atualização, uma imagem em cache é mantida para responder quando a origem não puder ser alcançada. Nunca abaixo da janela de revalidação. Faixa 0–604800; padrão 86400.

admin-settings-help-privacy-image-cache-stale-secs = Segundos após sua atualização, uma imagem em cache ainda é exibida de uma só vez enquanto é revalidada em segundo plano. Zero desliga. Faixa 0–86400; padrão 3600.

admin-settings-help-privacy-image-resize-enabled = Dimensione imagens remotas superdimensionadas até a largura solicitada pelo leitor (320, 640, 960, 1280 ou 1600 pixels). Imagens animadas são exibidas sem escala. Desativado por padrão.

admin-settings-help-privacy-image-resize-format = Auto escolhe o melhor codificador na compilação, que é PNG. Automático padrão.

admin-settings-help-privacy-image-resize-max-pixels = Imagens com mais pixels decodificados do que isso são exibidas sem escala; o tamanho é lido no cabeçalho antes de qualquer decodificação. Faixa 100.000–6.400.000; padrão 4000000.

admin-settings-help-privacy-image-resize-max-width = A imagem mais ampla que o redimensionamento produz. Faixa 320–1600; padrão 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Como Configurações > Segurança > Sessões nomeia a conta de onde vem uma sessão de acesso à caixa de correio vinculada: o endereço completo, o endereço com a primeira parte do nome mascarada ou nada. O valor é mostrado apenas ao proprietário das sessões, nunca a outros usuários. Padrão: endereço completo.

admin-settings-help-properties-folder-enabled = Ofereça Propriedades nos menus de pastas e no atalho. Quando desativado, a caixa de diálogo não fica disponível.

admin-settings-help-properties-folder-notes-enabled = Permita uma descrição ou nota em cada pasta.

admin-settings-help-properties-folder-notes-max-length = O maior número de caracteres que uma nota de pasta pode conter.

admin-settings-help-properties-folder-overrides-allowed = Quais configurações um usuário pode substituir para uma única pasta. Uma substituição não listada fica oculta e qualquer valor armazenado é ignorado.

admin-settings-help-properties-folder-sections = As seções da caixa de diálogo Propriedades da pasta: geral, remoto, tamanho, off-line, substituições, notas, ações. Uma seção não listada está oculta.

admin-settings-help-properties-folder-size-compute = Quando o tamanho de uma pasta no servidor é determinado: nunca, quando a caixa de diálogo é aberta ou em segundo plano. Contar uma caixa de correio grande custa uma varredura ao servidor.

admin-settings-help-properties-headers-raw = Permita que a caixa de diálogo Propriedades da mensagem liste os cabeçalhos brutos. Quando desativado, apenas o resumo analisado é exibido.

admin-settings-help-properties-message-enabled = Ofereça Propriedades nos menus de mensagens e no atalho. Quando desativado, a caixa de diálogo não fica disponível.

admin-settings-help-properties-message-permanent-delete = Se a caixa de diálogo da mensagem pode excluir uma mensagem definitivamente: permitir, perguntar duas vezes ou negar. Uma mensagem sob retenção legal nunca é excluída. A exclusão desta forma não pode ser desfeita.

admin-settings-help-properties-message-sections = As seções da caixa de diálogo Propriedades da mensagem: resumo, pontuação, cabeçalhos, anexos, ações. Uma seção não listada está oculta.

admin-settings-help-properties-offline-clear-enabled = Permitir que os usuários limpem o cache offline de uma única pasta na caixa de diálogo Propriedades.

admin-settings-help-properties-remote-location-visibility = Quanto do local do servidor os usuários veem: completo (host e caminho), host mascarado (host parcialmente oculto), somente caminho (caminho da caixa de correio sem o host) ou oculto.

admin-settings-help-properties-scoring-show = Mostre pontuação de spam, resultados de autenticação e sinais de segurança na caixa de diálogo Propriedades da mensagem.

admin-settings-help-properties-shortcut-enabled = Abra a caixa de diálogo Propriedades no teclado em uma pasta em foco ou mensagem selecionada.

admin-settings-help-properties-shortcut-key = A combinação de teclas que abre Propriedades; Alt+Enter por padrão.

admin-settings-help-reader-actions-archive = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-block-sender = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-colors = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-conversation = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-create-filter = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-delete = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-delete-sender = Oculto por padrão. Escolha a barra de ferramentas ou o menu Mais para ativá-lo quando o recurso de exclusão do remetente e o perfil do recurso permitirem. Os usuários podem substituir o posicionamento, a menos que estejam bloqueados.

admin-settings-help-reader-actions-edit-draft = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-export = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-finish-followup = Escolha barra de ferramentas, menu Mais ou oculto para completar mensagens sinalizadas e lembretes de acompanhamento.

admin-settings-help-reader-actions-flag = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-focus-focused = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-focus-other = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-forward = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-headers = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-links = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-move = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-not-spam = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-open-draft = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-pin = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-print = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-properties = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-read = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-reply = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-reply-all = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-save-contacts = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-save-sender = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-search = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-security = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-spam = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-sweep = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-view-source = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-reader-actions-zoom = Escolha barra de ferramentas, menu Mais ou oculto. Restrições de recursos, permissões e contexto de mensagens ainda se aplicam. Os usuários podem escolher a menos que esteja bloqueado.

admin-settings-help-retention-client-errors-event-days = Dias em que uma ocorrência individual de erro do navegador (seu rastreamento de pilha e classe do navegador) é mantida. Padrão 30. Um grupo é mantido por mais tempo, veja a próxima configuração.

admin-settings-help-retention-client-errors-group-days = Dias em que um grupo de erros do navegador (nome, mensagem, contagens) é mantido após ter sido visto pela última vez. Padrão 180. Nunca menor que a retenção de ocorrência.

admin-settings-help-retention-metrics-day-buckets-days = Dias de estatísticas diárias de desempenho mantidas antes de serem excluídas. 0 os mantém sem limite, o que permite que a tabela cresça lentamente enquanto o servidor estiver em execução.

admin-settings-help-retention-metrics-hour-buckets-days = Dias de estatísticas de desempenho por hora mantidos antes de serem mesclados em linhas diárias (pelo menos 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Horas de estatísticas de desempenho de um minuto mantidas antes de serem mescladas em linhas horárias (pelo menos 1).

admin-settings-help-retention-user-allow-strip-attachments = Se os usuários podem criar regras que removam anexos grandes de emails antigos. Desativado por padrão. Enquanto estiver desativada, a ação fica oculta no editor de regras, uma regra que a utiliza é recusada quando salva e as regras salvas anteriormente são ignoradas quando a tarefa é executada.

admin-settings-help-retention-user-calendar-enabled = Se os usuários podem escolher calendários cujos eventos são excluídos um determinado número de dias após seu término. Desativado por padrão. Enquanto desativada, a opção fica oculta nas Configurações, uma escolha é recusada quando salva e as opções salvas anteriormente não são executadas.

admin-settings-help-retention-user-calendar-max-days = Na maioria dos dias, um usuário pode esperar até que os eventos finalizados sejam excluídos. Um valor maior é recusado quando salvo e diminuído quando é executado.

admin-settings-help-retention-user-calendar-min-days = Menos dias após o término de um evento antes que uma regra de usuário possa excluí-lo. Um valor menor é recusado quando salvo e aumentado quando executado.

admin-settings-help-retention-user-notes-enabled = Se os usuários podem optar por remover notas não alteradas por um determinado número de dias. Desativado por padrão. As notas removidas podem ser restauradas até que a janela de desfazer das regras de retenção termine.

admin-settings-help-retention-user-notes-max-days = Na maioria dos dias, um usuário pode esperar até que as notas não alteradas sejam removidas. Um valor maior é recusado quando salvo e diminuído quando é executado.

admin-settings-help-retention-user-notes-min-days = Menos dias em que uma nota deve permanecer intacta antes que uma regra de usuário possa removê-la. Um valor menor é recusado quando salvo e aumentado quando executado.

admin-settings-help-retention-user-strip-keep-inline-images = Se as imagens mostradas no corpo da mensagem são mantidas quando os anexos são removidos.

admin-settings-help-retention-user-strip-min-age-days = O menor número de dias que uma mensagem deve ser antiga antes que uma regra possa remover seus anexos. Um valor menor é recusado quando a regra é salva e aumentado quando ela é executada.

admin-settings-help-retention-user-strip-min-attachment-kb = Somente anexos com pelo menos esse tamanho serão removidos. Anexos menores permanecem com a mensagem.

admin-settings-help-retention-user-strip-undo-days = Dias em que a mensagem original permanece na Lixeira depois que seus anexos foram removidos. -1 segue a janela de desfazer para regras de retenção. 0 exclui o original de uma vez.

admin-settings-help-server-shutdown-grace-seconds = Quanto tempo um servidor interrompido permite que as solicitações em andamento terminem antes de fechar as conexões, em segundos. A variável MW_HTTP_SHUTDOWN_TIMEOUT_MS a substitui. Aplica-se na próxima partida.

admin-settings-help-session-last-activity-flush-seconds = O horário da última visualização de uma sessão é gravado no máximo com esta frequência, em segundos. Zero escreve em cada solicitação. Não altera os limites ociosos ou absolutos. Aplica-se na próxima partida.

admin-settings-help-session-persist-across-restarts = Quando ativado, as sessões de caixa de correio, aplicativo e console permanecem válidas após a reinicialização do servidor. Quando desativado, todas as sessões terminam quando o servidor é iniciado. Aplica-se na próxima partida.

admin-settings-key-attachments-bar-click-to-toggle = Clique na barra de anexos para alternar

admin-settings-key-attachments-bar-download-all = Botão baixar tudo

admin-settings-key-attachments-bar-show-download-buttons = Botões de download de anexos

admin-settings-key-attachments-bar-zip-inline-images = Imagens embutidas em Baixar tudo ZIP

admin-settings-key-cap-reading-delete-sender = Permitir a exclusão de todos os e-mails de um remetente

admin-settings-key-compose-feature-family = Recursos do compositor

admin-settings-key-compose-headers-allow-auto-submitted = Permitir envio automático

admin-settings-key-compose-headers-allow-custom = Permitir cabeçalhos personalizados

admin-settings-key-compose-headers-allow-return-receipt-to = Permitir recebimento de devolução

admin-settings-key-compose-headers-allow-threading-edit = Permitir edição de cabeçalhos de threading

admin-settings-key-compose-headers-forced = Cabeçalhos forçados

admin-settings-key-compose-insert-image-resize-default = Predefinição de tamanho de imagem

admin-settings-key-compose-insert-max-inline-image-mb = Maior imagem embutida (MB)

admin-settings-key-compose-layout-default-font = Fonte de mensagem padrão

admin-settings-key-compose-reply-to-allow-override = Permitir alteração de resposta para

admin-settings-key-compose-reply-to-show = Linha Responder para

admin-settings-key-compose-sent-folder-fixed-id = Cópia enviada: pasta fixa

admin-settings-key-compose-sent-folder-mode = Cópia enviada: regra de pasta

admin-settings-key-compose-sent-folder-per-identity = Cópia enviada: pasta por identidade

admin-settings-key-compose-sent-folder-rules = Cópia enviada: regras

admin-settings-key-compose-sent-folder-skip-copy = Cópia enviada: não salve

admin-settings-key-compose-styles-custom = Estilos personalizados

admin-settings-key-compose-styles-default-set = Conjunto de estilo

admin-settings-key-compose-styles-max-custom = Estilos personalizados por usuário

admin-settings-key-contacts-save-from-message-add-note = Nota sobre contatos salvos de uma mensagem

admin-settings-key-contacts-save-from-message-default-book = Catálogo de endereços para contatos salvos de uma mensagem

admin-settings-key-contacts-save-from-message-default-scope = Contatos marcados primeiro ao salvar de uma mensagem

admin-settings-key-contacts-save-from-message-enabled = Salvar contatos de uma mensagem

admin-settings-key-contacts-save-from-message-skip-automated = Ignore remetentes automáticos ao salvar contatos

admin-settings-key-contacts-save-sender-enabled = Salvar remetente como contato

admin-settings-key-contacts-save-sender-use-reply-to = Salvar resposta para remetentes da lista

admin-settings-key-dsr-admin-step-up = Confirme novamente antes de aprovar

admin-settings-key-export-default-format = Exportar formato padrão da caixa de diálogo

admin-settings-key-export-default-scope = Escopo padrão da caixa de diálogo de exportação

admin-settings-key-export-dialog-enabled = Caixa de diálogo Exportar

admin-settings-key-export-folder-enabled = Exportação de pasta

admin-settings-key-export-folder-format = Formato de exportação de pasta

admin-settings-key-export-folder-include-subfolders = Incluir subpastas na exportação

admin-settings-key-export-folder-max-bytes = Limite de tamanho de exportação de pasta

admin-settings-key-export-folder-max-messages = Limite de mensagens de exportação de pasta

admin-settings-key-export-formats = Exportar formatos de diálogo

admin-settings-key-export-message-download-enabled = Download de mensagens

admin-settings-key-folders-hide-empty = Ocultar pastas vazias

admin-settings-key-folders-pinned-hide-from-tree = Ocultar pastas fixadas da árvore

admin-settings-key-mail-badge-auth = Selo de autenticação

admin-settings-key-mail-badge-density = Tamanho do selo do leitor

admin-settings-key-mail-badge-folder = Selo de pasta

admin-settings-key-mail-badge-headers = Emblemas de cabeçalho

admin-settings-key-mail-badge-labels = Etiquetas de crachá de leitor

admin-settings-key-mail-badge-list = Selo da lista de discussão

admin-settings-key-mail-badge-sanitized = Crachá HTML higienizado

admin-settings-key-mail-badge-spam = Selo de pontuação de spam

admin-settings-key-mail-brand-avatars-require-auth = Autenticação necessária para logotipos de marcas

admin-settings-key-mail-compose-header-auto-collapse-seconds = Atraso no colapso automático do cabeçalho do compositor

admin-settings-key-mail-compose-priority-default = Prioridade de mensagem padrão

admin-settings-key-mail-compose-priority-enabled = Seletor de prioridade no compositor

admin-settings-key-mail-expiry-hide-expired = Mensagens expiradas

admin-settings-key-mail-list-keep = Mantenha sempre

admin-settings-key-mail-list-page-size = Tamanho da página da lista de mensagens

admin-settings-key-mail-list-unsubscribe-mode = Lista de discussão Cancelar inscrição

admin-settings-key-mail-message-list-group = Agrupamento de mensagens

admin-settings-key-mail-priority-marks = Marcas de prioridade

admin-settings-key-mail-priority-marks-style = Estilo de marca de prioridade

admin-settings-key-mail-priority-send-headers = Cabeçalhos prioritários escritos no envio

admin-settings-key-mail-priority-trust-sender-headers = Confiar na prioridade definida pelo remetente

admin-settings-key-mail-reply-from-header-priority = Cabeçalhos que escolhem a identidade da resposta

admin-settings-key-mail-search-default-scope = Escopo de pesquisa de e-mail padrão

admin-settings-key-metrics-dimensions-enabled = Divida as estatísticas por tipo de cliente e classe de dispositivo

admin-settings-key-metrics-persist-enabled = Mantenha as estatísticas nas reinicializações

admin-settings-key-metrics-persist-flush-seconds = Intervalo de salvamento de estatísticas (segundos)

admin-settings-key-observability-client-capture-console = Relatar erros do console

admin-settings-key-observability-client-capture-csp = Denunciar violações CSP

admin-settings-key-observability-client-enabled = Relatórios de erros do navegador

admin-settings-key-observability-client-forward = Relatórios de erros de encaminhamento

admin-settings-key-observability-client-include-account-hash = Adicionar hash da conta aos relatórios de erros

admin-settings-key-observability-client-max-per-session = Relatórios de erros por carregamento de página

admin-settings-key-observability-client-sample-rate = Taxa de amostragem do relatório de erros

admin-settings-key-observability-client-source-maps = Resolver rastreamentos de pilha com mapas de origem

admin-settings-key-privacy-error-reports = Enviar relatórios de erros

admin-settings-key-privacy-image-cache-max-stale-secs = Manter obsoleto quando a origem falhar (segundos)

admin-settings-key-privacy-image-cache-stale-secs = Servir obsoleto durante a revalidação (segundos)

admin-settings-key-privacy-image-resize-enabled = Redimensionar imagens grandes

admin-settings-key-privacy-image-resize-format = Formato de imagem redimensionada

admin-settings-key-privacy-image-resize-max-pixels = Maior imagem para redimensionar (pixels)

admin-settings-key-privacy-image-resize-max-width = Imagem redimensionada mais ampla (pixels)

admin-settings-key-privacy-sessions-show-linked-origin = Origem da caixa de correio vinculada em sessões

admin-settings-key-properties-folder-enabled = Caixa de diálogo de propriedades da pasta

admin-settings-key-properties-folder-notes-enabled = Notas de pasta

admin-settings-key-properties-folder-notes-max-length = Comprimento da nota da pasta

admin-settings-key-properties-folder-overrides-allowed = Substituições por pasta permitidas

admin-settings-key-properties-folder-sections = Seções de diálogo de pasta

admin-settings-key-properties-folder-size-compute = Cálculo do tamanho da pasta

admin-settings-key-properties-headers-raw = Cabeçalhos brutos

admin-settings-key-properties-message-enabled = Caixa de diálogo de propriedades da mensagem

admin-settings-key-properties-message-permanent-delete = Exclusão permanente das propriedades

admin-settings-key-properties-message-sections = Seções da caixa de diálogo de mensagens

admin-settings-key-properties-offline-clear-enabled = Limpe o cache offline por pasta

admin-settings-key-properties-remote-location-visibility = Detalhe da localização remota

admin-settings-key-properties-scoring-show = Detalhes de pontuação e autenticação

admin-settings-key-properties-shortcut-enabled = Atalho de propriedades

admin-settings-key-properties-shortcut-key = Tecla de atalho de propriedades

admin-settings-key-reader-actions-archive = Posicionamento de ação de arquivo

admin-settings-key-reader-actions-block-sender = Bloquear posicionamento de ação do remetente

admin-settings-key-reader-actions-colors = Posicionamento da ação das cores da mensagem

admin-settings-key-reader-actions-conversation = Ver o posicionamento da ação de conversa

admin-settings-key-reader-actions-create-filter = Criar posicionamento de ação de filtro

admin-settings-key-reader-actions-delete = Excluir posicionamento de ação

admin-settings-key-reader-actions-delete-sender = Excluir todas as mensagens da ação do remetente

admin-settings-key-reader-actions-edit-draft = Editar posicionamento de ação de rascunho

admin-settings-key-reader-actions-export = Exportar posicionamento de ação

admin-settings-key-reader-actions-finish-followup = Marcar o acompanhamento como ação concluída

admin-settings-key-reader-actions-flag = Posicionamento de ação sinalizar/desmarcar

admin-settings-key-reader-actions-focus-focused = Mude para o posicionamento de ação focada

admin-settings-key-reader-actions-focus-other = Mover para outro posicionamento de ação

admin-settings-key-reader-actions-forward = Posicionamento de ação direta

admin-settings-key-reader-actions-headers = Posicionamento de ação de cabeçalhos brutos

admin-settings-key-reader-actions-links = Revise o posicionamento da ação dos links

admin-settings-key-reader-actions-move = Mover posicionamento de ação

admin-settings-key-reader-actions-not-spam = Não é posicionamento de ação de spam

admin-settings-key-reader-actions-open-draft = Abrir posicionamento de ação de rascunho de resposta

admin-settings-key-reader-actions-pin = Fixar/desafixar posicionamento de ação

admin-settings-key-reader-actions-print = Imprimir posicionamento de ação

admin-settings-key-reader-actions-properties = Posicionamento de ação de propriedades de mensagem

admin-settings-key-reader-actions-read = Posicionamento de ação lida/não lida

admin-settings-key-reader-actions-reply = Posicionamento da ação de resposta

admin-settings-key-reader-actions-reply-all = Responder a todos os posicionamentos de ação

admin-settings-key-reader-actions-save-contacts = Salvar posicionamento de ação de contatos de mensagens

admin-settings-key-reader-actions-save-sender = Salvar posicionamento da ação do remetente

admin-settings-key-reader-actions-search = Posicionamento de ação de pesquisa relacionada

admin-settings-key-reader-actions-security = Posicionamento de ação no modo de segurança

admin-settings-key-reader-actions-spam = Marcar como posicionamento de ação de spam

admin-settings-key-reader-actions-sweep = Varredura no posicionamento da ação do remetente

admin-settings-key-reader-actions-view-source = Ver o posicionamento da ação da origem da mensagem

admin-settings-key-reader-actions-zoom = Posicionamento da ação de zoom

admin-settings-key-retention-client-errors-event-days = Ocorrências de erro mantidas (dias)

admin-settings-key-retention-client-errors-group-days = Grupos de erros mantidos (dias)

admin-settings-key-retention-metrics-day-buckets-days = Estatísticas diárias mantidas (dias)

admin-settings-key-retention-metrics-hour-buckets-days = Estatísticas horárias mantidas (dias)

admin-settings-key-retention-metrics-minute-buckets-hours = Estatísticas de minutos mantidas (horas)

admin-settings-key-retention-user-allow-strip-attachments = Os usuários podem remover anexos de e-mails antigos

admin-settings-key-retention-user-calendar-enabled = Os usuários podem excluir eventos de calendário encerrados

admin-settings-key-retention-user-calendar-max-days = Na maioria dos dias antes do término, os eventos são excluídos

admin-settings-key-retention-user-calendar-min-days = O menor número de dias antes do término dos eventos é excluído

admin-settings-key-retention-user-notes-enabled = Os usuários podem remover notas intocadas

admin-settings-key-retention-user-notes-max-days = Na maioria dos dias antes das notas intocadas serem removidas

admin-settings-key-retention-user-notes-min-days = Menos dias antes que as notas intocadas sejam removidas

admin-settings-key-retention-user-strip-keep-inline-images = Mantenha imagens embutidas

admin-settings-key-retention-user-strip-min-age-days = Menos dias antes que os anexos sejam removidos

admin-settings-key-retention-user-strip-min-attachment-kb = Menor anexo removido (KB)

admin-settings-key-retention-user-strip-undo-days = Janela de desfazer após a remoção dos anexos (dias)

admin-settings-key-server-shutdown-grace-seconds = Período de carência de desligamento

admin-settings-key-session-last-activity-flush-seconds = Intervalo de gravação visto pela última vez

admin-settings-key-session-persist-across-restarts = Mantenha as sessões durante as reinicializações

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Desativado por padrão. Salve instantâneos de e-mail criptografados com escopo de conta nas recargas do navegador sem expirar. Os limites de entrada e bytes existentes ainda se aplicam. Os usuários podem limpar as cópias salvas; desabilitar esta preferência os remove do navegador ativo. Isso não concede permissão de armazenamento persistente do navegador ou download de anexos.

admin-settings-help-mail-conversations-allowed = Permitir que os usuários agrupem respostas relacionadas em conversas. Disponível por padrão, mas a preferência do usuário começa. A desativação da disponibilidade substitui a escolha do usuário sem alterar o agrupamento de categorias.

admin-settings-help-mail-conversations-enabled = Desativado por padrão. Agrupar respostas carregadas por thread, independentemente de Agrupar por. Defina o padrão ou bloqueie esta preferência. A disponibilidade de conversa também deve estar habilitada.

admin-settings-key-mail-cache-keep-indefinitely = Manter o cache de e-mail do navegador indefinidamente

admin-settings-key-mail-conversations-allowed = Disponibilizar o encadeamento de conversas

admin-settings-key-mail-conversations-enabled = Agrupar mensagens em conversas

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Até 100 domínios de remetentes exatos em letras minúsculas cujos links de mensagens revisados são ativados automaticamente. Os subdomínios são separados. Isso não confia em sites de destino nem permite imagens remotas. Vazio por padrão; os usuários podem alterá-lo, a menos que estejam bloqueados.

admin-settings-key-links-trusted-sender-domains = Domínios de remetentes confiáveis para links de mensagens

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Mostrar detalhes de download e arquivo ao lado do título de visualização como ícones (padrão), rótulos ou ícones e rótulos. Pode ser configurado e bloqueado globalmente, por domínio ou por usuário.

admin-settings-help-reader-attachment-thumbnails = Desativado por padrão. Permitir que anexos de imagens visíveis carreguem visualizações em miniatura. As regras de segurança de anexos ainda se aplicam. Pode ser configurado e bloqueado globalmente, por domínio ou por usuário.

admin-settings-key-reader-attachment-action-style = Ações de visualização de anexos

admin-settings-key-reader-attachment-thumbnails = Miniaturas de anexos de imagens

admin-settings-key-mail-compose-auto-add-mention-recipients = Adicionar automaticamente as pessoas mencionadas aos destinatários
admin-settings-help-mail-compose-auto-add-mention-recipients = Ao selecionar uma pessoa nas sugestões de @menções, adicioná-la a Para, a menos que já esteja em Para, Cc ou Cco. Desativado por padrão.

admin-settings-linked-disable-title = Desativar o acesso salvo às contas vinculadas?
admin-settings-linked-disable-body = Salvar esta alteração remove as credenciais salvas para renovar o acesso às contas vinculadas. Os usuários precisarão entrar novamente nessas contas quando o acesso precisar ser renovado. Reativar esta opção não restaura essas credenciais.
admin-settings-linked-disable-confirm = Desativar e limpar o acesso salvo

admin-settings-key-reader-auto-hide-header = Ocultar automaticamente os cabeçalhos das mensagens
admin-settings-help-reader-auto-hide-header = Abrir cada mensagem com os detalhes do remetente, destinatários e metadados recolhidos. O assunto e as ações permanecem visíveis. Use Mostrar cabeçalho para ver os detalhes. Desativado por padrão.

admin-settings-key-time-region = Região
admin-settings-help-time-region = auto ou um código de região de duas letras, como GB, US ou PT.

admin-settings-key-time-date-format = Formato de data
admin-settings-help-time-date-format = auto, short, medium, long, full ou um padrão como yyyy-MM-dd ou dd/MM/yyyy. Coloque texto literal entre 'aspas'.

admin-settings-key-time-time-format = Formato de hora
admin-settings-help-time-time-format = auto, short, medium, long, full ou HH:mm:ss / h:mm a. H = 24 horas, h = 12 horas, a = AM/PM.

admin-settings-key-time-hour-cycle = Relógio
admin-settings-help-time-hour-cycle = Os símbolos H ou h de um padrão personalizado têm prioridade.

admin-settings-key-time-first-day-of-week = Primeiro dia da semana
admin-settings-help-time-first-day-of-week = Automático segue as convenções da região.

admin-settings-key-compliance-mail-enabled = Ativar email
admin-settings-help-compliance-mail-enabled = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-host = Servidor SMTP
admin-settings-help-compliance-mail-host = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-port = Porta SMTP
admin-settings-help-compliance-mail-port = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-security = Modo TLS
admin-settings-help-compliance-mail-security = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-from = Endereço do remetente
admin-settings-help-compliance-mail-from = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-username = Usuário SMTP
admin-settings-help-compliance-mail-username = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-allowPrivate = Permitir servidor privado
admin-settings-help-compliance-mail-allowPrivate = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-dsr-notify-emailAdmins = Notificar operadores sobre solicitações de dados
admin-settings-help-dsr-notify-emailAdmins = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-dsr-notify-operatorAddresses = Destinatários operadores
admin-settings-help-dsr-notify-operatorAddresses = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-dsr-notify-emailRequester = Notificar solicitante por email
admin-settings-help-dsr-notify-emailRequester = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-breach-notify-email = Notificar operadores de violações
admin-settings-help-breach-notify-email = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-breach-subjects-email = Notificar pessoas afetadas por email
admin-settings-help-breach-subjects-email = Configure um servidor SMTP com TLS. O email usa a rota proxy system_mail. Revise aceitações incertas antes de reenviar.

admin-settings-key-compliance-mail-requireProxy = Exigir rota proxy
admin-settings-help-compliance-mail-requireProxy = Exigir rota proxy (system_mail).
admin-settings-key-retention-complianceMail-days = Retenção do histórico de emails concluídos (dias)
admin-settings-help-retention-complianceMail-days = Dias para manter entregas concluídas. Entregas pendentes e incertas permanecem até a resolução.

admin-settings-key-incidents-auto-enabled = Detectar possíveis incidentes automaticamente
admin-settings-help-incidents-auto-enabled = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.
admin-settings-key-incidents-auto-failureSamples = Amostras de falha antes da detecção
admin-settings-help-incidents-auto-failureSamples = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.
admin-settings-key-incidents-nis2-trustService = Usar prazo de notificação de serviços de confiança
admin-settings-help-incidents-nis2-trustService = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.
admin-settings-key-incidents-notify-email = Enviar atualizações e lembretes de incidentes por email
admin-settings-help-incidents-notify-email = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.
admin-settings-key-incidents-notify-operatorAddresses = Destinatários de email dos operadores
admin-settings-help-incidents-notify-operatorAddresses = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.
admin-settings-key-incidents-retention-days = Retenção de incidentes encerrados (dias)
admin-settings-help-incidents-retention-days = Falhas de integridade e sinais de segurança das sessões criam possíveis incidentes. Avalie a relevância e verifique a recuperação. Lacunas de monitoramento não comprovam indisponibilidade.

admin-settings-key-mail-snooze-presets = Opções de adiamento
admin-settings-help-mail-snooze-presets = Adicione, renomeie, reordene, edite ou oculte opções. O menu sempre permite escolher uma data e hora.

admin-settings-key-reader-actions-view-draft = Ver rascunho da resposta
admin-settings-help-reader-actions-view-draft = Ver rascunho da resposta

admin-settings-key-notifications-view-enabled = Ativar a visualização Notificações
admin-settings-help-notifications-view-enabled = Controla a central de notificações. Os avisos de segurança, de e-mail e do navegador mantêm suas próprias configurações.
admin-settings-key-notifications-view-show-in-sidebar = Mostrar Notificações na barra lateral
admin-settings-help-notifications-view-show-in-sidebar = Controla a central de notificações. Os avisos de segurança, de e-mail e do navegador mantêm suas próprias configurações.

admin-settings-key-mail-unified-account = Conta virtual unificada
admin-settings-help-mail-unified-account = Adicionar uma conta virtual ao seletor de contas, com pastas e contadores combinados. Independente da visualização unificada de mensagens.

admin-settings-key-accounts-settings-inherit-groups = Herdar preferências da conta principal
admin-settings-help-accounts-settings-inherit-groups = Os grupos selecionados usam valores da conta principal quando não há uma substituição nesta caixa. As escolhas locais e as políticas do administrador prevalecem. Temas, assinaturas, credenciais e IDs de pastas não são herdados.

admin-settings-key-mail-badge-importance = Importância
admin-settings-help-mail-badge-importance = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-follow-up = Acompanhamento
admin-settings-help-mail-badge-follow-up = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-snooze = Adiar
admin-settings-help-mail-badge-snooze = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-receipts = Confirmações de leitura
admin-settings-help-mail-badge-receipts = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-delivery = Status de entrega
admin-settings-help-mail-badge-delivery = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-dates = Datas da mensagem
admin-settings-help-mail-badge-dates = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-sensitivity = Sensibilidade
admin-settings-help-mail-badge-sensitivity = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-reply-by = Prazo de resposta
admin-settings-help-mail-badge-reply-by = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-expires = Validade
admin-settings-help-mail-badge-expires = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.
admin-settings-key-mail-badge-supersedes = Mensagens substituídas
admin-settings-help-mail-badge-supersedes = Escolha sempre, apenas quando relevante ou nunca. Destaca valores incomuns, lembretes ativos, confirmações, eventos de entrega ou prazos próximos. Os dados da mensagem são mantidos.

admin-settings-key-mail-follow-up-presets = Marcar para acompanhamento
admin-settings-help-mail-follow-up-presets = Escolha horários de acompanhamento separados do adiamento. O acompanhamento mantém as mensagens visíveis; o adiamento oculta até a data escolhida. As escolhas locais podem substituir padrões herdados quando a política do administrador permitir. Apenas Mais tarde hoje, Amanhã e Próxima semana são ativados por padrão. As outras opções estão disponíveis para ativar ou personalizar.

admin-settings-key-reader-file-previews = Visualizações de arquivos
admin-settings-help-reader-file-previews = Escolha os arquivos que podem abrir no aplicativo e as ferramentas exibidas. As restrições de anexos e as permissões de download do administrador continuam valendo. Barra de ferramentas exibe a ferramenta junto à visualização; Menu Mais move para os três pontos; Oculto remove. Os grupos PDF mantêm a disposição original até serem alterados. As permissões de download e impressão prevalecem.
