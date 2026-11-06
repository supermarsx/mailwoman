# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Administração completa de todas as permissões, incluindo administradores, funções, segurança, roteamento de e-mail e configuração de serviços. Reserve esta função para operadores de confiança responsáveis por toda a implantação.
admin-role-purpose-domain_admin = Administração diária de usuários e domínios: criar, atualizar, desativar ou excluir usuários; gerenciar cotas, recursos dos usuários, regras de login do domínio, configurações e identidade visual; revogar sessões e gerenciar bloqueios. Inclui também diagnósticos de e-mail e exportações de uso e de dados. Não concede o gerenciamento de funções de administrador nem alterações em servidores upstream.
admin-role-purpose-helpdesk = Suporte a contas e triagem de incidentes: inspecionar usuários, atividade de auditoria e uso; alterar sinalizadores de usuários, desativar contas e revogar sessões; ver bloqueios e executar simulações de login. Não concede a criação ou exclusão de usuários, alterações de cota nem gravação das configurações gerais.
admin-role-purpose-auditor = Inspeção e relatórios: concede todas as permissões que terminam em .read, além da exportação de auditoria e de uso. Não concede alterações de configuração nem de contas. Os relatórios exportados podem conter informações administrativas sensíveis.
admin-role-purpose-security_officer = Operações de segurança: gerenciar as políticas de segurança e MFA, a política de login e os bloqueios; desativar usuários, revogar sessões ou chaves de API; inspecionar e exportar a atividade de auditoria; ler os logs de autenticação e executar diagnósticos. A configuração de SSO pode ser lida, mas esta função não concede a edição do SSO nem o gerenciamento de funções de administrador.
admin-role-purpose-custom = Uma função personalizada com { $count } permissões selecionadas. A autoridade dela vem da lista exata de permissões abaixo, não do nome. Revise cada ação antes de atribuí-la.
admin-role-purpose-scope = O escopo de domínio é atribuído quando esta função é concedida a um administrador. Exceções de permitir/negar no nível da conta podem alterar o acesso efetivo. Uma descrição não concede permissões adicionais.
admin-role-purpose-editor = Escolha as tarefas que este administrador precisa executar e selecione as permissões correspondentes. Acesso de leitura, exportação de dados e alteração de registros são ações separadas. Revise os identificadores exatos das permissões antes de salvar.
admin-role-purpose-selection = Estas descrições explicam as ações selecionadas. Elas não ampliam as permissões enviadas ao servidor.
admin-role-purpose-read = Ver informações desta área.
admin-role-purpose-export = Exportar informações desta área.
admin-role-purpose-revoke = Revogar as sessões ou credenciais indicadas por esta permissão.
admin-role-purpose-delete = Excluir os registros abrangidos por esta permissão.
admin-role-purpose-disable = Desativar as contas abrangidas por esta permissão.
admin-role-purpose-create = Criar os registros abrangidos por esta permissão.
admin-role-purpose-specific = Executar a ação específica indicada por esta permissão; revise o identificador antes de concedê-la.
admin-role-purpose-description = Finalidade e responsabilidades
admin-role-purpose-description-help = Explique quem deve receber esta função personalizada e as tarefas que essa pessoa realiza (até 1024 caracteres). Este texto não altera as permissões.
