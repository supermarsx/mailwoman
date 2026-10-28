# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliotecas e licenças de terceiros
about-tp-search = Pesquisar por nome, versão ou licença
about-tp-filter = Filtrar por licença
about-tp-filter-all = Todas ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Nenhuma entrada corresponde.
        [one] A mostrar { $shown } de { $total } entrada.
       *[other] A mostrar { $shown } de { $total } entradas.
    }
about-tp-empty = Nada corresponde a esta pesquisa e licença.
about-tp-clear = Limpar pesquisa e filtro
about-tp-more = Mostrar mais (faltam { $n })

about-tp-kind-crate = Crate Rust
about-tp-kind-npm = Pacote npm
about-tp-kind-font = Tipo de letra
about-tp-kind-icons = Ícones
about-tp-kind-image = Imagens
about-tp-kind-os = Pacote da imagem

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crates Rust no servidor, na aplicação web e nos plugins
about-tp-group-shells = Crates Rust apenas nas aplicações de computador e móveis
about-tp-group-web = Pacotes npm na aplicação web
about-tp-group-assets = Tipos de letra, ícones e imagens
about-tp-group-image = Ficheiros da imagem do servidor
about-tp-group-build = Ferramentas de compilação: executadas durante a compilação, não distribuídas

about-tp-licence = Licença:
about-tp-licence-none = não declarada
about-tp-licence-in-file = consulte o ficheiro de direitos de autor
about-tp-used-in = Utilizado em: { $parts }
about-tp-artifact-server = servidor
about-tp-artifact-web = aplicação web
about-tp-artifact-plugins = plugins
about-tp-artifact-desktop = aplicação de computador
about-tp-artifact-mobile = aplicação móvel
about-tp-note-bundled = Incluído na aplicação web.
about-tp-note-fonts-pull = Não incluído: transferido pelo operador do servidor com `mailwoman fonts pull`.
about-tp-note-first-party = Criado para o Mailwoman, sob a licença do próprio projeto.
about-tp-note-static-libc = Ligado aos binários do servidor da imagem FROM-scratch.
about-tp-note-alpine-ca = O pacote de certificados de AC da imagem FROM-scratch, obtido do Alpine Linux.
about-tp-in-images = Em: { $images }
about-tp-image-runtime = a imagem de execução distroless
about-tp-image-static = a imagem FROM-scratch

about-tp-repository = Código-fonte
about-tp-homepage = Página inicial
about-tp-new-tab = de { $name } (abre num novo separador)

about-tp-text-toggle =
    { $count ->
        [one] Texto da licença
       *[other] Textos das licenças ({ $count })
    }
about-tp-text-open-for = Texto da licença de { $name }
about-tp-dialog-title = Texto da licença: { $name } { $version }
about-tp-close = Fechar
about-tp-text-loading = A carregar o texto da licença…
about-tp-text-error = Não foi possível carregar os textos das licenças.
about-tp-retry = Tentar novamente
about-tp-text-missing = Este texto não consta da lista. Volte a gerá-lo com scripts/about/gen-third-party.mjs.
about-tp-text-label = Texto da licença de { $name }: { $file }
about-tp-standard = Licença { $licence }
about-tp-standard-note = (texto padrão, sem ficheiro no pacote)
about-tp-no-text = Não existe texto de licença disponível para esta entrada.
