# Composer Format tab features

Every control of the composer's Format tab is a feature an administrator or a user can switch off. Each has a setting `compose.feature.<id>` (Boolean, on by default, user and admin scope, with an admin lock and default). The list lives in `apps/web/src/components/compose/composeFeatures.ts`; Settings and Admin render their cards from it. A switched-off feature is not drawn and its keyboard shortcut does nothing.

Both visual editors (the rich text editor and the native HTML editor) honour the same flags.

## Format features

| Id | What it controls | Shortcut |
| --- | --- | --- |
| `format.fontSizeStep` | Increase and decrease font size buttons (standard sizes 8 to 72 pt; Shift steps 1 pt) | Ctrl+Shift+> and Ctrl+Shift+<, Ctrl+] and Ctrl+[ for 1 pt |
| `format.lists` | The bulleted and numbered list buttons | |
| `format.listTypes` | The dropdown on each list button: bullet shapes, numbering styles, checklist, outline and 1.1.1 numbering, restart, continue, set start value | |
| `format.indent` | Increase and decrease indent (40 px steps; list items change level) | Tab and Shift+Tab in a list |
| `format.clearFormatting` | Clear formatting and its menu (all, text only, paragraph only, everything including links) | Ctrl+Space, Ctrl+\ |
| `format.painter` | The Format Painter group: Copy formatting, Apply formatting, the chip. Double-click Copy formatting to paint several selections until Esc | |
| `format.lineSpacing` | Line spacing (1.0, 1.15, 1.5, 2.0, custom) and space before and after | |
| `format.direction` | Left-to-right and right-to-left buttons | |
| `format.pasteOptions` | Keep formatting, merge formatting, text only | |
| `format.strikethrough` | The strikethrough button | |
| `format.doubleStrikethrough`, `format.superscript`, `format.subscript`, `format.smallCaps`, `format.allCaps`, `format.underlineStyle`, `format.ligatures`, `format.textScale`, `format.textOutline` | The entries of the Text effects menu | Ctrl+= subscript, Ctrl+Shift+= superscript |
| `format.stylesRibbon` | The Styles group | Ctrl+Alt+0 to 4 |
| `format.sort` | The Sort control | |

## What goes out in the message

The sent HTML is plain: inline `style` attributes and legacy tags, no classes and no editor attributes.

| Feature | Written as |
| --- | --- |
| Bullet shapes disc, circle, square | `<ul style="list-style-type: square">` |
| Numbering 1. a. A. i. I. | `<ol style="list-style-type: lower-alpha" type="a">` (the `type` attribute is for Outlook) |
| Start value | `<ol start="4">` |
| Dash, arrow, tick, checklist, `1)`, `01.`, `1.1.1` | `list-style-type: none` and the marker as text in front of each item (`<span>1.1&nbsp;</span>`). A checklist uses the glyphs U+2610 and U+2611 |
| Indent | `margin-left: 40px` per step, or a nested list for a list item |
| Text direction | `dir="rtl"` or `dir="ltr"` on the paragraph |
| Line spacing and space before and after | `line-height`, `margin-top`, `margin-bottom` |

## Text effects and mail clients

| Effect | Written as | Support |
| --- | --- | --- |
| Superscript, subscript | `<sup>`, `<sub>` | Works in most mail apps |
| All caps | `text-transform: uppercase` (the typed text is not changed) | Works in most mail apps |
| Character spacing | `letter-spacing: -0.05em` or `0.1em` | Works in most mail apps |
| Small caps | `font-variant: small-caps` | Some apps ignore this |
| Double strikethrough | `text-decoration: line-through; text-decoration-style: double` | Some apps ignore this: they show a single line |
| Underline style | `text-decoration: underline; text-decoration-style: double, dotted, dashed or wavy`; thick uses `text-decoration-thickness: 2px` | Some apps ignore this: they show a single underline |
| Ligatures off | `font-variant: no-common-ligatures` | Some apps ignore this |
| Text scale 50 to 200% | `font-stretch` with a matching `letter-spacing` | Some apps ignore this. There is no mail-safe character width scaling; the letter spacing is the part that shows |
| Outline | Four 1 px `text-shadow` copies in the chosen colour | Some apps ignore this: they show plain text. `-webkit-text-stroke` is not used because the sanitiser does not allow it |
| Shadow | `text-shadow: 1px 1px 2px` | Some apps ignore this |

The Text effects menu shows this support level next to each effect. The sanitiser (`mw-sanitize`, used by the server and by the editor in the browser) already allows each of these properties, and `apps/web/src/components/compose/NativeFormatTab.test.tsx` checks that they survive it.

## Not built

The Font dialog launcher (Ctrl+D), custom and user-defined styles, style sets and the settings `compose.styles.maxCustom` and `compose.styles.defaultSet` are not part of this release. The Styles group is the existing gallery (Normal, headings 1 to 6, Title, Subtitle, No spacing, Quote, Code).
