# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Provedor de identidade
admin-identity-description = Permita que aplicativos aprovados façam login com o Mailwoman. Isso é separado dos provedores de login externos e do acesso à API de e-mail. O código de autorização com PKCE é obrigatório; os outros grants não estão disponíveis.
admin-identity-configure = Configurar o emissor
admin-identity-new = Adicionar cliente de identidade
admin-identity-edit = Editar cliente
admin-identity-enabled = Ativado
admin-identity-disabled = Desativado
admin-identity-all = Todos
admin-identity-discovery = Abrir o documento de descoberta
admin-identity-search = Buscar clientes
admin-identity-status = Status
admin-identity-clients = Clientes de identidade
admin-identity-client = Identidade do cliente
admin-identity-client-id = ID do cliente
admin-identity-name = Nome
admin-identity-method = Autenticação do cliente
admin-identity-public = Cliente público (PKCE, sem segredo)
admin-identity-pkce = Todo cliente deve usar PKCE S256. Os segredos de clientes confidenciais são gerados aqui e exibidos uma única vez.
admin-identity-rotate-secret = Substituir o segredo do cliente existente
admin-identity-redirects = Destinos registrados
admin-identity-redirect-help = Um URI HTTPS exato por linha; HTTP em loopback numérico é permitido para aplicativos locais. Curingas e fragmentos são recusados.
admin-identity-callbacks = URIs de callback de autorização
admin-identity-logout-uris = URIs de redirecionamento após o logout
admin-identity-scopes = Escopos de identidade permitidos
admin-identity-actions = Ações
admin-identity-delete = Excluir cliente
admin-identity-delete-confirm = Excluir o acesso de identidade de { $name }? Os grants de identidade existentes deixam de funcionar e o registro de cliente somente de identidade é removido.
admin-identity-empty = Nenhum cliente de identidade correspondente.
admin-identity-cancel = Cancelar
admin-identity-save = Salvar
admin-identity-done = Concluído
admin-identity-secret = Segredo do cliente gerado
admin-identity-secret-once = Copie este segredo para o aplicativo dependente agora. Ele não poderá ser exibido novamente. Nunca coloque um segredo confidencial em código de navegador.
admin-identity-issuer = URL HTTPS pública do emissor
admin-identity-issuer-help = Use a origem HTTPS exata desta implantação e o caminho /oidc, incluindo qualquer prefixo da implantação. Alterar o emissor ou a configuração revoga os grants de identidade existentes.
admin-identity-rotate-key = Rotacionar a chave de assinatura
admin-identity-key-help = A rotação exige verificação recente do administrador. As chaves públicas anteriores permanecem disponíveis por 15 minutos para que os tokens emitidos recentemente concluam sua curta duração.
