import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { RetentionRules, type RetentionApi } from './RetentionRules.tsx';
import { blankRule, emptyScope, type Preview, type RetentionOverview, type Rule } from '../../api/retention.ts';
import { retentionFolder, setRetentionFolder } from '../../state/retentionFocus.ts';
import type { Mailbox } from '../../api/jmap-types.ts';

const box = (id: string, name: string, role: string | null = null): Mailbox => ({
  id, name, role, parentId: null, sortOrder: 0, totalEmails: 0, unreadEmails: 0,
} as Mailbox);
const folders = [box('inbox', 'Inbox', 'inbox'), box('work', 'Work')];

const rule = (over: Partial<Rule> = {}): Rule => ({
  id: 'r1', name: 'Newsletters', enabled: true, order: 1, template: null, createdAt: 1, updatedAt: 1,
  scope: { ...emptyScope(), labels: ['newsletter'] }, action: { kind: 'delete', days: 30, target: null },
  exclude: { flagged: true, unread: false, tags: ['keep'] }, ...over,
});

const overview = (over: Partial<RetentionOverview> = {}): RetentionOverview => ({
  policy: { enabled: true, minDays: 7, maxDays: 365, maxRules: 3, allowPermanentDelete: false, undoDays: 7, mandatory: [{ role: 'trash', days: 30 }], protectedFolders: ['drafts'] },
  settings: { enabled: true, trashFirst: true, undoDays: 7, pausedUntil: 0, paused: false, lastRunAt: 1_800_000_000, nextRunAt: null },
  folderDefaults: [], rules: [rule()],
  templates: [{ id: 'trash30', name: 'Trash: delete after 30 days', rule: { scope: emptyScope(), action: { kind: 'delete', days: 30, target: null }, exclude: blankRule().exclude } }],
  runs: [{ id: 'x', startedAt: 1_800_000_000, finishedAt: 1_800_000_010, trigger: 'scheduler', matched: 12, acted: 10, errors: 0 }],
  ...over,
});

const preview = (over: Partial<Preview> = {}): Preview => ({
  skipped: null, scanned: 100, matched: 1243, would: { moved: 0, trashed: 1243, purged: 0, markedRead: 0, stamped: 0 },
  done: { moved: 0, trashed: 0, purged: 0, markedRead: 0, stamped: 0 }, errors: 0, truncated: false, bySource: {},
  sample: [{ id: 'm1', folder: 'inbox', subject: 'Weekly deals', from: 'shop@example.test', receivedAt: 1_700_000_000, action: 'trash', source: 'rule:r1' }],
  ...over,
});

const api = (over: Partial<RetentionApi> = {}): RetentionApi => ({
  load: vi.fn(async () => overview()),
  create: vi.fn(async () => rule({ id: 'r2' })),
  update: vi.fn(async () => rule()),
  remove: vi.fn(async () => ({ deleted: true })),
  adopt: vi.fn(async () => rule({ id: 'r3' })),
  settings: vi.fn(async () => overview()),
  preview: vi.fn(async () => preview()),
  run: vi.fn(async () => preview({ done: { moved: 2, trashed: 5, purged: 1, markedRead: 0, stamped: 0 } })),
  ...over,
});

afterEach(() => { cleanup(); setRetentionFolder(null); });

describe('Settings > Retention', () => {
  it('shows the rules, the limits the server enforces and the last run', async () => {
    render(() => <RetentionRules folders={() => folders} api={api()} />);
    expect(await screen.findByTestId('retention-rule-r1')).toHaveTextContent('Newsletters');
    expect(screen.getByTestId('retention-policy')).toHaveTextContent('at least 7 days and at most 365 days');
    expect(screen.getByTestId('retention-policy')).toHaveTextContent('Rules cannot delete mail permanently');
    expect(screen.getByTestId('retention-policy')).toHaveTextContent('older than 30 days is deleted');
    expect(screen.getByTestId('retention-runs')).toHaveTextContent('12');
  });

  it('keeps the Trash step when the administrator does not allow permanent deletion', async () => {
    render(() => <RetentionRules folders={() => folders} api={api()} />);
    const toggle = await screen.findByTestId('retention-trash-first');
    expect(toggle).toBeChecked();
    expect(toggle).toBeDisabled();
  });

  it('previews a rule with the count and examples before it is saved', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    fireEvent.click(await screen.findByTestId('retention-add'));
    const editor = await screen.findByTestId('retention-editor');
    fireEvent.input(within(editor).getByLabelText('Name'), { target: { value: 'Old mail' } });
    fireEvent.click(within(editor).getByTestId('retention-preview'));
    const result = await screen.findByTestId('retention-preview-result');
    expect(result).toHaveTextContent('This would change 1,243 message(s).');
    expect(result).toHaveTextContent('Weekly deals');
    expect(a.preview).toHaveBeenCalledTimes(1);
    expect(a.create).not.toHaveBeenCalled();
  });

  it('refuses a deleting rule below the floor without asking the server', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    fireEvent.click(await screen.findByTestId('retention-add'));
    const editor = await screen.findByTestId('retention-editor');
    fireEvent.input(within(editor).getByLabelText('Name'), { target: { value: 'Quick' } });
    fireEvent.input(within(editor).getByLabelText('After days'), { target: { value: '2' } });
    expect(within(editor).getByTestId('retention-preview')).toBeDisabled();
    fireEvent.click(within(editor).getByTestId('retention-save'));
    expect(await within(editor).findByRole('alert')).toHaveTextContent('at least 7 days');
    expect(a.create).not.toHaveBeenCalled();
  });

  it('saves a rule and reloads', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    fireEvent.click(await screen.findByTestId('retention-add'));
    const editor = await screen.findByTestId('retention-editor');
    fireEvent.input(within(editor).getByLabelText('Name'), { target: { value: 'Old mail' } });
    fireEvent.input(within(editor).getByLabelText('Labels (comma separated)'), { target: { value: 'promo, ads' } });
    fireEvent.click(within(editor).getByTestId('retention-save'));
    await waitFor(() => expect(a.create).toHaveBeenCalledTimes(1));
    const saved = vi.mocked(a.create).mock.calls[0]![0];
    expect(saved.name).toBe('Old mail');
    expect(saved.scope.labels).toEqual(['promo', 'ads']);
    expect(saved.exclude.tags).toEqual(['keep']);
    await waitFor(() => expect(screen.queryByTestId('retention-editor')).toBeNull());
    expect(a.load).toHaveBeenCalledTimes(2);
  });

  it('pauses a single rule, adopts a template and deletes a rule', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    const row = await screen.findByTestId('retention-rule-r1');
    fireEvent.click(within(row).getByRole('checkbox'));
    await waitFor(() => expect(a.update).toHaveBeenCalledWith('r1', expect.objectContaining({ enabled: false })));
    await waitFor(() => expect(screen.getByTestId('retention-adopt-trash30')).toBeEnabled());
    fireEvent.click(screen.getByTestId('retention-adopt-trash30'));
    await waitFor(() => expect(a.adopt).toHaveBeenCalledWith('trash30'));
    await waitFor(() => expect(within(screen.getByTestId('retention-rule-r1')).getByText('Delete')).toBeEnabled());
    fireEvent.click(within(screen.getByTestId('retention-rule-r1')).getByText('Delete'));
    await waitFor(() => expect(a.remove).toHaveBeenCalledWith('r1'));
  });

  it('stops adding rules at the administrator limit', async () => {
    const rules = [rule({ id: 'a' }), rule({ id: 'b' }), rule({ id: 'c' })];
    render(() => <RetentionRules folders={() => folders} api={api({ load: vi.fn(async () => overview({ rules })) })} />);
    expect(await screen.findByTestId('retention-add')).toBeDisabled();
    expect(screen.getByTestId('retention-adopt-trash30')).toBeDisabled();
  });

  it('pauses and resumes every rule', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    fireEvent.click(await screen.findByTestId('retention-pause'));
    await waitFor(() => expect(a.settings).toHaveBeenCalledWith({ pausedUntil: expect.any(Number) }));
    cleanup();
    const paused = overview({ settings: { ...overview().settings, paused: true, pausedUntil: 1_900_000_000 } });
    const b = api({ load: vi.fn(async () => paused) });
    render(() => <RetentionRules folders={() => folders} api={b} />);
    fireEvent.click(await screen.findByTestId('retention-resume'));
    await waitFor(() => expect(b.settings).toHaveBeenCalledWith({ pausedUntil: 0 }));
  });

  it('runs now and reports the count, or why nothing ran', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    fireEvent.click(await screen.findByTestId('retention-run'));
    expect(await screen.findByTestId('retention-summary')).toHaveTextContent('8 message(s) changed.');
    cleanup();
    render(() => <RetentionRules folders={() => folders} api={api({ run: vi.fn(async () => preview({ skipped: 'legalHold', matched: 0, sample: [] })) })} />);
    fireEvent.click(await screen.findByTestId('retention-run'));
    expect(await screen.findByTestId('retention-summary')).toHaveTextContent('legal hold');
  });

  it('starts a rule for the folder chosen in the folder menu', async () => {
    setRetentionFolder('work');
    render(() => <RetentionRules folders={() => folders} api={api()} />);
    const editor = await screen.findByTestId('retention-editor');
    expect(within(editor).getByLabelText('Name')).toHaveValue('Work');
    expect(within(editor).getByLabelText('Folder', { selector: '#ret-folder' })).toHaveValue('work');
    expect(retentionFolder()).toBeNull();
  });

  it('says so when the feature is off on the server', async () => {
    const off = overview({ policy: { ...overview().policy, enabled: false } });
    render(() => <RetentionRules folders={() => folders} api={api({ load: vi.fn(async () => off) })} />);
    expect(await screen.findByText(/turned off on this server/)).toBeInTheDocument();
    expect(screen.getByTestId('retention-add')).toBeDisabled();
  });

  it('edits per-folder defaults and saves them through the settings call', async () => {
    const a = api();
    render(() => <RetentionRules folders={() => folders} api={a} />);
    const row = await screen.findByTestId('retention-fd-work');
    expect(screen.getByTestId('retention-fd-limits')).toHaveTextContent('at least 7 days and at most 365 days');
    expect(screen.getByTestId('retention-fd-inbox')).toBeInTheDocument();
    const select = within(row).getByLabelText('What happens to mail in Work');
    fireEvent.change(select, { target: { value: 'trash' } });
    const days = within(row).getByLabelText('Days before Work is processed');
    fireEvent.input(days, { target: { value: '3' } });
    expect(screen.getByTestId('retention-fd-problems')).toHaveTextContent('Work');
    expect(screen.getByTestId('retention-fd-save')).toBeDisabled();
    fireEvent.input(days, { target: { value: '45' } });
    fireEvent.click(screen.getByTestId('retention-fd-save'));
    await waitFor(() => expect(a.settings).toHaveBeenCalledWith({ folderDefaults: [expect.objectContaining({ folder: 'work', name: 'Work', action: { kind: 'trash', days: 45, target: null } })] }));
  });
});

describe('Settings > Retention: removing attachments', () => {
  const stripRule = (): Rule => rule({ id: 'r9', name: 'Big files', action: { kind: 'stripAttachments', days: 120, target: null }, scope: { ...emptyScope(), hasAttachment: true } });
  const allowed = (): RetentionOverview => overview({ policy: { ...overview().policy, allowStripAttachments: true, strip: { minAgeDays: 90, minAttachmentKb: 100, keepInlineImages: true, undoDays: 7 } } });

  it('hides the action and says why while the administrator has it off', async () => {
    const off = overview({ rules: [stripRule()] });
    render(() => <RetentionRules folders={() => folders} api={api({ load: vi.fn(async () => off) })} />);
    expect(await screen.findByTestId('retention-strip-locked')).toHaveTextContent('turned off by your administrator');
    expect(screen.getByTestId('retention-rule-strip-off')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('retention-add'));
    const editor = await screen.findByTestId('retention-editor');
    const options = [...within(editor).getByLabelText('What to do').querySelectorAll('option')].map((o) => (o as HTMLOptionElement).value);
    expect(options).toContain('markRead');
    expect(options).not.toContain('stripAttachments');
  });

  it('offers the action when allowed, refuses a short wait locally and previews the freed space', async () => {
    const a = api({
      load: vi.fn(async () => allowed()),
      preview: vi.fn(async () => preview({ matched: 4, would: { moved: 0, trashed: 0, purged: 0, markedRead: 0, stamped: 0, stripped: 4, reclaimedBytes: 5 * 1024 * 1024 }, sample: [] })),
    });
    render(() => <RetentionRules folders={() => folders} api={a} />);
    expect(await screen.findByTestId('retention-strip-allowed')).toHaveTextContent('100 KB');
    fireEvent.click(screen.getByTestId('retention-add'));
    const editor = await screen.findByTestId('retention-editor');
    fireEvent.input(within(editor).getByLabelText('Name'), { target: { value: 'Big files' } });
    fireEvent.change(within(editor).getByLabelText('What to do'), { target: { value: 'stripAttachments' } });
    expect(within(editor).getByTestId('retention-strip-limits')).toHaveTextContent('at least 100 KB');
    fireEvent.input(within(editor).getByLabelText('After days'), { target: { value: '30' } });
    fireEvent.click(within(editor).getByTestId('retention-save'));
    expect(await within(editor).findByRole('alert')).toHaveTextContent('must wait at least 90 days');
    expect(a.create).not.toHaveBeenCalled();
    fireEvent.input(within(editor).getByLabelText('After days'), { target: { value: '120' } });
    fireEvent.click(within(editor).getByTestId('retention-preview'));
    expect(await screen.findByTestId('retention-strip-preview')).toHaveTextContent('4 message(s) would lose attachments, freeing 5.0 MB');
  });
});
