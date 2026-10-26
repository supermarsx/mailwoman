import type { NotificationConfig } from '../screens/Settings/types.ts';

export interface NotificationMessage {
  subject?: string | null;
  from?: readonly { email?: string | null; name?: string | null }[] | null;
  mailboxNames?: readonly string[];
  mailboxIds?: Readonly<Record<string, boolean>> | null;
}

/** Missing/malformed policy never grants delivery. Rules are evaluated in order;
 * the first matching sender/subject/mailbox rule wins. */
export function shouldNotify(config: NotificationConfig | undefined, message: NotificationMessage, now = new Date()): boolean {
  if (!config?.enabled) return false;
  if (config.quietHours.enabled) {
    try {
      const parts = new Intl.DateTimeFormat('en-GB', { timeZone: config.quietHours.timezone ?? 'UTC', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).formatToParts(now);
      const minute = Number(parts.find(p => p.type === 'hour')?.value) * 60 + Number(parts.find(p => p.type === 'minute')?.value);
      const parse = (value: string): number => {
        if (!/^(?:[01]\d|2[0-3]):[0-5]\d$/.test(value)) throw new Error('Invalid time');
        return Number(value.slice(0, 2)) * 60 + Number(value.slice(3));
      };
      const start = parse(config.quietHours.start), end = parse(config.quietHours.end);
      if (!Number.isFinite(minute) || (start < end ? minute >= start && minute < end : start > end && (minute >= start || minute < end))) return false;
    } catch { return false; }
  }
  const values = [message.subject ?? '', ...(message.from ?? []).flatMap(from => [from.email ?? '', from.name ?? '']), ...Object.keys(message.mailboxIds ?? {}), ...(message.mailboxNames ?? [])].map(value => value.toLowerCase());
  for (const rule of config.rules) {
    const match = rule.match.trim().toLowerCase();
    if (match && values.some(value => value.includes(match))) return rule.action === 'notify';
  }
  return true;
}
