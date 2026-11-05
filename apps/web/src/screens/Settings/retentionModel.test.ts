import { describe, expect, it } from 'vitest';
import { actionMessage, daysFromNow, isDestructive, joinList, needsDays, ruleProblem, splitList } from './retentionModel.ts';
import { blankRule, type RetentionPolicy } from '../../api/retention.ts';

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
