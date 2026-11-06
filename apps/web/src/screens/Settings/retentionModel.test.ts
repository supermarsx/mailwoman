import { describe, expect, it } from 'vitest';
import { actionMessage, daysFromNow, defaultsPayload, flattenFolders, folderProblem, isDestructive, joinList, needsDays, ruleProblem, settingsFromDefaults, splitList } from './retentionModel.ts';
import { blankRule, emptyScope, type FolderDefault, type RetentionPolicy } from '../../api/retention.ts';

const policy = (over: Partial<RetentionPolicy> = {}): RetentionPolicy => ({
  enabled: true, minDays: 30, maxDays: 365, maxRules: 50, allowPermanentDelete: false, undoDays: 7, mandatory: [], protectedFolders: ['drafts'], ...over,
});

describe('retention model', () => {
  it('splits comma and line separated lists without empties or duplicates', () => {
    expect(splitList('a, b ,,c\nb')).toEqual(['a', 'b', 'c']);
    expect(splitList('  ')).toEqual([]);
    expect(joinList(['x', 'y'])).toBe('x, y');
  });

  it('knows which actions delete and which need a number of days', () => {
    expect(isDestructive('delete')).toBe(true);
    expect(isDestructive('trash')).toBe(true);
    expect(isDestructive('archive')).toBe(false);
    expect(needsDays('keep')).toBe(false);
    expect(needsDays('markRead')).toBe(true);
  });

  it('holds a deleting rule to the administrator floor and ceiling, as the server does', () => {
    const rule = blankRule();
    rule.name = 'News';
    expect(ruleProblem({ ...rule, action: { ...rule.action, days: 10 } }, policy())?.id).toBe('retention-error-min');
    expect(ruleProblem({ ...rule, action: { ...rule.action, days: 400 } }, policy())).toEqual({ id: 'retention-error-max', args: { days: 365 } });
    expect(ruleProblem({ ...rule, action: { ...rule.action, days: 400 } }, policy({ maxDays: 0 }))).toBeNull();
    // Archiving is not deleting, so the floor does not apply.
    expect(ruleProblem({ ...rule, action: { kind: 'archive', days: 1, target: null } }, policy())).toBeNull();
    expect(ruleProblem({ ...rule, action: { kind: 'move', days: 40, target: null } }, policy())?.id).toBe('retention-error-target');
    expect(ruleProblem({ ...rule, name: ' ' }, policy())?.id).toBe('retention-error-name');
    expect(ruleProblem({ ...rule, action: { ...rule.action, days: 1.5 } }, policy())?.id).toBe('retention-error-days');
  });

  it('describes what a rule does, with the permanent wording only when it applies', () => {
    const rule = blankRule();
    expect(actionMessage(rule, false)).toEqual({ id: 'retention-summary-delete', args: { days: 90 } });
    expect(actionMessage(rule, true).id).toBe('retention-summary-delete-permanent');
    expect(actionMessage({ ...rule, action: { kind: 'keep', days: 0, target: null } }, true).id).toBe('retention-summary-keep');
    expect(actionMessage({ ...rule, action: { kind: 'archive', days: 5, target: null } }, false).id).toBe('retention-summary-archive');
  });

  it('counts whole days to the next run', () => {
    expect(daysFromNow(1_000 + 86_400 + 1, 1_000)).toBe(2);
    expect(daysFromNow(500, 1_000)).toBe(0);
  });
});

describe('folder defaults', () => {
  const f = (id: string, name: string, parentId: string | null = null, sortOrder = 0) => ({ id, name, parentId, sortOrder, role: null });
  const def = (id: string, folder: string, kind: 'trash' | 'move' | 'delete', days: number, enabled = true): FolderDefault => ({
    id, name: folder, enabled, body: { scope: { ...emptyScope(), folders: [folder] }, action: { kind, days, target: kind === 'move' ? 'x' : null }, exclude: { flagged: true, unread: false, tags: ['k'] } },
  });

  it('orders folders as a tree with depth', () => {
    const flat = flattenFolders([f('c', 'Child', 'a'), f('b', 'B', null, 2), f('a', 'A', null, 1), f('o', 'Orphan', 'gone', 3)]);
    expect(flat.map((x) => [x.id, x.depth])).toEqual([['a', 0], ['c', 1], ['b', 0], ['o', 0]]);
  });

  it('keeps ids and exclusions of existing entries and entries it cannot express', () => {
    const folders = flattenFolders([f('a', 'A'), f('b', 'B'), f('c', 'C')]);
    const existing = [def('d1', 'a', 'trash', 60), def('d2', 'b', 'move', 10), def('d3', 'c', 'delete', 90)];
    const settings = settingsFromDefaults(existing);
    expect(settings.get('a')).toEqual({ choice: 'trash', days: 60 });
    expect(settings.has('b')).toBe(false);
    settings.set('a', { choice: 'trash', days: 45 });
    settings.set('c', { choice: 'inherit', days: 90 });
    const payload = defaultsPayload(folders, settings, existing);
    expect(payload.find((p) => p.folder === 'a')).toMatchObject({ id: 'd1', action: { kind: 'trash', days: 45 }, exclude: { tags: ['k'] } });
    expect(payload.find((p) => p.folder === 'b')).toMatchObject({ id: 'd2', action: { kind: 'move' } });
    expect(payload.some((p) => p.folder === 'c')).toBe(false);
  });

  it('adds a new default with the safe exclusions and zero days for keep', () => {
    const folders = flattenFolders([f('a', 'A')]);
    const payload = defaultsPayload(folders, new Map([['a', { choice: 'keep' as const, days: 99 }]]), []);
    expect(payload).toEqual([{ name: 'A', enabled: true, folder: 'a', action: { kind: 'keep', days: 0, target: null }, exclude: { flagged: true, unread: false, tags: ['keep'] } }]);
  });

  it('holds destructive choices to the administrator floor and ceiling', () => {
    const p = policy({ minDays: 30, maxDays: 365 });
    expect(folderProblem({ choice: 'trash', days: 10 }, p)?.id).toBe('retention-error-min');
    expect(folderProblem({ choice: 'delete', days: 400 }, p)?.id).toBe('retention-error-max');
    expect(folderProblem({ choice: 'archive', days: 10 }, p)).toBeNull();
    expect(folderProblem({ choice: 'inherit', days: 1 }, p)).toBeNull();
  });
});
