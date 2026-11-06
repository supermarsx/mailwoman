# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Exclusão e pseudonimização
compliance-card-erasure-coverage = Substitua os identificadores de uma pessoa no histórico de auditoria e nos logs e veja quais backups ainda guardam pessoas que foram excluídas.
admin-erasure-title = Exclusão e pseudonimização
admin-erasure-intro = Pseudonimizar pessoa substitui o endereço e os IDs de conta de uma pessoa no histórico de auditoria e nos arquivos de log rotacionados por um pseudônimo estável com chave. A cadeia de hashes da auditoria é selada novamente para que a verificação continue passando. Toda exclusão e pseudonimização também deixa uma tombstone, de modo que a restauração de um backup mais antigo exclui essas pessoas novamente.
admin-erasure-docs = O registro do que é excluído, exportado e mantido em cada tabela está em docs/admin/erasure-coverage.md.
admin-erasure-scrub = Pseudonimizar pessoa
admin-erasure-scrub-note = Execute a pré-visualização primeiro: ela conta as linhas e as linhas de log que mudariam e não altera nada.
admin-erasure-address = Endereço
admin-erasure-preview = Pré-visualizar
admin-erasure-run = Pseudonimizar pessoa
admin-erasure-dry-note = Somente pré-visualização. Nada foi alterado.
admin-erasure-done-note = Pseudonimizada. A pessoa agora aparece como { $pseudonym }.
admin-erasure-nothing = Nenhum histórico cita esta pessoa.
admin-erasure-col-table = Tabela ou arquivo
admin-erasure-col-rows = Linhas
admin-erasure-confirm-title = Pseudonimizar esta pessoa no histórico?
admin-erasure-confirm-body = As entradas de auditoria e as linhas de log que citam a pessoa são reescritas com o pseudônimo. Isso não pode ser desfeito; o endereço não pode ser recuperado a partir do pseudônimo sem a chave.
admin-erasure-settings = Configurações
admin-erasure-keys = Chaves de pseudônimo
admin-erasure-keys-note = Os pseudônimos são criados com a geração de chave atual. A rotação adiciona uma geração; as mais antigas permanecem até que a retenção definida acima as remova.
admin-erasure-rotate = Rotacionar chave
admin-erasure-keys-none = Nenhuma chave foi criada ainda. A primeira pseudonimização cria uma.
admin-erasure-col-gen = Geração
admin-erasure-col-created = Criada
admin-erasure-col-state = Estado
admin-erasure-key-current = Atual
admin-erasure-key-retired = Aposentada, mantida
admin-erasure-key-destroy = Aposentada, excluída em { $when }
admin-erasure-chain = Cadeia de auditoria
admin-erasure-chain-note = Cada entrada de auditoria guarda o hash da anterior. Uma pseudonimização reescreve entradas, sela a cadeia novamente e registra o hash de topo antigo e o novo.
admin-erasure-seal = Selar novas entradas
admin-erasure-chain-ok = A cadeia é verificada: { $rows } entradas seladas, { $unsealed } aguardando selagem.
admin-erasure-chain-bad = A cadeia não é verificada; primeira entrada inválida na posição { $seq }.
admin-erasure-col-time = Hora
admin-erasure-col-range = Posições
admin-erasure-backups = Backups
admin-erasure-backups-note = Um backup registra a marca d'água das tombstones quando é criado. A marca d'água atual é { $watermark }.
admin-erasure-col-label = Rótulo
admin-erasure-col-location = Local
admin-erasure-col-watermark = Marca d'água
admin-erasure-col-marker = Excluídos desde
admin-erasure-col-actions = Ações
admin-erasure-backups-none = Nenhum backup registrado.
admin-erasure-contains-erased = Contém pessoas excluídas ({ $count })
admin-erasure-no-marker = Nenhum
admin-erasure-forget = Esquecer
admin-erasure-register = Registrar backup
