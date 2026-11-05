import { describe, it, expect, beforeEach,vi } from 'vitest';
import {saveDraft,deleteDraft} from './compose/drafts-store.ts';
import { screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import { clearSettings, loadSettings } from '../api/settings.ts';

describe('MessageList (virtualized)', () => {
  beforeEach(() => localStorage.clear());
  it('shows pending reply separately from actual answered state with a direct sibling draft action',async()=>{
    const open=vi.fn(),{app,result}=renderWithApp(()=><MessageList onOpenReplyDraft={open}/>,{emails:[mkEmail('pending'),mkEmail('answered',{keywords:{$answered:true,$seen:true}})]});
    await app.login({username:'me@example.org',password:'p'});await screen.findByText('Subject pending');
    saveDraft({id:'reply-draft',accountId:'acct1',storageAuthority:'test-storage-authority-acct1',to:'a@example.org',subject:'Re: Subject pending',bodyHtml:'<p>Draft</p>',bodyText:'Draft',savedAt:1,source:{emailId:'pending',keyword:'$answered'}});
    const control=await screen.findByRole('button',{name:'Open draft'}),slot=control.closest('li')!;
    expect(slot.querySelector('[data-testid="row-reply-draft"]')).not.toBeNull();expect(slot.querySelector('[data-testid="row-answered"]')).toBeNull();expect(control.parentElement?.closest('button')).toBeNull();
    fireEvent.click(control);expect(open).toHaveBeenCalledWith('reply-draft');expect(app.openEmail()).toBeNull();expect(result.container.querySelectorAll('[data-testid="row-answered"]')).toHaveLength(1);
    deleteDraft('reply-draft','acct1','test-storage-authority-acct1');await waitFor(()=>expect(screen.queryByRole('button',{name:'Open draft'})).toBeNull());
  });

  it('keeps sparse message previews readable when leetspeak is enabled and restored', async () => {
    // The live JMAP fixture can return null previews even though the static model
    // annotates this field as a string. Rendering used to tolerate those values.
    const sparse = mkEmail('sparse', { subject: 'Readable message', preview: null as unknown as string });
    const { app, result } = renderWithApp(() => <MessageList />, { emails: [sparse] });
    await app.login({ username: 'me@example.org', password: 'p' });
    await screen.findByText('Readable message');
    try {
      await loadSettings({ refresh: true, fetcher: async () => new Response(JSON.stringify({ settings: { 'appearance.leetspeak': { value: true, source: 'user', locked: false } } }), { headers: { 'content-type': 'application/json' } }) });
      await screen.findByText('R34d48l3 m355493');
      expect(result.container.querySelector('.list__preview')?.textContent).toBe('');
      expect(app.messages()[0]?.subject).toBe('Readable message');
      expect(app.messages()[0]?.preview).toBeNull();
      clearSettings();
      await screen.findByText('Readable message');
    } finally { clearSettings(); }
  });

  it('mounts only a window of rows for a huge list', async () => {
    const emails = Array.from({ length: 5000 }, (_, i) =>
      mkEmail(`m${i}`, { subject: `Message ${i}` }),
    );
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ username: 'me@example.org', password: 'p' });

    await waitFor(() => expect(app.messages().length).toBe(5000));
    // Only the viewport window (± overscan) is in the DOM, not all 5000 rows.
    const rows = result.container.querySelectorAll('.list__row');
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.length).toBeLessThan(40);
    // The spacer still reflects the full list height so the scrollbar is honest.
    const items = result.container.querySelector('.list__items') as HTMLElement;
    expect(items.style.height).toBe(`${5000 * 72}px`);
  });

  it('renders tag chips from the registry for a labeled message', async () => {
    const { app } = renderWithApp(() => <MessageList />, {
      emails: [mkEmail('a', { keywords: { $seen: true, work: true } })],
    });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(1));
    // 'work' is a seeded registry tag → its display name renders as a chip.
    expect(await screen.findByText('Work')).toBeInTheDocument();
  });

  it('exposes list semantics: each row announces its position in the full list', async () => {
    const emails = Array.from({ length: 50 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(50));

    const firstItem = result.container.querySelector('.list__slot') as HTMLElement;
    expect(firstItem.getAttribute('role')).toBe('listitem');
    expect(firstItem.getAttribute('aria-setsize')).toBe('50');
    expect(firstItem.getAttribute('aria-posinset')).toBe('1');
    // Roving tabindex: the cursor row is the single tab stop.
    const firstRow = firstItem.querySelector('.list__row') as HTMLElement;
    expect(firstRow.getAttribute('tabindex')).toBe('0');
  });

  it('floats a pinned message to the top row', async () => {
    const { app, result } = renderWithApp(() => <MessageList />, {
      emails: [mkEmail('a', { subject: 'Alpha' }), mkEmail('b', { subject: 'Bravo', pinned: true })],
    });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(2));
    const firstRow = result.container.querySelector('.list__row') as HTMLElement;
    expect(firstRow.textContent).toContain('Bravo');
  });

  // ── W17 density → virtualized row height ──────────────────────────────────
  it('scales the virtualized row height with the density preference', async () => {
    const emails = Array.from({ length: 20 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(20));

    const items = () => result.container.querySelector('.list__items') as HTMLElement;
    // Default density (cozy) keeps the historical 72px row.
    expect(items().style.height).toBe(`${20 * 72}px`);

    app.setDensity('compact');
    await waitFor(() => expect(items().style.height).toBe(`${20 * 56}px`));

    app.setDensity('relaxed');
    await waitFor(() => expect(items().style.height).toBe(`${20 * 88}px`));
  });

  // ── W2 conversation threading ─────────────────────────────────────────────
  it('folds a shared-threadId conversation into one expandable head row', async () => {
    const emails = [
      mkEmail('m3', { threadId: 't1', subject: 'Re: Hi', receivedAt: '2026-01-03T00:00:00Z' }),
      mkEmail('m2', { threadId: 't1', subject: 'Re: Hi', receivedAt: '2026-01-02T00:00:00Z' }),
      mkEmail('m1', { threadId: 't1', subject: 'Hi', receivedAt: '2026-01-01T00:00:00Z' }),
      mkEmail('solo', { subject: 'Alone' }),
    ];
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(4));

    // One conversation head (rep = newest 'Re: Hi') + one singleton row.
    const heads = result.container.querySelectorAll('[data-testid="thread-head"]');
    expect(heads.length).toBe(1);
    // The oldest member's subject ('Hi') is hidden until the thread expands.
    expect(screen.queryByText('Hi')).toBeNull();

    fireEvent.click(screen.getByTestId('thread-toggle'));

    // Expanded: the member with subject 'Hi' is now mounted.
    await waitFor(() => expect(screen.queryByText('Hi')).not.toBeNull());
    expect(screen.getByTestId('thread-toggle').getAttribute('aria-expanded')).toBe('true');
  });

  it('leaves a thread-less list rendering exactly as a flat list (no thread heads)', async () => {
    const emails = Array.from({ length: 8 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(8));
    expect(result.container.querySelectorAll('[data-testid="thread-head"]').length).toBe(0);
    expect(result.container.querySelectorAll('.list__row').length).toBe(8);
  });

  it('uses the table mode and hides body previews when effective settings request it',async()=>{
    const {app,result}=renderWithApp(()=><MessageList/>,{emails:[mkEmail('a',{preview:'Private preview'})]});
    await app.login({username:'me@example.org',password:'p'});
    try{
      await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.messageListStyle':{value:'table',source:'global',locked:false},'mail.messageListPreview':{value:false,source:'user',locked:false}}}),{headers:{'content-type':'application/json'}})});
      expect(result.container.querySelector('.list')).toHaveAttribute('data-list-style','table');
      expect(result.container.querySelector('.list__preview')).toBeNull();
      expect(screen.getByRole('combobox',{name:'Message order'})).toBeInTheDocument();
    }finally{clearSettings();}
  });

  // ── W3 reading-pane position control ──────────────────────────────────────
  it('drives the reading-pane position from the list toolbar', async () => {
    const { app } = renderWithApp(() => <MessageList />, { emails: [mkEmail('a')] });
    await app.login({ username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(1));

    const root = document.documentElement;
    fireEvent.click(screen.getByRole('button',{name:'Reading pane'}));fireEvent.click(screen.getByRole('menuitemradio',{name:'Bottom'}));
    expect(root.getAttribute('data-reading-pane')).toBe('bottom');
    expect(screen.getByRole('button',{name:'Reading pane'})).toHaveAttribute('title','Reading pane: Bottom');

    fireEvent.click(screen.getByRole('button',{name:'Reading pane'}));fireEvent.click(screen.getByRole('menuitemradio',{name:'Off'}));
    expect(root.getAttribute('data-reading-pane')).toBe('off');

    // Reset to the default so the module singleton doesn't leak into other specs.
    fireEvent.click(screen.getByRole('button',{name:'Reading pane'}));fireEvent.click(screen.getByRole('menuitemradio',{name:'Right'}));
    expect(root.getAttribute('data-reading-pane')).toBe('right');
  });
});
