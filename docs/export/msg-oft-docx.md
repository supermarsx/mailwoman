# MSG / OFT / DOCX export (V7)

V7 (release 26.8.0) extends `mw-export` with three formats in addition to the existing
mbox / Markdown / text / html2md exporters:

| Format | Container | Library | What it is |
|---|---|---|---|
| `.msg` | OLE/CFB (MS-OXMSG) | `cfb` + Mailwoman's own MAPI encoder | an Outlook single-message file |
| `.oft` | OLE/CFB (MS-OXMSG) | `cfb` + the MAPI encoder | an Outlook template |
| `.docx` | OOXML (ZIP) | `docx-rs` | a Word document of the rendered message |

Existing export formats are byte-unchanged.

## Scope: what is preserved

The committed floor is **faithful body + attachments + headers**:

- the message body (the rendered content goes through the existing sanitizer + print
  pipeline),
- attachments,
- the standard headers (From/To/Cc/Subject/Date, etc.).

Rendered exports reuse the same sanitize/print path as the rest of Mailwoman, so an
exported `.docx` reflects what the sanitized message looks like.

## Fidelity boundary (honest, per §28.8)

Deep OLE write fidelity is **best-effort**, not a committed guarantee:

- embedded objects and custom/named MAPI properties are not guaranteed to round-trip
  with full fidelity,
- the goal is that Outlook (and compatible readers) open the `.msg`/`.oft` with the
  body, attachments, and headers intact — not a byte-exact reproduction of an
  Outlook-authored file's every named property.

Deep write fidelity (embedded objects, custom named properties) is tracked for
post-1.0.

## Importing `.oft` (untrusted CFB parsing)

Reading an `.oft` means parsing an untrusted OLE/CFB container. That parse is isolated
behind a **size-limited, disposable render boundary** (a `Cfb`/`Msg` render job routed
through the same disposable-child mechanism used for other untrusted document
rendering), so a malformed container cannot affect the main process.

## CFB fuzzing

`mw-export` ships a fuzz target for the CFB parse path (`crates/mw-export/fuzz/`,
§25). The `wasm`/plugin CI does not run the fuzzer; it is built and smoke-run to keep
the parser exercised.

## Supply chain note

`docx-rs` (MIT) is at 0.4.22, which depends on `quick-xml` 0.41. Up to 26.19 it was
at 0.4.20, which pinned `quick-xml` 0.36.2 with two denial-of-service advisories in
the quick-xml reader (RUSTSEC-2026-0194 / -0195); those were a `cargo deny` ignore
from 26.8 (`docs/RELEASE-NOTES-26.8.md`) and were removed in 26.20 when 0.36.2 left
the lock. Every quick-xml consumer in the tree is now on 0.41.

`docx-rs` 0.4.22 depends on `zip` 8.6, which is in the affected range of three `zip`
advisories (RUSTSEC-2026-0367 / -0368 / -0369; fixed in zip 9). All three concern
`ZipArchive::extract` writing an archive's entries to a filesystem. docx-rs does not
call `extract`, and Mailwoman uses docx-rs to write DOCX only; it never reads a
`.docx` through it. The three advisories are a `cargo deny` ignore in `deny.toml`,
to be removed when docx-rs releases on zip 9 or later.
