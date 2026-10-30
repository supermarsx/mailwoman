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
reader-attachments-show = Show attachments
reader-attachments-hide = Hide attachments
