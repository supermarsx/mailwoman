# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: about. Source: ../en/about.ftl

about-tp-label = Bibliotecas de terceiros e licenças
about-tp-search = Pesquisar por nome, versão ou licença
about-tp-filter = Filtrar por licença
about-tp-filter-all = Todas ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Nenhuma entrada corresponde.
        [one] Mostrando { $shown } de { $total } entrada.
       *[other] Mostrando { $shown } de { $total } entradas.
    }
about-tp-empty = Nada corresponde a esta pesquisa e licença.
about-tp-clear = Limpar pesquisa e filtro
about-tp-more = Mostrar mais ({ $n } restantes)

about-tp-kind-crate = Crate Rust
about-tp-kind-npm = Pacote npm
about-tp-kind-font = Fonte
about-tp-kind-icons = Ícones
about-tp-kind-image = Imagens
about-tp-kind-os = Pacote da imagem

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crates Rust no servidor, no app web e nos plugins
about-tp-group-shells = Crates Rust apenas nos apps para desktop e celular
about-tp-group-web = Pacotes npm no app web
about-tp-group-assets = Fontes, ícones e imagens
about-tp-group-image = Arquivos da imagem do servidor
about-tp-group-build = Ferramentas de build: executadas durante o build, não distribuídas

about-tp-licence = Licença:
about-tp-licence-none = não declarada
about-tp-licence-in-file = consulte o arquivo de copyright
about-tp-used-in = Usado em: { $parts }
about-tp-artifact-server = servidor
about-tp-artifact-web = app web
about-tp-artifact-plugins = plugins
about-tp-artifact-desktop = app para desktop
about-tp-artifact-mobile = app para celular
about-tp-note-bundled = Incluído no app web.
about-tp-note-fonts-pull = Não incluído: baixado pelo operador do servidor com `mailwoman fonts pull`.
about-tp-note-first-party = Feito para o Mailwoman, sob a licença do próprio projeto.
about-tp-note-static-libc = Vinculado aos binários do servidor da imagem FROM-scratch.
about-tp-note-alpine-ca = O pacote de certificados de CA da imagem FROM-scratch, obtido do Alpine Linux.
about-tp-in-images = Em: { $images }
about-tp-image-runtime = a imagem de runtime distroless
about-tp-image-static = a imagem FROM-scratch

about-tp-repository = Código-fonte
about-tp-homepage = Página inicial
about-tp-new-tab = de { $name } (abre em uma nova aba)

about-tp-text-toggle =
    { $count ->
        [one] Texto da licença
       *[other] Textos das licenças ({ $count })
    }
about-tp-text-open-for = Texto da licença de { $name }
about-tp-dialog-title = Texto da licença: { $name } { $version }
about-tp-close = Fechar
about-tp-text-loading = Carregando o texto da licença…
about-tp-text-error = Não foi possível carregar os textos das licenças.
about-tp-retry = Tentar novamente
about-tp-text-missing = Este texto está faltando na lista. Gere-o novamente com scripts/about/gen-third-party.mjs.
about-tp-text-label = Texto da licença de { $name }: { $file }
about-tp-standard = Licença { $licence }
about-tp-standard-note = (texto padrão, sem arquivo no pacote)
about-tp-no-text = Nenhum texto de licença disponível para esta entrada.
