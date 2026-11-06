// Bytes -> displayable text for the plain-text attachment viewer.
//
// Total by construction: every input yields a result, never a throw. A charset
// label the platform does not know falls back to UTF-8, invalid sequences become
// U+FFFD, a file that is not text is reported as `binary`, and a file past the
// cap is cut (on a character boundary) with the full size reported so the viewer
// can offer the download instead of rendering megabytes.

/** Most bytes decoded and shown; the rest is reachable through the download. */
export const TEXT_PREVIEW_CAP = 512 * 1024;
/** A rendered line is cut here so one minified line cannot stall layout. */
export const TEXT_LINE_CAP = 20_000;

export type DecodedText =
  | { kind: 'empty' }
  | { kind: 'binary'; bytes: number }
  | { kind: 'text'; text: string; encoding: string; totalBytes: number; truncated: boolean; lines: number; lineCut: boolean };

/** The `charset=` parameter of a Content-Type value, lower-cased, or null. */
export function declaredCharset(mime: string): string | null {
  const m = /;\s*charset\s*=\s*"?([^";\s]+)"?/i.exec(mime);
  return m?.[1] ? m[1].toLowerCase() : null;
}

function bomEncoding(b: Uint8Array): { encoding: string; skip: number } | null {
  if (b.length >= 3 && b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf) return { encoding: 'utf-8', skip: 3 };
  if (b.length >= 2 && b[0] === 0xff && b[1] === 0xfe) return { encoding: 'utf-16le', skip: 2 };
  if (b.length >= 2 && b[0] === 0xfe && b[1] === 0xff) return { encoding: 'utf-16be', skip: 2 };
  return null;
}

function decoderFor(label: string): TextDecoder | null {
  try {
    return new TextDecoder(label, { fatal: false });
  } catch {
    return null;
  }
}

/** Does the sample look like text? NUL bytes or a high share of control bytes say no. */
function looksBinary(sample: Uint8Array): boolean {
  if (sample.length === 0) return false;
  let control = 0;
  for (const byte of sample) {
    if (byte === 0) return true;
    if (byte < 9 || (byte > 13 && byte < 32 && byte !== 27)) control += 1;
  }
  return control / sample.length > 0.1;
}

export function decodeText(input: Uint8Array, mime = ''): DecodedText {
  if (input.length === 0) return { kind: 'empty' };
  const bom = bomEncoding(input);
  const wide = bom?.encoding.startsWith('utf-16') === true;
  if (!wide && looksBinary(input.subarray(0, 8192))) return { kind: 'binary', bytes: input.length };
  const truncated = input.length > TEXT_PREVIEW_CAP;
  // An even cut for UTF-16 keeps code units whole; a UTF-8 cut leaves at worst one
  // trailing replacement character, trimmed below.
  const end = truncated ? (wide ? TEXT_PREVIEW_CAP & ~1 : TEXT_PREVIEW_CAP) : input.length;
  const body = input.subarray(bom?.skip ?? 0, end);
  const label = bom?.encoding ?? declaredCharset(mime) ?? 'utf-8';
  let decoder = decoderFor(label);
  let encoding = label;
  if (decoder === null) {
    decoder = decoderFor('utf-8');
    encoding = 'utf-8';
  }
  let text = '';
  try {
    text = decoder?.decode(body) ?? '';
  } catch {
    text = '';
  }
  if (truncated && text.endsWith('�')) text = text.slice(0, -1);
  if (text.length === 0 && body.length > 0) return { kind: 'binary', bytes: input.length };
  let lineCut = false;
  const rows = text.split(/\r\n|\r|\n/);
  const clipped = rows.map((row) => {
    if (row.length <= TEXT_LINE_CAP) return row;
    lineCut = true;
    return row.slice(0, TEXT_LINE_CAP);
  });
  return { kind: 'text', text: lineCut ? clipped.join('\n') : rows.join('\n'), encoding, totalBytes: input.length, truncated, lines: rows.length, lineCut };
}
