// Side-by-side conflict resolver (plan #7, SPEC §11.4). Lists the overlapping
// instance pairs the controller detected and, for the selected pair, compares
// the two events side by side and offers concrete resolutions:
//   • reschedule — move the later event to start when the earlier one ends;
//   • shorten    — trim the earlier event so it ends when the later one starts;
//   • tentative  — mark the later event tentative;
//   • double-book — keep both, but mark the later event's time as free so it no
//                   longer counts against free/busy;
//   • keep both  — accept the overlap as-is.
// When either event has attendees the mutation goes through the controller's
// `updateEvent`, which the engine turns into an iTIP update send (the "update
// sends" the plan calls for) — surfaced here as a notice.
//
// A free/busy grid (wiring the previously-unused `queryFreeBusy`) shows each
// attendee's busy/tentative/free time across the conflict window so the user can
// pick a clear slot.
//
// a11y (WCAG 2.2 AA): a focus-trapped `role=dialog` (Escape closes, focus is
// restored on close); the free/busy grid is a semantic table with row/column
// headers and per-cell text (status is never conveyed by color alone); every
// user-controlled title/email is bidi-isolated.

import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n';
import type { CalendarEvent } from '../../api/pim-types.ts';
import type { CalendarController } from './controller.ts';
import type { FreeBusyBlock } from './api.ts';
import type { ConflictPair } from './types.ts';
import { dateToLocal, formatFull, formatTime, localToDate, startOfDay } from './datetime.ts';
import { formatDuration, parseDuration } from './recurrence.ts';
import * as css from './calendar.css.ts';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** One resolved side of a conflict pair: the master + its concrete bounds. */
interface Side {
  event: CalendarEvent;
  start: Date;
  end: Date;
}

type FbStatus = 'busy' | 'tentative' | 'free';

export interface ConflictResolverProps {
  controller: CalendarController;
  onClose: () => void;
}

function endOf(ev: CalendarEvent): Date {
  return new Date(localToDate(ev.start).getTime() + parseDuration(ev.duration));
}

export function ConflictResolver(props: ConflictResolverProps): JSX.Element {
  const c = props.controller;
  const [selected, setSelected] = createSignal(0);
  const [busy, setBusy] = createSignal(false);
  const [blocks, setBlocks] = createSignal<FreeBusyBlock[]>([]);

  // Keep the selection in range as pairs resolve away.
  createEffect(() => {
    const n = c.conflicts().length;
    if (n > 0 && selected() >= n) setSelected(n - 1);
  });

  /** The selected pair, resolved to its two masters ordered earliest-first. */
  const pair = createMemo<{ pair: ConflictPair; earlier: Side; later: Side } | null>(() => {
    const p = c.conflicts()[selected()];
    if (p === undefined) return null;
    const ea = c.masterById(p.a);
    const eb = c.masterById(p.b);
    if (ea === undefined || eb === undefined) return null;
    const sa: Side = { event: ea, start: localToDate(ea.start), end: endOf(ea) };
    const sb: Side = { event: eb, start: localToDate(eb.start), end: endOf(eb) };
    const [earlier, later] = sa.start <= sb.start ? [sa, sb] : [sb, sa];
    return { pair: p, earlier, later };
  });

  const hasAttendees = createMemo(() => {
    const pr = pair();
    if (pr === null) return false;
    return Object.keys(pr.earlier.event.participants).length > 0 || Object.keys(pr.later.event.participants).length > 0;
  });

  /** The union of attendee emails across both events (rows of the grid). */
  const principals = createMemo<string[]>(() => {
    const pr = pair();
    if (pr === null) return [];
    const set = new Set<string>();
    for (const side of [pr.earlier, pr.later]) {
      for (const part of Object.values(side.event.participants)) {
        if (part.email !== '') set.add(part.email);
      }
    }
    if (set.size === 0) set.add('me@example.com');
    return [...set];
  });

  /** The conflict day + the hour span the grid covers (union ± 1h, clamped). */
  const grid = createMemo(() => {
    const pr = pair();
    if (pr === null) return { day: startOfDay(new Date()), hours: [] as number[] };
    const day = startOfDay(pr.earlier.start);
    const startH = Math.max(0, Math.min(pr.earlier.start.getHours(), pr.later.start.getHours()) - 1);
    const endH = Math.min(24, Math.max(pr.earlier.end.getHours() + (pr.earlier.end.getMinutes() > 0 ? 1 : 0), pr.later.end.getHours() + (pr.later.end.getMinutes() > 0 ? 1 : 0)) + 1);
    const hours: number[] = [];
    for (let h = startH; h < endH; h += 1) hours.push(h);
    return { day, hours };
  });

  // Wire `queryFreeBusy`: refetch the grid whenever the selected pair changes.
  createEffect(() => {
    const pr = pair();
    const ps = principals();
    if (pr === null || ps.length === 0) {
      setBlocks([]);
      return;
    }
    const g = grid();
    const from = dateToLocal(g.day);
    const to = dateToLocal(new Date(g.day.getTime() + 24 * 3600 * 1000));
    void c.queryFreeBusy(ps, from, to).then(setBlocks);
  });

  function statusAt(principal: string, hour: number): FbStatus {
    const g = grid();
    const slotStart = new Date(g.day.getTime() + hour * 3600 * 1000);
    const slotEnd = new Date(slotStart.getTime() + 3600 * 1000);
    let status: FbStatus = 'free';
    for (const b of blocks()) {
      if (b.principal !== principal) continue;
      const bs = localToDate(b.start);
      const be = localToDate(b.end);
      if (bs < slotEnd && be > slotStart) {
        if (b.status === 'busy') return 'busy';
        status = 'tentative';
      }
    }
    return status;
  }

  // ── resolutions ──
  async function run(fn: () => Promise<void>): Promise<void> {
    if (busy()) return;
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
    }
  }

  function reschedule(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { start: dateToLocal(pr.earlier.end) }));
  }

  function shorten(): void {
    const pr = pair();
    if (pr === null) return;
    const ms = pr.later.start.getTime() - pr.earlier.start.getTime();
    if (ms <= 0) return;
    void run(() => c.updateEvent(pr.earlier.event.id, { duration: formatDuration(ms) }));
  }

  function markTentative(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { status: 'tentative' }));
  }

  function doubleBook(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { freeBusyStatus: 'free' }));
  }

  function keepBoth(): void {
    // Accept the overlap unchanged. If it is the last pair, close the resolver.
    if (c.conflicts().length <= 1) props.onClose();
    else setSelected((i) => (i + 1) % c.conflicts().length);
  }

  // ── focus trap ──
  let dialogRef!: HTMLDivElement;
  let restoreEl: HTMLElement | null = null;
  onMount(() => {
    restoreEl = (document.activeElement as HTMLElement | null) ?? null;
    const first = dialogRef.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? dialogRef).focus();
  });
  onCleanup(() => restoreEl?.focus?.());

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      props.onClose();
      return;
    }
    if (e.key !== 'Tab') return;
    const nodes = Array.from(dialogRef.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (nodes.length === 0) return;
    const first = nodes[0]!;
    const last = nodes[nodes.length - 1]!;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function sidePanel(side: Side, which: 'earlier' | 'later'): JSX.Element {
    const cal = c.calendars().find((k) => k.id === side.event.calendarId);
    return (
      <div class={css.resolverSide} data-testid={`resolver-${which}`}>
        <h4 class={css.resolverSideTitle}><bdi>{side.event.title}</bdi></h4>
        <dl class={css.resolverMeta}>
          <dt>{t('calendar-field-start')}</dt>
          <dd>{formatFull(side.start)}</dd>
          <dt>{t('calendar-resolver-time')}</dt>
          <dd>{formatTime(side.start)} – {formatTime(side.end)}</dd>
          <dt>{t('calendar-field-calendar')}</dt>
          <dd><bdi>{cal?.name ?? ''}</bdi></dd>
          <Show when={side.event.locations[0] !== undefined}>
            <dt>{t('calendar-field-location')}</dt>
            <dd><bdi>{side.event.locations[0]!.name}</bdi></dd>
          </Show>
          <dt>{t('calendar-field-status')}</dt>
          <dd>{t(`calendar-status-${side.event.status}`)}</dd>
          <dt>{t('calendar-attendees')}</dt>
          <dd>{t('calendar-events-count-people', { count: Object.keys(side.event.participants).length })}</dd>
        </dl>
      </div>
    );
  }

  return (
    <div class={css.dialogBackdrop} onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div
        ref={dialogRef}
        class={css.resolverDialog}
        role="dialog"
        tabindex={-1}
        aria-label={t('calendar-resolver-title')}
        aria-modal="true"
        onKeyDown={onKeyDown}
      >
        <h2 style={{ margin: 0, 'font-size': '1.1rem' }}>{t('calendar-resolver-title')}</h2>

        <Show
          when={pair()}
          fallback={<p class={css.dimText}>{t('calendar-resolver-none')}</p>}
        >
          {(pr) => (
            <>
              <Show when={c.conflicts().length > 1}>
                <div class={css.row}>
                  <label class={css.label} for="resolver-pair">{t('calendar-resolver-pick')}</label>
                  <select
                    id="resolver-pair"
                    class={css.input}
                    value={selected()}
                    onChange={(e) => setSelected(Number(e.currentTarget.value))}
                  >
                    <For each={c.conflicts()}>
                      {(cp, i) => (
                        <option value={i()}>
                          {t('calendar-resolver-pair-n', {
                            n: i() + 1,
                            total: c.conflicts().length,
                            a: isolate(c.masterById(cp.a)?.title ?? ''),
                            b: isolate(c.masterById(cp.b)?.title ?? ''),
                          })}
                        </option>
                      )}
                    </For>
                  </select>
                </div>
              </Show>

              <div class={css.resolverGridTwo}>
                {sidePanel(pr().earlier, 'earlier')}
                {sidePanel(pr().later, 'later')}
              </div>

              <p class={css.dimText}>
                {t('calendar-resolver-overlap', {
                  start: formatTime(localToDate(pr().pair.overlapStart)),
                  end: formatTime(localToDate(pr().pair.overlapEnd)),
                })}
              </p>

              {/* free/busy grid — consumes queryFreeBusy */}
              <Show when={grid().hours.length > 0}>
                <div class={css.fbScroll}>
                  <table class={css.fbGrid} data-testid="freebusy-grid">
                    <caption class={css.srOnly}>{t('calendar-fb-caption')}</caption>
                    <thead>
                      <tr>
                        <th scope="col" class={css.fbCorner}>{t('calendar-fb-attendee')}</th>
                        <For each={grid().hours}>
                          {(h) => <th scope="col" class={css.fbHead}>{String(h).padStart(2, '0')}</th>}
                        </For>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={principals()}>
                        {(p) => (
                          <tr>
                            <th scope="row" class={css.fbRowHead}><bdi>{p}</bdi></th>
                            <For each={grid().hours}>
                              {(h) => {
                                const s = (): FbStatus => statusAt(p, h);
                                return (
                                  <td
                                    class={css.fbCell[s()]}
                                    aria-label={t('calendar-fb-cell', {
                                      principal: isolate(p),
                                      hour: `${String(h).padStart(2, '0')}:00`,
                                      status: t(`calendar-fb-${s()}`),
                                    })}
                                  >
                                    <span aria-hidden="true">{s() === 'free' ? '' : s() === 'busy' ? '●' : '◐'}</span>
                                  </td>
                                );
                              }}
                            </For>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </div>
              </Show>

              <Show when={hasAttendees()}>
                <p class={css.dimText} role="note">{t('calendar-resolver-update-note')}</p>
              </Show>

              <div class={css.resolverActions}>
                <button type="button" class={css.primaryButton} disabled={busy()} onClick={reschedule}>
                  {t('calendar-resolver-reschedule')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={shorten}>
                  {t('calendar-resolver-shorten')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={markTentative}>
                  {t('calendar-resolver-tentative')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={doubleBook}>
                  {t('calendar-resolver-double-book')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={keepBoth}>
                  {t('calendar-resolver-keep')}
                </button>
              </div>
            </>
          )}
        </Show>

        <div class={css.dialogActions}>
          <span class={css.spacer} />
          <button type="button" class={css.button} onClick={props.onClose}>{t('common-close')}</button>
        </div>
      </div>
    </div>
  );
}
