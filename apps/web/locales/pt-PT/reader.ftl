# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pt-PT · Module: reader. Source: ../en/reader.ftl

reader-settings-title = Apresentação das mensagens
reader-settings-help = Escolha como as mensagens são apresentadas. As predefinições do administrador aplicam-se até escolher uma alternativa. O conteúdo das mensagens permanece isolado da aplicação.
reader-colorMode = Cores das mensagens
reader-colorMode-help = «Automático» adapta as cores das mensagens nos temas escuros. «Original» preserva as cores do remetente; «adaptar» utiliza o tema ativo; «inverter» inverte as cores, mas preserva as cores das imagens.
reader-fontSize = Tamanho da letra de leitura (px)
reader-fontSize-help = Tamanho de letra base do texto das mensagens, de 12 a 24 píxeis. A formatação do remetente pode ainda definir o tamanho dos títulos.
reader-lineHeight = Altura de linha de leitura
reader-lineHeight-help = Espaço entre as linhas do texto das mensagens, de 1,2 a 2,2 vezes o tamanho da letra.
reader-contentWidth = Largura de leitura
reader-contentWidth-help = Limite as linhas longas para uma leitura confortável ou utilize todo o painel da mensagem.
reader-loading = Aspeto do carregamento
reader-loading-help = Escolha linhas de marcador discretas, um indicador de texto ou nenhum indicador visual de carregamento. Os leitores de ecrã continuam a receber o estado do carregamento.
reader-loadingDelayMs = Atraso do indicador de carregamento (ms)
reader-loadingDelayMs-help = Evita que um indicador pisque em pedidos rápidos. Isto atrasa apenas o indicador, nunca a própria mensagem.
reader-option-auto = Automático com o tema
reader-option-original = Cores originais da mensagem
reader-option-adaptive = Adaptar ao tema
reader-option-invert = Inverter as cores da mensagem
reader-option-comfortable = Confortável
reader-option-wide = Larga
reader-option-full = Painel inteiro
reader-option-skeleton = Linhas de marcador
reader-option-indicator = Indicador de texto
reader-option-none = Sem indicador visual
reader-inherit = Utilizar a predefinição do administrador

reader-actionStyle = Friso de ações da mensagem
reader-actionStyle-help = Os ícones compactos deixam mais espaço para a mensagem. Cada ação mantém o seu nome para o teclado e uma descrição explicativa. Escolha etiquetas visíveis se preferir texto junto aos ícones.
reader-option-icons = Ícones compactos
reader-option-labels = Ícones e etiquetas

# t59: zoom do corpo da mensagem na barra de ações do leitor
reader-zoom-group = Zoom da mensagem
reader-zoom-in = Ampliar
reader-zoom-in-help = Aumentar a mensagem. Só a mensagem muda de tamanho.
reader-zoom-out = Reduzir
reader-zoom-out-help = Diminuir a mensagem. Só a mensagem muda de tamanho.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Zoom da mensagem { $level }%. Repor 100%
reader-zoom-reset-help = Repor a mensagem a 100%
reader-zoom-reset-menu = Repor zoom ({ $level }%)
reader-zoom = Zoom das mensagens (%)
reader-zoom-help = O tamanho com que as mensagens abrem, de 50% a 200%. Só o corpo da mensagem muda de escala, nunca a aplicação, e a impressão usa sempre 100%. Os botões de zoom junto às ações da mensagem também o alteram.

reader-attachments-summary = { $count ->
    [one] 1 anexo
   *[other] { $count } anexos
} · { $size }
reader-attachments-inline-summary = { $count ->
    [one] 1 imagem incorporada
   *[other] { $count } imagens incorporadas
} · { $size }
reader-attachments-inline-group = { $count ->
    [one] 1 imagem incorporada
   *[other] { $count } imagens incorporadas
}
reader-attachments-show = Mostrar anexos
reader-attachments-hide = Ocultar anexos
reader-attachmentsDensity = Densidade dos anexos
reader-attachmentsDensity-help = O modo compacto mostra pequenos botões de ficheiros; o confortável mostra pré-visualizações maiores quando expandido.
reader-option-compact = Compacto
reader-attachmentsCollapsed = Recolher anexos por predefinição
reader-attachmentsCollapsed-help = Iniciar cada mensagem com um resumo compacto dos anexos. Pode expandir essa mensagem sem alterar a predefinição.

reader-card-text = Texto e cores
reader-card-text-help = Tamanho, espaçamento, largura e cores do texto das mensagens.
reader-card-actions = Ações e anexos
reader-card-actions-help = A barra de ações da mensagem e como os anexos começam.
reader-card-loading = A carregar
reader-card-loading-help = O que vê enquanto uma mensagem abre.
settings-mail-card-list = Lista de mensagens
settings-mail-card-list-help = Ações das linhas e o que a lista faz quando age sobre uma mensagem.
reader-attachments-download = Transferir { $name }
reader-attachments-download-label = Transferir { $name } ({ $size })
reader-attachments-download-blocked = Não é possível transferir { $name }: { $reason }
reader-attachments-download-off = As transferências estão desativadas
reader-attachments-download-all = Transferir tudo
reader-attachments-download-all-label = Transferir todos os anexos num ficheiro ZIP
reader-attachments-download-all-busy = A preparar o ficheiro ZIP
reader-attachments-download-all-none = Não há anexos para transferir
reader-attachments-download-all-blocked = A política de anexos bloqueia todos os ficheiros aqui
reader-card-attachments-bar = Barra de anexos
reader-card-attachments-bar-help = Como a barra por cima dos anexos de uma mensagem responde e o que oferece. As linhas bloqueadas são definidas pelo administrador.
reader-attachmentsBarClickToToggle = Clicar na barra para expandir ou recolher
reader-attachmentsBarClickToToggle-help = Clicar em qualquer ponto da barra de anexos alterna-a. Se estiver desligado, só o botão da seta o faz.
reader-attachmentsBarShowDownloadButtons = Botões de transferência nos anexos
reader-attachmentsBarShowDownloadButtons-help = Mostrar o botão de transferência de cada anexo ao passar o rato ou com foco, sempre, ou nunca.
reader-attachmentsBarDownloadAll = Botão Transferir tudo
reader-attachmentsBarDownloadAll-help = Mostrar na barra um botão que transfere os anexos num único ficheiro ZIP.
reader-attachmentsBarZipInlineImages = Incluir imagens em linha no ZIP
reader-attachmentsBarZipInlineImages-help = As imagens que aparecem dentro do texto da mensagem ficam de fora de Transferir tudo, a menos que isto esteja ligado.
reader-option-always = Sempre
reader-option-hover = Ao passar o rato ou com foco
reader-option-never = Nunca
reader-attachments-download-all-warned = { $count ->
    [one] Um ficheiro que exige confirmação ficou de fora. Transfira-o individualmente.
   *[other] { $count } ficheiros que exigem confirmação ficaram de fora. Transfira-os um a um.
 }
