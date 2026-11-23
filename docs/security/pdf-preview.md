# PDF attachment preview

PDF attachments open in Mozilla's PDF.js viewer, served by your Mailwoman server from
`apps/web/public/pdfjs` (no third-party CDN). This page describes how the viewer is
isolated from the rest of the app and what a hostile PDF can and cannot do.

<!-- screenshot: id=security/pdf-preview; screen=Mailbox > message with a PDF attachment > preview; state=sample PDF open, toolbar visible; element=whole viewer -->

## How the bytes reach the viewer

1. The app checks that the attachment URL is one of its own attachment URLs, then
   downloads the bytes with the signed-in session, through the same authorization
   checks as any other attachment.
2. It loads the viewer page `pdfjs/web/mailwoman.html` in an iframe sandboxed with
   `allow-scripts allow-downloads allow-modals` and **without** `allow-same-origin`,
   with `referrerpolicy="no-referrer"`. The frame therefore runs in an opaque origin:
   it cannot read the app's cookies, storage or DOM, and it is never given a session
   token or a document URL.
3. The bytes go to the frame by `postMessage`. Each opening uses a fresh random nonce;
   the frame accepts messages only from its parent window, pins the origin of the
   first message it receives, and ignores messages without the current nonce. The app
   in turn accepts messages only from that frame with the current nonce.

The viewer page strips its own query string and does not load documents from URL
parameters. It carries its own Content Security Policy:

```
default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' data:;
style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; font-src 'self' blob: data:;
connect-src 'self' blob: data:; media-src blob:; frame-src 'none'; object-src 'none';
base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'; trusted-types default
```

## What is turned off

The viewer starts with these PDF.js options: `enableScripting: false` (no PDF
JavaScript), `isEvalSupported: false`, `enableXfa: false` (no XFA forms), alt-text
generation and model download off, signature editor off, preferences and history off.
A PDF's own print action (OpenAction) cannot open a print dialog; printing works only
from the toolbar or keyboard.

Links inside a PDF do not navigate the frame. The app shows the destination and asks
you to confirm (**Open link**) before opening it. Files embedded in the PDF are saved
locally rather than opened.

## Download and print

**Download** and **Print** in the viewer follow
[`mail.allowDownload`](/admin/settings#mail.allowdownload) and
[`mail.allowPrint`](/admin/settings#mail.allowprint) (both default on) and the PDF
tools chosen in [`reader.filePreviews`](/admin/settings#reader.filepreviews). When the
viewer saves a file it sends the bytes to the app, which re-checks the download
permission and validates the payload, MIME type and file name before starting a local
download.

::: warning What these switches do not do
Turning off download or print hides those actions in the app. The server refuses
explicit save-to-disk downloads when `mail.allowDownload` is off, but the preview
itself has to deliver the PDF's bytes to the browser, so a user who can preview a file
can still keep a copy (for example through the browser's developer tools or a
screenshot). Treat these settings as a policy signal, not as copy protection.
:::

## Limits and failure handling

- The viewer code loads only when a PDF is opened. Closing or replacing the attachment
  aborts the download, closes the document and removes the frame.
- If the frame does not answer, or does not finish opening the document, within 45
  seconds, the preview fails with an error instead of waiting forever. A password
  prompt stops that timer.
- The app passes its theme to the viewer as a fixed set of colour and font values;
  these restyle the viewer's toolbars and dialogs, not the PDF's pages.
- PDF.js parses untrusted input. A bug in PDF.js could still affect the viewer frame;
  the sandbox and CSP limit what code running there can reach, but they are not proof
  against browser bugs.

## Vendored files

`apps/web/public/pdfjs/manifest.json` records the upstream release archive, its
SHA-256, and a hash for each vendored file. The PDF scripting bundle, source maps,
the example PDF and the debugger are not included. To reproduce or verify the files:

```sh
python apps/web/scripts/vendor-pdf-viewer.py          # fetch the pinned archive and rebuild
python apps/web/scripts/vendor-pdf-viewer.py --check  # verify without fetching or writing
```

Local integration code lives in `apps/web/public/pdfjs/bootstrap.mjs`,
`apps/web/public/pdfjs/mailwoman.css` and `apps/web/src/viewers/PdfViewer.tsx`.

## Related

- [Attachment preview settings](/deploy/attachment-preview)
- [Settings reference](/admin/settings)
