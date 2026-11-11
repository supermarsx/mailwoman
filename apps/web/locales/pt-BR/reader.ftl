# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-BR · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Exibição de mensagens
reader-settings-help = Escolha como as mensagens são exibidas. Os padrões do administrador valem até você escolher uma substituição. O conteúdo da mensagem permanece isolado do aplicativo.
reader-colorMode = Cores da mensagem
reader-colorMode-help = Automático adapta as cores das mensagens em temas escuros. Original preserva as cores do remetente; adaptável usa o tema ativo; inversão inverte as cores preservando as cores das imagens.
reader-fontSize = Tamanho da fonte de leitura (px)
reader-fontSize-help = Tamanho base da fonte do texto das mensagens, de 12 a 24 pixels. A formatação do remetente ainda pode definir o tamanho dos títulos.
reader-lineHeight = Altura da linha de leitura
reader-lineHeight-help = Espaço entre as linhas do texto das mensagens, de 1,2 a 2,2 vezes o tamanho da fonte.
reader-contentWidth = Largura de leitura
reader-contentWidth-help = Limite linhas longas para uma leitura confortável ou use todo o painel da mensagem.
reader-loading = Aparência do carregamento
reader-loading-help = Escolha linhas de espaço reservado discretas, um indicador de texto ou nenhum indicador visual de carregamento. Os leitores de tela ainda recebem o status de carregamento.
reader-loadingDelayMs = Atraso do indicador de carregamento (ms)
reader-loadingDelayMs-help = Evite que um indicador pisque em solicitações rápidas. Apenas o indicador é atrasado, nunca a mensagem.
reader-option-auto = Automático com o tema
reader-option-original = Cores originais da mensagem
reader-option-adaptive = Adaptar ao tema
reader-option-invert = Inverter as cores da mensagem
reader-option-comfortable = Confortável
reader-option-wide = Ampla
reader-option-full = Painel inteiro
reader-option-skeleton = Linhas de espaço reservado
reader-option-indicator = Indicador de texto
reader-option-none = Nenhum indicador visual
reader-inherit = Usar o padrão do administrador
reader-actionStyle = Faixa de ações da mensagem
reader-actionStyle-help = Ícones compactos deixam mais espaço para a mensagem. Cada ação mantém seu nome para o teclado e uma dica explicativa. Escolha rótulos visíveis se preferir texto ao lado dos ícones.
reader-option-icons = Ícones compactos
reader-option-labels = Ícones e rótulos
reader-zoom-group = Zoom da mensagem
reader-zoom-in = Aumentar zoom
reader-zoom-in-help = Deixa a mensagem maior. Somente a mensagem muda de tamanho.
reader-zoom-out = Diminuir zoom
reader-zoom-out-help = Deixa a mensagem menor. Somente a mensagem muda de tamanho.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Zoom da mensagem { $level }%. Redefinir para 100%
reader-zoom-reset-help = Redefinir a mensagem para 100%
reader-zoom-reset-menu = Redefinir zoom ({ $level }%)
reader-zoom = Zoom da mensagem (%)
reader-zoom-help = O tamanho com que as mensagens são abertas, de 50% a 200%. Somente o corpo da mensagem é ampliado, nunca o aplicativo, e a impressão sempre usa 100%. Os botões de zoom ao lado das ações da mensagem também o alteram.
reader-attachments-summary = { $count ->
    [one] 1 anexo
   *[other] { $count } anexos
 } · { $size }
reader-attachments-show = Mostrar anexos
reader-attachments-hide = Ocultar anexos
reader-attachmentsDensity = Densidade dos anexos
reader-attachmentsDensity-help = Compacta mostra botões de arquivo pequenos; confortável mostra pré-visualizações maiores quando expandida.
reader-option-compact = Compacta
reader-attachmentsCollapsed = Recolher anexos por padrão
reader-attachmentsCollapsed-help = Iniciar cada mensagem com um resumo compacto dos anexos. Você pode expandir essa mensagem sem alterar este padrão.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Texto e cores
reader-card-text-help = Tamanho, espaçamento, largura e cores do texto das mensagens.
reader-card-actions = Ações e anexos
reader-card-actions-help = A faixa de ações da mensagem e como os anexos são exibidos inicialmente.
reader-card-loading = Carregamento
reader-card-loading-help = O que aparece enquanto uma mensagem é aberta.
settings-mail-card-list = Lista de mensagens
settings-mail-card-list-help = Ações nas linhas e o que a lista faz quando você age sobre uma mensagem.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Baixar { $name }

reader-attachments-download-all = Baixar tudo

reader-attachments-download-all-blocked = A política de anexos bloqueia todos os arquivos aqui

reader-attachments-download-all-busy = Preparando o arquivo ZIP

reader-attachments-download-all-label = Baixe todos os anexos como um arquivo ZIP

reader-attachments-download-all-none = Não há anexos para download

reader-attachments-download-all-warned = { $count ->
        [0] { $count } arquivos que precisam de confirmação foram deixados de fora. Baixe-os um por um.
        [one] Um arquivo que precisa de confirmação foi deixado de fora. Faça o download sozinho.
        [many] { $count } arquivos que precisam de confirmação foram deixados de fora. Baixe-os um por um.
       *[other] { $count } arquivos que precisam de confirmação foram deixados de fora. Baixe-os um por um.
    }

reader-attachments-download-blocked = { $name } não pode ser baixado: { $reason }

reader-attachments-download-label = Baixar { $name } ({ $size })

reader-attachments-download-off = Os downloads estão desativados

reader-attachments-inline-group = { $count ->
        [0] { $count } imagens embutidas
        [one] { $count } imagem embutida
        [many] { $count } imagens embutidas
       *[other] { $count } imagens embutidas
    }

reader-attachments-inline-summary = { $count ->
        [0] { $count } imagens embutidas
        [one] { $count } imagem embutida
        [many] { $count } imagens embutidas
       *[other] { $count } imagens embutidas
    } · { $size }

reader-attachmentsBarClickToToggle = Clique na barra para expandir ou recolher

reader-attachmentsBarClickToToggle-help = Clicar em qualquer lugar da barra de anexos a alterna. Quando desativado, apenas o botão de seta funciona.

reader-attachmentsBarDownloadAll = Botão baixar tudo

reader-attachmentsBarDownloadAll-help = Mostre um botão na barra que baixa os anexos como um arquivo ZIP.

reader-attachmentsBarShowDownloadButtons = Botões de download em anexos

reader-attachmentsBarShowDownloadButtons-help = Mostre o botão de download de cada anexo ao passar o mouse ou focar, sempre ou nunca.

reader-attachmentsBarZipInlineImages = Incluir imagens embutidas no ZIP

reader-attachmentsBarZipInlineImages-help = As imagens que aparecem dentro do texto da mensagem são deixadas de fora de Baixar tudo, a menos que esta opção esteja ativada.

reader-card-attachments-bar = Barra de anexos

reader-card-attachments-bar-help = Como a barra acima dos anexos de uma mensagem responde e o que ela oferece. As linhas marcadas como bloqueadas são definidas pelo seu administrador.

reader-option-always = Sempre

reader-option-hover = Ao passar o mouse ou focar

reader-option-never = Nunca

reader-card-header = Cabeçalho da mensagem
reader-autoHideHeader = Ocultar automaticamente os cabeçalhos das mensagens
reader-autoHideHeader-help = Abrir cada mensagem com os detalhes do remetente, destinatários e metadados recolhidos. O assunto e as ações permanecem visíveis. Use Mostrar cabeçalho para ver os detalhes. Desativado por padrão.
reader-header-show = Mostrar cabeçalho
reader-header-hide = Ocultar cabeçalho
