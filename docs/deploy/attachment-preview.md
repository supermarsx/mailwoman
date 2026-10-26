# Attachment preview isolation and supported formats

The authenticated attachment-preview route passes decoded bytes to a disposable `mw-render` child. The child's no-import WASM guest parses TNEF/MAPI and RFC822/MIME. The render child sanitizes guest HTML with the existing mail sanitizer before returning it. The parent never parses TNEF or MIME attachment structure. Remote resources are not fetched while previewing.

Supported in this bounded implementation:

- `message/rfc822`/EML: decoded subject and addresses, date, plain and HTML bodies, transfer-decoded attachments, and nested message bytes.
- `application/ms-tnef`/winmail.dat: legacy and MAPI binary attachments, Unicode filenames, named/multivalue property traversal, declared common Windows/East Asian/UTF8 codepages, plain/HTML message body properties, and embedded-message object bytes.
- Attachment bytes remain unchanged. Filenames are normalized display/download suggestions; no file is extracted to a filesystem. OLE objects are opaque downloads, never executed.

Limits are 4 MiB raw input, 6 MiB JSON input frame, 64 returned attachments, 4 MiB aggregate extracted attachment data, 1 MiB for each body representation, 256 MIME parts and three nested message levels. The WASM store caps memory at512 MiB and guest execution at5 seconds. The server independently limits concurrent workers, lifetime and output bytes. The child bounds stdin before allocating a complete line. Failed checksums, corrupt lengths, unsupported property types and excessive counts fail closed.

Remaining limits: compressed RTF bodies are not rendered and produce a warning. OLE rendering, external-reference retrieval, Outlook-specific appointment/task UI, arbitrary legacy codepages and encrypted inner message decryption are unsupported. Unrecognized codepages currently use Windows-1252 for display strings; binary extraction remains unchanged. Nested message attachments are returned as downloadable message bytes; the viewer decides whether to request a further bounded preview. This is not a claim of complete Outlook/TNEF fidelity.

On Linux the existing render-child kernel jail must install successfully. On other platforms the documented process-isolation/WASM protections remain, without claiming Linux seccomp/Landlock enforcement.

The canonical guest is built using `rust:1.98.1-bookworm` and `crates/mw-media-wasm/build.sh`. The script honors the authorized `CARGO_TARGET_DIR`. Independent source mounts produced identical bytes; `plugins/verify-artifacts.py` pins the hash and zero-import/export contract. Its committed-tree mode is expected to refuse an uncommitted artifact; rerun that mode after integration.

The retained independent [MimeKit corpus](https://github.com/jstedfast/MimeKit/tree/a250ce09775904ad1af532fc1f92c5d01cc89e37/UnitTests/TestData/tnef) includes license, provenance and hashes under `crates/mw-media-wasm/fixtures/tnef`. Tests compare binary attachments exactly and normalize CRLF/LF for upstream text fixture comparisons. Format parsing follows Microsoft's [TNEF framing](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxtnef/b03b5704-ef54-4390-b981-dc78f8e79a05) and [MAPI property encapsulation](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxtnef/c27c89ec-7f8b-48d3-9e27-dc7600d23854).
