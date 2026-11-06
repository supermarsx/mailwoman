# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contatos
contacts-search = Pesquisar contatos
contacts-new = Novo contato
contacts-find-duplicates = Encontrar duplicados
contacts-loading = Carregando…
contacts-none = Nenhum contato.
contacts-list = Lista de contatos
contacts-favorite = Favoritar { $name }
contacts-export-vcard = Exportar vCard
contacts-export-csv = Exportar CSV
contacts-select-hint = Selecione um contato para ver o cartão.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Catálogos de endereços e grupos
contacts-address-books = Catálogos de endereços
contacts-all = Todos os contatos
contacts-favorites = Favoritos
contacts-groups = Grupos
contacts-new-group = Novo grupo
contacts-new-group-name = Nome do novo grupo
contacts-group-name-placeholder = Nome do grupo
contacts-create = Criar
contacts-data = Dados
contacts-import = Importar…

# -- Business card -----------------------------------------------------------
contacts-card = Contato { $name }
contacts-favorited = Favoritado
contacts-favorite-action = Favoritar
contacts-email = E-mail
contacts-phone = Telefone
contacts-dates = Datas
contacts-groups-label = Grupos
contacts-notes = Notas
contacts-security-key = Chave de segurança
contacts-key-on-file = Há uma chave/certificado registrado (somente exibição até o suporte a PGP/S-MIME chegar).
contacts-no-key = Nenhuma chave registrada.
contacts-directory-security = Segurança do diretório
contacts-member-of = Membro de
contacts-group-membership = Membros de { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Novo contato
contacts-edit-contact = Editar contato
contacts-full-name = Nome completo
contacts-given = Nome
contacts-given-placeholder = Nome
contacts-surname = Sobrenome
contacts-surname-placeholder = Sobrenome
contacts-organization = Organização
contacts-job-title = Cargo
contacts-job-title-placeholder = Cargo
contacts-email-n = E-mail { $n }
contacts-email-label = Rótulo do e-mail { $n }
contacts-email-label-placeholder = trabalho
contacts-remove-email = Remover e-mail { $n }
contacts-add-email = Adicionar e-mail
contacts-phone-n = Telefone { $n }
contacts-remove-phone = Remover telefone { $n }
contacts-add-phone = Adicionar telefone
contacts-security-key-label = Chave de segurança (espaço reservado opaco)
contacts-security-key-placeholder = Chave PGP / certificado S/MIME (somente exibição)
contacts-favorite-word = Favorito

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importar contatos
contacts-imported = { $count ->
    [one] { $count } contato importado
   *[other] { $count } contatos importados
}.
contacts-import-hint = Cole um vCard (.vcf) ou CSV, ou escolha um arquivo. O formato é detectado automaticamente.
contacts-import-file = Importar arquivo
contacts-paste = Colar vCard ou CSV
contacts-preview = Pré-visualizar
contacts-map-columns = Mapear colunas
contacts-col = Coluna
contacts-maps-to = Corresponde a
contacts-sample = Exemplo
contacts-map-column = Mapear coluna { $header }
contacts-preview-count = Pré-visualização ({ $count })
contacts-import-preview = Pré-visualização da importação
contacts-no-name = (sem nome)
contacts-no-email = sem e-mail
contacts-import-n = Importar { $count ->
    [one] { $count } contato
   *[other] { $count } contatos
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Mesclar duplicados
contacts-merge-review-hint = Revise o cartão mesclado. Os outros cartões viram marcadores de exclusão (reversível).
contacts-merged-preview = Pré-visualização mesclada
contacts-emails = E-mails
contacts-phones = Telefones
contacts-merge-confirm = Mesclar contatos
contacts-no-duplicates = Nenhum duplicado encontrado.
contacts-possible-duplicates = { $count } possíveis duplicados
contacts-review-merge = Revisar mesclagem

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Detalhes do contato
contacts-address-recipients = Mostrar destinatários
contacts-address-save = Salvar contato
contacts-address-compose = Enviar e-mail
contacts-address-unsaved = Este endereço não está salvo nos seus contatos.
contacts-address-book = Agenda de endereços
contacts-address-lookup-failed = Não foi possível carregar as informações do contato. Tente novamente.
contacts-address-save-failed = Não foi possível salvar o contato. Tente novamente.
contacts-address-saved = Contato salvo.
contacts-address-loading = Carregando contato…
contacts-editor-person = Nome e trabalho
contacts-editor-reach = E-mail e telefone
contacts-editor-other = Notas e preferências
contacts-editor-save-error = Não foi possível salvar o contato. Suas alterações continuam aqui. Tente novamente.
contacts-group-editor-help = Crie um grupo em uma agenda de endereços e depois adicione membros a partir dos cartões de contato.
contacts-group-details = Detalhes do grupo
contacts-group-save-error = Não foi possível salvar o grupo. Suas alterações continuam aqui. Tente novamente.
contacts-edit-group = Editar grupo de contatos
contacts-edit-named-group = Editar grupo { $name }
contacts-group-name = Nome do grupo
contacts-group-edit-help = Atualize o nome deste grupo. A agenda de endereços e os membros permanecem inalterados.
contacts-back-to-list = Voltar aos contatos
contacts-membership-title = Participação no grupo
contacts-membership-help = Escolha um grupo e adicione ou remova membros. As alterações são salvas conforme você as faz.
contacts-membership-no-groups = Ainda não há grupos. Crie um para começar a adicionar membros.
contacts-membership-pick = Selecione um grupo para ver os membros.
contacts-membership-of = Membros de { $name }
contacts-membership-members = Membros
contacts-membership-add = Digite um nome ou endereço para adicionar um membro
contacts-membership-add-group = Digite o nome de um grupo para adicionar este contato
contacts-membership-count = { $count ->
    [one] 1 membro
   *[other] { $count } membros
 }
contacts-membership-failed = Algumas alterações de participação não puderam ser salvas. A lista mostra o que está armazenado.
contacts-membership-manage = Gerenciar participação no grupo
contacts-import-source = Origem
contacts-import-book = Agenda de endereços
contacts-import-review = Revisão
contacts-import-empty = Escolha um arquivo ou cole texto vCard ou CSV e selecione Pré-visualizar. Os contatos encontrados aparecem aqui, e você pode deixar de fora qualquer um deles.
contacts-merge-sets = Possíveis duplicados
contacts-merge-set = Duplicados de { $name }
contacts-merge-keep = Cartão a manter
contacts-merge-done = Mesclado em { $name }.
contacts-address-actions = Ações do contato
contacts-address-save-to = Salvar nos contatos
contacts-address-goto = Ir para o contato
contacts-address-copy = Copiar informações do contato
contacts-address-copied = Informações do contato copiadas.
contacts-address-copy-failed = Não foi possível copiar as informações do contato.
contacts-share = Compartilhar contato
contacts-share-email = Enviar contato por e-mail
contacts-share-copy = Copiar como vCard
contacts-share-download = Baixar .vcf
contacts-share-native = Compartilhar…
contacts-share-copied = vCard copiado.
contacts-share-copy-failed = Não foi possível copiar o vCard.
contacts-share-downloaded = vCard baixado.
contacts-share-failed = Não foi possível compartilhar o contato.
contacts-share-attach-failed = Não foi possível anexar o arquivo .vcf, então o vCard está no texto da mensagem.
contacts-address-saved-favorite = Contato salvo e adicionado aos favoritos.
contacts-favorite-add = Adicionar aos favoritos
contacts-favorite-remove = Remover dos favoritos
contacts-favorite-save = Salvar e adicionar aos favoritos
contacts-favorite-added = { $name } adicionado aos favoritos.
contacts-favorite-removed = { $name } removido dos favoritos.
contacts-favorite-failed = Não foi possível atualizar os favoritos.
contacts-select-row = Selecionar { $name }
contacts-selected-count = { $count } selecionados
contacts-select-clear = Limpar seleção
contacts-bulk-actions = Ações para os contatos selecionados
contacts-bulk-favorites-added = { $count } adicionados aos favoritos.
contacts-bulk-favorites-removed = { $count } removidos dos favoritos.
contacts-row-menu = Ações para { $name }
contacts-favorites-empty = Ainda não há favoritos. Use a estrela em um contato para adicionar um.
