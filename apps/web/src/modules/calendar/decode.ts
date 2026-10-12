// Reading an imported calendar file as text.
//
// Outlook writes `.hol` holiday files as UTF-16LE with a byte order mark (an
// installed OUTLOOK.HOL starts `FF FE`). `File.text()` is specified as a UTF-8
// decode, and browsers differ on such a file (measured 2026-10 on Chromium 149,
// WebKit 26.5 and Firefox 151): Chromium and WebKit read the mark and return
// the text; Firefox decodes UTF-8, which replaces the mark with U+FFFD U+FFFD,
// leaves a NUL beside every character and turns each character outside ASCII
// into U+FFFD. `CalendarEvent/import` takes the document as a JSON string
// (`event_import`, crates/mw-engine/src/pim/events.rs), so the server is sent
// text and cannot undo that. The bytes are therefore decoded here, the same
// way in every browser.
//
// The rule: a UTF-16 byte order mark selects UTF-16 — `FF FE` little-endian,
// `FE FF` big-endian — and anything else is UTF-8, as it was. Nothing is
// guessed from the content: UTF-16 without a mark is read as UTF-8, and the
// server refuses what that produces. UTF-32 is not read (its little-endian mark
// begins `FF FE`; the result is likewise refused).

/** Decode the bytes of a calendar file by its byte order mark. */
export function decodeCalendarBytes(bytes: Uint8Array): string {
  const encoding =
    bytes[0] === 0xff && bytes[1] === 0xfe ? 'utf-16le' : bytes[0] === 0xfe && bytes[1] === 0xff ? 'utf-16be' : 'utf-8';
  // The decoder drops the mark it was chosen by (and a UTF-8 one), and puts
  // U+FFFD where the bytes are not valid in that encoding.
  return new TextDecoder(encoding).decode(bytes);
}

/** Read a picked file's bytes and decode them with {@link decodeCalendarBytes}. */
export function readCalendarFile(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(decodeCalendarBytes(new Uint8Array(reader.result as ArrayBuffer)));
    reader.onerror = () => reject(reader.error ?? new Error('the file could not be read'));
    reader.readAsArrayBuffer(file);
  });
}
