# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: empty-folder. Source: ../en/empty-folder.ftl

# Empty a whole folder (t45): the folder menu, the folder tab and the Trash toolbar.
empty-folder-action = Esvaziar pasta…
empty-folder-title = Esvaziar «{ $name }»?
# Ordinary folder: the messages move to Trash.
empty-folder-body-trash = { $count ->
    [one] A mensagem em { $name } será movida para o Lixo, onde a poderá restaurar.
   *[other] As { $count } mensagens em { $name } serão movidas para o Lixo, onde as poderá restaurar.
} As subpastas e as respetivas mensagens não são alteradas.
# Trash itself: permanent.
empty-folder-body-destroy-trash = { $count ->
    [one] A mensagem no Lixo será eliminada permanentemente. Esta ação não pode ser anulada.
   *[other] As { $count } mensagens no Lixo serão eliminadas permanentemente. Esta ação não pode ser anulada.
} As subpastas e as respetivas mensagens não são alteradas.
# Junk / Spam: permanent.
empty-folder-body-destroy-junk = { $count ->
    [one] A mensagem em { $name } será eliminada permanentemente. Esta ação não pode ser anulada.
   *[other] As { $count } mensagens em { $name } serão eliminadas permanentemente. Esta ação não pode ser anulada.
} As subpastas e as respetivas mensagens não são alteradas.
# An ordinary folder in an account that has no Trash folder: permanent.
empty-folder-body-destroy-no-trash = Esta conta não tem pasta Lixo, pelo que { $count ->
    [one] a mensagem em { $name } será eliminada permanentemente.
   *[other] as { $count } mensagens em { $name } serão eliminadas permanentemente.
} Esta ação não pode ser anulada. As subpastas e as respetivas mensagens não são alteradas.
empty-folder-confirm-trash = Mover para o Lixo
empty-folder-confirm-destroy = Eliminar permanentemente
empty-folder-cancel = Cancelar
empty-folder-already-empty = «{ $name }» já está vazia.
empty-folder-prepare-failed = Não foi possível contar as mensagens de «{ $name }», pelo que nada foi alterado. Verifique a ligação e tente novamente.
empty-folder-syncing = «{ $name }» ainda está a ser sincronizada, pelo que não é possível contar as mensagens. Nada foi alterado; tente novamente daqui a pouco.
empty-folder-running = Já está a ser esvaziada uma pasta. Pare-a a partir da respetiva notificação antes de iniciar outra.
empty-folder-progress-trash = A mover mensagens de { $name } para o Lixo: { $done } de { $total }.
empty-folder-progress-destroy = A eliminar mensagens de { $name }: { $done } de { $total }.
empty-folder-stop = Parar
empty-folder-stopping = A parar após o lote atual. { $done } de { $total } concluídas até agora.
empty-folder-done-trash = { $done ->
    [one] 1 mensagem movida de { $name } para o Lixo.
   *[other] { $done } mensagens movidas de { $name } para o Lixo.
}
empty-folder-done-destroy = { $done ->
    [one] 1 mensagem de { $name } eliminada permanentemente.
   *[other] { $done } mensagens de { $name } eliminadas permanentemente.
}
empty-folder-stopped = Parado. { $done } de { $total } mensagens concluídas; { $remaining } ainda em { $name }.
empty-folder-refused = Parado. { $done } de { $total } mensagens concluídas; o servidor de correio recusou { $refused }. { $remaining } ainda em { $name }.
empty-folder-failed = O esvaziamento de { $name } parou com um erro após { $done } mensagens. O último lote pode ou não ter sido concluído; verifique a pasta antes de tentar novamente.
