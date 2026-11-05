// Pure helpers of the breach register screen.
import type { BreachClock, BreachSummary } from '../../../api/admin/breaches.ts';

export const SETTING_GROUPS: readonly { id: string; keys: readonly string[] }[] = [
  { id: 'clock', keys: ['breach.enabled', 'breach.noticeHours', 'breach.reminders.hours', 'breach.reminders.overdueRepeatHours', 'breach.retention.years'] },
  { id: 'authority', keys: ['breach.authority.name', 'breach.authority.url', 'breach.authority.contact', 'breach.dpo.name', 'breach.dpo.email', 'breach.dpo.phone'] },
  { id: 'templates', keys: ['breach.template.subject', 'breach.template.body'] },
  { id: 'channels', keys: ['breach.notify.audit', 'breach.notify.webhookUrls', 'breach.notify.operatorAddresses', 'breach.subjects.inApp', 'breach.subjects.inbox', 'breach.subjects.webhook'] },
  {
    id: 'automation',
    keys: [
      'breach.auto.enabled', 'breach.auto.startClock', 'breach.auto.assignee', 'breach.auto.escalateAfterHours', 'breach.auto.escalateTo', 'breach.auto.windowMinutes',
      'breach.auto.theft.threshold', 'breach.auto.credentialStuffing.threshold', 'breach.auto.massForward.threshold', 'breach.auto.roleEscalation.threshold',
      'breach.auto.exportVolume.threshold', 'breach.auto.auditChain.threshold', 'breach.auto.backupAnomaly.threshold', 'breach.auto.dsrExport.threshold',
    ],
  },
];

export interface Remaining {
  overdue: boolean;
  hours: number;
  minutes: number;
}

/** Whole hours and minutes to (or past) the deadline, measured from `nowSecs`. */
export function remaining(deadlineAt: number, nowSecs: number): Remaining {
  const diff = deadlineAt - nowSecs;
  const abs = Math.abs(diff);
  return { overdue: diff < 0, hours: Math.floor(abs / 3600), minutes: Math.floor((abs % 3600) / 60) };
}

export type ClockTone = 'ok' | 'soon' | 'overdue' | 'stopped' | 'idle';

/** How the clock reads: a draft is idle, under 24 hours is soon, past the deadline is overdue. */
export function clockTone(clock: BreachClock, nowSecs: number): ClockTone {
  switch (clock.phase) {
    case 'not-started': return 'idle';
    case 'stopped-on-time': case 'stopped-late': case 'not-required': return 'stopped';
    default: {
      if (clock.deadlineAt === null) return 'idle';
      const left = clock.deadlineAt - nowSecs;
      return left < 0 ? 'overdue' : left <= 24 * 3600 ? 'soon' : 'ok';
    }
  }
}

export const NEXT_STATUSES: readonly string[] = ['detected', 'assessing', 'contained', 'notified-authority', 'notified-subjects', 'closed', 'not-notifiable'];

/** Statuses whose move needs a written reason. */
export function needsReason(from: string, to: string): boolean {
  return to === 'not-notifiable' || to === 'closed' || ((from === 'closed' || from === 'not-notifiable') && ['assessing', 'contained', 'detected'].includes(to));
}

export const fileName = (reference: string, what: string, ext: string): string => `${reference}-${what}.${ext}`;

export const yearFileName = (year: string, ext: string): string => `breach-register-${year}.${ext}`;

/** A value for an `<input type="datetime-local">` in local time. */
export function toLocalInput(secs: number): string {
  const d = new Date(secs * 1000);
  const p = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

export function fromLocalInput(value: string): number | undefined {
  if (!value) return undefined;
  const ms = new Date(value).getTime();
  return Number.isFinite(ms) ? Math.floor(ms / 1000) : undefined;
}

export const splitList = (text: string): string[] => text.split(/[\n,;]/).map((s) => s.trim()).filter(Boolean);

/** Overdue and due-soon counts for the list banner, from the rows themselves. */
export function urgent(rows: readonly BreachSummary[], nowSecs: number): { overdue: number; soon: number } {
  let overdue = 0;
  let soon = 0;
  for (const r of rows) {
    const tone = clockTone(r.clock, nowSecs);
    if (tone === 'overdue') overdue += 1;
    else if (tone === 'soon') soon += 1;
  }
  return { overdue, soon };
}

export function saveFile(name: string, text: string, mime: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: mime }));
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
}
