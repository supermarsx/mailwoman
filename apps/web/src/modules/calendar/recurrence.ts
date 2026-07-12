// Client-side recurrence expansion + the recurrence model the editor produces
// (plan §3 e4, common set: FREQ / INTERVAL / BYDAY / BYMONTHDAY / COUNT / UNTIL /
// EXDATE). The engine (`mw-ics` + `rrule`) is the authoritative TZID-aware
// expander (plan §1.12); this module powers the mock backend, the editor's live
// preview, and offline display, over the same `EventInstance` shape the engine
// returns. It intentionally covers the common set only — exotic BYSETPOS /
// BYWEEKNO / BYYEARDAY chains are engine-side best-effort (plan §0 cut list).

import type { CalendarEvent } from '../../api/pim-types.ts';
import type { EventInstance } from './types.ts';
import { addDays, addMonths, addYears, dateToCalDate, localToDate, parseLocal } from './datetime.ts';

/** Weekday codes, JSCalendar `NDay.day` style. */
export type Weekday = 'mo' | 'tu' | 'we' | 'th' | 'fr' | 'sa' | 'su';

/** The frozen recurrence-editor model — a typed projection of the free-form
 *  JSCalendar `RecurrenceRule` JSON stored on `CalendarEvent.recurrenceRules`. */
export interface RecurrenceRule {
  frequency: 'daily' | 'weekly' | 'monthly' | 'yearly';
  /** Every `interval` periods (default 1). */
  interval?: number;
  /** Stop after this many occurrences. Mutually exclusive with `until`. */
  count?: number;
  /** Stop on/after this local date-time. Mutually exclusive with `count`. */
  until?: string;
  /** Weekly: which weekdays. */
  byDay?: Weekday[];
  /** Monthly: which days of the month (1-31). */
  byMonthDay?: number[];
}

const WEEKDAY_INDEX: Record<Weekday, number> = { su: 0, mo: 1, tu: 2, we: 3, th: 4, fr: 5, sa: 6 };
const WEEKDAY_ORDER: Weekday[] = ['su', 'mo', 'tu', 'we', 'th', 'fr', 'sa'];
export const WEEKDAY_LABEL: Record<Weekday, string> = {
  mo: 'Mon',
  tu: 'Tue',
  we: 'Wed',
  th: 'Thu',
  fr: 'Fri',
  sa: 'Sat',
  su: 'Sun',
};

const MAX_OCCURRENCES = 750; // hard cap so a malformed COUNT-less rule can't spin.

// ── ISO 8601 duration ────────────────────────────────────────────────────────

/** Parse an ISO 8601 duration (`"PT1H30M"`, `"P1D"`) to whole milliseconds. */
export function parseDuration(iso: string): number {
  const m = /^P(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?$/.exec(iso.trim());
  if (m === null) return 0;
  const [, d, h, mi, s] = m;
  return (
    (Number(d ?? 0) * 86400 + Number(h ?? 0) * 3600 + Number(mi ?? 0) * 60 + Number(s ?? 0)) * 1000
  );
}

/** Emit an ISO 8601 duration for a millisecond span (`3600000` → `"PT1H"`). */
export function formatDuration(ms: number): string {
  let secs = Math.max(0, Math.round(ms / 1000));
  const days = Math.floor(secs / 86400);
  secs -= days * 86400;
  const hours = Math.floor(secs / 3600);
  secs -= hours * 3600;
  const mins = Math.floor(secs / 60);
  secs -= mins * 60;
  const date = days > 0 ? `${days}D` : '';
  let time = '';
  if (hours > 0) time += `${hours}H`;
  if (mins > 0) time += `${mins}M`;
  if (secs > 0) time += `${secs}S`;
  if (date === '' && time === '') return 'PT0S';
  return `P${date}${time === '' ? '' : `T${time}`}`;
}

// ── Rule <-> JSON ────────────────────────────────────────────────────────────

/** Narrow the free-form stored JSON to the typed editor model, or `null`. */
export function parseRule(raw: Record<string, unknown>): RecurrenceRule | null {
  const freq = raw['frequency'];
  if (freq !== 'daily' && freq !== 'weekly' && freq !== 'monthly' && freq !== 'yearly') return null;
  const rule: RecurrenceRule = { frequency: freq };
  if (typeof raw['interval'] === 'number' && raw['interval'] > 1) rule.interval = raw['interval'];
  if (typeof raw['count'] === 'number') rule.count = raw['count'];
  if (typeof raw['until'] === 'string') rule.until = raw['until'];
  const byDay = raw['byDay'];
  if (Array.isArray(byDay)) {
    const days = byDay.filter((x): x is Weekday => typeof x === 'string' && x in WEEKDAY_INDEX);
    if (days.length > 0) rule.byDay = days;
  }
  const byMonthDay = raw['byMonthDay'];
  if (Array.isArray(byMonthDay)) {
    const nums = byMonthDay.filter((x): x is number => typeof x === 'number');
    if (nums.length > 0) rule.byMonthDay = nums;
  }
  return rule;
}

/** Serialize the editor model back to storable JSON. */
export function ruleToJson(rule: RecurrenceRule): Record<string, unknown> {
  const out: Record<string, unknown> = { frequency: rule.frequency };
  if (rule.interval !== undefined && rule.interval > 1) out['interval'] = rule.interval;
  if (rule.count !== undefined) out['count'] = rule.count;
  if (rule.until !== undefined) out['until'] = rule.until;
  if (rule.byDay !== undefined && rule.byDay.length > 0) out['byDay'] = rule.byDay;
  if (rule.byMonthDay !== undefined && rule.byMonthDay.length > 0) out['byMonthDay'] = rule.byMonthDay;
  return out;
}

/** The first typed rule on an event, if any. */
export function firstRule(event: CalendarEvent): RecurrenceRule | null {
  const raw = event.recurrenceRules[0];
  return raw === undefined ? null : parseRule(raw);
}

/** A human-readable summary of a rule (`"Every 2 weeks on Mon, Wed"`). */
export function describeRule(rule: RecurrenceRule): string {
  const n = rule.interval ?? 1;
  const unit = { daily: 'day', weekly: 'week', monthly: 'month', yearly: 'year' }[rule.frequency];
  let s = n === 1 ? `Every ${unit}` : `Every ${n} ${unit}s`;
  if (rule.frequency === 'weekly' && rule.byDay !== undefined && rule.byDay.length > 0) {
    const ordered = [...rule.byDay].sort((a, b) => WEEKDAY_INDEX[a] - WEEKDAY_INDEX[b]);
    s += ` on ${ordered.map((d) => WEEKDAY_LABEL[d]).join(', ')}`;
  }
  if (rule.frequency === 'monthly' && rule.byMonthDay !== undefined && rule.byMonthDay.length > 0) {
    s += ` on day ${[...rule.byMonthDay].sort((a, b) => a - b).join(', ')}`;
  }
  if (rule.count !== undefined) s += `, ${rule.count} times`;
  else if (rule.until !== undefined) s += `, until ${rule.until.slice(0, 10)}`;
  return s;
}

// ── Expansion ────────────────────────────────────────────────────────────────

/**
 * Expand a single master event into concrete `EventInstance`s overlapping
 * `[windowStart, windowEnd)`. Non-recurring events yield 0 or 1 instance;
 * recurring events walk occurrences applying INTERVAL / BYDAY / BYMONTHDAY /
 * COUNT / UNTIL and skipping EXDATE (`excludedRecurrenceDates`). Per-instance
 * `recurrenceOverrides` patch the title / start / duration of a given date.
 */
export function expandEvent(
  event: CalendarEvent,
  windowStart: Date,
  windowEnd: Date,
  color: string,
): EventInstance[] {
  const durationMs = parseDuration(event.duration) || (event.showWithoutTime ? 86400000 : 3600000);
  const rule = firstRule(event);
  const excluded = new Set(event.excludedRecurrenceDates);

  const emit = (occStart: Date, recurring: boolean): EventInstance | null => {
    const calDate = dateToCalDate(occStart);
    if (excluded.has(calDate)) return null;
    const override = event.recurrenceOverrides[calDate] ?? event.recurrenceOverrides[occStart.toISOString()];
    let start = occStart;
    let ms = durationMs;
    if (override !== undefined) {
      if (typeof override['start'] === 'string') start = localToDate(override['start']);
      if (typeof override['duration'] === 'string') ms = parseDuration(override['duration']);
      if (override['excluded'] === true) return null;
    }
    const end = new Date(start.getTime() + ms);
    if (end <= windowStart || start >= windowEnd) return null;
    return {
      key: `${event.id}:${occStart.getTime()}`,
      event,
      start,
      end,
      allDay: event.showWithoutTime,
      recurring,
      color,
    };
  };

  if (rule === null) {
    const inst = emit(localToDate(event.start), false);
    return inst === null ? [] : [inst];
  }

  const out: EventInstance[] = [];
  const seed = parseLocal(event.start);
  const until = rule.until !== undefined ? localToDate(rule.until) : null;
  const interval = Math.max(1, rule.interval ?? 1);
  let emitted = 0;
  let guard = 0;

  const consider = (occStart: Date): boolean => {
    // returns false to stop the whole walk (COUNT/UNTIL reached)
    if (until !== null && occStart > until) return false;
    if (rule.count !== undefined && emitted >= rule.count) return false;
    const inst = emit(occStart, true);
    // COUNT counts generated occurrences, not just those inside the window.
    emitted += 1;
    if (inst !== null && occStart < windowEnd) out.push(inst);
    return true;
  };

  if (rule.frequency === 'weekly') {
    const days =
      rule.byDay !== undefined && rule.byDay.length > 0
        ? rule.byDay.map((d) => WEEKDAY_INDEX[d])
        : [localToDate(event.start).getDay()];
    // Walk week by week from the seed's week start; within each active week emit
    // the selected weekdays in order.
    let weekBase = weekStartFrom(localToDate(event.start));
    const seedDate = localToDate(event.start);
    while (guard < MAX_OCCURRENCES) {
      guard += 1;
      let stop = false;
      for (let dow = 0; dow < 7 && !stop; dow += 1) {
        const day = addDays(weekBase, dow);
        if (day < startOfSameDay(seedDate)) continue;
        if (!days.includes(day.getDay())) continue;
        const occ = withTime(day, seed);
        if (!consider(occ)) {
          stop = true;
          break;
        }
        if (occ > windowEnd && (until === null || occ <= until)) {
          // Past the window but rule may still be counting; keep going only if
          // a COUNT is pending, else we can stop to bound work.
          if (rule.count === undefined) {
            stop = true;
            break;
          }
        }
      }
      if (stop && emitted >= (rule.count ?? Number.POSITIVE_INFINITY)) break;
      if (stop && rule.count === undefined) break;
      weekBase = addDays(weekBase, 7 * interval);
      if (weekBase > windowEnd && rule.count === undefined) break;
    }
    return out;
  }

  // daily / monthly / yearly: single-track walk from the seed.
  let cursor = localToDate(event.start);
  while (guard < MAX_OCCURRENCES) {
    guard += 1;
    if (rule.frequency === 'monthly' && rule.byMonthDay !== undefined && rule.byMonthDay.length > 0) {
      let stop = false;
      for (const dom of [...rule.byMonthDay].sort((a, b) => a - b)) {
        const occ = new Date(cursor.getFullYear(), cursor.getMonth(), dom, seed.hour, seed.minute, seed.second);
        if (occ.getMonth() !== cursor.getMonth()) continue; // clamp invalid (e.g. Feb 30)
        if (!consider(occ)) {
          stop = true;
          break;
        }
      }
      if (stop) break;
      cursor = addMonths(startOfMonthLocal(cursor), interval);
    } else {
      if (!consider(cursor)) break;
      cursor = step(cursor, rule.frequency, interval);
    }
    if (cursor > windowEnd && rule.count === undefined && until === null) break;
    if (cursor > windowEnd && rule.count === undefined && until !== null && cursor > until) break;
  }
  return out;
}

function step(d: Date, freq: RecurrenceRule['frequency'], interval: number): Date {
  switch (freq) {
    case 'daily':
      return addDays(d, interval);
    case 'monthly':
      return addMonths(d, interval);
    case 'yearly':
      return addYears(d, interval);
    case 'weekly':
      return addDays(d, 7 * interval);
    default:
      return addDays(d, interval);
  }
}

function withTime(day: Date, seed: { hour: number; minute: number; second: number }): Date {
  return new Date(day.getFullYear(), day.getMonth(), day.getDate(), seed.hour, seed.minute, seed.second);
}
function startOfSameDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0, 0);
}
function startOfMonthLocal(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1, 0, 0, 0, 0);
}
function weekStartFrom(d: Date): Date {
  // Monday-based week to match the grid default.
  const day = d.getDay();
  const diff = (day - 1 + 7) % 7;
  return addDays(startOfSameDay(d), -diff);
}

/** The set of weekday codes as an ordered UI list. */
export const WEEKDAYS: Weekday[] = ['mo', 'tu', 'we', 'th', 'fr', 'sa', 'su'];
export { WEEKDAY_ORDER };
