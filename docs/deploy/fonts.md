---
title: Fonts
---

# Fonts

The Mailwoman web client brings its own typefaces and never loads a font from another site, so it runs under a content security policy that only allows fonts from its own origin. This page lists the fonts that are built in, explains how people pick them, and documents the command `mailwoman fonts pull`, which downloads Google Fonts files for self-hosting.

## The fonts that are built in

Fonts are packaged into the web client when it is built. Nothing is fetched at run time from a font service.

| Font | Where it appears | Licence file served with the client |
| --- | --- | --- |
| Jost | The default interface typeface, and the message editor | Packaged with the source (`apps/web/src/assets/fonts/jost`) |
| Inter, Roboto, Open Sans | Interface font choices of the same name, and the message editor | `/fonts/licenses/<name>.txt` |
| Newsreader, JetBrains Mono | The interface choices `serif` and `mono`, and the message editor | `/fonts/licenses/<name>.txt` |
| Manrope | An interface font choice and the message editor | `/font-licenses/manrope.txt` |
| Source Sans 3, Source Serif 4, Lora, Montserrat, Nunito Sans, Merriweather | The message editor only | `/font-licenses/<name>.txt` |

The message editor also offers Arial, Verdana, Tahoma, Trebuchet MS, Georgia, Times New Roman and Courier New. Mailwoman does not ship these. They are the visitor's own system fonts, and the editor writes a fallback list such as `Arial, Helvetica, sans-serif`.

::: warning
A message does not carry its fonts. The recipient's mail program shows the font if it has it, and otherwise one from the fallback list that the editor wrote into the message.
:::

## Who chooses what

| Setting | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Interface font** ([`appearance.font`](/admin/settings#appearance.font)) | The typeface of the web client: `default`, `system`, `serif`, `mono`, `inter`, `roboto`, `open-sans`, `jost` or `manrope` | `default` | Administrators globally and per domain, and users, unless locked |
| **Default message font** ([`compose.layout.defaultFont`](/admin/settings#compose.layout.defaultfont)) | Font, size in pixels and colour for new text in the message editor, written as `family\|size\|colour` | empty | Administrators globally and per domain, and users, unless locked |

## Download fonts with `mailwoman fonts pull`

The command fetches a stylesheet from Google Fonts, downloads each font file in it, and writes them to a folder together with a stylesheet whose `url()` entries point at your own path. It keeps Google's per-script subsets, so a browser only downloads the scripts a page uses. It does not edit the files, so it needs no font tools.

```sh
mailwoman fonts pull "Inter:wght@400;500;600;700" \
  --out fonts \
  --url-prefix /fonts \
  --css-name fonts.css
```

| Argument | What it does | Default |
| --- | --- | --- |
| `<family>...` | One or more Google Fonts specifications, such as `Inter:wght@400;700` or `Fraunces:ital@1`. At least one is required | none |
| `--out <dir>` | Folder for the font files and the stylesheet. It is created if needed | `fonts` |
| `--url-prefix <path>` | Path written into the stylesheet in front of each file name | `/fonts` |
| `--css-name <name>` | File name of the stylesheet inside `--out` | `fonts.css` |
| `--text "<characters>"` | Restricts the download to these characters | all characters |

Each font file is named `<family>-<subset>-<weight>-<style>.woff2`, for example `inter-latin-400-normal.woff2`. When it finishes the command prints `pulled N font face(s)`, the path of the stylesheet and the number of files.

The command downloads from `fonts.googleapis.com` directly. It ignores `HTTP_PROXY` and similar variables, so the machine that runs it needs a direct connection.

### The output is not used automatically

The web client does not load the stylesheet that this command writes. Its typefaces come from the build, as described above. When `MW_WEB_DIR` is set, Mailwoman serves the files in that folder, so you can put the output there, for example under `fonts/`, but a page or stylesheet of your own has to refer to `fonts.css` for the fonts to be used.

## Troubleshooting

- **`fetching Google Fonts stylesheet` or `downloading <address>` followed by a network error.** The machine has no direct route to Google, or a firewall blocks it. A proxy variable does not help, the command ignores it. Run it on a machine with access and copy the folder.
- **`no @font-face rules in the fetched stylesheet`.** Google answered without any font. Check the family specification, for example the spelling of the family and of axis lists such as `wght@400;700`.
- **A status error such as `400 Bad Request`.** The specification is not one that Google accepts.

## Related

- [Branding](/admin/branding) for the name, logo and icons
- [Reverse proxy](/deploy/reverse-proxy) for serving the application behind a path prefix
- [Composer features](/admin/composer-features) for the editor controls people can use
