// `decode.ts`: an imported calendar file is decoded by its byte order mark.
//
// The bytes are made here, by `utf16`, from text in the layout of Outlook's own
// OUTLOOK.HOL (`[Name] n`, `Description,YYYY/M/D`, CRLF); nothing in this file
// goes through the function under test to produce its expected value.

import { describe, it, expect } from 'vitest';
import { decodeCalendarBytes, readCalendarFile } from './decode.ts';

const PACK = '[Portugal] 2\r\nAno Novo,2026/1/1\r\nDia da Região,2026/7/1\r\n';

/** `text` in UTF-16, code unit by code unit, with or without its mark. */
function utf16(text: string, order: 'le' | 'be', mark = true): Uint8Array<ArrayBuffer> {
  const out: number[] = mark ? (order === 'le' ? [0xff, 0xfe] : [0xfe, 0xff]) : [];
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (order === 'le') out.push(unit & 0xff, unit >> 8);
    else out.push(unit >> 8, unit & 0xff);
  }
  return Uint8Array.from(out);
}

describe('decodeCalendarBytes', () => {
  it('a UTF-8 read of a UTF-16 file loses the mark and the characters outside ASCII', () => {
    // What `File.text()` does in a browser that follows its specification
    // (Firefox), and why the decode cannot be left to the server.
    const asUtf8 = new TextDecoder().decode(utf16(PACK, 'le'));
    expect(asUtf8.startsWith('��[\0P\0')).toBe(true);
    expect(asUtf8).toContain('R\0e\0g\0i\0�\0o\0');
    expect(asUtf8).not.toContain('ã');
  });

  it('reads UTF-16LE with its byte order mark, and drops the mark', () => {
    const bytes = utf16(PACK, 'le');
    expect([...bytes.subarray(0, 4)]).toEqual([0xff, 0xfe, 0x5b, 0x00]);
    expect(decodeCalendarBytes(bytes)).toBe(PACK);
  });

  it('reads UTF-16BE with its byte order mark, and drops the mark', () => {
    const bytes = utf16(PACK, 'be');
    expect([...bytes.subarray(0, 4)]).toEqual([0xfe, 0xff, 0x00, 0x5b]);
    expect(decodeCalendarBytes(bytes)).toBe(PACK);
  });

  it('reads anything else as UTF-8, with or without a UTF-8 mark', () => {
    const utf8 = new TextEncoder().encode(PACK);
    expect(decodeCalendarBytes(utf8)).toBe(PACK);
    expect(decodeCalendarBytes(Uint8Array.from([0xef, 0xbb, 0xbf, ...utf8]))).toBe(PACK);
    expect(decodeCalendarBytes(new Uint8Array(0))).toBe('');
  });

  it('does not guess: UTF-16 without a mark is read as UTF-8', () => {
    for (const order of ['le', 'be'] as const) {
      const text = decodeCalendarBytes(utf16(PACK, order, false));
      expect(text).not.toBe(PACK);
      expect(text).toContain('\0');
    }
  });

  it('a file cut off inside a code unit keeps what came before', () => {
    const cut = utf16(PACK, 'le').subarray(0, -1);
    expect(decodeCalendarBytes(cut).startsWith('[Portugal] 2\r\nAno Novo,2026/1/1\r\n')).toBe(true);
  });
});

describe('readCalendarFile', () => {
  it('decodes the bytes of a picked file', async () => {
    await expect(readCalendarFile(new File([utf16(PACK, 'le')], 'pt.hol'))).resolves.toBe(PACK);
    await expect(readCalendarFile(new File([utf16(PACK, 'be')], 'pt.hol'))).resolves.toBe(PACK);
    await expect(readCalendarFile(new File([PACK], 'pt.hol'))).resolves.toBe(PACK);
  });
});
