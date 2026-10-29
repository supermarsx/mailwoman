// Pure helpers for the reader's event card (t53): when an event happens in the
// viewer's zone, its recurrence in words, and the links in its location and
// description. Everything here reads untrusted calendar text and returns plain
// strings or typed segments; nothing is rendered as HTML and nothing is fetched.

import { formatDateWith, formatList, t } from '../../i18n';
import { validTimeZone, wallTimeCandidates } from '../../i18n/timezones.ts';

/** The event fields the card reads (a subset of the engine projection). */
export interface CardEvent {
  uid?: string;
  title?: string;
  description?: string;
  locations?: Array<{ name?: string }>;
  start?: string;
  timeZone?: string | null;
  duration?: string;
  showWithoutTime?: boolean;
  recurrenceRules?: Array<Record<string, unknown>>;
  excludedRecurrenceDates?: string[];
  recurrenceOverrides?: Record<string, unknown>;
  status?: string;
  sequence?: number;
  participants?: Record<string, CardParticipant>;
}

export interface CardParticipant {
  name?: string;
  email?: string;
  role?: string;
  participationStatus?: string;
  expectReply?: boolean;
  kind?: string;
}

/** ISO-8601 duration → milliseconds (`P1D`, `PT1H30M`, `P2W`); 0 when malformed. */
export function durationMs(iso: string | undefined): number {
  const m = /^([+-])?P(?:(\d+)W)?(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?$/.exec(iso ?? '');
  if (!m) return 0;
  const [, sign, w, d, h, mi, s] = m;
  const ms = (((Number(w ?? 0) * 7 + Number(d ?? 0)) * 24 + Number(h ?? 0)) * 60 + Number(mi ?? 0)) * 60_000 + Number(s ?? 0) * 1000;
  return sign === '-' ? -ms : ms;
}

/** When an event starts and ends. A timed event is an instant; an all-day one is
 *  a pair of calendar dates (`end` exclusive) carried as UTC midnights. */
export interface EventRange {
  start: Date;
  end: Date;
  allDay: boolean;
}

/** The instant a projection `LocalDateTime` names in `zone` (a floating time is
 *  read in the viewer's zone). The earlier of two in a repeated hour. */
export function instantOf(local: string, zone: string | null | undefined, viewerZone: string): Date | null {
  const value = local.length === 16 ? `${local}:00` : local;
  if (zone === 'UTC' || zone === 'Etc/UTC') {
    const d = new Date(`${value}Z`);
    return Number.isNaN(d.getTime()) ? null : d;
  }
  const z = zone && validTimeZone(zone) ? zone : viewerZone;
  const found = wallTimeCandidates(value, z);
  if (found[0]) return found[0];
  // A wall time skipped by a DST change: an hour later is what clients show.
  const plusHour = value.replace(/T(\d{2})/, (_, h: string) => `T${String(Math.min(23, Number(h) + 1)).padStart(2, '0')}`);
  return wallTimeCandidates(plusHour, z)[0] ?? null;
}

export function eventRange(ev: CardEvent, viewerZone: string): EventRange | null {
  const start = ev.start ?? '';
  if (!start) return null;
  if (ev.showWithoutTime || !start.includes('T')) {
    const s = new Date(`${start.slice(0, 10)}T00:00:00Z`);
    if (Number.isNaN(s.getTime())) return null;
    const days = Math.max(1, Math.round(durationMs(ev.duration || 'P1D') / 86_400_000));
    return { start: s, end: new Date(s.getTime() + days * 86_400_000), allDay: true };
  }
  const s = instantOf(start, ev.timeZone ?? null, viewerZone);
  if (!s) return null;
  return { start: s, end: new Date(s.getTime() + Math.max(0, durationMs(ev.duration))), allDay: false };
}

function sameDayIn(a: Date, b: Date, zone: string): boolean {
  const key = (d: Date): string => formatDateWith(d, { timeZone: zone, year: 'numeric', month: '2-digit', day: '2-digit' });
  return key(a) === key(b);
}

/** "Tue, 3 Nov 2026, 10:00 – 11:00" in `zone` (localized); all-day ranges in dates. */
export function formatRange(range: EventRange, zone: string): string {
  const day: Intl.DateTimeFormatOptions = { weekday: 'short', day: 'numeric', month: 'short', year: 'numeric' };
  if (range.allDay) {
    const last = new Date(range.end.getTime() - 86_400_000);
    const first = formatDateWith(range.start, { ...day, timeZone: 'UTC' });
    const text = last.getTime() <= range.start.getTime() ? first : `${first} – ${formatDateWith(last, { ...day, timeZone: 'UTC' })}`;
    return t('calendar-card-all-day', { when: text });
  }
  const time: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit', timeZone: zone };
  const startText = `${formatDateWith(range.start, { ...day, timeZone: zone })}, ${formatDateWith(range.start, time)}`;
  if (range.end.getTime() <= range.start.getTime()) return startText;
  const endText = sameDayIn(range.start, range.end, zone)
    ? formatDateWith(range.end, time)
    : `${formatDateWith(range.end, { ...day, timeZone: zone })}, ${formatDateWith(range.end, time)}`;
  return `${startText} – ${endText}`;
}

/** The UTC offset of `zone` at `at`, in minutes. */
export function offsetMinutes(zone: string, at: Date): number {
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat('en-US', { timeZone: zone, hourCycle: 'h23', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
      .formatToParts(at).map((p) => [p.type, p.value]),
  );
  const wall = Date.UTC(Number(parts['year']), Number(parts['month']) - 1, Number(parts['day']), Number(parts['hour']), Number(parts['minute']));
  return Math.round((wall - Math.floor(at.getTime() / 60_000) * 60_000) / 60_000);
}

/** The organizer's own wall time, when their zone shows a different clock from
 *  the viewer's at that moment; otherwise `null`. */
export function organizerTime(ev: CardEvent, range: EventRange, viewerZone: string): string | null {
  const zone = ev.timeZone;
  if (range.allDay || !zone || zone === 'UTC' || zone === 'Etc/UTC' || !validTimeZone(zone)) return null;
  if (offsetMinutes(zone, range.start) === offsetMinutes(viewerZone, range.start)) return null;
  return t('calendar-card-organizer-time', { when: formatRange(range, zone), zone });
}

// ── recurrence ───────────────────────────────────────────────────────────────

const DAY_CODES = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'] as const;

/** A weekday name in the active locale (`0` = Sunday). */
function weekdayName(index: number): string {
  // 2026-11-01 is a Sunday.
  return formatDateWith(new Date(Date.UTC(2026, 10, 1 + index, 12)), { weekday: 'long', timeZone: 'UTC' });
}

function untilDate(raw: string): Date | null {
  const m = /^(\d{4})(\d{2})(\d{2})(?:T(\d{2})(\d{2})(\d{2})Z?)?$/.exec(raw);
  if (!m) return null;
  return new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 12));
}

/** One RRULE in plain words ("Every Tuesday until 3 Dec 2026"), or `null` when
 *  the event does not repeat. A rule this does not read fully (BYSETPOS, an
 *  ordinal weekday, …) is described as a custom schedule, never wrongly. */
export function describeRecurrence(ev: CardEvent): string | null {
  const rule = ev.recurrenceRules?.[0];
  const raw = typeof rule?.['rrule'] === 'string' ? (rule['rrule'] as string) : null;
  if (!raw) return null;
  const parts = new Map<string, string>();
  for (const piece of raw.toUpperCase().split(';')) {
    const [k, v] = piece.split('=');
    if (k && v !== undefined) parts.set(k.trim(), v.trim());
  }
  const known = new Set(['FREQ', 'INTERVAL', 'COUNT', 'UNTIL', 'BYDAY', 'BYMONTHDAY', 'WKST']);
  const freq = parts.get('FREQ');
  const days = (parts.get('BYDAY') ?? '').split(',').filter(Boolean);
  const plainDays = days.every((d) => (DAY_CODES as readonly string[]).includes(d));
  if (!freq || ![...parts.keys()].every((k) => known.has(k)) || !plainDays || (days.length > 0 && freq !== 'WEEKLY')) {
    return t('calendar-card-repeat-custom');
  }
  const interval = Math.max(1, Number(parts.get('INTERVAL') ?? 1) || 1);
  let text: string;
  if (freq === 'WEEKLY' && days.length > 0) {
    const names = days.map((d) => (DAY_CODES as readonly string[]).indexOf(d)).sort((a, b) => ((a + 6) % 7) - ((b + 6) % 7)).map(weekdayName);
    text = t('calendar-card-repeat-weekly-on', { interval, days: formatList(names) });
  } else if (freq === 'DAILY') text = t('calendar-card-repeat-daily', { interval });
  else if (freq === 'WEEKLY') text = t('calendar-card-repeat-weekly', { interval });
  else if (freq === 'MONTHLY') {
    const monthDay = parts.get('BYMONTHDAY');
    text = monthDay && /^\d{1,2}$/.test(monthDay)
      ? t('calendar-card-repeat-monthly-day', { interval, day: Number(monthDay) })
      : t('calendar-card-repeat-monthly', { interval });
  } else if (freq === 'YEARLY') text = t('calendar-card-repeat-yearly', { interval });
  else return t('calendar-card-repeat-custom');
  const count = Number(parts.get('COUNT') ?? 0);
  const until = parts.get('UNTIL') ? untilDate(parts.get('UNTIL')!) : null;
  if (count > 0) text = t('calendar-card-repeat-count', { rule: text, count });
  else if (until) text = t('calendar-card-repeat-until', { rule: text, date: formatDateWith(until, { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' }) });
  const skipped = ev.excludedRecurrenceDates?.length ?? 0;
  if (skipped > 0) text = t('calendar-card-repeat-except', { rule: text, count: skipped });
  return text;
}

// ── links ────────────────────────────────────────────────────────────────────

export type MeetingProvider = 'teams' | 'zoom' | 'meet' | 'jitsi' | 'webex';

const PROVIDERS: Array<[MeetingProvider, RegExp]> = [
  ['teams', /^(?:teams\.microsoft\.com|teams\.live\.com)$/],
  ['zoom', /(?:^|\.)zoom\.(?:us|com)$/],
  ['meet', /^meet\.google\.com$/],
  ['jitsi', /(?:^meet\.jit\.si|(?:^|\.)jitsi\.[a-z.]+)$/],
  ['webex', /(?:^|\.)webex\.com$/],
];

/** An `http(s)` URL as a string, or `null` for anything else (a `javascript:`,
 *  `data:` or `file:` value is never a link). */
export function safeUrl(text: string): string | null {
  const trimmed = text.trim();
  if (!/^https?:\/\//i.test(trimmed) || trimmed.length > 2048 || /\s/.test(trimmed)) return null;
  try {
    const url = new URL(trimmed);
    return url.protocol === 'https:' || url.protocol === 'http:' ? url.href : null;
  } catch {
    return null;
  }
}

const URL_IN_TEXT = /https?:\/\/[^\s<>"'()]+[^\s<>"'().,;:!?]/gi;

/** Split text into plain runs and `http(s)` links. */
export function linkSegments(text: string): Array<{ text: string; url?: string }> {
  const out: Array<{ text: string; url?: string }> = [];
  let last = 0;
  for (const m of text.matchAll(URL_IN_TEXT)) {
    const at = m.index ?? 0;
    const url = safeUrl(m[0]);
    if (!url) continue;
    if (at > last) out.push({ text: text.slice(last, at) });
    out.push({ text: m[0], url });
    last = at + m[0].length;
  }
  if (last < text.length) out.push({ text: text.slice(last) });
  return out;
}

/** The online-meeting join link in the location or description, if any:
 *  only a link whose host is one of the known meeting services. */
export function joinLink(ev: CardEvent): { provider: MeetingProvider; url: string } | null {
  const texts = [...(ev.locations ?? []).map((l) => l.name ?? ''), ev.description ?? ''];
  for (const text of texts) {
    for (const seg of linkSegments(text)) {
      if (!seg.url) continue;
      const host = new URL(seg.url).hostname.toLowerCase();
      const found = PROVIDERS.find(([, re]) => re.test(host));
      if (found) return { provider: found[0], url: seg.url };
    }
  }
  return null;
}

// ── people ───────────────────────────────────────────────────────────────────

export function participantAddress(key: string, p: CardParticipant): string {
  return (p.email && p.email.length > 0 ? p.email : key).replace(/^mailto:/i, '');
}

export function sameAddress(a: string | null | undefined, b: string | null | undefined): boolean {
  const norm = (s: string): string => s.trim().replace(/^mailto:/i, '').toLowerCase();
  return !!a && !!b && norm(a) !== '' && norm(a) === norm(b);
}

/** The organizer entry and the other participants (organizer first excluded). */
export function people(ev: CardEvent): { organizer: [string, CardParticipant] | null; attendees: Array<[string, CardParticipant]> } {
  const entries = Object.entries(ev.participants ?? {});
  const organizer = entries.find(([, p]) => p.role === 'organizer') ?? null;
  return { organizer, attendees: entries.filter(([, p]) => p.role !== 'organizer') };
}

/** The description, bounded for display. */
export const DESCRIPTION_LIMIT = 4000;
export function displayDescription(text: string | undefined): { text: string; clipped: boolean } {
  // Control characters other than tab and line break are dropped.
  const clean = [...(text ?? '').replace(/\r\n?/g, '\n')]
    .filter((ch) => { const c = ch.charCodeAt(0); return ch === '\n' || ch === '\t' || (c >= 0x20 && c !== 0x7f); })
    .join('').trim();
  return clean.length > DESCRIPTION_LIMIT ? { text: clean.slice(0, DESCRIPTION_LIMIT), clipped: true } : { text: clean, clipped: false };
}
