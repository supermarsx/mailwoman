reader-settings-title = Message display
reader-settings-help = Choose how messages are displayed. Administrator defaults apply until you choose an override. Message content remains isolated from the application.
reader-colorMode = Message colors
reader-colorMode-help = Automatic adapts message colors in dark themes. Original preserves the sender's colors; adaptive uses the active theme; inversion reverses colors while preserving image colors.
reader-fontSize = Reading font size (px)
reader-fontSize-help = Base font size for message text, from 12 to 24 pixels. Sender formatting can still define heading sizes.
reader-lineHeight = Reading line height
reader-lineHeight-help = Space between lines of message text, from 1.2 to 2.2 times the font size.
reader-contentWidth = Reading width
reader-contentWidth-help = Limit long lines for comfortable reading or use the full message pane.
reader-loading = Loading appearance
reader-loading-help = Choose calm placeholder lines, a text indicator, or no visual loading indicator. Screen readers still receive loading status.
reader-loadingDelayMs = Loading indicator delay (ms)
reader-loadingDelayMs-help = Avoid flashing an indicator for fast requests. This delays only the indicator, never the message itself.
reader-option-auto = Automatic with theme
reader-option-original = Original message colors
reader-option-adaptive = Adapt to theme
reader-option-invert = Invert message colors
reader-option-comfortable = Comfortable
reader-option-wide = Wide
reader-option-full = Full pane
reader-option-skeleton = Placeholder lines
reader-option-indicator = Text indicator
reader-option-none = No visual indicator
reader-inherit = Use administrator default

reader-actionStyle = Message action ribbon
reader-actionStyle-help = Compact icons leave more room for the message. Every action keeps its keyboard name and an explanatory tooltip. Choose visible labels if you prefer text beside the icons.
reader-option-icons = Compact icons
reader-option-labels = Icons and labels

# t59: message-body zoom in the reader action row
reader-zoom-group = Message zoom
reader-zoom-in = Zoom in
reader-zoom-in-help = Make the message larger. Only the message changes size.
reader-zoom-out = Zoom out
reader-zoom-out-help = Make the message smaller. Only the message changes size.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Message zoom { $level }%. Reset to 100%
reader-zoom-reset-help = Reset the message to 100%
reader-zoom-reset-menu = Reset zoom ({ $level }%)
reader-zoom = Message zoom (%)
reader-zoom-help = The size messages open at, from 50% to 200%. Only the message body scales, never the app, and printing always uses 100%. The zoom buttons beside the message actions change it too.

reader-attachments-summary = { $count ->
    [one] 1 attachment
   *[other] { $count } attachments
} · { $size }
reader-attachments-inline-summary = { $count ->
    [one] 1 inline image
   *[other] { $count } inline images
} · { $size }
reader-attachments-inline-group = { $count ->
    [one] 1 inline image
   *[other] { $count } inline images
}
reader-attachments-show = Show attachments
reader-attachments-hide = Hide attachments
reader-attachmentsDensity = Attachment density
reader-attachmentsDensity-help = Compact shows small file buttons; comfortable shows larger previews when expanded.
reader-option-compact = Compact
reader-attachmentsCollapsed = Collapse attachments by default
reader-attachmentsCollapsed-help = Start each message with a compact attachment summary. You can expand that message without changing this default.

reader-card-text = Text and colors
reader-card-text-help = Size, spacing, width and colors of message text.
reader-card-actions = Actions and attachments
reader-card-actions-help = The message action ribbon and how attachments start.
reader-card-loading = Loading
reader-card-loading-help = What you see while a message opens.
settings-mail-card-list = Message list
settings-mail-card-list-help = Row actions and what the list does when you act on a message.
reader-attachments-download = Download { $name }
reader-attachments-download-label = Download { $name } ({ $size })
reader-attachments-download-blocked = { $name } cannot be downloaded: { $reason }
reader-attachments-download-off = Downloads are turned off
reader-attachments-download-all = Download all
reader-attachments-download-all-label = Download all attachments as a ZIP file
reader-attachments-download-all-busy = Preparing the ZIP file
reader-attachments-download-all-none = There are no attachments to download
reader-attachments-download-all-blocked = The attachment policy blocks every file here
reader-card-attachments-bar = Attachments bar
reader-card-attachments-bar-help = How the bar above a message's attachments responds and what it offers. Lines marked locked are set by your administrator.
reader-attachmentsBarClickToToggle = Click the bar to expand or collapse
reader-attachmentsBarClickToToggle-help = Clicking anywhere on the attachments bar toggles it. When off, only the arrow button does.
reader-attachmentsBarShowDownloadButtons = Download buttons on attachments
reader-attachmentsBarShowDownloadButtons-help = Show each attachment's download button on hover or focus, always, or never.
reader-attachmentsBarDownloadAll = Download all button
reader-attachmentsBarDownloadAll-help = Show a button on the bar that downloads the attachments as one ZIP file.
reader-attachmentsBarZipInlineImages = Include inline images in the ZIP
reader-attachmentsBarZipInlineImages-help = Images that appear inside the message text are left out of Download all unless this is on.
reader-option-always = Always
reader-option-hover = On hover or focus
reader-option-never = Never
reader-attachments-download-all-warned = { $count ->
    [one] One file that needs a confirmation was left out. Download it on its own.
   *[other] { $count } files that need a confirmation were left out. Download them one by one.
 }

reader-card-header = Message header
reader-autoHideHeader = Automatically hide message headers
reader-autoHideHeader-help = Open each message with sender, recipient and metadata details collapsed. The subject and actions stay visible. Use Show header to reveal the details. Off by default.
reader-header-show = Show header
reader-header-hide = Hide header
