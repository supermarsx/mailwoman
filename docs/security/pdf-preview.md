# PDF attachment preview

PDF attachments use the complete, self-hosted Mozilla PDF.js generic viewer,
pinned to version 6.1.200. The viewer includes page navigation, search, zoom and
fit modes, rotation, scrolling and spread modes, presentation, thumbnails,
outline, embedded attachments, optional-content layers, document properties,
text selection, supported forms and annotation editors, download/save and print.
Tools that require unavailable document features are handled by PDF.js itself.
Experimental signature editing and online AI features are disabled.

The application fetches an attachment through the existing authorization and
attachment-policy checks before previewing it. The viewer receives those bytes
through an origin-, frame- and nonce-bound channel; it does not receive session
tokens or arbitrary document URLs. Its fixed entry document ignores file/query
parameters and uses a separate restrictive CSP. PDF scripting, XFA, dynamic
evaluation, remote AI/model downloads and PDF-triggered automatic printing are
disabled. External links require the application's destination confirmation;
embedded attachments download locally. Manual printing remains available.

The viewer is loaded only when opening a PDF. PDF.js's rendering queue retains
a viewport-sized page cache rather than allocating a canvas for every page.
Closing or replacing the attachment aborts the parent byte fetch, closes the
document and destroys the frame/worker. A failed or blocked bootstrap has a
finite deadline; password entry remains interactive without a typing deadline.
The active app theme is passed as a restricted set of color/font tokens and
does not restyle the PDF's document content.

The bridge also follows live changes to the app's appearance, accent, UI font,
direction and surface/text/control colors, including viewer dialogs and menus.
An explicit light or dark app appearance takes precedence over browser defaults.

PDF.js Save sends the current PDF bytes through the frame-, instance- and
nonce-bound bridge. The parent checks the current download permission and
validates the byte payload, MIME type and filename before starting a local
download. This avoids opaque-origin frame downloads failing in Firefox and
preserves saved form fields and annotations. Object URLs are released after
download initiation or when the viewer closes.

Upstream files, all required font/CMap/codec licenses and localization catalogs
live in `apps/web/public/pdfjs`. `manifest.json` records archive provenance and
individual asset hashes. The PDF scripting/QuickJS bundles, source maps, example
PDF and debugger are excluded. The existing root-served worker is verified to
match the pinned viewer exactly.

To reproduce the assets, run `python apps/web/scripts/vendor-pdf-viewer.py`.
An archive path may be supplied for offline reproduction; its pinned archive
SHA-256 is verified identically. Run
`python apps/web/scripts/vendor-pdf-viewer.py --check` to verify the upstream
files, matching worker and generated app entry without fetching or rewriting
anything. Local integration code is kept separately in `bootstrap.mjs`,
`mailwoman.css` and the lazy `PdfViewer.tsx` component.
