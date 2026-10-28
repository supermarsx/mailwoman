# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl

admin-role-purpose-super_admin = Administração completa de todas as capacidades, inclusive administradores, funções, segurança, encaminhamento de correio e configuração de serviços. Reserve esta função para operadores fidedignos responsáveis por toda a implementação.
admin-role-purpose-domain_admin = Administração diária de utilizadores e domínios: criar, atualizar, desativar ou eliminar utilizadores; gerir quotas, capacidades dos utilizadores, regras de início de sessão do domínio, definições e personalização da marca; revogar sessões e gerir banimentos. Inclui também diagnósticos de correio e exportações de utilização e de dados. Não concede a gestão de funções de administrador nem alterações a servidores a montante.
admin-role-purpose-helpdesk = Apoio a contas e triagem de incidentes: inspecionar utilizadores, atividade de auditoria e utilização; alterar sinalizadores de utilizadores, desativar contas e revogar sessões; ver banimentos e executar simulações de início de sessão. Não concede a criação ou eliminação de utilizadores, alterações de quotas nem escrita de definições gerais.
admin-role-purpose-auditor = Inspeção e relatórios: concede todas as capacidades terminadas em .read, mais a exportação de auditoria e de utilização. Não concede alterações de configuração nem modificações de contas. Os relatórios exportados podem conter informações administrativas sensíveis.
admin-role-purpose-security_officer = Operações de segurança: gerir as políticas de segurança e MFA, a política de início de sessão e os banimentos; desativar utilizadores, revogar sessões ou chaves de API; inspecionar e exportar a atividade de auditoria; ler os registos de autenticação e executar diagnósticos. A configuração SSO pode ser lida, mas esta função não concede a edição do SSO nem a gestão de funções de administrador.
admin-role-purpose-custom = Uma função personalizada com { $count } permissões selecionadas. A sua autoridade provém da lista exata de permissões abaixo, não do nome. Reveja cada ação antes de a atribuir.
admin-role-purpose-scope = O âmbito de domínio é atribuído quando esta função é concedida a um administrador. As substituições de permissão/recusa ao nível da conta podem alterar o acesso efetivo. Uma descrição não concede permissões adicionais.
admin-role-purpose-editor = Escolha as tarefas que este administrador tem de realizar e, em seguida, selecione as permissões correspondentes. O acesso de leitura, a exportação de dados e a alteração de registos são ações separadas. Reveja os identificadores exatos das capacidades antes de guardar.
admin-role-purpose-selection = Estas descrições explicam as ações selecionadas. Não alargam as permissões enviadas para o servidor.
admin-role-purpose-read = Ver informações nesta área.
admin-role-purpose-export = Exportar informações desta área.
admin-role-purpose-revoke = Revogar as sessões ou credenciais indicadas por esta permissão.
admin-role-purpose-delete = Eliminar os registos abrangidos por esta permissão.
admin-role-purpose-disable = Desativar as contas abrangidas por esta permissão.
admin-role-purpose-create = Criar registos abrangidos por esta permissão.
admin-role-purpose-specific = Executar a ação específica indicada por esta capacidade; reveja o respetivo identificador antes de a conceder.
admin-role-purpose-description = Finalidade e responsabilidades
admin-role-purpose-description-help = Explique quem deve receber esta função personalizada e as tarefas que desempenha (até 1024 carateres). Este texto não altera as respetivas permissões.
