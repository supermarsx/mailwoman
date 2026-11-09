# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = Pasta movida: { $name }
folder-dnd-moved-many = { $count } pastas movidas
folder-dnd-undo = Desfazer
folder-dnd-cancel = Cancelar
folder-dnd-handle = Arraste { $name } para movê-la
folder-dnd-locked-hint = A ordem das pastas é fixada pelo seu administrador.
folder-dnd-forced-hint = Fixada pelo seu administrador. Não pode ser movida nem desafixada.
folder-dnd-disabled-hint = Arrastar e soltar pastas está desativado.
folder-dnd-system-hint = As pastas do sistema não podem ser movidas.
folder-dnd-pinned-group = Fixadas
folder-dnd-top-level = Mover para o nível superior
folder-dnd-problem-disabled = A movimentação de pastas está desativada.
folder-dnd-problem-locked = A ordem das pastas é fixada pelo seu administrador.
folder-dnd-problem-self = Uma pasta não pode ser movida para dentro dela mesma.
folder-dnd-problem-descendant = Uma pasta não pode ser movida para dentro de uma de suas próprias subpastas.
folder-dnd-problem-system = As pastas do sistema não podem ser movidas para cá.
folder-dnd-problem-no-children = Essa pasta não pode conter subpastas.
folder-dnd-problem-nesting-off = Seu administrador não permite aninhar pastas.
folder-dnd-problem-depth = Isso aninharia as pastas em um nível profundo demais.
folder-dnd-problem-forced = Esta pasta está fixada pelo seu administrador.
folder-dnd-problem-unknown = Essa pasta não existe mais.
folder-dnd-problem-noop = A pasta já está lá.
folder-dnd-problem-collision = Já existe ali uma pasta chamada { $name }. Renomeie-a primeiro.
folder-dnd-problem-server = O servidor não moveu { $name }. Nada foi alterado.
folder-dnd-announce-up = { $name } movida para cima
folder-dnd-announce-down = { $name } movida para baixo
folder-dnd-announce-indent = { $name } movida para dentro da pasta acima
folder-dnd-announce-outdent = { $name } movida um nível para cima
folder-dnd-nest-title = Mover { $name } para cá?
folder-dnd-nest-title-many = Mover { $count } pastas para cá?
folder-dnd-nest-body = Escolha se isso altera a pasta no servidor de e-mail ou apenas a forma como este aplicativo a exibe.
folder-dnd-nest-legend = Como movê-la
folder-dnd-nest-local = Organizar somente neste aplicativo
folder-dnd-nest-local-help = Apenas uma exibição. A pasta continua onde está no servidor e em outros aplicativos de e-mail.
folder-dnd-nest-server = Mover no servidor
folder-dnd-nest-server-help = Altera a pasta em todos os lugares, em todos os aplicativos de e-mail que usam esta conta.
folder-dnd-nest-remember = Lembrar a minha escolha
folder-dnd-nest-confirm = Mover
folder-dnd-arranged-only = Organizada somente neste aplicativo
folder-dnd-arranged-only-help = O lugar dela no servidor é diferente: { $path }
folder-dnd-apply-server = Aplicar no servidor
folder-dnd-reset = Restaurar a ordem padrão
folder-dnd-reset-done = Ordem das pastas restaurada
folder-dnd-move-to = Mover { $name } para...
folder-dnd-layout-link = Layout das pastas
folder-dnd-reorder-enable = Reorganizar pastas arrastando
folder-dnd-nest-mode = Quando eu aninhar uma pasta
folder-dnd-nest-mode-ask = Perguntar
folder-dnd-nest-mode-server = Mover no servidor
folder-dnd-nest-mode-local = Organizar somente neste aplicativo
folder-dnd-new-placement = As novas pastas aparecem
folder-dnd-new-placement-end = Depois das pastas do mesmo nível
folder-dnd-new-placement-alphabetical = Em ordem alfabética
folder-dnd-handles = Mostrar alças de arrastar
folder-dnd-handles-hover = Ao passar o mouse
folder-dnd-handles-always = Sempre
folder-dnd-arrange-title = Organizar pastas
folder-dnd-arrange-help = Arraste uma pasta para reordená-la, solte-a sobre outra pasta para aninhá-la ou use Alt e as setas em uma pasta com foco.
folder-dnd-admin-locked = Definido pelo seu administrador
folder-dnd-arrange-menu = Mover para…
folder-star-add = Marcar pasta com estrela
folder-star-remove = Remover estrela da pasta
folder-star-starred = Com estrela
folder-star-toggle = Marcar { $name } com estrela
folder-star-only = Mostrar somente as com estrela
folder-star-only-help = Mostra somente as pastas com estrela e as pastas que as contêm.
folder-dnd-indent = Mover { $name } para dentro da pasta acima
folder-dnd-outdent = Mover { $name } um nível para cima
folder-dnd-more-actions = Organizar
folder-dnd-selected-move = Mover selecionadas para…
folder-dnd-locked-row = Bloqueada
folder-dnd-problem-pin = As pastas fixadas ficam acima das demais. Desafixe-a primeiro para movê-la para lá.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = Vazio…

folder-bar-label = Ações para as pastas selecionadas

folder-bar-mark-read = Marcar tudo como lido

folder-bar-pin = Fixar

folder-bar-star = Estrela

folder-bar-unpin = Liberar

folder-bar-unstar = Remover estrela

folder-dnd-handle-hint = Arraste para mover esta pasta

folder-pinned-group = Fixado

folder-pinned-unpin-zone = Solte aqui para soltar

settings-folders-bulk-delete-body = { $folders ->
        [0] { $folders } pastas (com suas subpastas) e as mensagens { $messages } nelas serão excluídas. Isto não pode ser desfeito.
        [one] { $folders } pasta e as mensagens { $messages } nela contidas serão excluídas. Isto não pode ser desfeito.
        [many] { $folders } pastas (com suas subpastas) e as mensagens { $messages } nelas serão excluídas. Isto não pode ser desfeito.
       *[other] { $folders } pastas (com suas subpastas) e as mensagens { $messages } nelas serão excluídas. Isto não pode ser desfeito.
    }

settings-folders-bulk-delete-confirm = { $count ->
        [0] Exclua { $count } pastas
        [one] Excluir { $count } pasta
        [many] Exclua { $count } pastas
       *[other] Exclua { $count } pastas
    }

settings-folders-bulk-delete-done = { $count ->
        [0] Excluiu { $count } pastas
        [one] { $count } pasta excluída
        [many] Excluiu { $count } pastas
       *[other] Excluiu { $count } pastas
    }

settings-folders-bulk-delete-failed = Nada foi excluído. Não excluído: { $names }

settings-folders-bulk-delete-list = Pastas para excluir

settings-folders-bulk-delete-none = Nenhuma das pastas selecionadas pode ser excluída (as pastas do sistema e as pastas que você não pode excluir são ignoradas).

settings-folders-bulk-delete-others = Também não excluído: { $names }

settings-folders-bulk-delete-partial = Pastas { $done } excluídas. Não excluído: { $names }

settings-folders-bulk-delete-partial-done = Pastas { $done } excluídas.

settings-folders-bulk-delete-refused-legalHold = { $count ->
        [0] Não excluído porque a conta está sob retenção legal ({ $count } pastas): { $names }
        [one] Não excluído porque a conta está sob retenção legal: { $names }
        [many] Não excluído porque a conta está sob retenção legal ({ $count } pastas): { $names }
       *[other] Não excluído porque a conta está sob retenção legal ({ $count } pastas): { $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
        [0] Não excluído porque o processamento da conta é restrito ({ $count } pastas): { $names }
        [one] Não excluído porque o processamento da conta é restrito: { $names }
        [many] Não excluído porque o processamento da conta é restrito ({ $count } pastas): { $names }
       *[other] Não excluído porque o processamento da conta é restrito ({ $count } pastas): { $names }
    }

settings-folders-bulk-delete-skipped = { $count } as pastas selecionadas são ignoradas porque não podem ser excluídas: { $names }

settings-folders-bulk-delete-subfolders = Exclua também as subpastas de cada pasta

settings-folders-bulk-delete-subfolders-off = Apenas as pastas selecionadas são excluídas. Uma pasta que ainda possui uma subpasta fora da seleção é mantida.

settings-folders-bulk-delete-title = { $count ->
        [0] Excluir { $count } pastas?
        [one] Excluir { $count } pasta?
        [many] Excluir { $count } pastas?
       *[other] Excluir { $count } pastas?
    }

settings-folders-bulk-empty-body = { $total } mensagens nas pastas { $folders } serão removidas: { $toTrash } serão movidas para a Lixeira e poderão ser desfeitas, { $permanent } serão excluídas definitivamente e não poderão ser recuperadas. As subpastas não são tocadas.

settings-folders-bulk-empty-confirm = Esvazie as pastas

settings-folders-bulk-empty-counting = Contando as mensagens em cada pasta…

settings-folders-bulk-empty-done = Esvaziou { $count } pastas, { $messages } mensagens

settings-folders-bulk-empty-fail-refused = o servidor recusou parte dela; { $remaining } mensagens permanecem

settings-folders-bulk-empty-fail-stopped = parou; { $remaining } mensagens permanecem

settings-folders-bulk-empty-fail-uncertain = o resultado do último lote é desconhecido; verifique a pasta

settings-folders-bulk-empty-list = Pastas para esvaziar

settings-folders-bulk-empty-partial = Pastas { $done } esvaziadas; { $failed } falhou. Veja a lista.

settings-folders-bulk-empty-permanent = { $count } mensagens são excluídas para sempre

settings-folders-bulk-empty-progress = { $done } de { $of } pastas concluídas, { $messages } mensagens processadas. Agora: { $name }

settings-folders-bulk-empty-skip-count-failed = não pôde ser contado, ignorado

settings-folders-bulk-empty-skip-empty = já vazio, ignorado

settings-folders-bulk-empty-skip-permanent-denied = seria excluído para sempre, o que o administrador desativou, ignorou

settings-folders-bulk-empty-skip-search = uma pesquisa salva, ignorada

settings-folders-bulk-empty-skip-unsupported = esta conta não pode esvaziá-la, ignorada

settings-folders-bulk-empty-title = { $count ->
        [0] Esvaziar { $count } pastas?
        [one] Esvaziar { $count } pasta?
        [many] Esvaziar { $count } pastas?
       *[other] Esvaziar { $count } pastas?
    }

settings-folders-bulk-empty-to-trash = { $count } mensagens são movidas para a Lixeira

settings-folders-bulk-empty-undo-label = { $count } pastas esvaziadas ({ $messages } mensagens)

settings-folders-delete-selected = Excluir…

settings-folders-select-user = Selecione todas as pastas do usuário

settings-folders-subscribe-failed = As assinaturas não puderam ser alteradas.

settings-folders-subscribe-partial = Pastas { $done } alteradas; { $failed } não pôde ser alterado.

settings-folders-subscribe-selected = Inscrever-se

settings-folders-subscribed = { $count ->
        [0] Inscrito em { $count } pastas
        [one] Inscrito em { $count } pasta
        [many] Inscrito em { $count } pastas
       *[other] Inscrito em { $count } pastas
    }

settings-folders-unsubscribe-selected = Cancelar inscrição

settings-folders-unsubscribed = { $count ->
        [0] Cancelou inscrição em { $count } pastas
        [one] Cancelada inscrição em { $count } pasta
        [many] Cancelou inscrição em { $count } pastas
       *[other] Cancelou inscrição em { $count } pastas
    }
