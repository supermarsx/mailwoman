import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { JobsPanel, type JobsApi, type RetentionJob } from './JobsPanel.tsx';
import { renderInShell } from '../shell/testkit.tsx';

const job = (over: Partial<RetentionJob> = {}): RetentionJob => ({
  kind: 'gc_uploads', taskId: 't1', enabled: true, nextRunAt: 1_800_000_000, armAfter: 1_800_500_000,
  previewOnly: true, olderThanHours: 720,
  lastRun: { at: 1_799_990_000, status: 'ok', summary: 'preview', dryRun: true },
  lastPreview: { at: 1_799_990_000, wouldChange: 42, summary: 'preview' },
  ...over,
});

const api = (jobs: RetentionJob[], setEnabled?: JobsApi['setEnabled']): JobsApi => ({
  list: vi.fn(async () => ({ jobs, now: 1_800_000_000 })),
  setEnabled: setEnabled ?? vi.fn(async (kind, enabled) => job({ kind, enabled, previewOnly: false, armAfter: null })),
});

describe('Admin retention jobs', () => {
  it('shows the switch, the preview count and the date deleting starts', async () => {
    renderInShell(() => <JobsPanel api={api([
      job(),
      job({ kind: 'user_retention', enabled: false, previewOnly: false, armAfter: null, lastPreview: null }),
    ])} />);
    const row = await screen.findByTestId('retention-job-gc_uploads');
    expect((screen.getByTestId('retention-job-switch-gc_uploads') as HTMLInputElement).checked).toBe(true);
    expect(screen.getByTestId('retention-job-arms-gc_uploads')).toBeTruthy();
    expect(screen.getByTestId('retention-job-preview-gc_uploads').textContent).toContain('42');
    expect(row).toBeTruthy();
    expect((screen.getByTestId('retention-job-switch-user_retention') as HTMLInputElement).checked).toBe(false);
    expect(screen.queryByTestId('retention-job-arms-user_retention')).toBeNull();
    cleanup();
  });

  it('switches a job and shows what the server returned', async () => {
    const setEnabled = vi.fn(async (kind: RetentionJob['kind'], enabled: boolean) =>
      job({ kind, enabled, previewOnly: false, armAfter: null }));
    renderInShell(() => <JobsPanel api={api([job({ enabled: true })], setEnabled)} />);
    const sw = await screen.findByTestId('retention-job-switch-gc_uploads');
    fireEvent.change(sw, { target: { checked: false } });
    await waitFor(() => expect(setEnabled).toHaveBeenCalledWith('gc_uploads', false));
    await waitFor(() => expect((screen.getByTestId('retention-job-switch-gc_uploads') as HTMLInputElement).checked).toBe(false));
    expect(screen.queryByTestId('retention-job-arms-gc_uploads')).toBeNull();
    cleanup();
  });

  it('reports a failed switch', async () => {
    const setEnabled = vi.fn(async () => { throw new Error('refused'); });
    renderInShell(() => <JobsPanel api={api([job()], setEnabled)} />);
    fireEvent.change(await screen.findByTestId('retention-job-switch-gc_uploads'), { target: { checked: false } });
    await waitFor(() => expect(screen.getAllByRole('alert').length).toBeGreaterThan(0));
    cleanup();
  });
});
