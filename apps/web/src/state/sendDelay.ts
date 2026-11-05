// Undo-send ("recall") window and send-later helpers.
//
// The engine holds every send for the window, so the Outbox is the durable
// queue: a message waits there until its time and can be recalled until the
// engine claims it. The server clamps whatever the client asks for to the
// administrator's ceiling, so the values here only shape the controls.
//
// Settings: mail.send.recallDelaySeconds (the user's choice, 0 = send at once),
// mail.send.recallDelayMaxSeconds, mail.send.allowRecall, mail.send.maxDelayedSeconds.

import { settings } from '../api/settings.ts';

export const RECALL_DELAY_KEY = 'mail.send.recallDelaySeconds';
export const RECALL_MAX_KEY = 'mail.send.recallDelayMaxSeconds';
export const ALLOW_RECALL_KEY = 'mail.send.allowRecall';
export const HORIZON_KEY = 'mail.send.maxDelayedSeconds';

export const DEFAULT_RECALL_SECONDS = 10;
export const MAX_RECALL_SECONDS = 1800;
export const DEFAULT_HORIZON_SECONDS = 90 * 86_400;

/** The selector's choices, in seconds. Anything else is a custom value. */
export const RECALL_PRESETS: readonly number[] = [0, 5, 10, 20, 30, 60, 120, 300, 600, 1800];

function num(key: string, fallback: number): number {
  const v = settings()[key]?.value;
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

/** Whether the administrator allows recalling at all. */
export function recallAllowed(): boolean {
  const v = settings()[ALLOW_RECALL_KEY]?.value;
  return typeof v === 'boolean' ? v : true;
}

/** The longest window this user may pick. */
export function recallCapSeconds(): number {
  return Math.max(0, Math.min(MAX_RECALL_SECONDS, num(RECALL_MAX_KEY, MAX_RECALL_SECONDS)));
}

/** How far ahead a message may be scheduled; 0 means scheduling is off. */
export function horizonSeconds(): number {
  return Math.max(0, num(HORIZON_KEY, DEFAULT_HORIZON_SECONDS));
}

/** The window applied to the next send: the user's choice within the cap. */
export function effectiveRecallSeconds(): number {
  if (!recallAllowed()) return 0;
  const chosen = Math.round(num(RECALL_DELAY_KEY, DEFAULT_RECALL_SECONDS));
  return Math.max(0, Math.min(chosen, recallCapSeconds()));
}

/** Presets that fit under `cap`, plus the cap itself when it falls between presets. */
export function recallChoices(cap: number): number[] {
  const list = RECALL_PRESETS.filter((s) => s <= cap);
  if (!list.includes(cap) && cap > 0) list.push(cap);
  return list;
}

/** m:ss for a countdown. */
export function formatCountdown(totalSeconds: number): string {
  const s = Math.max(0, Math.ceil(totalSeconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

export interface SendLaterPreset {
  /** Fluent id suffix under `send-later-preset-`. */
  id: 'hour' | 'today' | 'tomorrow' | 'monday';
  at: Date;
}

function atHour(base: Date, dayOffset: number, hour: number): Date {
  const d = new Date(base.getFullYear(), base.getMonth(), base.getDate() + dayOffset, hour, 0, 0, 0);
  return d;
}

/** The quick choices at `now`, in the viewer's local time, within the horizon. */
export function sendLaterPresets(now: Date, horizon = horizonSeconds()): SendLaterPreset[] {
  const out: SendLaterPreset[] = [{ id: 'hour', at: new Date(now.getTime() + 3_600_000) }];
  const evening = atHour(now, 0, 17);
  if (evening.getTime() - now.getTime() >= 2 * 3_600_000) out.push({ id: 'today', at: evening });
  const tomorrow = atHour(now, 1, 9);
  out.push({ id: 'tomorrow', at: tomorrow });
  const toMonday = ((8 - now.getDay()) % 7) || 7;
  const monday = atHour(now, toMonday, 9);
  if (monday.getTime() !== tomorrow.getTime()) out.push({ id: 'monday', at: monday });
  return out.filter((p) => (p.at.getTime() - now.getTime()) / 1000 <= horizon);
}

export type SendAtCheck = 'ok' | 'invalid' | 'past' | 'horizon' | 'off';

/** Future only, and within the administrator's horizon. */
export function checkSendAt(at: Date, now: Date, horizon = horizonSeconds()): SendAtCheck {
  if (Number.isNaN(at.getTime())) return 'invalid';
  if (at.getTime() <= now.getTime()) return 'past';
  if (horizon <= 0) return 'off';
  if ((at.getTime() - now.getTime()) / 1000 > horizon) return 'horizon';
  return 'ok';
}

/** `yyyy-MM-ddTHH:mm` in local time, the value a `datetime-local` input takes. */
export function toLocalInput(d: Date): string {
  const p = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** The viewer's IANA time zone, for display next to a picked time. */
export function localTimeZone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  } catch {
    return 'UTC';
  }
}
