# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contactos
contacts-search = Pesquisar contactos
contacts-new = Novo contacto
contacts-find-duplicates = Procurar duplicados
contacts-loading = A carregar…
contacts-none = Sem contactos.
contacts-list = Lista de contactos
contacts-favorite = Adicionar { $name } aos favoritos
contacts-export-vcard = Exportar vCard
contacts-export-csv = Exportar CSV
contacts-select-hint = Selecione um contacto para ver o respetivo cartão.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Livros de endereços e grupos
contacts-address-books = Livros de endereços
contacts-all = Todos os contactos
contacts-favorites = Favoritos
contacts-groups = Grupos
contacts-new-group = Novo grupo
contacts-new-group-name = Nome do novo grupo
contacts-group-name-placeholder = Nome do grupo
contacts-create = Criar
contacts-data = Dados
contacts-import = Importar…

# -- Business card -----------------------------------------------------------
contacts-card = Contacto { $name }
contacts-favorited = Nos favoritos
contacts-favorite-action = Favorito
contacts-email = E-mail
contacts-phone = Telefone
contacts-dates = Datas
contacts-groups-label = Grupos
contacts-notes = Notas
contacts-security-key = Chave de segurança
contacts-key-on-file = Existe uma chave/certificado guardado (apenas para visualização até haver suporte para PGP/S-MIME).
contacts-no-key = Nenhuma chave guardada.
contacts-directory-security = Segurança do diretório
contacts-member-of = Membro de
contacts-group-membership = Membros de { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Novo contacto
contacts-edit-contact = Editar contacto
contacts-full-name = Nome completo
contacts-given = Nome próprio
contacts-given-placeholder = Nome
contacts-surname = Apelido
contacts-surname-placeholder = Apelido
contacts-organization = Organização
contacts-job-title = Cargo
contacts-job-title-placeholder = Cargo
contacts-email-n = E-mail { $n }
contacts-email-label = Etiqueta do e-mail { $n }
contacts-email-label-placeholder = trabalho
contacts-remove-email = Remover e-mail { $n }
contacts-add-email = Adicionar e-mail
contacts-phone-n = Telefone { $n }
contacts-remove-phone = Remover telefone { $n }
contacts-add-phone = Adicionar telefone
contacts-security-key-label = Chave de segurança (marcador opaco)
contacts-security-key-placeholder = Chave PGP / certificado S-MIME (apenas visualização)
contacts-favorite-word = Favorito

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Importar contactos
contacts-imported = { $count ->
    [one] Foi importado { $count } contacto
   *[other] Foram importados { $count } contactos
}.
contacts-import-hint = Cole um vCard (.vcf) ou CSV, ou escolha um ficheiro. O formato é detetado automaticamente.
contacts-import-file = Importar ficheiro
contacts-paste = Colar vCard ou CSV
contacts-preview = Pré-visualizar
contacts-map-columns = Mapear colunas
contacts-col = Coluna
contacts-maps-to = Corresponde a
contacts-sample = Exemplo
contacts-map-column = Mapear coluna { $header }
contacts-preview-count = Pré-visualizar ({ $count })
contacts-import-preview = Pré-visualização da importação
contacts-no-name = (sem nome)
contacts-no-email = sem e-mail
contacts-import-n = Importar { $count ->
    [one] { $count } contacto
   *[other] { $count } contactos
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Unir duplicados
contacts-merge-review-hint = Reveja o cartão unido. Os outros cartões ficam marcados como eliminados (reversível).
contacts-merged-preview = Pré-visualização da união
contacts-emails = E-mails
contacts-phones = Telefones
contacts-merge-confirm = Unir contactos
contacts-no-duplicates = Não foram encontrados duplicados.
contacts-possible-duplicates = { $count } possíveis duplicados
contacts-review-merge = Rever união

contacts-editor-person = Nome e trabalho
contacts-editor-reach = E-mail e telefone
contacts-editor-other = Notas e preferências
contacts-editor-save-error = Não foi possível guardar o contacto. As suas alterações continuam aqui. Tente novamente.

contacts-group-editor-help = Crie um grupo num livro de endereços e, depois, adicione membros a partir dos respetivos cartões de contacto.
contacts-group-details = Detalhes do grupo
contacts-group-save-error = Não foi possível guardar o grupo. As suas alterações continuam aqui. Tente novamente.

contacts-edit-group = Editar grupo de contactos
contacts-edit-named-group = Editar grupo { $name }
contacts-group-name = Nome do grupo
contacts-group-edit-help = Atualize o nome deste grupo. O livro de endereços e os membros mantêm-se inalterados.

# -- Páginas de gestão (t39 R6) ------------------------------------------------
contacts-back-to-list = Voltar aos contactos
contacts-membership-title = Membros dos grupos
contacts-membership-help = Escolha um grupo e adicione ou remova os seus membros. As alterações são guardadas à medida que as faz.
contacts-membership-no-groups = Ainda não há grupos. Crie um para começar a adicionar membros.
contacts-membership-pick = Selecione um grupo para ver os seus membros.
contacts-membership-of = Membros de { $name }
contacts-membership-members = Membros
contacts-membership-add = Escreva um nome ou endereço para adicionar um membro
contacts-membership-add-group = Escreva o nome de um grupo para adicionar este contacto
contacts-membership-count = { $count ->
    [one] 1 membro
   *[other] { $count } membros
}
contacts-membership-failed = Não foi possível guardar algumas alterações aos membros. A lista mostra o que está guardado.
contacts-membership-manage = Gerir membros dos grupos
contacts-import-source = Origem
contacts-import-book = Livro de endereços
contacts-import-review = Revisão
contacts-import-empty = Escolha um ficheiro ou cole texto vCard ou CSV e selecione Pré-visualizar. Os contactos encontrados aparecem aqui e pode excluir qualquer um deles.
contacts-merge-sets = Possíveis duplicados
contacts-merge-set = Duplicados de { $name }
contacts-merge-keep = Cartão a manter
contacts-merge-done = Unido em { $name }.
