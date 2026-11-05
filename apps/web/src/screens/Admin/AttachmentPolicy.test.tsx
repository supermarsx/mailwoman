import { describe, it, expect, vi, afterEach } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminAttachmentPolicy } from './AttachmentPolicy.tsx';
import { ATTACHMENT_TABS, attachmentRouteTab, attachmentTabForKey } from './shell/attachmentNavigation.ts';
import { renderInShell } from './shell/testkit.tsx';
import type { AdminSetting } from '../../api/admin/settings.ts';

const base: AdminSetting = { key: 'attachments.downloadAllowed', family: false, kind: { type: 'bool' }, default: true, nullable: false, scopes: 'global+domain', userOverride: false, category: 'attachments', mapsTo: '', env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: null, allowUserOverride: true, globalLocked: false, updatedBy: null, updatedAt: null, effective: { value: true, source: 'default' }, envState: null, overrides: {}, etag: '"v1"' };
const list = (key: string, value: string[]): AdminSetting => ({ ...base, key, kind: { type: 'stringList', maxItems: 100, maxLength: 127 }, default: [], effective: { value, source: value.length ? 'global' : 'default' }, globalValue: value.length ? value : null });
const rows: AdminSetting[] = [
  base,
  { ...base, key: 'attachments.previewAllowed' },
  { ...base, key: 'attachments.maxBytes', kind: { type: 'int', min: 0, max: 4294967296 }, default: 0, effective: { value: 26214400, source: 'global' }, globalValue: 26214400 },
  list('attachments.allowedExtensions', []),
  list('attachments.blockedExtensions', ['exe']),
  list('attachments.warnExtensions', ['exe', 'zip']),
  list('attachments.allowedMimeTypes', []),
  list('attachments.blockedMimeTypes', ['image/*']),
  list('attachments.warnMimeTypes', []),
];
const load = vi.fn(async () => ({ kind: 'ok' as const, data: rows, etag: null }));
const route = (...rest: string[]) => () => ({ section: 'attachments', rest });
afterEach(() => vi.unstubAllGlobals());

describe('attachment policy, edited in place', () => {
  it('shows the Lists tab with chip lists and no Edit policy links', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route() });
    const blocked = await screen.findByTestId('attachment-list-blocked-extension');
    expect(within(blocked).getByText('exe')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Edit policy' })).toBeNull();
    expect(screen.getByRole('navigation', { name: 'Attachment policy' })).toBeInTheDocument();
  });

  it('adds a normalised extension and saves the list', async () => {
    const bodies: unknown[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_u: string, init?: RequestInit) => {
      bodies.push(JSON.parse(String(init?.body)));
      return new Response(JSON.stringify({ ...rows[4], globalValue: ['exe', 'pdf'], etag: '"v2"' }), { status: 200, headers: { 'content-type': 'application/json', ETag: '"v2"' } });
    }));
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route() });
    const editor = await screen.findByTestId('attachment-list-blocked-extension');
    const input = within(editor).getByRole('textbox', { name: /Blocked file extensions|Extension/i });
    fireEvent.input(input, { target: { value: '*.PDF' } });
    fireEvent.click(within(editor).getByRole('button', { name: 'Add' }));
    expect(within(editor).getByText('pdf')).toBeInTheDocument();
    const card = editor.closest('[data-testid="inline-setting"]') as HTMLElement;
    fireEvent.click(within(card).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(bodies).toEqual([{ globalValue: ['exe', 'pdf'] }]));
  });

  it('rejects an invalid entry in words and does not change the list', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route() });
    const editor = await screen.findByTestId('attachment-list-blocked-mime');
    fireEvent.input(within(editor).getByPlaceholderText(/MIME type/), { target: { value: 'notamime' } });
    fireEvent.click(within(editor).getByRole('button', { name: 'Add' }));
    expect(await within(editor).findByText(/Not a MIME type/)).toBeInTheDocument();
    expect(within(editor).getByText('image/*')).toBeInTheDocument();
  });

  it('reports a duplicate and bulk-pasted invalid entries', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route() });
    const editor = await screen.findByTestId('attachment-list-blocked-extension');
    fireEvent.click(within(editor).getByText('Paste many'));
    fireEvent.input(within(editor).getByRole('textbox', { name: 'Paste many' }), { target: { value: 'exe bat a/b' } });
    fireEvent.click(within(editor).getByRole('button', { name: 'Add all' }));
    expect(await within(editor).findByText(/Already in the list: exe/)).toBeInTheDocument();
    expect(within(editor).getByText(/not a valid rule: a\/b/)).toBeInTheDocument();
    expect(within(editor).getByText('bat')).toBeInTheDocument();
  });

  it('removes a rule with the chip button', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route() });
    const editor = await screen.findByTestId('attachment-list-blocked-extension');
    fireEvent.click(within(editor).getByRole('button', { name: 'Remove exe' }));
    expect(within(editor).queryByText('exe')).toBeNull();
    expect(within(editor).getByText('No rules in this list.')).toBeInTheDocument();
  });

  it('warns when a rule is in a blocked and a warning list', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route('warnings') });
    const editor = await screen.findByTestId('attachment-list-warn-extension');
    expect(within(editor).getAllByTestId('attachment-conflict')[0]).toHaveTextContent('the warning never shows');
  });

  it('edits the size limit with units', async () => {
    const bodies: unknown[] = [];
    vi.stubGlobal('fetch', vi.fn(async (_u: string, init?: RequestInit) => {
      bodies.push(JSON.parse(String(init?.body)));
      return new Response(JSON.stringify({ ...rows[2], globalValue: 10485760, etag: '"v2"' }), { status: 200, headers: { 'content-type': 'application/json', ETag: '"v2"' } });
    }));
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route('limits') });
    const editor = await screen.findByTestId('attachment-max-bytes');
    const amount = within(editor).getByRole('textbox') as HTMLInputElement;
    expect(amount.value).toBe('25');
    expect(within(editor).getByRole('combobox')).toHaveValue('MiB');
    fireEvent.input(amount, { target: { value: '10' } });
    fireEvent.click(within(editor.closest('[data-testid="inline-setting"]') as HTMLElement).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(bodies).toEqual([{ globalValue: 10485760 }]));
  });

  it('tests a file with the server evaluator and shows the decision', async () => {
    const simulate = vi.fn(async () => ({ decision: { downloadAllowed: false, previewAllowed: false, warning: false, reason: 'attachmentBlocked' }, matched: ['blockedExtensions'] }));
    renderInShell(() => <AdminAttachmentPolicy load={load} simulate={simulate} />, { route: route('test') });
    fireEvent.input(await screen.findByLabelText('File name'), { target: { value: 'setup.EXE' } });
    fireEvent.input(screen.getByLabelText('Size'), { target: { value: '2' } });
    fireEvent.click(screen.getByRole('button', { name: 'Test' }));
    const result = await screen.findByTestId('simulator-result');
    expect(result).toHaveAttribute('data-verdict', 'blocked');
    expect(result).toHaveTextContent('A block rule matches.');
    expect(result).toHaveTextContent('blocked extensions');
    expect(simulate).toHaveBeenCalledWith({ filename: 'setup.EXE', mime: 'application/octet-stream', size: 2048 });
  });

  it('offers common MIME types in a grouped combobox and also accepts a custom one', async () => {
    const simulate = vi.fn(async () => ({ decision: { downloadAllowed: true, previewAllowed: false, warning: false, reason: null }, matched: [] }));
    renderInShell(() => <AdminAttachmentPolicy load={load} simulate={simulate} />, { route: route('test') });
    const combo = await screen.findByRole('combobox', { name: 'MIME type' });
    fireEvent.focus(combo);
    fireEvent.input(combo, { target: { value: 'pdf' } });
    fireEvent.keyDown(combo, { key: 'ArrowDown' });
    fireEvent.keyDown(combo, { key: 'Enter' });
    expect(combo).toHaveValue('application/pdf');
    fireEvent.input(combo, { target: { value: 'application/x-custom' } });
    expect(screen.queryByText(/not a type\/subtype/)).not.toBeInTheDocument();
    fireEvent.input(combo, { target: { value: 'notamime' } });
    expect(screen.getByText(/not a type\/subtype/)).toBeInTheDocument();
    expect(combo).toHaveAttribute('aria-invalid', 'true');
    fireEvent.input(screen.getByLabelText('File name'), { target: { value: 'a.bin' } });
    fireEvent.click(screen.getByRole('button', { name: 'Test' }));
    await waitFor(() => expect(simulate).toHaveBeenCalledWith({ filename: 'a.bin', mime: 'notamime', size: 0 }));
  });

  it('fills name, type and size from a chosen file and remembers the type for the session', async () => {
    const simulate = vi.fn(async () => ({ decision: { downloadAllowed: true, previewAllowed: false, warning: false, reason: null }, matched: [] }));
    const view = renderInShell(() => <AdminAttachmentPolicy load={load} simulate={simulate} />, { route: route('test') });
    const picker = (await screen.findByLabelText('Fill in from a file (optional)')) as HTMLInputElement;
    const file = new File([new Uint8Array(2048)], 'report.docx', { type: '' });
    fireEvent.change(picker, { target: { files: [file] } });
    expect(screen.getByLabelText('File name')).toHaveValue('report.docx');
    expect(screen.getByRole('combobox', { name: 'MIME type' })).toHaveValue('application/vnd.openxmlformats-officedocument.wordprocessingml.document');
    fireEvent.click(screen.getByRole('button', { name: 'Test' }));
    await waitFor(() => expect(simulate).toHaveBeenCalledTimes(1));
    view.unmount();
    renderInShell(() => <AdminAttachmentPolicy load={load} simulate={simulate} />, { route: route('test') });
    expect(await screen.findByRole('combobox', { name: 'MIME type' })).toHaveValue('application/vnd.openxmlformats-officedocument.wordprocessingml.document');
    sessionStorage.clear();
  });

  it('asks for a file name before testing', async () => {
    const simulate = vi.fn();
    renderInShell(() => <AdminAttachmentPolicy load={load} simulate={simulate} />, { route: route('test') });
    fireEvent.click(await screen.findByRole('button', { name: 'Test' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Enter a file name');
    expect(simulate).not.toHaveBeenCalled();
  });

  it('keeps old per-key deep links: #/attachments/<key> opens the owning tab on that row', async () => {
    expect(attachmentTabForKey('attachments.maxBytes')).toBe('limits');
    expect(attachmentTabForKey('attachments.warnMimeTypes')).toBe('warnings');
    expect(attachmentRouteTab({ section: 'attachments', rest: ['attachments.previewAllowed'] })).toEqual({ tab: 'delivery', focusKey: 'attachments.previewAllowed' });
    expect(ATTACHMENT_TABS.map((tab) => tab.id)).toEqual(['lists', 'limits', 'delivery', 'warnings', 'test']);
    renderInShell(() => <AdminAttachmentPolicy load={load} />, { route: route('attachments.previewAllowed') });
    await screen.findAllByTestId('inline-setting');
    const focused = document.querySelector('[data-setting-key="attachments.previewAllowed"]') as HTMLElement;
    await waitFor(() => expect(focused).toContainElement(document.activeElement as HTMLElement));
    expect(screen.getByRole('navigation', { name: 'Attachment policy' })).toHaveTextContent('Download and preview');
  });

  it('reports unavailable configuration rather than inventing policy defaults', async () => {
    renderInShell(() => <AdminAttachmentPolicy load={async () => ({ kind: 'unavailable' })} />, { route: route() });
    expect(await screen.findByRole('alert')).toHaveTextContent('not available');
    expect(screen.queryByTestId('inline-setting')).toBeNull();
  });
});
