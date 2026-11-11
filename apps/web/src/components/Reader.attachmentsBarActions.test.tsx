import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import type { AppState } from '../state/store.ts';
import type { Email } from '../api/jmap-types.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';
import { AttachmentsPane } from './Reader.tsx';
import {createSignal} from 'solid-js';

const downloadOne = vi.fn((..._args: unknown[]) => Promise.resolve());
const downloadZip = vi.fn((..._args: unknown[]) => Promise.resolve());
vi.mock('./messageAttachmentDownload.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('./messageAttachmentDownload.ts')>()),
  downloadOne: (...args: unknown[]) => downloadOne(...args),
  downloadZip: (...args: unknown[]) => downloadZip(...args),
}));

const policy = { previewAllowed: true, downloadAllowed: true, maxBytes: 0, allowedExtensions: [], blockedExtensions: ['exe'], warnExtensions: ['js'], allowedMimeTypes: [], blockedMimeTypes: [], warnMimeTypes: [] };
const PDF = { blobId: 'b.1', type: 'application/pdf', name: 'a.pdf', size: 1024 };
const DOC = { blobId: 'b.2', type: 'application/msword', name: 'b.doc', size: 2048 };
const EXE = { blobId: 'b.3', type: 'application/x-msdownload', name: 'bad.exe', size: 10 };
const INLINE = { blobId: 'b.4', type: 'image/png', name: 'logo.png', size: 99, cid: 'logo@x', disposition: 'inline' };

type Values = Record<string, { value: unknown; locked?: boolean }>;
const apply = (values: Values) => loadSettings({ refresh: true, fetcher: async () => new Response(JSON.stringify({ settings: Object.fromEntries(Object.entries(values).map(([key, entry]) => [key, { value: entry.value, source: 'global', locked: entry.locked ?? false }])) }), { headers: { 'content-type': 'application/json' } }) });

async function setup(attachments: unknown[], values: Values = {}) {
  const showToast = vi.fn();
  vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(new Response(JSON.stringify(policy), { status: 200, headers: { 'content-type': 'application/json' } }))));
  await apply({ 'reader.attachmentsCollapsed': { value: true }, ...values });
  const app = { accountId: () => 'a1', downloadUrl: () => 'https://m/d/{accountId}/{blobId}/{name}?type={type}', showToast, openMessage: vi.fn(), nextcloud: { enabled: () => false, ensureEnabled: () => Promise.resolve(), service: {} } } as unknown as AppState;
  const [parts,setParts]=createSignal(attachments);
  const [messageId,setMessageId]=createSignal('e1');
  const view = render(() => <AppContext.Provider value={app}><AttachmentsPane email={{ id: messageId(), hasAttachment: true, attachments:parts() } as unknown as Email} /></AppContext.Provider>);
  return { app, showToast, view,setParts,setMessageId };
}
type View = Awaited<ReturnType<typeof setup>>['view'];
const expanded = (view: View) => view.getByTestId('reader-attachments-toggle').getAttribute('aria-expanded');
const open: Values = { 'reader.attachmentsCollapsed': { value: false } };
const settle = () => new Promise((r) => setTimeout(r, 30));

beforeEach(() => { downloadOne.mockClear(); downloadZip.mockClear(); });
afterEach(() => { cleanup(); clearSettings(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

describe('attachments bar toggle', () => {
  it('updates an already expanded attachment list without toggling the bar',async()=>{
    const {view,setParts}=await setup([PDF],open);
    await waitFor(()=>expect(view.container.querySelector('[data-policy="ready"]')).not.toBeNull());
    expect(view.getByTitle('a.pdf')).toBeVisible();
    setParts([DOC]);
    await waitFor(()=>expect(view.getByTitle('b.doc')).toBeVisible());
    expect(view.queryByTitle('a.pdf')).toBeNull();expect(expanded(view)).toBe('true');
  });
  it('updates files and inline images while both sections are already open',async()=>{
    const {view,setParts}=await setup([PDF,INLINE],open);
    await waitFor(()=>expect(view.container.querySelector('[data-policy="ready"]')).not.toBeNull());
    expect(view.getByTitle('a.pdf')).toBeVisible();
    const inline=view.getByTestId('reader-attachments-inline') as HTMLDetailsElement;
    inline.open=true;
    setParts([DOC,{...INLINE,blobId:'new-inline',name:'updated-logo.png'}]);
    await waitFor(()=>expect(view.getByTitle('updated-logo.png')).toBeVisible());
    expect(view.getByTitle('b.doc')).toBeVisible();expect(view.queryByTitle('logo.png')).toBeNull();
    expect(inline.open).toBe(true);expect(expanded(view)).toBe('true');
  });
  it('shows the next message attachments when expanded is the configured default',async()=>{
    const {view,setParts,setMessageId}=await setup([PDF],open);
    await waitFor(()=>expect(view.container.querySelector('[data-policy="ready"]')).not.toBeNull());
    expect(view.getByTitle('a.pdf')).toBeVisible();
    setMessageId('e2');setParts([DOC]);
    await waitFor(()=>expect(view.getByTitle('b.doc')).toBeVisible());
    expect(view.queryByTitle('a.pdf')).toBeNull();expect(expanded(view)).toBe('true');
  });
  it('toggles when the header is clicked anywhere, and aria-expanded follows', async () => {
    const { view } = await setup([PDF]);
    expect(expanded(view)).toBe('false');
    const row = view.getByTestId('reader-attachments-bar').querySelector('[data-toggle-row]')!;
    fireEvent.click(row);
    expect(expanded(view)).toBe('true');
    fireEvent.click(row.querySelector('[data-tooltip]')!);
    expect(expanded(view)).toBe('false');
  });
  it('toggles once (not twice) when the chevron button itself is clicked', async () => {
    const { view } = await setup([PDF]);
    fireEvent.click(view.getByTestId('reader-attachments-toggle'));
    expect(expanded(view)).toBe('true');
  });
  it('does not toggle from the Download all button or an item download button', async () => {
    const { view } = await setup([PDF, DOC], open);
    expect(expanded(view)).toBe('true');
    fireEvent.click(view.getByTestId('reader-attachments-download-all'));
    expect(expanded(view)).toBe('true');
    await waitFor(() => expect(view.queryAllByTestId('attachment-download').length).toBe(2));
    fireEvent.click(view.getAllByTestId('attachment-download')[0]!);
    expect(expanded(view)).toBe('true');
  });
  it('toggles from the keyboard on the summary (Enter and Space)', async () => {
    const { view } = await setup([PDF]);
    const summary = view.getByTestId('reader-attachments-bar').querySelector('[data-tooltip][tabindex="0"]')!;
    fireEvent.keyDown(summary, { key: 'Enter' });
    expect(expanded(view)).toBe('true');
    fireEvent.keyDown(summary, { key: ' ' });
    expect(expanded(view)).toBe('false');
  });
  it('a finished text selection does not toggle', async () => {
    const { view } = await setup([PDF]);
    const row = view.getByTestId('reader-attachments-bar').querySelector('[data-toggle-row]')!;
    vi.spyOn(globalThis, 'getSelection').mockReturnValue({ isCollapsed: false, toString: () => 'attach' } as unknown as Selection);
    fireEvent.click(row);
    expect(expanded(view)).toBe('false');
  });
  it('only the chevron toggles when attachments.bar.clickToToggle is off', async () => {
    const { view } = await setup([PDF], { 'attachments.bar.clickToToggle': { value: false, locked: true } });
    expect(view.getByTestId('reader-attachments-bar').querySelector('[data-toggle-row]')).toBeNull();
    fireEvent.click(view.getByTestId('reader-attachments-toggle'));
    expect(expanded(view)).toBe('true');
  });
});

describe('attachment download buttons', () => {
  it('downloads one attachment through the shared helper with its name and part', async () => {
    const { view, app } = await setup([PDF, DOC], open);
    await waitFor(() => expect(view.getAllByTestId('attachment-download').length).toBe(2));
    const button = view.getAllByTestId('attachment-download')[0]!;
    expect(button.getAttribute('aria-label')).toContain('a.pdf');
    expect(button.getAttribute('aria-label')).toContain('1');
    expect(button.getAttribute('data-tooltip')).toMatch(/^Download .?a.pdf.?$/);
    fireEvent.click(button);
    await waitFor(() => expect(downloadOne).toHaveBeenCalledTimes(1));
    const [calledApp, emailId, named, acknowledge] = downloadOne.mock.calls[0] as unknown as [unknown, string, { name: string; part: { blobId: string } }, boolean];
    expect(calledApp).toBe(app);
    expect(emailId).toBe('e1');
    expect(named.name).toBe('a.pdf');
    expect(named.part.blobId).toBe('b.1');
    expect(acknowledge).toBe(false);
  });
  it('a blocked file is disabled with its reason and never downloads', async () => {
    const { view } = await setup([EXE], open);
    await waitFor(() => expect(view.getByTestId('attachment-download')).toBeTruthy());
    const button = view.getByTestId('attachment-download');
    expect(button.getAttribute('aria-disabled')).toBe('true');
    expect(button.getAttribute('data-tooltip')).toContain('bad.exe');
    fireEvent.click(button);
    expect(downloadOne).not.toHaveBeenCalled();
  });
  it('is disabled while mail.allowDownload is off', async () => {
    const { view } = await setup([PDF], { ...open, 'mail.allowDownload': { value: false, locked: true } });
    await waitFor(() => expect(view.getByTestId('attachment-download')).toBeTruthy());
    expect(view.getByTestId('attachment-download').getAttribute('aria-disabled')).toBe('true');
    expect(view.getByTestId('reader-attachments-download-all')).toBeDisabled();
  });
  it('honours the mode: never removes the buttons, always and hover mark the wrapper', async () => {
    const never = await setup([PDF], { ...open, 'attachments.bar.showDownloadButtons': { value: 'never', locked: true } });
    await waitFor(() => expect(never.view.container.querySelector('[data-policy="ready"]')).not.toBeNull());
    expect(never.view.queryByTestId('attachment-download')).toBeNull();
    cleanup();
    const always = await setup([PDF], { ...open, 'attachments.bar.showDownloadButtons': { value: 'always' } });
    await waitFor(() => expect(always.view.getByTestId('attachment-download')).toBeTruthy());
    expect(always.view.container.querySelector('.mw-thumb-wrap')!.getAttribute('data-action-mode')).toBe('always');
    cleanup();
    const hover = await setup([PDF], open);
    await waitFor(() => expect(hover.view.getByTestId('attachment-download')).toBeTruthy());
    expect(hover.view.container.querySelector('.mw-thumb-wrap')!.getAttribute('data-action-mode')).toBe('hover');
  });
});

describe('Download all', () => {
  type Group = { emailId: string; items: { name: string }[] };
  const names = (call: unknown[]) => (call[1] as Group[])[0]!.items.map((i) => i.name);
  it('zips the allowed files and passes blocked ones so the summary counts them', async () => {
    const { view } = await setup([PDF, DOC, EXE]);
    await settle();
    fireEvent.click(view.getByTestId('reader-attachments-download-all'));
    await waitFor(() => expect(downloadZip).toHaveBeenCalledTimes(1));
    expect((downloadZip.mock.calls[0]![1] as Group[])).toHaveLength(1);
    expect(names(downloadZip.mock.calls[0]!)).toEqual(['a.pdf', 'b.doc', 'bad.exe']);
    expect(downloadOne).not.toHaveBeenCalled();
  });
  it('downloads a single allowed attachment directly, not as a ZIP', async () => {
    const { view } = await setup([PDF]);
    await settle();
    fireEvent.click(view.getByTestId('reader-attachments-download-all'));
    await waitFor(() => expect(downloadOne).toHaveBeenCalledTimes(1));
    expect(downloadZip).not.toHaveBeenCalled();
  });
  it('leaves inline cid images out unless zipInlineImages is on', async () => {
    const off = await setup([PDF, DOC, INLINE]);
    await settle();
    fireEvent.click(off.view.getByTestId('reader-attachments-download-all'));
    await waitFor(() => expect(downloadZip).toHaveBeenCalledTimes(1));
    expect(names(downloadZip.mock.calls[0]!)).toEqual(['a.pdf', 'b.doc']);
    cleanup();
    downloadZip.mockClear();
    const on = await setup([PDF, DOC, INLINE], { 'attachments.bar.zipInlineImages': { value: true } });
    await settle();
    fireEvent.click(on.view.getByTestId('reader-attachments-download-all'));
    await waitFor(() => expect(downloadZip).toHaveBeenCalledTimes(1));
    expect(names(downloadZip.mock.calls[0]!)).toHaveLength(3);
  });
  it('tells the user when every file is blocked and downloads nothing', async () => {
    const { view, showToast } = await setup([EXE]);
    await settle();
    fireEvent.click(view.getByTestId('reader-attachments-download-all'));
    await waitFor(() => expect(showToast).toHaveBeenCalledWith('error', expect.any(String)));
    expect(downloadZip).not.toHaveBeenCalled();
    expect(downloadOne).not.toHaveBeenCalled();
  });
  it('shows a busy spinner and disables the button while the ZIP is built', async () => {
    let finish: () => void = () => undefined;
    downloadZip.mockImplementationOnce(() => new Promise<void>((resolve) => { finish = resolve; }));
    const { view } = await setup([PDF, DOC]);
    await settle();
    const button = view.getByTestId('reader-attachments-download-all');
    fireEvent.click(button);
    await waitFor(() => expect(button).toBeDisabled());
    expect(button.querySelector('[data-mw-spinner]')).not.toBeNull();
    finish();
    await waitFor(() => expect(button).not.toBeDisabled());
  });
  it('is not offered when attachments.bar.downloadAll is off', async () => {
    const { view } = await setup([PDF], { 'attachments.bar.downloadAll': { value: false, locked: true } });
    expect(view.queryByTestId('reader-attachments-download-all')).toBeNull();
  });
});
