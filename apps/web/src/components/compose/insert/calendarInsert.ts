import {formatDateWith} from '../../../i18n/format.ts';
import {localeWeekStart} from '../../../modules/calendar/datetime.ts';
import {regionalWeekStart} from '../../../i18n/regional.ts';
// Calendar blocks for the Insert tab: free slots, an agenda, a month snapshot, an
// event summary, and the iCalendar (.ics) that goes with an event. All output is
// mail-safe HTML (tables, inline styles) built from plain data; the caller fetches
// the events and supplies the labels.

export type Visibility = 'busy' | 'titles';

export interface Interval { start: string; end: string }
export interface AgendaEvent extends Interval { title: string; location?: string; allDay?: boolean }

const esc = (s: string): string =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

// ── time zone helpers ───────────────────────────────────────────────────────

function partsIn(ms: number, tz: string): { y: number; m: number; d: number; h: number; min: number; wd: number } {
  const fmt = new Intl.DateTimeFormat('en-US', { timeZone: tz, hourCycle: 'h23', year: 'numeric', month: 'numeric', day: 'numeric', hour: 'numeric', minute: 'numeric', weekday: 'short' });
  const o: Record<string, string> = {};
  for (const p of fmt.formatToParts(new Date(ms))) o[p.type] = p.value;
  const wd = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].indexOf(o['weekday'] ?? '');
  return { y: Number(o['year']), m: Number(o['month']), d: Number(o['day']), h: Number(o['hour']) % 24, min: Number(o['minute']), wd };
}

/** The UTC instant of a wall-clock time in `tz`. */
export function zonedToUtc(y: number, m: number, d: number, h: number, min: number, tz: string): number {
  const guess = Date.UTC(y, m - 1, d, h, min);
  const at = (ms: number): number => { const p = partsIn(ms, tz); return Date.UTC(p.y, p.m - 1, p.d, p.h, p.min) - ms; };
  let result = guess - at(guess);
  result = guess - at(result);
  return result;
}

const pad = (n: number): string => String(n).padStart(2, '0');
const timeIn = (ms: number, tz: string, locale?: string): string => formatDateWith(new Date(ms),{timeZone:tz,hour:'2-digit',minute:'2-digit'},locale);
const dayLabel = (ms: number, tz: string, locale?: string): string => formatDateWith(new Date(ms),{timeZone:tz,weekday:'short',day:'numeric',month:'short'},locale);

const TABLE = 'border-collapse:collapse;font-family:Arial,Helvetica,sans-serif;font-size:13px;color:#222222';
const CELL = 'border:1px solid #c8c8c8;padding:5px 10px;vertical-align:top';
const HEAD = `${CELL};background:#f0f0f0;text-align:left`;

// ── availability ────────────────────────────────────────────────────────────

export interface AvailabilityOptions {
  /** First and last day, inclusive, as YYYY-MM-DD in `tz`. */
  from: string;
  to: string;
  tz: string;
  workStart?: number;
  workEnd?: number;
  minMinutes?: number;
  skipWeekends?: boolean;
  locale?: string;
  /** Labels. */
  labels: { day: string; free: string; none: string; note: string };
}

/** Free slots (as [startMs, endMs]) in one day's working window, minus the busy intervals. */
export function freeSlots(busy: readonly Interval[], windowStart: number, windowEnd: number, minMinutes: number): [number, number][] {
  const blocks = busy
    .map((b) => [Date.parse(b.start), Date.parse(b.end)] as const)
    .filter(([s, e]) => Number.isFinite(s) && Number.isFinite(e) && e > windowStart && s < windowEnd)
    .sort((a, b) => a[0] - b[0]);
  const out: [number, number][] = [];
  let cursor = windowStart;
  for (const [s, e] of blocks) {
    if (s > cursor) out.push([cursor, Math.min(s, windowEnd)]);
    cursor = Math.max(cursor, e);
  }
  if (cursor < windowEnd) out.push([cursor, windowEnd]);
  return out.filter(([s, e]) => (e - s) / 60000 >= minMinutes);
}

/** A compact table of the free slots per day. Only free time is shown, never what fills the rest. */
export function availabilityHtml(busy: readonly Interval[], o: AvailabilityOptions): string {
  const [fy, fm, fd] = o.from.split('-').map(Number) as [number, number, number];
  const [ty, tm, td] = o.to.split('-').map(Number) as [number, number, number];
  const last = Date.UTC(ty, tm - 1, td);
  const rows: string[] = [];
  for (let day = Date.UTC(fy, fm - 1, fd); day <= last && rows.length < 62; day += 86_400_000) {
    const dt = new Date(day);
    const [y, m, d] = [dt.getUTCFullYear(), dt.getUTCMonth() + 1, dt.getUTCDate()];
    const startMs = zonedToUtc(y, m, d, o.workStart ?? 9, 0, o.tz);
    if (o.skipWeekends !== false && [0, 6].includes(partsIn(startMs, o.tz).wd)) continue;
    const endMs = zonedToUtc(y, m, d, o.workEnd ?? 17, 0, o.tz);
    const slots = freeSlots(busy, startMs, endMs, o.minMinutes ?? 30);
    const text = slots.length === 0 ? esc(o.labels.none) : slots.map(([s, e]) => `${timeIn(s, o.tz, o.locale)}\u2013${timeIn(e, o.tz, o.locale)}`).join(', ');
    rows.push(`<tr><td style="${CELL}">${esc(dayLabel(startMs, o.tz, o.locale))}</td><td style="${CELL}">${text}</td></tr>`);
  }
  const note = esc(o.labels.note.replace('{tz}', o.tz));
  return `<table data-mw-calendar="availability" cellpadding="0" cellspacing="0" style="${TABLE}"><thead><tr><th style="${HEAD}">${esc(o.labels.day)}</th><th style="${HEAD}">${esc(o.labels.free)}</th></tr></thead><tbody>${rows.join('')}</tbody></table><p style="font-family:Arial,Helvetica,sans-serif;font-size:12px;color:#555555;margin:4px 0 0">${note}</p>`;
}

// ── agenda ──────────────────────────────────────────────────────────────────

export interface AgendaOptions { tz: string; visibility: Visibility; locale?: string; labels: { busy: string; allDay: string; empty: string; note: string } }

export function agendaHtml(events: readonly AgendaEvent[], o: AgendaOptions): string {
  const sorted = [...events].sort((a, b) => Date.parse(a.start) - Date.parse(b.start));
  if (sorted.length === 0) return `<p style="font-family:Arial,Helvetica,sans-serif;font-size:13px">${esc(o.labels.empty)}</p>`;
  let day = '';
  const rows: string[] = [];
  for (const ev of sorted) {
    const s = Date.parse(ev.start), e = Date.parse(ev.end);
    const label = dayLabel(s, o.tz, o.locale);
    if (label !== day) { day = label; rows.push(`<tr><td colspan="2" style="${HEAD}"><strong>${esc(label)}</strong></td></tr>`); }
    const when = ev.allDay ? o.labels.allDay : `${timeIn(s, o.tz, o.locale)}\u2013${timeIn(e, o.tz, o.locale)}`;
    const title = o.visibility === 'titles' ? ev.title : o.labels.busy;
    const place = o.visibility === 'titles' && ev.location ? `<br><span style="color:#555555">${esc(ev.location)}</span>` : '';
    rows.push(`<tr><td style="${CELL};white-space:nowrap">${esc(when)}</td><td style="${CELL}">${esc(title)}${place}</td></tr>`);
  }
  return `<table data-mw-calendar="agenda" cellpadding="0" cellspacing="0" style="${TABLE}"><tbody>${rows.join('')}</tbody></table><p style="font-family:Arial,Helvetica,sans-serif;font-size:12px;color:#555555;margin:4px 0 0">${esc(o.labels.note.replace('{tz}', o.tz))}</p>`;
}

// ── month snapshot ──────────────────────────────────────────────────────────

export interface SnapshotOptions { year: number; month: number; tz: string; visibility: Visibility; weekStartsOn?: number; locale?: string; labels: { note: string } }

export function snapshotHtml(events: readonly AgendaEvent[], o: SnapshotOptions): string {
  const startDow = o.weekStartsOn ?? regionalWeekStart() ?? localeWeekStart(o.locale);
  const first = Date.UTC(o.year, o.month - 1, 1);
  const days = new Date(o.year, o.month, 0).getDate();
  const lead = (new Date(first).getUTCDay() - startDow + 7) % 7;
  const byDay = new Map<number, AgendaEvent[]>();
  for (const ev of events) {
    const p = partsIn(Date.parse(ev.start), o.tz);
    if (p.y === o.year && p.m === o.month) byDay.set(p.d, [...(byDay.get(p.d) ?? []), ev]);
  }
  const names = Array.from({ length: 7 }, (_, i) => formatDateWith(new Date(Date.UTC(2024,0,7+((startDow+i)%7))),{weekday:'short',timeZone:'UTC'},o.locale));
  const cells: string[] = [];
  for (let i = 0; i < lead; i++) cells.push(`<td style="${CELL}"></td>`);
  for (let d = 1; d <= days; d++) {
    const list = byDay.get(d) ?? [];
    const detail = list.length === 0 ? '' : o.visibility === 'titles'
      ? list.slice(0, 2).map((ev) => `<br><span style="font-size:11px;color:#1a56b0">${esc(ev.title)}</span>`).join('') + (list.length > 2 ? `<br><span style="font-size:11px">+${list.length - 2}</span>` : '')
      : '<br><span style="font-size:11px;color:#1a56b0">\u25CF</span>';
    cells.push(`<td style="${CELL};width:14%;height:48px"><strong>${d}</strong>${detail}</td>`);
  }
  while (cells.length % 7 !== 0) cells.push(`<td style="${CELL}"></td>`);
  const rows: string[] = [];
  for (let i = 0; i < cells.length; i += 7) rows.push(`<tr>${cells.slice(i, i + 7).join('')}</tr>`);
  const title = formatDateWith(new Date(first),{month:'long',year:'numeric',timeZone:'UTC'},o.locale);
  return `<table data-mw-calendar="snapshot" cellpadding="0" cellspacing="0" style="${TABLE}"><caption style="text-align:left;font-weight:bold;padding:4px 0">${esc(title)}</caption><thead><tr>${names.map((n) => `<th style="${HEAD}">${esc(n)}</th>`).join('')}</tr></thead><tbody>${rows.join('')}</tbody></table><p style="font-family:Arial,Helvetica,sans-serif;font-size:12px;color:#555555;margin:4px 0 0">${esc(o.labels.note.replace('{tz}', o.tz))}</p>`;
}

// ── event: summary block and .ics ───────────────────────────────────────────

export interface EventDraft {
  uid: string;
  title: string;
  /** UTC instants. */
  start: number;
  end: number;
  tz: string;
  location?: string;
  description?: string;
  attendees: string[];
  organizer: { name: string; email: string };
}

export function eventSummaryHtml(ev: EventDraft, labels: { when: string; where: string; who: string }, locale?: string): string {
  const fmt = {format:(date:Date)=>formatDateWith(date,{timeZone:ev.tz,dateStyle:'full',timeStyle:'short'},locale)};
  const row = (k: string, v: string): string => (v === '' ? '' : `<tr><td style="${CELL};color:#555555">${esc(k)}</td><td style="${CELL}">${v}</td></tr>`);
  return `<table data-mw-calendar="event" cellpadding="0" cellspacing="0" style="${TABLE}"><tbody><tr><td colspan="2" style="${HEAD}"><strong>${esc(ev.title)}</strong></td></tr>${row(labels.when, `${esc(fmt.format(new Date(ev.start)))} \u2013 ${esc(timeIn(ev.end, ev.tz, locale))} (${esc(ev.tz)})`)}${row(labels.where, esc(ev.location ?? ''))}${row(labels.who, esc(ev.attendees.join(', ')))}${ev.description ? `<tr><td colspan="2" style="${CELL}">${esc(ev.description).replace(/\n/g, '<br>')}</td></tr>` : ''}</tbody></table>`;
}

const icsText = (s: string): string => s.replace(/\\/g, '\\\\').replace(/;/g, '\\;').replace(/,/g, '\\,').replace(/\r?\n/g, '\\n');
const icsParam = (s: string): string => `"${s.replace(/["\r\n]/g, '')}"`;
const icsDate = (ms: number): string => new Date(ms).toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '');

/** RFC 5545 line folding at 75 octets. */
export function foldLine(line: string): string {
  const enc = new TextEncoder();
  if (enc.encode(line).length <= 75) return line;
  const out: string[] = [];
  let cur = '';
  for (const ch of line) {
    const limit = out.length === 0 ? 75 : 74;
    if (enc.encode(cur + ch).length > limit) { out.push(cur); cur = ch; } else cur += ch;
  }
  out.push(cur);
  return out.join('\r\n ');
}

/** The iCalendar document: METHOD:REQUEST with ORGANIZER and ATTENDEEs when there are attendees, else METHOD:PUBLISH. */
export function buildEventIcs(ev: EventDraft, now: number = Date.now()): { method: 'REQUEST' | 'PUBLISH'; text: string } {
  const method = ev.attendees.length > 0 ? 'REQUEST' : 'PUBLISH';
  const lines = [
    'BEGIN:VCALENDAR', 'VERSION:2.0', 'PRODID:-//Mailwoman//Composer//EN', 'CALSCALE:GREGORIAN', `METHOD:${method}`,
    'BEGIN:VEVENT', `UID:${ev.uid}`, `DTSTAMP:${icsDate(now)}`, `DTSTART:${icsDate(ev.start)}`, `DTEND:${icsDate(ev.end)}`,
    `X-WR-TIMEZONE:${ev.tz}`, `SUMMARY:${icsText(ev.title)}`, 'SEQUENCE:0', 'STATUS:CONFIRMED',
  ];
  if (ev.location) lines.push(`LOCATION:${icsText(ev.location)}`);
  if (ev.description) lines.push(`DESCRIPTION:${icsText(ev.description)}`);
  if (method === 'REQUEST') {
    lines.push(`ORGANIZER;CN=${icsParam(ev.organizer.name || ev.organizer.email)}:mailto:${ev.organizer.email}`);
    for (const a of ev.attendees) lines.push(`ATTENDEE;CN=${icsParam(a)};ROLE=REQ-PARTICIPANT;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:${a}`);
  }
  lines.push('END:VEVENT', 'END:VCALENDAR');
  return { method, text: `${lines.map(foldLine).join('\r\n')}\r\n` };
}

/** Local wall-clock inputs (`YYYY-MM-DD` and `HH:MM`) in `tz` to a UTC instant. */
export function localToInstant(date: string, time: string, tz: string): number {
  const [y, m, d] = date.split('-').map(Number) as [number, number, number];
  const [h, mi] = time.split(':').map(Number) as [number, number];
  return zonedToUtc(y, m, d, h, mi, tz);
}

export const pad2 = pad;
