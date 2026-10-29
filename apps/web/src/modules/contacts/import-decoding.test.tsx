// The contacts import reads a picked file by its byte order mark (t31-e11).
//
// `File.text()` is specified as a UTF-8 decode, so in Firefox a UTF-16 vCard
// comes back with U+FFFD for its mark and a NUL beside every character, and
// parses to no contact. The import page (t39 R6) decodes with the calendar's
// `readCalendarFile` instead (`modules/calendar/decode.ts`). jsdom has no
// `Blob.text`; the tests give it the specified one, so a page that still
// called it would read these files as Firefox does.
//
// The bytes are made here, by `utf16`, code unit by code unit; nothing in this
// file goes through the decoder to produce an expected value.

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, within } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import { createAppState } from '../../state/store.ts';
import { ContactsModule } from './index.tsx';
import { makeContactsClient, defaultSeed } from './mockClient.ts';

const VCARD = 'BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Zoë Ångström\r\nEMAIL:zoe@example.org\r\nEND:VCARD\r\n';

/** `text` in UTF-16 with its byte order mark. */
function utf16(text: string, order: 'le' | 'be'): Uint8Array<ArrayBuffer> {
  const out: number[] = order === 'le' ? [0xff, 0xfe] : [0xfe, 0xff];
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (order === 'le') out.push(unit & 0xff, unit >> 8);
    else out.push(unit >> 8, unit & 0xff);
  }
  return Uint8Array.from(out);
}

async function pickFile(bytes: Uint8Array<ArrayBuffer>): Promise<HTMLElement> {
  render(() => <AppContext.Provider value={createAppState(makeContactsClient(defaultSeed()))}>{ContactsModule()}</AppContext.Provider>);
  await screen.findByText('Ada Lovelace');
  fireEvent.click(screen.getByRole('button', { name: 'Import…' }));
  const dialog = await screen.findByTestId('contacts-import-page');
  const file = new File([bytes], 'contacts.vcf', { type: 'text/vcard' });
  fireEvent.change(within(dialog).getByLabelText('Import file'), { target: { files: [file] } });
  return dialog;
}

/** `Blob.text` as specified: the bytes, decoded as UTF-8. */
function specifiedText(this: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(new TextDecoder('utf-8').decode(reader.result as ArrayBuffer));
    reader.onerror = () => reject(reader.error ?? new Error('read failed'));
    reader.readAsArrayBuffer(this);
  });
}

describe('contacts import decoding', () => {
  const ownText = Object.getOwnPropertyDescriptor(Blob.prototype, 'text');
  beforeEach(() => {
    localStorage.clear();
    Object.defineProperty(Blob.prototype, 'text', { configurable: true, writable: true, value: specifiedText });
  });
  afterEach(() => {
    if (ownText === undefined) delete (Blob.prototype as { text?: unknown }).text;
    else Object.defineProperty(Blob.prototype, 'text', ownText);
  });

  for (const order of ['le', 'be'] as const) {
    it(`reads a UTF-16${order.toUpperCase()} vCard with its byte order mark`, async () => {
      const bytes = utf16(VCARD, order);
      expect([...bytes.subarray(0, 2)]).toEqual(order === 'le' ? [0xff, 0xfe] : [0xfe, 0xff]);
      const dialog = await pickFile(bytes);
      const preview = await within(dialog).findByRole('list', { name: 'Import preview' });
      expect(preview).toHaveTextContent('Zoë Ångström'); expect(preview).toHaveTextContent('zoe@example.org');
      // A textarea's value has its line breaks normalized to LF.
      expect(within(dialog).getByLabelText('Paste vCard or CSV')).toHaveValue(VCARD.replace(/\r\n/g, '\n'));
    });
  }

  it('reads a UTF-8 vCard as before', async () => {
    const dialog = await pickFile(new TextEncoder().encode(VCARD) as Uint8Array<ArrayBuffer>);
    const preview = await within(dialog).findByRole('list', { name: 'Import preview' });
    expect(preview).toHaveTextContent('Zoë Ångström'); expect(preview).toHaveTextContent('zoe@example.org');
  });
});
