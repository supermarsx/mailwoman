# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Aviso de plugin de interface não assinado
plugins-unsigned-title = Há plugins não assinados em execução
plugins-unsigned-text = Estes plugins foram admitidos sem uma assinatura verificada. Eles rodam em sandbox, mas o código deles não foi conferido com uma chave confiável.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Os plugins do motor rodam em uma sandbox WebAssembly. Um plugin só roda depois de ser registrado, aprovado, receber pelo menos uma permissão e ser ativado. Ele só pode usar as permissões concedidas aqui.
plugins-admin-unsigned-banner = Um plugin de terceiros sem assinatura está carregado. Ele roda porque um administrador permitiu; o servidor apenas confere o arquivo com o digest aprovado.
plugins-admin-load-error = Não foi possível carregar os plugins.
plugins-admin-register-heading = Registrar um plugin
plugins-admin-register-component = Componente
plugins-admin-register-third-party = Componente de terceiros
plugins-admin-register-id = ID do plugin
plugins-admin-register-id-hint = O servidor lê o componente do arquivo com o nome deste ID no diretório de plugins de terceiros. Antes, aprove o digest desse arquivo na lista de permissões abaixo.
plugins-admin-register-name = Nome
plugins-admin-register-version = Versão
plugins-admin-register-capabilities = Permissões que o plugin declara
plugins-admin-register-hosts = Hosts que o plugin pode contatar
plugins-admin-register-hosts-hint-first-party = Nomes de host separados por vírgula. Deixe em branco para usar os hosts padrão do componente.
plugins-admin-register-hosts-hint-third-party = Nomes de host separados por vírgula. Deixe em branco para nenhum acesso à rede.
plugins-admin-register-submit = Registrar
plugins-admin-trust-first-party = Próprio, digest incorporado
plugins-admin-trust-third-party = De terceiros, digest aprovado
plugins-admin-status-loaded = Carregado
plugins-admin-status-not-loaded = Não carregado
plugins-admin-status-restart = Reinicialização necessária
plugins-admin-restart-to-load = É necessário reiniciar para carregar este plugin.
plugins-admin-restart-to-apply = É necessário reiniciar para aplicar estas alterações. Algumas contas desta ponte não são atendidas até lá.
plugins-admin-reason-not-approved = Não carregado: nenhum administrador aprovou este plugin.
plugins-admin-reason-disabled = Não carregado: o plugin não está ativado.
plugins-admin-reason-unsigned-not-allowed = Não carregado: o plugin não tem assinatura e não foi autorizado a rodar sem uma.
plugins-admin-reason-no-grant = Não carregado: nenhuma permissão de que ele precisa foi concedida.
plugins-admin-reason-component-unavailable = Não carregado: nenhum arquivo de componente passou na verificação de digest.
plugins-admin-reason-load-failed = Não carregado: o servidor não conseguiu carregar o componente. Consulte o log do servidor.
plugins-admin-reason-proxy-mode = Não carregado: este servidor roda em modo proxy e não processa e-mail por conta própria.
plugins-admin-reason-no-account-binding = Não carregado: nenhuma conta está vinculada a esta ponte.
plugins-admin-reason-no-host-caller = Não carregado: esta versão do servidor não chama os hooks que este plugin oferece.
plugins-admin-reason-another-classifier-active = Não carregado: outro classificador de spam está carregado. Só um roda por vez.
plugins-admin-running-with = Rodando com: { $capabilities }
plugins-admin-hosts = Hosts: { $hosts }
plugins-admin-hosts-none = Hosts: nenhum
plugins-admin-grants-heading = Permissões concedidas
plugins-admin-grants-hint = O plugin só pode usar as permissões marcadas aqui. Salvar substitui a concessão anterior.
plugins-admin-grant-for = Conceder { $capability } a { $name }
plugins-admin-grants-save = Salvar permissões
plugins-admin-endpoint = Endereço do classificador
plugins-admin-endpoint-for = Endereço do classificador para { $name }
plugins-admin-endpoint-hint = Onde o plugin acessa o daemon dele. O host também precisa estar na lista de hosts do plugin. Deixe em branco para usar o padrão do componente.
plugins-admin-endpoint-save = Salvar endereço
plugins-admin-test = Testar classificador
plugins-admin-test-verdict = Resultado do teste: { $verdict }
plugins-admin-uninstall = Desinstalar
plugins-admin-uninstall-confirm = Confirmar desinstalação
plugins-admin-uninstall-note = A desinstalação remove o registro, as permissões e as configurações armazenadas.
plugins-admin-uninstall-detail = A desinstalação exclui os dados armazenados do plugin de todas as contas, as fixações dele na lista de permissões, o registro e as permissões. O arquivo do componente no disco não é excluído.
plugins-admin-revoke-detail = A revogação remove a aprovação deste digest e desativa o plugin. Se o plugin estiver rodando, ele é interrompido imediatamente.
plugins-admin-error-already-registered = Este plugin já está registrado. Desinstale-o antes de registrá-lo novamente.
plugins-admin-error-component-unavailable = Nenhum arquivo de componente deste plugin corresponde ao digest incorporado ao servidor.
plugins-admin-error-digest-not-approved = Nenhum arquivo de componente com digest aprovado foi encontrado para este ID. Antes, aprove o digest do arquivo na lista de permissões.
plugins-admin-error-not-approved = Aprove o plugin antes de ativá-lo.
plugins-admin-error-unsigned-not-allowed = Este plugin não tem assinatura. Permita que ele rode sem assinatura antes de ativá-lo.
plugins-admin-error-not-loaded = O plugin não está carregado, então não pode ser testado.
plugins-admin-error-classifier-error = A chamada ao classificador falhou: { $detail }
plugins-admin-error-other = O servidor recusou a solicitação: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = Plugins de interface
plugins-admin-ui-title = Plugins de interface
plugins-admin-ui-intro = Os plugins de interface adicionam controles ao cliente web. Cada um roda em um quadro em sandbox e só pode usar as permissões concedidas aqui. Um plugin é oferecido aos usuários conectados depois de aprovado e ativado.
plugins-admin-ui-empty = Nenhum plugin de interface registrado.
plugins-admin-ui-load-error = Não foi possível carregar os plugins de interface.
plugins-admin-ui-register-heading = Registrar um plugin de interface
plugins-admin-ui-manifest = Manifesto (JSON)
plugins-admin-ui-bundle = Arquivo do pacote
plugins-admin-ui-bundle-hint = Obrigatório para um plugin assinado: a assinatura é conferida com este arquivo.
plugins-admin-ui-allow-unsigned = Permitir que este plugin seja registrado sem assinatura
plugins-admin-ui-register = Registrar
plugins-admin-ui-manifest-invalid = O manifesto não é um JSON válido.
plugins-admin-ui-refused = O servidor recusou a solicitação: { $detail }
plugins-admin-ui-approve = Aprovar e ativar
plugins-admin-ui-extension-points = Pontos de extensão: { $points }
plugins-admin-ui-grants-heading = Conceder uma permissão
plugins-admin-ui-grants-note = O servidor não informa quais permissões já foram concedidas. Conceder uma permissão novamente substitui a concessão anterior.
plugins-admin-ui-grant = Conceder { $capability }
plugins-admin-ui-grant-for = Conceder { $capability } a { $name }
plugins-admin-ui-grant-hosts = Hosts para { $capability } em { $name }
plugins-admin-ui-grant-hosts-hint = Nomes de host separados por vírgula dos quais o plugin pode buscar dados.
plugins-admin-ui-granted = { $capability } concedida.
plugins-admin-ui-delete = Excluir
plugins-admin-ui-delete-confirm = Confirmar exclusão
