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
| `format.stylesRibbon` | The Styles group: tiles, the all-styles dropdown, style sets, Reset to Normal | Ctrl+Alt+0 to 6, Ctrl+Shift+N |
| `format.customStyles` | Create style from selection, Update style, keeping your own styles | |
| `format.fontDialog` | The Font dialog (family, size, colour, effects, underline style, spacing, live preview, Set as default) | Ctrl+D |
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

## Styles

The Styles group has four tiles with live previews (Normal, No spacing, Heading 1, Heading 2), an All styles dropdown and three buttons: Create style from selection, Update style to match selection and Reset to Normal. The dropdown lists Normal, No spacing, Heading 1 to 6, Title, Subtitle, Strong, Emphasis, Intense emphasis, Quote, Intense quote, List paragraph, Task list item, Code (inline), Code and Caption, then the four style sets (Default, Modern, Classic, Compact) and the user's own styles.

A style is written as plain tags and inline CSS: `h1` to `h6`, `blockquote`, `pre` or `p`, with `margin-top`/`margin-bottom`/`line-height`/`text-align`/`margin-left` on the block and a `span` with `font-size`, `font-family`, `color`, `font-weight` and `font-style` (or the rich editor's `strong`/`em`) around the text. A Task list item is a checklist (see above). No classes and no editor attributes are sent.

| Setting | What it does |
| --- | --- |
| `compose.styles.defaultSet` | The style set a user starts with (`default`, `modern`, `classic`, `compact`). Users switch set in the gallery; an administrator can lock it |
| `compose.styles.maxCustom` | Most custom styles a user keeps (0 to 100, default 20). 0 turns Create style off. Updating a built-in style stores an override that does not count |
| `compose.styles.custom` | The user's styles as JSON, written by the gallery |

The Font dialog's Set as default writes `compose.layout.defaultFont` (family|size in px|colour), the setting the Layout tab uses.

## Not built

- Fallbacks for clients that ignore an effect. A nested-span pair for double strikethrough would draw a different line in the clients that support it, and a small caps fallback of smaller uppercase spans would change the typed text, so both stay as one declaration that such clients show as plain text.
- `-webkit-text-stroke` for the outline. It is not in the `mw-sanitize` allow-list (`crates/mw-sanitize/src/lib.rs`, `is_allowed_property`). Adding it needs that allow-list extended with a test and the wasm rebuilt.
- `font-variant-ligatures` is not in the allow-list either, so the ligature effect uses `font-variant: no-common-ligatures`.
- Style shortcuts in the native editor use the style set from the settings, not a set picked in the gallery during the same session.
