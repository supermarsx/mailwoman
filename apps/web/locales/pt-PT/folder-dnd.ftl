# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: folder-dnd. Source: ../en/folder-dnd.ftl

# Organização das pastas por arrastar e largar (barra lateral e Definições > Pastas e barra lateral).
folder-dnd-moved = Pasta movida: { $name }
folder-dnd-moved-many = { $count } pastas movidas
folder-dnd-undo = Anular
folder-dnd-cancel = Cancelar
folder-dnd-handle = Arraste { $name } para a mover
folder-dnd-locked-hint = A ordem das pastas está fixada pelo administrador.
folder-dnd-forced-hint = Fixada pelo administrador. Não pode ser movida nem desafixada.
folder-dnd-disabled-hint = Arrastar e largar pastas está desligado.
folder-dnd-system-hint = As pastas do sistema não podem ser movidas.
folder-dnd-pinned-group = Fixadas
folder-dnd-top-level = Mover para o nível superior
folder-dnd-problem-disabled = Mover pastas está desligado.
folder-dnd-problem-locked = A ordem das pastas está fixada pelo administrador.
folder-dnd-problem-self = Uma pasta não pode ser movida para dentro de si própria.
folder-dnd-problem-descendant = Uma pasta não pode ser movida para dentro de uma das suas subpastas.
folder-dnd-problem-system = As pastas do sistema não podem ser movidas para aqui.
folder-dnd-problem-no-children = Essa pasta não pode conter subpastas.
folder-dnd-problem-nesting-off = O administrador não permite aninhar pastas.
folder-dnd-problem-depth = Isso aninharia as pastas demasiado fundo.
folder-dnd-problem-forced = Esta pasta está fixada pelo administrador.
folder-dnd-problem-unknown = Essa pasta já não existe.
folder-dnd-problem-noop = A pasta já está aí.
folder-dnd-problem-collision = Já existe aí uma pasta chamada { $name }. Mude-lhe primeiro o nome.
folder-dnd-problem-server = O servidor não moveu { $name }. Nada foi alterado.
folder-dnd-announce-up = { $name } movida para cima
folder-dnd-announce-down = { $name } movida para baixo
folder-dnd-announce-indent = { $name } movida para dentro da pasta acima
folder-dnd-announce-outdent = { $name } subiu um nível
folder-dnd-nest-title = Mover { $name } para aqui?
folder-dnd-nest-title-many = Mover { $count } pastas para aqui?
folder-dnd-nest-body = Escolha se isto altera a pasta no servidor de correio ou apenas a forma como esta aplicação a mostra.
folder-dnd-nest-legend = Como a mover
folder-dnd-nest-local = Apenas organizar nesta aplicação
folder-dnd-nest-local-help = Só na vista. A pasta fica onde está no servidor e noutras aplicações de correio.
folder-dnd-nest-server = Mover no servidor
folder-dnd-nest-server-help = Altera a pasta em todo o lado, em todas as aplicações de correio que usam esta conta.
folder-dnd-nest-remember = Lembrar a minha escolha
folder-dnd-nest-confirm = Mover
folder-dnd-arranged-only = Organizada apenas nesta aplicação
folder-dnd-arranged-only-help = O seu lugar no servidor é diferente: { $path }
folder-dnd-apply-server = Aplicar no servidor
folder-dnd-reset = Repor a ordem predefinida
folder-dnd-reset-done = Ordem das pastas reposta
folder-dnd-move-to = Mover { $name } para...
folder-dnd-layout-link = Disposição das pastas
folder-dnd-reorder-enable = Reorganizar pastas arrastando
folder-dnd-nest-mode = Quando aninho uma pasta
folder-dnd-nest-mode-ask = Perguntar-me
folder-dnd-nest-mode-server = Mover no servidor
folder-dnd-nest-mode-local = Apenas organizar nesta aplicação
folder-dnd-new-placement = As pastas novas aparecem
folder-dnd-new-placement-end = Depois das irmãs
folder-dnd-new-placement-alphabetical = Por ordem alfabética
folder-dnd-handles = Mostrar pegas de arrasto
folder-dnd-handles-hover = Ao passar o rato
folder-dnd-handles-always = Sempre
folder-dnd-arrange-title = Organizar pastas
folder-dnd-arrange-help = Arraste uma pasta para a reordenar, largue-a sobre outra para a aninhar, ou use Alt e as setas numa pasta com foco.
folder-dnd-admin-locked = Definido pelo administrador
folder-dnd-arrange-menu = Mover para…
folder-star-add = Marcar pasta com estrela
folder-star-remove = Retirar estrela da pasta
folder-star-starred = Com estrela
folder-star-toggle = Marcar { $name } com estrela
folder-star-only = Mostrar só pastas com estrela
folder-star-only-help = Mostra apenas as pastas com estrela e as pastas que as contêm.
folder-dnd-indent = Mover { $name } para dentro da pasta acima
folder-dnd-outdent = Subir { $name } um nível
folder-dnd-more-actions = Organizar
folder-dnd-selected-move = Mover selecionadas para…
folder-dnd-locked-row = Bloqueada
folder-dnd-problem-pin = As pastas fixadas ficam acima das outras. Desafixe-a primeiro para a mover para aí.

folder-dnd-handle-hint = Arraste para mover esta pasta
folder-pinned-group = Fixadas
folder-pinned-unpin-zone = Largue aqui para desafixar

settings-folders-subscribe-selected = Subscrever
settings-folders-unsubscribe-selected = Cancelar subscrição
settings-folders-delete-selected = Eliminar…
settings-folders-subscribed = { $count ->
    [one] Subscreveu { $count } pasta
   *[other] Subscreveu { $count } pastas
}
settings-folders-unsubscribed = { $count ->
    [one] Cancelou a subscrição de { $count } pasta
   *[other] Cancelou a subscrição de { $count } pastas
}
settings-folders-subscribe-partial = Alteradas { $done } pastas; { $failed } não puderam ser alteradas.
settings-folders-subscribe-failed = Não foi possível alterar as subscrições.
settings-folders-bulk-delete-none = Nenhuma das pastas selecionadas pode ser eliminada (as pastas do sistema e as que não pode eliminar são ignoradas).
settings-folders-bulk-delete-title = { $count ->
    [one] Eliminar { $count } pasta?
   *[other] Eliminar { $count } pastas?
}
settings-folders-bulk-delete-body = { $folders ->
    [one] { $folders } pasta e as { $messages } mensagens que contém serão eliminadas. Isto não pode ser anulado.
   *[other] { $folders } pastas (com as subpastas) e as { $messages } mensagens que contêm serão eliminadas. Isto não pode ser anulado.
}
settings-folders-bulk-delete-confirm = { $count ->
    [one] Eliminar { $count } pasta
   *[other] Eliminar { $count } pastas
}
settings-folders-bulk-delete-list = Pastas a eliminar
settings-folders-bulk-delete-skipped = { $count } pastas selecionadas foram ignoradas porque não podem ser eliminadas: { $names }
settings-folders-bulk-delete-done = { $count ->
    [one] Eliminou { $count } pasta
   *[other] Eliminou { $count } pastas
}
settings-folders-bulk-delete-partial = Eliminadas { $done } pastas. Não eliminadas: { $names }
settings-folders-bulk-delete-failed = Nada foi eliminado. Não eliminadas: { $names }

# Barra de seleção e esvaziar em lote
folder-bar-label = Ações para as pastas selecionadas
folder-bar-empty = Esvaziar…
folder-bar-mark-read = Marcar tudo como lido
folder-bar-pin = Fixar
folder-bar-unpin = Desafixar
folder-bar-star = Marcar com estrela
folder-bar-unstar = Retirar a estrela
settings-folders-bulk-empty-title = { $count ->
    [one] Esvaziar { $count } pasta?
   *[other] Esvaziar { $count } pastas?
}
settings-folders-bulk-empty-counting = A contar as mensagens de cada pasta…
settings-folders-bulk-empty-body = { $total } mensagens serão removidas. { $permanent ->
    [0] Nada é eliminado definitivamente: as mensagens vão primeiro para o Lixo.
   *[other] { $permanent } delas são eliminadas definitivamente e não podem ser recuperadas.
} As subpastas não são afetadas.
settings-folders-bulk-empty-confirm = Esvaziar as pastas
settings-folders-bulk-empty-list = Pastas a esvaziar
settings-folders-bulk-empty-to-trash = { $count } mensagens vão para o Lixo
settings-folders-bulk-empty-permanent = { $count } mensagens são eliminadas definitivamente
settings-folders-bulk-empty-progress = { $done } de { $of } pastas concluídas, { $messages } mensagens processadas. Agora: { $name }
settings-folders-bulk-empty-skip-search = pesquisa guardada, ignorada
settings-folders-bulk-empty-skip-unsupported = esta conta não a pode esvaziar, ignorada
settings-folders-bulk-empty-skip-empty = já está vazia, ignorada
settings-folders-bulk-empty-skip-count-failed = não foi possível contar, ignorada
settings-folders-bulk-empty-fail-refused = o servidor recusou parte; restam { $remaining } mensagens
settings-folders-bulk-empty-fail-stopped = interrompido; restam { $remaining } mensagens
settings-folders-bulk-empty-fail-uncertain = o resultado do último lote é desconhecido; verifique a pasta
settings-folders-bulk-empty-done = Esvaziou { $count } pastas, { $messages } mensagens
settings-folders-bulk-empty-partial = Esvaziadas { $done } pastas; { $failed } falharam. Veja a lista.

settings-folders-select-user = Selecionar todas as pastas do utilizador

settings-folders-bulk-delete-subfolders = Eliminar também as subpastas de cada pasta
settings-folders-bulk-delete-subfolders-off = Só as pastas selecionadas são eliminadas. Uma pasta que ainda tenha uma subpasta fora da seleção é mantida.
