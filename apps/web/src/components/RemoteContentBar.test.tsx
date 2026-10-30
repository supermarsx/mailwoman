// Remote-images banner (t55): one line, short wording, counts only when known,
// Show images / Undo, the less common grant scopes in a menu, no banner text
// that reads "0", and no re-render storm while counts settle.
import { describe, it, expect, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { RemoteContentBar } from './RemoteContentBar.tsx';
import type { BlockedContentReport, RemoteImageGrant } from '../api/remote-images.ts';

function report(over: Partial<BlockedContentReport> = {}): BlockedContentReport {
  return { blockedHosts: ['cdn.example', 'tracker.evil'], blockedCount: 3, trackerCount: 0, ...over };
}

const CTX = { emailId: 'M1', sender: 'bob@spam.example' } as const;
const banner = (): HTMLElement => screen.getByTestId('remote-content-bar');
const openMenu = (): void => { fireEvent.click(screen.getByRole('button', { name: 'More image options' })); };

describe('RemoteContentBar — blocked (ask)', () => {
  it('is one short line with no count when no trackers are known', () => {
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    expect(screen.getByRole('status')).toHaveTextContent(/^Remote images blocked$/);
    expect(banner().textContent).not.toMatch(/\b0\b/);
    // The host list and explanation are in the tooltip, not the row.
    expect(screen.queryByText('cdn.example')).toBeNull();
    const tip = screen.getByRole('status').getAttribute('data-tooltip') ?? '';
    expect(tip).toContain('cdn.example');
    expect(tip).toContain('tracker.evil');
  });

  it('adds the tracker count (HTML + server-decoded) only when above zero', () => {
    render(() => <RemoteContentBar {...CTX} report={report({ trackerCount: 2 })} decodedTrackers={{ count: 1, limited: false }} />);
    expect(screen.getByRole('status')).toHaveTextContent('Remote images blocked · 3 trackers removed');
  });

  it('Show images grants the single-message scope', () => {
    const onGrant = vi.fn().mockResolvedValue(undefined);
    render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    expect(onGrant).toHaveBeenCalledWith({ kind: 'single', value: 'M1' }, expect.anything());
  });

  const MENU: [RegExp, { kind: string; value: string }][] = [
    [/Always show images from .bob@spam\.example./, { kind: 'per-sender', value: 'bob@spam.example' }],
    [/Always show images from .spam\.example.$/, { kind: 'per-domain', value: 'spam.example' }],
    [/Always show images in all mail/, { kind: 'all', value: '' }],
  ];
  for (const [label, scope] of MENU) {
    it(`menu "${String(label)}" grants ${scope.kind}`, () => {
      const onGrant = vi.fn().mockResolvedValue(undefined);
      const { unmount } = render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
      openMenu();
      fireEvent.click(screen.getByRole('menuitem', { name: label }));
      expect(onGrant).toHaveBeenCalledWith(scope, expect.anything());
      unmount();
    });
  }

  it('the menu links to privacy settings only when the reader can open them', () => {
    const open = vi.fn();
    const { unmount } = render(() => <RemoteContentBar {...CTX} report={report()} onOpenPrivacySettings={open} />);
    openMenu();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Privacy settings' }));
    expect(open).toHaveBeenCalledOnce();
    unmount();
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    openMenu();
    expect(screen.queryByRole('menuitem', { name: 'Privacy settings' })).toBeNull();
  });

  it('drops the sender and domain choices that cannot be expressed', () => {
    render(() => <RemoteContentBar emailId="M1" sender="" report={report()} />);
    openMenu();
    expect(screen.queryByRole('menuitem', { name: /Always show images from/ })).toBeNull();
    expect(screen.getByRole('menuitem', { name: 'Always show images in all mail' })).toBeInTheDocument();
  });

  it('a failed grant raises an error toast and keeps the blocked state', async () => {
    const onGrant = vi.fn().mockRejectedValue(new Error('x'));
    render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await Promise.resolve();
    expect(screen.getByRole('status')).toHaveTextContent('Remote images blocked');
  });
});

describe('RemoteContentBar — shown (ask, granted)', () => {
  const grant: RemoteImageGrant = { scopeKind: 'per-sender', scopeValue: 'bob@spam.example', grantedAt: '2026-07-19T00:00:00Z' };

  it('collapses to "Images shown · Undo"', () => {
    render(() => <RemoteContentBar {...CTX} report={report({ blockedCount: 0, blockedHosts: [] })} activeGrant={grant} />);
    expect(screen.getByRole('status')).toHaveTextContent(/^Images shown$/);
    expect(screen.getByRole('button', { name: /^Undo/ })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Show images' })).toBeNull();
  });

  it('Undo revokes the covering grant', () => {
    const onRevoke = vi.fn().mockResolvedValue(undefined);
    render(() => <RemoteContentBar {...CTX} report={report()} activeGrant={grant} onRevoke={onRevoke} />);
    fireEvent.click(screen.getByRole('button', { name: /^Undo/ }));
    expect(onRevoke).toHaveBeenCalledWith({ kind: 'per-sender', value: 'bob@spam.example' }, expect.anything());
  });

  it('show → shown → undo → blocked in one mounted banner', async () => {
    const [g, setG] = createSignal<RemoteImageGrant | null>(null);
    render(() => <RemoteContentBar {...CTX} report={report()} activeGrant={g()} onGrant={(s) => { setG({ scopeKind: s.kind, scopeValue: s.value, grantedAt: '' }); }} onRevoke={() => { setG(null); }} />);
    const el = banner();
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await Promise.resolve(); await Promise.resolve();
    expect(screen.getByRole('status')).toHaveTextContent('Images shown');
    fireEvent.click(screen.getByRole('button', { name: /^Undo/ }));
    await Promise.resolve(); await Promise.resolve();
    expect(screen.getByRole('status')).toHaveTextContent('Remote images blocked');
    expect(banner()).toBe(el);
  });
});

describe('RemoteContentBar — other policy modes', () => {
  it('proxy mode states what happened and offers no grant', () => {
    render(() => <RemoteContentBar {...CTX} mode="proxy" report={report({ trackerCount: 1 })} onOpenPrivacySettings={() => undefined} />);
    expect(screen.getByRole('status')).toHaveTextContent('Images loaded through the server · 1 tracker removed');
    expect(screen.queryByRole('button', { name: 'Show images' })).toBeNull();
    openMenu();
    expect(screen.getAllByRole('menuitem').map((m) => m.textContent)).toEqual(['Privacy settings']);
  });

  it('block mode never offers a grant and never shows a zero count', () => {
    render(() => <RemoteContentBar {...CTX} mode="block" report={report()} decodedTrackers={{ count: 0, limited: false }} />);
    expect(screen.getByRole('status')).toHaveTextContent(/^Remote images blocked by your privacy settings$/);
    expect(screen.queryByRole('button', { name: 'Show images' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'More image options' })).toBeNull();
  });
});

describe('RemoteContentBar — incremental counts', () => {
  it('a stream of count updates keeps one node, changes text only when the count changes, never says 0', () => {
    const [decoded, setDecoded] = createSignal<{ count: number; limited: boolean } | null>(null);
    render(() => <RemoteContentBar {...CTX} mode="proxy" report={report()} decodedTrackers={decoded()} />);
    const status = screen.getByRole('status');
    const texts = new Set<string>([status.textContent ?? '']);
    const observer = new MutationObserver(() => texts.add(status.textContent ?? ''));
    observer.observe(status, { subtree: true, childList: true, characterData: true });
    for (const n of [0, 0, 2, 2, 2]) setDecoded({ count: n, limited: false });
    observer.takeRecords().forEach(() => texts.add(status.textContent ?? ''));
    texts.add(status.textContent ?? '');
    observer.disconnect();
    expect([...texts]).toEqual(['Images loaded through the server', 'Images loaded through the server · 2 trackers removed']);
    expect(screen.getByRole('status')).toBe(status);
  });
});

// t82: the write recovers by itself where that is safe; the banner shows no
// toast for an attempt that later succeeds and ONE toast with a working Retry
// on the final failure.
describe('RemoteContentBar — recovery (t82)', () => {
  it('shows no error toast when the grant succeeds after retries', async () => {
    const { toasts, resetToasts } = await import('./toast/toast.ts');
    resetToasts();
    const onGrant = vi.fn(async (_scope: unknown, options?: { onSlow?: (s: 'retrying' | 'offline' | null) => void }) => {
      options?.onSlow?.('retrying');
      await Promise.resolve();
      options?.onSlow?.(null);
    });
    render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await vi.waitFor(() => expect(onGrant).toHaveBeenCalledTimes(1));
    await Promise.resolve(); await Promise.resolve();
    expect(toasts().filter((t) => t.tone === 'error')).toHaveLength(0);
  });

  it('says "Retrying…" on the button while recovery runs long', async () => {
    let finish!: () => void;
    const onGrant = vi.fn((_scope: unknown, options?: { onSlow?: (s: 'retrying' | 'offline' | null) => void }) => new Promise<void>((resolve) => { options?.onSlow?.('retrying'); finish = resolve; }));
    render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Show images' })).toHaveTextContent('Retrying…'));
    expect(screen.getByRole('button', { name: 'Show images' })).toHaveAttribute('aria-busy', 'true');
    finish();
    await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Show images' })).toHaveTextContent(/^Show images$/));
  });

  it('a final failure shows one toast with the reason, and its Retry grants', async () => {
    const { toasts, resetToasts } = await import('./toast/toast.ts');
    const { RecoveryError } = await import('../api/recovery.ts');
    resetToasts();
    const onGrant = vi.fn().mockRejectedValueOnce(new RecoveryError('server')).mockRejectedValueOnce(new RecoveryError('server')).mockResolvedValue(undefined);
    render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await vi.waitFor(() => expect(toasts().filter((t) => t.tone === 'error')).toHaveLength(1));
    fireEvent.click(screen.getByRole('button', { name: 'Show images' }));
    await vi.waitFor(() => expect(onGrant).toHaveBeenCalledTimes(2));
    await Promise.resolve();
    const errors = toasts().filter((t) => t.tone === 'error');
    expect(errors).toHaveLength(1); // the second failure replaced the first
    expect(errors[0]?.message).toBe('Could not update remote image permissions. The server did not answer in time.');
    await errors[0]?.action?.run();
    await vi.waitFor(() => expect(onGrant).toHaveBeenCalledTimes(3));
    expect(onGrant).toHaveBeenLastCalledWith({ kind: 'single', value: 'M1' }, expect.anything());
    await vi.waitFor(() => expect(toasts().filter((t) => t.tone === 'error')).toHaveLength(0));
  });
});


describe('RemoteContentBar failure recovery',()=>{
  it('keeps Undo and offers Retry for failed shown images without changing consent',()=>{
    const retry=vi.fn(),revoke=vi.fn();
    render(()=><RemoteContentBar {...CTX} report={report()} activeGrant={{scopeKind:'single',scopeValue:'M1',grantedAt:''}} imageFailures={{count:2,retrying:false}} onRetryImages={retry} onRevoke={revoke}/>);
    expect(screen.getByRole('button',{name:/^Undo/})).toBeVisible();fireEvent.click(screen.getByRole('button',{name:'Retry'}));expect(retry).toHaveBeenCalledOnce();expect(revoke).not.toHaveBeenCalled();
  });
  it('never offers Retry after privacy policy blocks the images',()=>{
    render(()=><RemoteContentBar {...CTX} mode="block" report={report()} imageFailures={{count:2,retrying:false}} onRetryImages={()=>undefined}/>);
    expect(screen.queryByRole('button',{name:'Retry'})).toBeNull();
  });
});
