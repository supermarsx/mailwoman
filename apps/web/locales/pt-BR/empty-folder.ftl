# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = Esvaziar pasta…
empty-folder-title = Esvaziar “{ $name }”?
empty-folder-body-trash = { $count ->
    [one] A mensagem em { $name } será movida para a Lixeira, onde você poderá restaurá-la.
   *[other] Todas as { $count } mensagens em { $name } serão movidas para a Lixeira, onde você poderá restaurá-las.
 } As subpastas e as mensagens nelas não são afetadas.
empty-folder-body-destroy-trash = { $count ->
    [one] A mensagem na Lixeira será excluída permanentemente. Isso não pode ser desfeito.
   *[other] Todas as { $count } mensagens na Lixeira serão excluídas permanentemente. Isso não pode ser desfeito.
 } As subpastas e as mensagens nelas não são afetadas.
empty-folder-body-destroy-junk = { $count ->
    [one] A mensagem em { $name } será excluída permanentemente. Isso não pode ser desfeito.
   *[other] Todas as { $count } mensagens em { $name } serão excluídas permanentemente. Isso não pode ser desfeito.
 } As subpastas e as mensagens nelas não são afetadas.
empty-folder-body-destroy-no-trash = Esta conta não tem pasta Lixeira, então { $count ->
    [one] a mensagem em { $name } será excluída permanentemente.
   *[other] todas as { $count } mensagens em { $name } serão excluídas permanentemente.
 } Isso não pode ser desfeito. As subpastas e as mensagens nelas não são afetadas.
empty-folder-confirm-trash = Mover para a Lixeira
empty-folder-confirm-destroy = Excluir permanentemente
empty-folder-cancel = Cancelar
empty-folder-already-empty = “{ $name }” já está vazia.
empty-folder-prepare-failed = Não foi possível contar as mensagens de “{ $name }”, então nada foi alterado. Verifique a conexão e tente novamente.
empty-folder-syncing = “{ $name }” ainda está sendo sincronizada, então ainda não pode ser contada. Nada foi alterado; tente novamente em instantes.
empty-folder-running = Uma pasta já está sendo esvaziada. Interrompa-a pela notificação antes de iniciar outra.
empty-folder-progress-trash = Movendo mensagens de { $name } para a Lixeira: { $done } de { $total }.
empty-folder-progress-destroy = Excluindo mensagens de { $name }: { $done } de { $total }.
empty-folder-stop = Parar
empty-folder-stopping = Parando após o lote atual. { $done } de { $total } concluídas até agora.
empty-folder-done-trash = { $done ->
    [one] 1 mensagem de { $name } movida para a Lixeira.
   *[other] { $done } mensagens de { $name } movidas para a Lixeira.
}
empty-folder-done-destroy = { $done ->
    [one] 1 mensagem de { $name } excluída permanentemente.
   *[other] { $done } mensagens de { $name } excluídas permanentemente.
}
empty-folder-stopped = Interrompido. { $done } de { $total } mensagens concluídas; { $remaining } ainda em { $name }.
empty-folder-refused = Interrompido. { $done } de { $total } mensagens concluídas; o servidor de e-mail recusou { $refused }. { $remaining } ainda em { $name }.
empty-folder-failed = O esvaziamento de { $name } parou com um erro após { $done } mensagens. O último lote pode ou não ter sido concluído; verifique a pasta antes de tentar novamente.
