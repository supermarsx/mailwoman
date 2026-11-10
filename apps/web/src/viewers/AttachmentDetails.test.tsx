import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { AttachmentDetailsButton, type AttachmentDetails } from './AttachmentDetails.tsx';
import {clearSettings,loadSettings} from '../api/settings.ts';
import { groupHex, parsePdfDate, viewerDetailsFrom } from './attachmentDetails.ts';

const DIGEST = '0123456789abcdef'.repeat(4);
const details = (over: Partial<AttachmentDetails> = {}): AttachmentDetails => ({
  name: 'Quarterly report.pdf', mime: 'application/pdf', size: 1_234_567, digest: DIGEST, type: 'PDF document', extension: 'pdf', detected: 'application/pdf', ...over,
});
afterEach(() => { cleanup(); clearSettings(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

describe('formatting', () => {
  it('groups a fingerprint into 8-character blocks', () => {
    expect(groupHex(DIGEST)).toHaveLength(8);
    expect(groupHex(DIGEST)[0]).toBe('01234567');
    expect(groupHex('abcdef', 4)).toEqual(['abcd', 'ef']);
    expect(groupHex('')).toEqual([]);
  });
  it('reads PDF dates, with and without a zone, and refuses anything else', () => {
    expect(parsePdfDate("D:20240131120000+01'00'")).toBe('2024-01-31T12:00:00+01:00');
    expect(parsePdfDate('D:20240131120000Z')).toBe('2024-01-31T12:00:00Z');
    expect(parsePdfDate('D:2024')).toBe('2024-01-01T00:00:00Z');
    expect(parsePdfDate('yesterday')).toBeNull();
    expect(parsePdfDate('D:20241341250000')).toBeNull();
    expect(parsePdfDate(42)).toBeNull();
  });
  it('bounds what a viewer frame reports', () => {
    expect(viewerDetailsFrom({ pages: 3, producer: 'Acme‮PDF\n', creator: 'x'.repeat(500), created: 'D:20240131120000Z', pdfVersion: '1.7', extra: 'ignored' })).toEqual({ pages: 3, producer: 'Acme PDF', creator: 'x'.repeat(200), created: '2024-01-31T12:00:00Z', pdfVersion: '1.7' });
    expect(viewerDetailsFrom({ pages: -1, producer: 7, created: 'nope' })).toEqual({});
    expect(viewerDetailsFrom(null)).toEqual({});
  });
});

describe('AttachmentDetailsButton', () => {
  it.each(['icons','labels','icons-labels'])('keeps an accessible name in %s presentation',async style=>{
    await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{'reader.attachmentActionStyle':{value:style,source:'user',locked:false}}}))});
    render(()=><AttachmentDetailsButton details={details()}/>);
    const button=screen.getByRole('button',{name:'File details'});
    expect(button.textContent).toBe(style==='icons'?'':'File details');
    expect(button.querySelector('svg')!==null).toBe(style!=='labels');
  });

  it('is a labelled button with a tooltip that opens a modal dialog and returns focus on Escape', async () => {
    render(() => <AttachmentDetailsButton details={details()}/>);
    const button = screen.getByRole('button', { name: 'File details' });
    expect(button.getAttribute('title')).toBe('Show file details and fingerprint');
    expect(button.getAttribute('aria-haspopup')).toBe('dialog');
    expect(button.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByRole('dialog')).toBeNull();
    button.focus();
    fireEvent.click(button);
    const dialog = await screen.findByRole('dialog');
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(dialog.getAttribute('aria-labelledby')).not.toBeNull();
    expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('File details and fingerprint');
    expect(button.getAttribute('aria-expanded')).toBe('true');
    expect(dialog.contains(document.activeElement)).toBe(true);
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(document.activeElement).toBe(button);
  });

  it('lists name, type, MIME, size and the grouped SHA-256 fingerprint with its algorithm and a verification hint', async () => {
    render(() => <AttachmentDetailsButton details={details()}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    const dialog = await screen.findByRole('dialog');
    const text = dialog.textContent ?? '';
    for (const expected of ['Quarterly report.pdf', 'PDF document', 'application/pdf', '1.2 MB', '1,234,567 bytes', 'SHA-256', 'Verify the fingerprint', 'It does not prove that the file is safe']) expect(text).toContain(expected);
    const groups = [...dialog.querySelectorAll('[data-testid="attachment-fingerprint"] span')].map(span => span.textContent);
    expect(groups).toHaveLength(8);
    expect(groups.join('')).toBe(DIGEST);
    // It is a definition list: terms and values pair up.
    expect(dialog.querySelectorAll('dl > dt').length).toBe(dialog.querySelectorAll('dl > dd').length);
  });

  it('shows the PDF facts its viewer reported, and omits the ones it did not', async () => {
    render(() => <AttachmentDetailsButton details={details({ extra: { pages: 12, producer: 'Skia/PDF m120', created: '2024-01-31T12:00:00Z', pdfVersion: '1.4', passwordRequired: false } })}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    const text = (await screen.findByRole('dialog')).textContent ?? '';
    for (const expected of ['Pages', '12', 'Producer', 'Skia/PDF m120', 'Created', 'PDF version', '1.4', 'No password was asked for']) expect(text).toContain(expected);
    expect(text).not.toContain('Created with');
    cleanup();
    render(() => <AttachmentDetailsButton details={details({ extra: { pages: 1, passwordRequired: true } })}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    expect((await screen.findByRole('dialog')).textContent).toContain('A password was required to open it');
  });

  it('copies the file name and the fingerprint (plain hex, no spaces) and says so', async () => {
    const writeText = vi.fn(async () => {});
    vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
    render(() => <AttachmentDetailsButton details={details()}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(screen.getByRole('button', { name: 'Copy fingerprint' }));
    await waitFor(() => expect(writeText).toHaveBeenLastCalledWith(DIGEST));
    await waitFor(() => expect(dialog.querySelector('[role="status"]')?.textContent).toBe('Copied to the clipboard.'));
    fireEvent.click(screen.getByRole('button', { name: 'Copy file name' }));
    await waitFor(() => expect(writeText).toHaveBeenLastCalledWith('Quarterly report.pdf'));
  });

  it('reports a clipboard that refuses, and a fingerprint that could not be computed', async () => {
    vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText: vi.fn(async () => { throw new Error('denied'); }) } });
    render(() => <AttachmentDetailsButton details={details({ digest: null })}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    const dialog = await screen.findByRole('dialog');
    expect(dialog.textContent).toContain('Fingerprint unavailable');
    expect(screen.queryByRole('button', { name: 'Copy fingerprint' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Copy file name' }));
    await waitFor(() => expect(dialog.querySelector('[role="status"]')?.textContent).toBe('Could not copy to the clipboard.'));
  });

  it('traps Tab inside the dialog and closes from its close button', async () => {
    render(() => <AttachmentDetailsButton details={details()}/>);
    fireEvent.click(screen.getByRole('button', { name: 'File details' }));
    const dialog = await screen.findByRole('dialog');
    const focusable = [...dialog.querySelectorAll<HTMLElement>('button')];
    expect(focusable.length).toBeGreaterThanOrEqual(3);
    focusable.at(-1)!.focus();
    fireEvent.keyDown(dialog, { key: 'Tab' });
    expect(document.activeElement).toBe(focusable[0]);
    fireEvent.click(focusable[0]!.getAttribute('aria-label') === 'Close' ? focusable[0]! : screen.getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });
});
