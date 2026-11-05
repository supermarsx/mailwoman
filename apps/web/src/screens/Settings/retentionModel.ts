// Pure helpers for Settings > Retention: list parsing, the limits a rule is held
// to, and the message that describes a rule in one line.
import type { ActionKind, RetentionPolicy, RuleInput } from '../../api/retention.ts';

/** `a, b ,c` to `['a', 'b', 'c']`; empty entries dropped, duplicates removed. */
export function splitList(text: string): string[] {
  return [...new Set(text.split(/[,\n]/).map((x) => x.trim()).filter(Boolean))];
}

export const joinList = (items: readonly string[]): string => items.join(', ');

/** Actions that remove mail from where it is or hide it for good: the days floor and ceiling apply. */
export const isDestructive = (kind: ActionKind): boolean => kind === 'delete' || kind === 'trash';

/** Whether the days field means anything for this action. */
export const needsDays = (kind: ActionKind): boolean => kind !== 'keep';

/** The first problem with a rule that the form can see without asking the server, as a message id. */
export function ruleProblem(rule: RuleInput, policy: RetentionPolicy): { id: string; args: Record<string, number> } | null {
  if (rule.name.trim() === '') return { id: 'retention-error-name', args: {} };
  if (!Number.isInteger(rule.action.days) || rule.action.days < 0 || rule.action.days > 3650) return { id: 'retention-error-days', args: {} };
  if (isDestructive(rule.action.kind)) {
    if (rule.action.days < policy.minDays) return { id: 'retention-error-min', args: { days: policy.minDays } };
    if (policy.maxDays > 0 && rule.action.days > policy.maxDays) return { id: 'retention-error-max', args: { days: policy.maxDays } };
  }
  if (rule.action.kind === 'move' && !rule.action.target) return { id: 'retention-error-target', args: {} };
  return null;
}

/** The message that describes what happens to the mail a rule's scope matches. */
export function actionMessage(rule: RuleInput, permanent: boolean): { id: string; args: Record<string, number> } {
  const { kind, days } = rule.action;
  if (kind === 'keep') return { id: 'retention-summary-keep', args: {} };
  if (kind === 'delete') return { id: permanent ? 'retention-summary-delete-permanent' : 'retention-summary-delete', args: { days } };
  return { id: `retention-summary-${kind}`, args: { days } };
}

/** Whole days between two unix times, for the "next run" line. */
export const daysFromNow = (unix: number, now: number = Date.now() / 1000): number => Math.max(0, Math.ceil((unix - now) / 86400));
