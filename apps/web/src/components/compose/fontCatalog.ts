/** Local composer choices. Bundled faces are served by Mailwoman, never a font CDN.
 * System entries are fallback stacks, not bundled proprietary font files.
 * Email recipients may use the fallback: outgoing messages do not embed these fonts.
 */
export const COMPOSER_FONTS = [
  { id: 'arial', label: 'Arial', stack: 'Arial, Helvetica, sans-serif', source: 'system' },
  { id: 'verdana', label: 'Verdana', stack: 'Verdana, Geneva, sans-serif', source: 'system' },
  { id: 'tahoma', label: 'Tahoma', stack: 'Tahoma, Geneva, sans-serif', source: 'system' },
  { id: 'trebuchet-ms', label: 'Trebuchet MS', stack: '"Trebuchet MS", Arial, sans-serif', source: 'system' },
  { id: 'georgia', label: 'Georgia', stack: 'Georgia, "Times New Roman", serif', source: 'system' },
  { id: 'times-new-roman', label: 'Times New Roman', stack: '"Times New Roman", Times, serif', source: 'system' },
  { id: 'courier-new', label: 'Courier New', stack: '"Courier New", Courier, monospace', source: 'system' },
  { id: 'jost', label: 'Jost', stack: 'Jost, Arial, sans-serif', source: 'bundled' },
  { id: 'inter', label: 'Inter', stack: '"Inter Variable", Inter, Arial, sans-serif', source: 'bundled' },
  { id: 'roboto', label: 'Roboto', stack: '"Roboto Variable", Roboto, Arial, sans-serif', source: 'bundled' },
  { id: 'open-sans', label: 'Open Sans', stack: '"Open Sans Variable", "Open Sans", Arial, sans-serif', source: 'bundled' },
  { id: 'newsreader', label: 'Newsreader', stack: '"Newsreader Variable", Newsreader, Georgia, serif', source: 'bundled' },
  { id: 'jetbrains-mono', label: 'JetBrains Mono', stack: '"JetBrains Mono Variable", "JetBrains Mono", "Courier New", monospace', source: 'bundled' },
  { id: 'source-sans-3', label: 'Source Sans 3', stack: '"Source Sans 3 Variable", "Source Sans 3", Arial, sans-serif', source: 'bundled' },
  { id: 'source-serif-4', label: 'Source Serif 4', stack: '"Source Serif 4 Variable", "Source Serif 4", Georgia, serif', source: 'bundled' },
  { id: 'lora', label: 'Lora', stack: '"Lora Variable", Lora, Georgia, serif', source: 'bundled' },
  { id: 'montserrat', label: 'Montserrat', stack: '"Montserrat Variable", Montserrat, Arial, sans-serif', source: 'bundled' },
  { id: 'nunito-sans', label: 'Nunito Sans', stack: '"Nunito Sans Variable", "Nunito Sans", Arial, sans-serif', source: 'bundled' },
  { id: 'merriweather', label: 'Merriweather', stack: '"Merriweather Variable", Merriweather, Georgia, serif', source: 'bundled' },
] as const;

export type ComposerFontId = (typeof COMPOSER_FONTS)[number]['id'];

export function composerFontStack(id: string): string | undefined {
  return COMPOSER_FONTS.find(font => font.id === id)?.stack;
}

/** Accept only a catalogue ID, legacy family label, or complete known stack.
 * No arbitrary CSS values, URLs, appended declarations, or unknown fallback lists.
 */
export function normalizeComposerFont(value: string): string | undefined {
  const normalized = value.trim().toLowerCase();
  return COMPOSER_FONTS.find(font =>
    font.id === normalized || font.label.toLowerCase() === normalized ||
    font.stack.toLowerCase() === normalized,
  )?.stack;
}
