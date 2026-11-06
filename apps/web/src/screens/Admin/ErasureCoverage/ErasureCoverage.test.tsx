import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import ErasureCoverage, { type ScrubApi } from './index.tsx';
import type { ScrubResult, ScrubState } from '../../../api/admin/scrub.ts';
import { renderInShell } from '../shell/testkit.tsx';

const state = (): ScrubState => ({
  settings: { enabled: true, pseudonymAlgorithm: 'hmac-sha256-16', includeLogs: true, autoRunOnErasure: false, keyRetentionDays: 0, propagateErasure: true, tombstoneRetentionDays: 3650 },
  keys: [{ gen: 1, createdAt: 1_800_000_000, retiredAt: null, destroyAfter: null }],
  auditChain: { rows: 4, unsealed: 0, gaps: 0, ok: true, firstBadSeq: null, headMatches: true, rechains: 1 },
  anchors: [{ id: 'a1', ts: 1_800_000_100, scrubId: 'scrubbed-1-abc', fromSeq: 2, toSeq: 4, oldHeadHash: 'aa', newHeadHash: 'bb', rowsChanged: 2, algorithm: 'hmac-sha256-16', keyGen: 1 }],
  tombstoneWatermark: 3,
  backups: [
    { id: 'b1', kind: 'store', label: 'nightly', location: '/backups/a.sqlite', createdAt: 1_800_000_000, tombstoneWatermark: 1, erasedSince: 2 },
    { id: 'b2', kind: 'store', label: 'fresh', location: '/backups/b.sqlite', createdAt: 1_800_000_200, tombstoneWatermark: 3, erasedSince: 0 },
  ],
  algorithms: ['hmac-sha256-16', 'hmac-sha256-32'],
});
const result = (dryRun: boolean): ScrubResult => ({ dryRun, counts: { audit_log: 2, log_lines: 5 }, errors: [], pseudonym: 'scrubbed-1-abc', keyGen: 1, anchor: dryRun ? null : 'a1', tombstone: dryRun ? null : 4 });
const api = (): ScrubApi => ({
  state: vi.fn(async () => state()), preview: vi.fn(async () => result(true)), run: vi.fn(async () => result(false)),
  rotate: vi.fn(async () => ({})), seal: vi.fn(async () => ({})), register: vi.fn(async () => ({})), remove: vi.fn(async () => ({})), settings: vi.fn(async () => []),
});

afterEach(() => cleanup());

describe('Admin erasure and scrubbing', () => {
  it('marks a backup that still holds people erased since it was made', async () => {
    renderInShell(() => <ErasureCoverage api={api()} />);
    const row = await screen.findByTestId('erasure-backup-b1');
    expect(row.textContent).toContain('2');
    expect(row.querySelector('[data-testid="erasure-marker"]')).not.toBeNull();
    expect(screen.getByTestId('erasure-backup-b2').querySelector('[data-testid="erasure-marker"]')).toBeNull();
  });

  it('previews counts first and runs the scrub only after confirmation', async () => {
    const a = api();
    renderInShell(() => <ErasureCoverage api={a} />);
    await screen.findByTestId('erasure-backups');
    expect((screen.getByTestId('erasure-run') as HTMLButtonElement).disabled).toBe(true);
    fireEvent.input(screen.getByTestId('erasure-address'), { target: { value: 'ana@example.org' } });
    fireEvent.click(screen.getByTestId('erasure-preview'));
    const out = await screen.findByTestId('erasure-result');
    expect(out.textContent).toContain('audit_log');
    expect(a.preview).toHaveBeenCalledWith('ana@example.org');
    expect(a.run).not.toHaveBeenCalled();
    fireEvent.click(screen.getByTestId('erasure-run'));
    const dialog = await screen.findByTestId('erasure-confirm');
    fireEvent.click(dialog.querySelector('button.btn--danger, button[data-confirm], .btn--danger') ?? dialog.querySelectorAll('button')[1]!);
    await waitFor(() => expect(a.run).toHaveBeenCalledWith('ana@example.org'));
  });

  it('shows the chain state and the anchors', async () => {
    renderInShell(() => <ErasureCoverage api={api()} />);
    expect((await screen.findByTestId('erasure-chain-state')).textContent).toContain('4');
    expect(screen.getByTestId('erasure-anchors').textContent).toContain('2 - 4');
  });
});
